// Axum Web Server for Fincept Terminal
// This module provides the HTTP server implementation using Axum.
//
// Endpoints:
// - POST /api/rpc - JSON-RPC endpoint for all commands
// - GET /api/health - Health check endpoint
// - GET /api/ready - Readiness check endpoint
// - WS /ws - WebSocket endpoint for real-time data (future)
//
// Production Features:
// - Request tracing with unique request IDs
// - Structured logging
// - CORS configuration
// - Health and readiness checks
//
// Usage:
// Run with: cargo run --bin fincept-server --features web

use axum::{
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

use super::rpc::dispatch;
use super::types::{HealthResponse, RpcRequest, RpcResponse, ServerConfig};

/// Server state with startup time for uptime tracking
pub struct ServerState {
    pub start_time: Instant,
    pub config: ServerConfig,
    pub request_count: std::sync::atomic::AtomicU64,
}

/// Start the Axum web server
pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database
    crate::database::initialize().await?;

    let server_state = Arc::new(ServerState {
        start_time: Instant::now(),
        config: config.clone(),
        request_count: std::sync::atomic::AtomicU64::new(0),
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Request ID layer for tracing
    let x_request_id = axum::http::HeaderName::from_static("x-request-id");

    // Build the router with middleware
    let app = Router::new()
        .route("/api/rpc", post(rpc_handler))
        .route("/api/health", get(health_handler))
        .route("/api/ready", get(ready_handler))
        .route("/", get(index_handler))
        .layer(middleware::from_fn_with_state(server_state.clone(), request_logging_middleware))
        .layer(cors)
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(SetRequestIdLayer::new(x_request_id.clone(), MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .with_state(server_state);

    // Start the server
    let addr = format!("{}:{}", config.host, config.port);
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║     FINCEPT TERMINAL WEB SERVER - PRODUCTION READY        ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  🚀 Server starting...                                    ║");
    println!("║  📍 Listening on: http://{:<29} ║", addr);
    println!("║                                                           ║");
    println!("║  Endpoints:                                               ║");
    println!("║  • POST /api/rpc    - JSON-RPC commands                   ║");
    println!("║  • GET  /api/health - Health check                        ║");
    println!("║  • GET  /api/ready  - Readiness check                     ║");
    println!("║  • GET  /           - API documentation                   ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Features:                                                ║");
    println!("║  ✓ Request tracing with X-Request-ID                      ║");
    println!("║  ✓ Structured logging                                     ║");
    println!("║  ✓ CORS enabled                                           ║");
    println!("║  ✓ Health & readiness checks                              ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Request logging middleware
async fn request_logging_middleware(
    State(state): State<Arc<ServerState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Increment request counter
    state.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Process request
    let response = next.run(request).await;

    // Log request completion
    let duration = start.elapsed();
    let status = response.status();
    
    // Log format: [request_id] METHOD /path -> STATUS (duration_ms)
    if status.is_success() {
        tracing::info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    } else {
        tracing::warn!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request failed"
        );
    }

    response
}

/// RPC endpoint handler
/// Accepts JSON-RPC style requests and dispatches to command handlers
async fn rpc_handler(
    Json(request): Json<RpcRequest>,
) -> impl IntoResponse {
    let cmd = request.cmd.clone();
    tracing::debug!(command = %cmd, "Processing RPC command");
    
    let response = dispatch(request).await;
    
    if response.success {
        tracing::debug!(command = %cmd, "RPC command succeeded");
    } else {
        tracing::warn!(command = %cmd, error = ?response.error, "RPC command failed");
    }
    
    Json(response)
}

/// Health check endpoint - always returns healthy if server is running
async fn health_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    let _total_requests = state.request_count.load(std::sync::atomic::Ordering::Relaxed);
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
    })
}

/// Readiness check endpoint - checks if server is ready to serve traffic
async fn ready_handler(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    // Check database connectivity
    match crate::database::pool::get_pool() {
        Ok(pool) => {
            match pool.get() {
                Ok(_) => {
                    let uptime = state.start_time.elapsed().as_secs();
                    (StatusCode::OK, Json(serde_json::json!({
                        "status": "ready",
                        "database": "connected",
                        "uptime_seconds": uptime
                    })))
                }
                Err(_) => {
                    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                        "status": "not_ready",
                        "database": "disconnected",
                        "error": "Database connection failed"
                    })))
                }
            }
        }
        Err(e) => {
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "status": "not_ready",
                "database": "error",
                "error": format!("Database pool error: {}", e)
            })))
        }
    }
}

