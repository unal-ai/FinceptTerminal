// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::collections::HashMap;
use std::process::{Child, Command, Stdio, ChildStdin};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Write};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};
use serde::Serialize;
use sha2::{Sha256, Digest};
use tauri::{Manager, Listener};

// Data sources and commands modules
mod data_sources;
pub mod commands;
pub mod command_registry;
mod utils;
mod setup;
pub mod database;
mod python_runtime;
mod worker_pool;
pub mod websocket;
pub mod barter_integration;

// Web server module (feature-gated)
#[cfg(feature = "web")]
pub mod server;

// mod finscript; // TODO: Implement FinScript module

// MCP Server Process with communication channels
pub struct MCPProcess {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    response_rx: Receiver<String>,
}

// Global state to manage MCP server processes
pub struct MCPState {
    pub processes: Mutex<HashMap<String, MCPProcess>>,
}

// Global state for WebSocket manager
pub struct WebSocketState {
    pub manager: Arc<tokio::sync::RwLock<websocket::WebSocketManager>>,
    pub router: Arc<tokio::sync::RwLock<websocket::MessageRouter>>,
    pub services: Arc<tokio::sync::RwLock<WebSocketServices>>,
}

pub struct WebSocketServices {
    pub paper_trading: websocket::services::PaperTradingService,
    pub arbitrage: websocket::services::ArbitrageService,
    pub portfolio: websocket::services::PortfolioService,
    pub monitoring: websocket::services::MonitoringService,
}

