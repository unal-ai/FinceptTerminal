// Axum Web Server for Fincept Terminal
// This module provides the HTTP server implementation using Axum.
//
// Endpoints:
// - POST /api/rpc - JSON-RPC endpoint for all commands
// - GET /api/health - Health check endpoint
// - WS /ws - WebSocket endpoint for real-time data (future)
//
// Usage:
// Run with: cargo run --bin fincept-server --features web

use axum::{
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};

use super::rpc::{dispatch, AppState};
use super::types::{HealthResponse, RpcRequest, RpcResponse, ServerConfig};

/// Server state with startup time for uptime tracking
pub struct ServerState {
    pub app_state: AppState,
    pub start_time: Instant,
    pub config: ServerConfig,
}

/// Start the Axum web server
pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database
    crate::database::initialize().await?;

    // Create application state
    let app_state = AppState {
        mcp_state: Arc::new(tokio::sync::RwLock::new(crate::MCPState {
            processes: std::sync::Mutex::new(std::collections::HashMap::new()),
        })),
        ws_state: Arc::new(tokio::sync::RwLock::new(crate::WebSocketState {
            manager: Arc::new(tokio::sync::RwLock::new(
                crate::websocket::WebSocketManager::new(
                    Arc::new(tokio::sync::RwLock::new(crate::websocket::MessageRouter::new()))
                )
            )),
            router: Arc::new(tokio::sync::RwLock::new(crate::websocket::MessageRouter::new())),
            services: Arc::new(tokio::sync::RwLock::new(crate::WebSocketServices {
                paper_trading: crate::websocket::services::PaperTradingService::new(),
                arbitrage: crate::websocket::services::ArbitrageService::new(),
                portfolio: crate::websocket::services::PortfolioService::new(),
                monitoring: crate::websocket::services::MonitoringService::default(),
            })),
        })),
        barter_state: Arc::new(tokio::sync::RwLock::new(
            crate::barter_integration::commands::BarterState::new(
                crate::barter_integration::types::TradingMode::Paper
            )
        )),
    };

    let server_state = Arc::new(ServerState {
        app_state,
        start_time: Instant::now(),
        config: config.clone(),
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        .route("/api/rpc", post(rpc_handler))
        .route("/api/health", get(health_handler))
        .route("/", get(index_handler))
        .layer(cors)
        .with_state(server_state);

    // Start the server
    let addr = format!("{}:{}", config.host, config.port);
    println!("🚀 Fincept Terminal Web Server starting...");
    println!("   Listening on: http://{}", addr);
    println!("   RPC Endpoint: POST http://{}/api/rpc", addr);
    println!("   Health Check: GET http://{}/api/health", addr);
    println!("");
    println!("   Example RPC call:");
    println!("   curl -X POST http://{}/api/rpc \\", addr);
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{\"cmd\": \"greet\", \"args\": {{\"name\": \"World\"}}}}'");
    println!("");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// RPC endpoint handler
/// Accepts JSON-RPC style requests and dispatches to command handlers
async fn rpc_handler(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<RpcRequest>,
) -> impl IntoResponse {
    let response = dispatch(request, &state.app_state).await;
    Json(response)
}

/// Health check endpoint
async fn health_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
    })
}

/// Index handler - returns API documentation
async fn index_handler() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Fincept Terminal API</title>
    <style>
        body { font-family: system-ui, -apple-system, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; }
        h1 { color: #1a1a1a; }
        pre { background: #f5f5f5; padding: 1rem; border-radius: 4px; overflow-x: auto; }
        code { background: #f5f5f5; padding: 0.2rem 0.4rem; border-radius: 2px; }
        .endpoint { margin: 1rem 0; padding: 1rem; border: 1px solid #ddd; border-radius: 4px; }
        .method { font-weight: bold; color: #0066cc; }
    </style>
</head>
<body>
    <h1>🚀 Fincept Terminal API</h1>
    <p>Welcome to the Fincept Terminal Web API. This server exposes the same functionality as the desktop application via a JSON-RPC interface.</p>
    
    <h2>Endpoints</h2>
    
    <div class="endpoint">
        <p><span class="method">POST</span> <code>/api/rpc</code></p>
        <p>JSON-RPC endpoint for all commands. Send a JSON body with:</p>
        <pre>{
  "cmd": "command_name",
  "args": { "param1": "value1" }
}</pre>
    </div>
    
    <div class="endpoint">
        <p><span class="method">GET</span> <code>/api/health</code></p>
        <p>Health check endpoint. Returns server status and uptime.</p>
    </div>
    
    <h2>Example Commands</h2>
    
    <h3>Get Market Quote</h3>
    <pre>curl -X POST http://localhost:3000/api/rpc \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "get_market_quote", "args": {"symbol": "AAPL"}}'</pre>
    
    <h3>Get Historical Data</h3>
    <pre>curl -X POST http://localhost:3000/api/rpc \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "get_historical_data", "args": {"symbol": "AAPL", "startDate": "2024-01-01", "endDate": "2024-12-31"}}'</pre>
    
    <h3>Database Health Check</h3>
    <pre>curl -X POST http://localhost:3000/api/rpc \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "db_check_health", "args": {}}'</pre>
    
    <p><em>Version: "#.to_string() + env!("CARGO_PKG_VERSION") + r#"</em></p>
</body>
</html>
"#;

    (StatusCode::OK, [("content-type", "text/html")], html)
}

/// Main entry point for the web server binary
#[cfg(feature = "web")]
pub fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    rt.block_on(async {
        let config = ServerConfig::default();
        if let Err(e) = run_server(config).await {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    });
}