/// Index handler - returns API documentation
async fn index_handler() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Fincept Terminal API</title>
    <style>
        body { font-family: system-ui, -apple-system, sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem; }
        h1 { color: #1a1a1a; }
        h2 { color: #333; border-bottom: 2px solid #3b82f6; padding-bottom: 0.5rem; margin-top: 2rem; }
        h3 { color: #555; margin-top: 1.5rem; }
        pre { background: #f5f5f5; padding: 1rem; border-radius: 4px; overflow-x: auto; }
        code { background: #f5f5f5; padding: 0.2rem 0.4rem; border-radius: 2px; }
        .endpoint { margin: 1rem 0; padding: 1rem; border: 1px solid #ddd; border-radius: 4px; }
        .method { font-weight: bold; color: #0066cc; }
        .category { margin: 1rem 0; }
        .category-title { font-weight: bold; color: #3b82f6; margin-bottom: 0.5rem; }
        ul { list-style-type: none; padding-left: 1rem; }
        li { margin: 0.3rem 0; }
        li code { font-weight: bold; }
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
    
    <h2>Available Commands</h2>
    
    <div class="category">
        <p class="category-title">📊 Market Data</p>
        <ul>
            <li><code>get_market_quote</code> - Get real-time stock quote</li>
            <li><code>get_market_quotes</code> - Get multiple stock quotes</li>
            <li><code>get_historical_data</code> - Get historical price data</li>
            <li><code>get_stock_info</code> - Get company information</li>
            <li><code>get_financials</code> - Get financial statements</li>
            <li><code>get_period_returns</code> - Get period returns (7D, 30D)</li>
            <li><code>check_market_data_health</code> - Check market data provider status</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">⚙️ Settings & Database</p>
        <ul>
            <li><code>db_check_health</code> - Check database status</li>
            <li><code>db_get_all_settings</code> - Get all settings</li>
            <li><code>db_get_setting</code> - Get a specific setting</li>
            <li><code>db_save_setting</code> - Save a setting</li>
            <li><code>check_setup_status</code> - Check system setup status</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">🔑 Credentials</p>
        <ul>
            <li><code>db_get_credentials</code> - Get all API credentials</li>
            <li><code>db_save_credential</code> - Save an API credential</li>
            <li><code>db_get_credential_by_service</code> - Get credential by service name</li>
            <li><code>db_delete_credential</code> - Delete a credential</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">🤖 LLM Configuration</p>
        <ul>
            <li><code>db_get_llm_configs</code> - Get LLM provider configs</li>
            <li><code>db_save_llm_config</code> - Save LLM config</li>
            <li><code>db_get_llm_global_settings</code> - Get global LLM settings</li>
            <li><code>db_save_llm_global_settings</code> - Save global LLM settings</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">💬 Chat Sessions</p>
        <ul>
            <li><code>db_create_chat_session</code> - Create new chat session</li>
            <li><code>db_get_chat_sessions</code> - Get chat session history</li>
            <li><code>db_add_chat_message</code> - Add message to session</li>
            <li><code>db_get_chat_messages</code> - Get messages for session</li>
            <li><code>db_delete_chat_session</code> - Delete a chat session</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">📁 Data Sources</p>
        <ul>
            <li><code>db_get_all_data_sources</code> - Get all data sources</li>
            <li><code>db_save_data_source</code> - Save a data source</li>
            <li><code>db_delete_data_source</code> - Delete a data source</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">💼 Portfolios</p>
        <ul>
            <li><code>db_list_portfolios</code> - List all portfolios</li>
            <li><code>db_get_portfolio</code> - Get portfolio by ID</li>
            <li><code>db_create_portfolio</code> - Create new portfolio</li>
            <li><code>db_delete_portfolio</code> - Delete a portfolio</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">👁️ Watchlists</p>
        <ul>
            <li><code>db_get_watchlists</code> - Get all watchlists</li>
            <li><code>db_create_watchlist</code> - Create new watchlist</li>
            <li><code>db_get_watchlist_stocks</code> - Get stocks in watchlist</li>
            <li><code>db_add_watchlist_stock</code> - Add stock to watchlist</li>
            <li><code>db_remove_watchlist_stock</code> - Remove stock from watchlist</li>
            <li><code>db_delete_watchlist</code> - Delete a watchlist</li>
        </ul>
    </div>

    <div class="category">
        <p class="category-title">🔧 Utilities</p>
        <ul>
            <li><code>greet</code> - Test endpoint</li>
            <li><code>sha256_hash</code> - Compute SHA256 hash</li>
        </ul>
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
    
    <h3>Create Watchlist</h3>
    <pre>curl -X POST http://localhost:3000/api/rpc \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "db_create_watchlist", "args": {"name": "Tech Stocks", "color": "#3b82f6"}}'</pre>

    <h3>Create Portfolio</h3>
    <pre>curl -X POST http://localhost:3000/api/rpc \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "db_create_portfolio", "args": {"name": "Main Portfolio", "currency": "USD"}}'</pre>
    
    <p><em>Version: "#.to_string() + env!("CARGO_PKG_VERSION") + r#"</em></p>
</body>
</html>
"#;

    (axum::http::StatusCode::OK, [("content-type", "text/html")], html)
}