#[derive(Debug, Serialize)]
struct SpawnResult {
    pid: u32,
    success: bool,
    error: Option<String>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Cleanup workflow on app close (called from frontend)
#[tauri::command]
async fn cleanup_running_workflows() -> Result<(), String> {
    // This is just a marker command - the actual cleanup happens in the frontend
    // via workflowService.cleanupRunningWorkflows()
    Ok(())
}

// Spawn an MCP server process with background stdout reader
#[tauri::command]
fn spawn_mcp_server(
    app: tauri::AppHandle,
    state: tauri::State<MCPState>,
    server_id: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
) -> Result<SpawnResult, String> {
    spawn_mcp_server_internal(
        Some(&app),
        &state,
        server_id,
        command,
        args,
        env,
    )
}

/// Spawn an MCP server process in a runtime-agnostic way.
///
/// This internal function provides the core logic for spawning MCP (Model Context Protocol)
/// server processes, and can be called from both Tauri command contexts and web server
/// RPC handlers.
///
/// # Parameters
///
/// * `app` - Optional Tauri application handle. Pass `Some(&app_handle)` when called from
///   a Tauri command to enable bundled Bun path resolution. Pass `None` when called from
///   a web server context where no Tauri runtime is available.
/// * `state` - Reference to the shared [`MCPState`] that tracks spawned MCP server processes.
/// * `server_id` - Unique identifier for this MCP server instance.
/// * `command` - The command to execute (e.g., "npx", "bunx", "node", or a direct path).
/// * `args` - Command-line arguments to pass to the spawned process.
/// * `env` - Environment variables to set for the spawned process.
///
/// # Bun/npx Substitution
///
/// When `command` is "npx" or "bunx", this function attempts to resolve the bundled Bun
/// executable path and substitutes the command with `bun x`, which provides equivalent
/// functionality. This ensures that MCP servers can be spawned consistently across
/// different runtime environments without requiring a separate Node.js installation.
///
/// # Returns
///
/// Returns a [`SpawnResult`] containing the process ID on success, or an error message
/// if the process could not be spawned.
pub(crate) fn spawn_mcp_server_internal(
    app: Option<&tauri::AppHandle>,
    state: &MCPState,
    server_id: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
) -> Result<SpawnResult, String> {
    // Determine if we should use bundled Bun (for npx/bunx commands)
    let (fixed_command, fixed_args) = if command == "npx" || command == "bunx" {
        // Try to get bundled Bun path
        match utils::python::get_bundled_bun_path_for_runtime(app) {
            Ok(bun_path) => {
                // Use 'bun x' which is equivalent to 'bunx' or 'npx'
                let mut new_args = vec!["x".to_string()];
                new_args.extend(args.clone());
                (bun_path.to_string_lossy().to_string(), new_args)
            }
            Err(_) => {
                // Fall back to system npx
                #[cfg(target_os = "windows")]
                let cmd = "npx.cmd".to_string();
                #[cfg(not(target_os = "windows"))]
                let cmd = "npx".to_string();
                (cmd, args.clone())
            }
        }
    } else {
        // Fix command for Windows - node/python need .exe extension
        #[cfg(target_os = "windows")]
        let cmd = if command == "node" {
            "node.exe".to_string()
        } else if command == "python" {
            "python.exe".to_string()
        } else {
            command.clone()
        };

        #[cfg(not(target_os = "windows"))]
        let cmd = command.clone();

        (cmd, args.clone())
    };

    // Build command
    let mut cmd = Command::new(&fixed_command);
    cmd.args(&fixed_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Add environment variables
    for (key, value) in env {
        cmd.env(key, value);
    }

    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    // Spawn process
    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id();

            // Extract stdin and stdout
            let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
            let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
            let stderr = child.stderr.take();

            // Create channel for responses
            let (response_tx, response_rx): (Sender<String>, Receiver<String>) = channel();

            // Spawn background thread to read stdout
            thread::spawn(move || {
                let reader = BufReader::new(stdout);

                for line in reader.lines() {
                    match line {
                        Ok(content) => {
                            if !content.trim().is_empty() {
                                if response_tx.send(content).is_err() {
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            });

            // Spawn background thread to read stderr (for debugging)
            if let Some(stderr) = stderr {
                let _server_id_clone = server_id.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(content) = line {
                            if !content.trim().is_empty() {
                                eprintln!("[MCP] {}", content);
                            }
                        }
                    }
                });
            }

            // Store process with communication channels
            let mcp_process = MCPProcess {
                child,
                stdin: Arc::new(Mutex::new(stdin)),
                response_rx,
            };

            let mut processes = state.processes.lock().unwrap();
            processes.insert(server_id.clone(), mcp_process);

            Ok(SpawnResult {
                pid,
                success: true,
                error: None,
            })
        }
        Err(e) => {
            eprintln!("[Tauri] Failed to spawn MCP server: {}", e);
            Ok(SpawnResult {
                pid: 0,
                success: false,
                error: Some(format!("Failed to spawn process: {}", e)),
            })
        }
    }
}

// Send JSON-RPC request to MCP server with timeout
#[tauri::command]
fn send_mcp_request(
    state: tauri::State<MCPState>,
    server_id: String,
    request: String,
) -> Result<String, String> {
    send_mcp_request_internal(&state, server_id, request)
}

pub(crate) fn send_mcp_request_internal(
    state: &MCPState,
    server_id: String,
    request: String,
) -> Result<String, String> {
    println!("[Tauri] Sending request to server {}: {}", server_id, request);

    let mut processes = state.processes.lock().unwrap();

    if let Some(mcp_process) = processes.get_mut(&server_id) {
        // Write request to stdin
        {
            let mut stdin = mcp_process.stdin.lock().unwrap();
            writeln!(stdin, "{}", request)
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin.flush()
                .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        }

        // Wait for response with timeout (30 seconds for initial package download)
        match mcp_process.response_rx.recv_timeout(Duration::from_secs(30)) {
            Ok(response) => {
                Ok(response)
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                Err("Timeout: No response from server within 30 seconds".to_string())
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err("Server process has terminated unexpectedly".to_string())
            }
        }
    } else {
        Err(format!("Server {} not found", server_id))
    }
}

// Send notification (fire and forget)
#[tauri::command]
fn send_mcp_notification(
    state: tauri::State<MCPState>,
    server_id: String,
    notification: String,
) -> Result<(), String> {
    send_mcp_notification_internal(&state, server_id, notification)
}

pub(crate) fn send_mcp_notification_internal(
    state: &MCPState,
    server_id: String,
    notification: String,
) -> Result<(), String> {
    let mut processes = state.processes.lock().unwrap();

    if let Some(mcp_process) = processes.get_mut(&server_id) {
        let mut stdin = mcp_process.stdin.lock().unwrap();
        writeln!(stdin, "{}", notification)
            .map_err(|e| format!("Failed to write notification: {}", e))?;
        stdin.flush()
            .map_err(|e| format!("Failed to flush: {}", e))?;
        Ok(())
    } else {
        Err(format!("Server {} not found", server_id))
    }
}

// Ping MCP server to check if alive
#[tauri::command]
fn ping_mcp_server(
    state: tauri::State<MCPState>,
    server_id: String,
) -> Result<bool, String> {
    ping_mcp_server_internal(&state, server_id)
}

pub(crate) fn ping_mcp_server_internal(
    state: &MCPState,
    server_id: String,
) -> Result<bool, String> {
    let mut processes = state.processes.lock().unwrap();

    if let Some(mcp_process) = processes.get_mut(&server_id) {
        // Check if process is still running
        match mcp_process.child.try_wait() {
            Ok(Some(_)) => Ok(false), // Process has exited
            Ok(None) => Ok(true),      // Process is still running
            Err(_) => Ok(false),       // Error checking status
        }
    } else {
        Ok(false) // Server not found
    }
}

// Kill MCP server
#[tauri::command]
fn kill_mcp_server(
    state: tauri::State<MCPState>,
    server_id: String,
) -> Result<(), String> {
    kill_mcp_server_internal(&state, server_id)
}

pub(crate) fn kill_mcp_server_internal(
    state: &MCPState,
    server_id: String,
) -> Result<(), String> {
    let mut processes = state.processes.lock().unwrap();

    if let Some(mut mcp_process) = processes.remove(&server_id) {
        match mcp_process.child.kill() {
            Ok(_) => {
                Ok(())
            }
            Err(e) => Err(format!("Failed to kill server: {}", e)),
        }
    } else {
        Ok(()) // Server not found, consider it killed
    }
}

// SHA256 hash for Fyers authentication
#[tauri::command]
fn sha256_hash(input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

// ============================================================================
// WEBSOCKET COMMANDS
// ============================================================================

/// Set WebSocket provider configuration
#[tauri::command]
async fn ws_set_config(
    state: tauri::State<'_, WebSocketState>,
    config: websocket::types::ProviderConfig,
) -> Result<(), String> {
    let manager = state.manager.read().await;
    manager.set_config(config.clone());
    Ok(())
}

/// Connect to WebSocket provider
#[tauri::command]
async fn ws_connect(
    state: tauri::State<'_, WebSocketState>,
    provider: String,
) -> Result<(), String> {
    eprintln!("[ws_connect] Called for provider: {}", provider);

    let manager = state.manager.read().await;
    let result = manager.connect(&provider).await
        .map_err(|e| e.to_string());

    match &result {
        Ok(_) => eprintln!("[ws_connect] ✓ Successfully connected to {}", provider),
        Err(e) => eprintln!("[ws_connect] ✗ Failed to connect to {}: {}", provider, e),
    }

    result
}

/// Disconnect from WebSocket provider
#[tauri::command]
async fn ws_disconnect(
    state: tauri::State<'_, WebSocketState>,
    provider: String,
) -> Result<(), String> {
    let manager = state.manager.read().await;
    manager.disconnect(&provider).await
        .map_err(|e| e.to_string())
}

/// Subscribe to WebSocket channel
#[tauri::command]
async fn ws_subscribe(
    state: tauri::State<'_, WebSocketState>,
    provider: String,
    symbol: String,
    channel: String,
    params: Option<serde_json::Value>,
) -> Result<(), String> {
    eprintln!("[ws_subscribe] Called: provider={}, symbol={}, channel={}", provider, symbol, channel);

    // Register frontend subscriber
    let topic = format!("{}.{}.{}", provider, channel, symbol);
    eprintln!("[ws_subscribe] Registering frontend subscriber for topic: {}", topic);
    state.router.write().await.subscribe_frontend(&topic);

    // Subscribe via manager
    eprintln!("[ws_subscribe] Calling manager.subscribe...");
    let manager = state.manager.read().await;
    let result = manager.subscribe(&provider, &symbol, &channel, params).await
        .map_err(|e| e.to_string());

    match &result {
        Ok(_) => eprintln!("[ws_subscribe] ✓ Successfully subscribed to {} {} {}", provider, symbol, channel),
        Err(e) => eprintln!("[ws_subscribe] ✗ Failed to subscribe: {}", e),
    }

    result
}

/// Unsubscribe from WebSocket channel
#[tauri::command]
async fn ws_unsubscribe(
    state: tauri::State<'_, WebSocketState>,
    provider: String,
    symbol: String,
    channel: String,
) -> Result<(), String> {
    // Unregister frontend subscriber
    state.router.write().await.unsubscribe_frontend(&format!("{}.{}.{}", provider, channel, symbol));

    // Unsubscribe via manager
    let manager = state.manager.read().await;
    manager.unsubscribe(&provider, &symbol, &channel).await
        .map_err(|e| e.to_string())
}

/// Get connection metrics for a provider
#[tauri::command]
async fn ws_get_metrics(
    state: tauri::State<'_, WebSocketState>,
    provider: String,
) -> Result<Option<websocket::types::ConnectionMetrics>, String> {
    let manager = state.manager.read().await;
    Ok(manager.get_metrics(&provider))
}

/// Get all connection metrics
#[tauri::command]
async fn ws_get_all_metrics(
    state: tauri::State<'_, WebSocketState>,
) -> Result<Vec<websocket::types::ConnectionMetrics>, String> {
    let manager = state.manager.read().await;
    Ok(manager.get_all_metrics())
}

/// Reconnect to provider
#[tauri::command]
async fn ws_reconnect(
    state: tauri::State<'_, WebSocketState>,
    provider: String,
) -> Result<(), String> {
    let manager = state.manager.read().await;
    manager.reconnect(&provider).await
        .map_err(|e| e.to_string())
}

// ============================================================================
// MONITORING COMMANDS
// ============================================================================

/// Add monitoring condition
#[tauri::command]
async fn monitor_add_condition(
    _app: tauri::AppHandle,
    state: tauri::State<'_, WebSocketState>,
    condition: websocket::services::monitoring::MonitorCondition,
) -> Result<i64, String> {
    use rusqlite::params;

    let pool = database::pool::get_pool().map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO monitor_conditions (provider, symbol, field, operator, value, value2, enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &condition.provider,
            &condition.symbol,
            match condition.field {
                websocket::services::monitoring::MonitorField::Price => "price",
                websocket::services::monitoring::MonitorField::Volume => "volume",
                websocket::services::monitoring::MonitorField::ChangePercent => "change_percent",
                websocket::services::monitoring::MonitorField::Spread => "spread",
            },
            match condition.operator {
                websocket::services::monitoring::MonitorOperator::GreaterThan => ">",
                websocket::services::monitoring::MonitorOperator::LessThan => "<",
                websocket::services::monitoring::MonitorOperator::GreaterThanOrEqual => ">=",
                websocket::services::monitoring::MonitorOperator::LessThanOrEqual => "<=",
                websocket::services::monitoring::MonitorOperator::Equal => "==",
                websocket::services::monitoring::MonitorOperator::Between => "between",
            },
            condition.value,
            condition.value2,
            if condition.enabled { 1 } else { 0 },
        ],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    // Reload conditions
    let services = state.services.read().await;
    let _ = services.monitoring.load_conditions().await;

    Ok(id)
}

/// Get all monitoring conditions
#[tauri::command]
async fn monitor_get_conditions(
    _app: tauri::AppHandle,
) -> Result<Vec<websocket::services::monitoring::MonitorCondition>, String> {
    let pool = database::pool::get_pool().map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, provider, symbol, field, operator, value, value2, enabled
         FROM monitor_conditions
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let conditions = stmt
        .query_map([], |row| {
            Ok(websocket::services::monitoring::MonitorCondition {
                id: Some(row.get(0)?),
                provider: row.get(1)?,
                symbol: row.get(2)?,
                field: websocket::services::monitoring::MonitorField::from_str(&row.get::<_, String>(3)?).unwrap(),
                operator: websocket::services::monitoring::MonitorOperator::from_str(&row.get::<_, String>(4)?).unwrap(),
                value: row.get(5)?,
                value2: row.get(6)?,
                enabled: row.get::<_, i32>(7)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(conditions)
}

/// Delete monitoring condition
#[tauri::command]
async fn monitor_delete_condition(
    _app: tauri::AppHandle,
    state: tauri::State<'_, WebSocketState>,
    id: i64,
) -> Result<(), String> {
    use rusqlite::params;

    let pool = database::pool::get_pool().map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM monitor_conditions WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    // Reload conditions
    let services = state.services.read().await;
    services.monitoring.load_conditions().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// Get recent alerts
#[tauri::command]
async fn monitor_get_alerts(
    _app: tauri::AppHandle,
    limit: i64,
) -> Result<Vec<websocket::services::monitoring::MonitorAlert>, String> {
    use rusqlite::params;

    let pool = database::pool::get_pool().map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, condition_id, provider, symbol, field, triggered_value, triggered_at
         FROM monitor_alerts
         ORDER BY triggered_at DESC
         LIMIT ?1"
    ).map_err(|e| e.to_string())?;

    let alerts = stmt
        .query_map(params![limit], |row| {
            Ok(websocket::services::monitoring::MonitorAlert {
                id: Some(row.get(0)?),
                condition_id: row.get(1)?,
                provider: row.get(2)?,
                symbol: row.get(3)?,
                field: websocket::services::monitoring::MonitorField::from_str(&row.get::<_, String>(4)?).unwrap(),
                triggered_value: row.get(5)?,
                triggered_at: row.get::<_, i64>(6)? as u64,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(alerts)
}

/// Load monitoring conditions on startup
#[tauri::command]
async fn monitor_load_conditions(
    state: tauri::State<'_, WebSocketState>,
) -> Result<(), String> {
    let services = state.services.read().await;
    services.monitoring.load_conditions().await.map_err(|e| e.to_string())
}

// Windows-specific imports to hide console windows
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows creation flags to hide console window
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// Execute Python script with arguments and environment variables
#[tauri::command]
fn execute_python_script(
    app: tauri::AppHandle,
    script_name: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let python_path = utils::python::get_python_path(&app)?;
    let script_path = utils::python::get_script_path(&app, &script_name)?;

    // Verify paths exist
    // Skip existence check for system Python commands (like "python" or "python3")
    // which are found in PATH but not as file paths
    let is_system_command = python_path.to_string_lossy() == "python"
        || python_path.to_string_lossy() == "python3"
        || python_path.to_string_lossy() == "python.exe";

    if !is_system_command && !python_path.exists() {
        return Err(format!("Python executable not found at: {:?}", python_path));
    }
    if !script_path.exists() {
        return Err(format!("Script not found at: {:?}", script_path));
    }

    let mut cmd = Command::new(&python_path);
    cmd.arg(&script_path).args(&args);

    // Add environment variables
    for (key, value) in env {
        cmd.env(key, value);
    }

    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|e| format!("Failed to parse output: {}", e))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Python script failed: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to execute Python script: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use crate::generate_handler_from_list;

    // Initialize high-performance Rust SQLite database
    // CRITICAL: Database is required for paper trading and other core features
    if let Err(e) = tokio::runtime::Runtime::new().unwrap().block_on(database::initialize()) {
        eprintln!("========================================");
        eprintln!("CRITICAL ERROR: Failed to initialize database!");
        eprintln!("Error: {}", e);
        eprintln!("The application cannot function without the database.");
        eprintln!("Please ensure you have write permissions to:");
        eprintln!("  Windows: %APPDATA%\\fincept-terminal");
        eprintln!("  macOS: ~/Library/Application Support/fincept-terminal");
        eprintln!("  Linux: ~/.local/share/fincept-terminal");
        eprintln!("========================================");
        // Note: We don't panic here to allow the app to show an error UI
        // The frontend will detect database failures via health checks
    }

    // Initialize WebSocket system
    let router = Arc::new(tokio::sync::RwLock::new(websocket::MessageRouter::new()));
    let manager = Arc::new(tokio::sync::RwLock::new(websocket::WebSocketManager::new(router.clone())));

    // Initialize services with default monitoring (will be configured in setup)
    let services = Arc::new(tokio::sync::RwLock::new(WebSocketServices {
        paper_trading: websocket::services::PaperTradingService::new(),
        arbitrage: websocket::services::ArbitrageService::new(),
        portfolio: websocket::services::PortfolioService::new(),
        monitoring: websocket::services::MonitoringService::default(),
    }));

    let ws_state = WebSocketState {
        manager: manager.clone(),
        router: router.clone(),
        services: services.clone(),
    };

    // Initialize Barter trading system (Paper mode by default)
    let barter_state = barter_integration::commands::BarterState::new(
        barter_integration::types::TradingMode::Paper
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_cors_fetch::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(MCPState {
            processes: Mutex::new(HashMap::new()),
        })
        .manage(commands::backtesting::BacktestingState::default())
        .manage(ws_state)
        .manage(barter_state)
        .setup(move |app| {
            // CRITICAL: Set app handle for router to emit WebSocket events to frontend
            let app_handle = app.handle().clone();
            let router_clone = router.clone();
            let services_clone = services.clone();

            // Get database path
            let db_path = app.path().app_data_dir()
                .map(|p| p.join("fincept_terminal.db").to_string_lossy().to_string())
                .unwrap_or_else(|_| "fincept_terminal.db".to_string());

            // Use tauri::async_runtime to spawn task in Tauri's runtime
            tauri::async_runtime::spawn(async move {
                // Set router app handle
                router_clone.write().await.set_app_handle(app_handle.clone());

                // Initialize monitoring service with proper database path
                let mut services_guard = services_clone.write().await;
                services_guard.monitoring = websocket::services::MonitoringService::new(db_path);
                services_guard.monitoring.set_app_handle(app_handle.clone());

                // Subscribe to ticker stream and start monitoring
                let ticker_rx = router_clone.read().await.subscribe_ticker();
                services_guard.monitoring.start_monitoring(ticker_rx);

                // Load existing conditions from database
                let _ = services_guard.monitoring.load_conditions().await;

                drop(services_guard); // Release the lock before listening

                // Listen for Fyers ticker events from frontend
                let router_clone_for_event = router_clone.clone();
                let _ = app_handle.listen("fyers_ticker", move |event: tauri::Event| {
                    if let Ok(payload_str) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                        if let Some(payload) = payload_str.as_object() {
                            // Parse ticker data from frontend event
                            let ticker = websocket::types::TickerData {
                                provider: payload.get("provider").and_then(|v| v.as_str()).unwrap_or("fyers").to_string(),
                                symbol: payload.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                price: payload.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                volume: payload.get("volume").and_then(|v| v.as_f64()),
                                bid: payload.get("bid").and_then(|v| v.as_f64()),
                                ask: payload.get("ask").and_then(|v| v.as_f64()),
                                bid_size: payload.get("bid_size").and_then(|v| v.as_f64()),
                                ask_size: payload.get("ask_size").and_then(|v| v.as_f64()),
                                high: payload.get("high").and_then(|v| v.as_f64()),
                                low: payload.get("low").and_then(|v| v.as_f64()),
                                open: payload.get("open").and_then(|v| v.as_f64()),
                                close: payload.get("close").and_then(|v| v.as_f64()),
                                change: payload.get("change").and_then(|v| v.as_f64()),
                                change_percent: payload.get("change_percent").and_then(|v| v.as_f64()),
                                timestamp: payload.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0),
                            };

                            // Route to monitoring service
                            let router = router_clone_for_event.clone();
                            tauri::async_runtime::spawn(async move {
                                router.read().await.route(websocket::types::MarketMessage::Ticker(ticker)).await;
                            });
                        }
                    }
                });
            });

            Ok(())
        })
        .invoke_handler(
            crate::for_each_tauri_command!(generate_handler_from_list)
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
