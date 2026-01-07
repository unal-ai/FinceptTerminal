// RPC Command Dispatcher
// This module maps command names to their handlers, similar to Tauri's invoke_handler.
// It allows reusing all existing command logic without modification.

use super::types::{RpcRequest, RpcResponse};
use serde_json::Value;
use std::sync::Arc;

/// Application state for the web server
/// This replaces Tauri's managed state system
pub struct AppState {
    pub mcp_state: Arc<tokio::sync::RwLock<crate::MCPState>>,
    pub ws_state: Arc<tokio::sync::RwLock<crate::WebSocketState>>,
    pub barter_state: Arc<tokio::sync::RwLock<crate::barter_integration::commands::BarterState>>,
}

/// Dispatch an RPC request to the appropriate command handler
/// 
/// This function acts as the central router, mapping command names to their
/// implementations. It mirrors the behavior of Tauri's invoke_handler macro.
pub async fn dispatch(request: RpcRequest, _state: &AppState) -> RpcResponse {
    let args = request.args;
    
    match request.cmd.as_str() {
        // ============================================================================
        // BASIC COMMANDS
        // ============================================================================
        "greet" => {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("World");
            RpcResponse::ok(format!("Hello, {}! You've been greeted from Rust Web Server!", name))
        }

        // ============================================================================
        // MARKET DATA COMMANDS
        // ============================================================================
        "get_market_quote" => {
            dispatch_market_quote(args).await
        }
        "get_market_quotes" => {
            dispatch_market_quotes(args).await
        }
        "get_period_returns" => {
            dispatch_period_returns(args).await
        }
        "check_market_data_health" => {
            dispatch_market_health().await
        }
        "get_historical_data" => {
            dispatch_historical_data(args).await
        }
        "get_stock_info" => {
            dispatch_stock_info(args).await
        }
        "get_financials" => {
            dispatch_financials(args).await
        }

        // ============================================================================
        // DATABASE COMMANDS
        // ============================================================================
        "db_check_health" => {
            dispatch_db_health().await
        }
        "db_get_all_settings" => {
            dispatch_db_get_all_settings().await
        }
        "db_get_setting" => {
            dispatch_db_get_setting(args).await
        }
        "db_save_setting" => {
            dispatch_db_save_setting(args).await
        }

        // ============================================================================
        // WATCHLIST COMMANDS
        // ============================================================================
        "db_get_watchlists" => {
            dispatch_db_get_watchlists().await
        }
        "db_create_watchlist" => {
            dispatch_db_create_watchlist(args).await
        }
        "db_get_watchlist_stocks" => {
            dispatch_db_get_watchlist_stocks(args).await
        }
        "db_add_watchlist_stock" => {
            dispatch_db_add_watchlist_stock(args).await
        }
        "db_remove_watchlist_stock" => {
            dispatch_db_remove_watchlist_stock(args).await
        }
        "db_delete_watchlist" => {
            dispatch_db_delete_watchlist(args).await
        }

        // ============================================================================
        // CREDENTIAL COMMANDS
        // ============================================================================
        "db_get_credentials" => {
            dispatch_db_get_credentials().await
        }
        "db_save_credential" => {
            dispatch_db_save_credential(args).await
        }
        "db_delete_credential" => {
            dispatch_db_delete_credential(args).await
        }

        // ============================================================================
        // LLM CONFIG COMMANDS
        // ============================================================================
        "db_get_llm_configs" => {
            dispatch_db_get_llm_configs().await
        }
        "db_save_llm_config" => {
            dispatch_db_save_llm_config(args).await
        }
        "db_get_llm_global_settings" => {
            dispatch_db_get_llm_global_settings().await
        }
        "db_save_llm_global_settings" => {
            dispatch_db_save_llm_global_settings(args).await
        }

        // ============================================================================
        // DATA SOURCE COMMANDS
        // ============================================================================
        "db_get_all_data_sources" => {
            dispatch_db_get_all_data_sources().await
        }
        "db_save_data_source" => {
            dispatch_db_save_data_source(args).await
        }
        "db_delete_data_source" => {
            dispatch_db_delete_data_source(args).await
        }

        // ============================================================================
        // PYTHON EXECUTION COMMANDS
        // ============================================================================
        "execute_yfinance_command" => {
            dispatch_execute_python_command("yfinance", args).await
        }
        "execute_polygon_command" => {
            dispatch_execute_python_command("polygon", args).await
        }
        "execute_fred_command" => {
            dispatch_execute_python_command("fred", args).await
        }

        // ============================================================================
        // NEWS COMMANDS
        // ============================================================================
        "fetch_all_rss_news" => {
            dispatch_fetch_rss_news(args).await
        }
        "get_rss_feed_count" => {
            dispatch_get_rss_feed_count().await
        }

        // ============================================================================
        // SETUP & UTILITY COMMANDS
        // ============================================================================
        "check_setup_status" => {
            dispatch_check_setup_status().await
        }
        "sha256_hash" => {
            let input = args.get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            let result = hasher.finalize();
            RpcResponse::ok(format!("{:x}", result))
        }

        // ============================================================================
        // CATCH-ALL FOR UNIMPLEMENTED COMMANDS
        // ============================================================================
        _ => {
            // For commands not yet implemented in web server,
            // return a helpful error message
            RpcResponse::err(format!(
                "Command '{}' is not yet available in web mode. \
                This command may only be available in the desktop application.",
                request.cmd
            ))
        }
    }
}

// ============================================================================
// MARKET DATA DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_market_quote(args: Value) -> RpcResponse {
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };

    // Use the data source directly (without Tauri app handle)
    match crate::data_sources::yfinance::YFinanceProviderWeb::get_quote(&symbol).await {
        Ok(quote) => RpcResponse::ok(quote),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_market_quotes(args: Value) -> RpcResponse {
    let symbols: Vec<String> = match args.get("symbols") {
        Some(v) => serde_json::from_value(v.clone()).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'symbols' parameter"),
    };

    match crate::data_sources::yfinance::YFinanceProviderWeb::get_quotes(&symbols).await {
        Ok(quotes) => RpcResponse::ok(quotes),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_period_returns(args: Value) -> RpcResponse {
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };

    match crate::data_sources::yfinance::YFinanceProviderWeb::get_period_returns(&symbol).await {
        Ok(returns) => RpcResponse::ok(returns),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_market_health() -> RpcResponse {
    match crate::data_sources::yfinance::YFinanceProviderWeb::health_check().await {
        Ok(healthy) => RpcResponse::ok(healthy),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_historical_data(args: Value) -> RpcResponse {
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let start_date = args.get("startDate").or(args.get("start_date"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let end_date = args.get("endDate").or(args.get("end_date"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    match crate::data_sources::yfinance::YFinanceProviderWeb::get_historical(&symbol, &start_date, &end_date).await {
        Ok(data) => RpcResponse::ok(serde_json::json!({
            "success": true,
            "data": data
        })),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_stock_info(args: Value) -> RpcResponse {
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };

    match crate::data_sources::yfinance::YFinanceProviderWeb::get_info(&symbol).await {
        Ok(info) => RpcResponse::ok(info),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_financials(args: Value) -> RpcResponse {
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };

    match crate::data_sources::yfinance::YFinanceProviderWeb::get_financials(&symbol).await {
        Ok(financials) => RpcResponse::ok(financials),
        Err(e) => RpcResponse::err(e),
    }
}

// ============================================================================
// DATABASE DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_health() -> RpcResponse {
    match crate::database::pool::get_pool() {
        Ok(pool) => {
            match pool.get() {
                Ok(_) => RpcResponse::ok(serde_json::json!({
                    "status": "healthy",
                    "message": "Database connection successful"
                })),
                Err(e) => RpcResponse::err(format!("Database connection failed: {}", e)),
            }
        }
        Err(e) => RpcResponse::err(format!("Database pool error: {}", e)),
    }
}

async fn dispatch_db_get_all_settings() -> RpcResponse {
    match crate::database::settings::get_all_settings() {
        Ok(settings) => RpcResponse::ok(settings),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_setting(args: Value) -> RpcResponse {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return RpcResponse::err("Missing 'key' parameter"),
    };

    match crate::database::settings::get_setting(&key) {
        Ok(value) => RpcResponse::ok(value),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_setting(args: Value) -> RpcResponse {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return RpcResponse::err("Missing 'key' parameter"),
    };
    let value = match args.get("value").and_then(|v| v.as_str()) {
        Some(v) => v.to_string(),
        None => return RpcResponse::err("Missing 'value' parameter"),
    };

    match crate::database::settings::save_setting(&key, &value) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_watchlists() -> RpcResponse {
    match crate::database::watchlists::get_watchlists() {
        Ok(watchlists) => RpcResponse::ok(watchlists),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_create_watchlist(args: Value) -> RpcResponse {
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return RpcResponse::err("Missing 'name' parameter"),
    };
    let description = args.get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match crate::database::watchlists::create_watchlist(&name, description.as_deref()) {
        Ok(id) => RpcResponse::ok(serde_json::json!({"id": id})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_watchlist_stocks(args: Value) -> RpcResponse {
    let watchlist_id = match args.get("watchlistId").or(args.get("watchlist_id")) {
        Some(v) => v.as_i64().unwrap_or(0) as i32,
        None => return RpcResponse::err("Missing 'watchlistId' parameter"),
    };

    match crate::database::watchlists::get_watchlist_stocks(watchlist_id) {
        Ok(stocks) => RpcResponse::ok(stocks),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_add_watchlist_stock(args: Value) -> RpcResponse {
    let watchlist_id = match args.get("watchlistId").or(args.get("watchlist_id")) {
        Some(v) => v.as_i64().unwrap_or(0) as i32,
        None => return RpcResponse::err("Missing 'watchlistId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let name = args.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::watchlists::add_watchlist_stock(watchlist_id, &symbol, name.as_deref()) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"added": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_remove_watchlist_stock(args: Value) -> RpcResponse {
    let watchlist_id = match args.get("watchlistId").or(args.get("watchlist_id")) {
        Some(v) => v.as_i64().unwrap_or(0) as i32,
        None => return RpcResponse::err("Missing 'watchlistId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };

    match crate::database::watchlists::remove_watchlist_stock(watchlist_id, &symbol) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"removed": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_watchlist(args: Value) -> RpcResponse {
    let id = match args.get("id") {
        Some(v) => v.as_i64().unwrap_or(0) as i32,
        None => return RpcResponse::err("Missing 'id' parameter"),
    };

    match crate::database::watchlists::delete_watchlist(id) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_credentials() -> RpcResponse {
    match crate::database::credentials::get_all_credentials() {
        Ok(creds) => RpcResponse::ok(creds),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_credential(args: Value) -> RpcResponse {
    let service = match args.get("service").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'service' parameter"),
    };
    let api_key = args.get("apiKey").or(args.get("api_key"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let api_secret = args.get("apiSecret").or(args.get("api_secret"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match crate::database::credentials::save_credential(&service, api_key.as_deref(), api_secret.as_deref()) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_credential(args: Value) -> RpcResponse {
    let service = match args.get("service").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'service' parameter"),
    };

    match crate::database::credentials::delete_credential(&service) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_llm_configs() -> RpcResponse {
    match crate::database::llm::get_llm_configs() {
        Ok(configs) => RpcResponse::ok(configs),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_llm_config(args: Value) -> RpcResponse {
    match crate::database::llm::save_llm_config(args) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_llm_global_settings() -> RpcResponse {
    match crate::database::llm::get_global_settings() {
        Ok(settings) => RpcResponse::ok(settings),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_llm_global_settings(args: Value) -> RpcResponse {
    match crate::database::llm::save_global_settings(args) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_all_data_sources() -> RpcResponse {
    match crate::database::data_sources::get_all_data_sources() {
        Ok(sources) => RpcResponse::ok(sources),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_data_source(args: Value) -> RpcResponse {
    match crate::database::data_sources::save_data_source(args) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_data_source(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };

    match crate::database::data_sources::delete_data_source(&id) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// PYTHON COMMAND DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_execute_python_command(script_type: &str, args: Value) -> RpcResponse {
    let command = match args.get("command").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return RpcResponse::err("Missing 'command' parameter"),
    };
    let params = args.get("params").cloned().unwrap_or(serde_json::json!({}));

    // Execute Python script
    match crate::utils::python::execute_python_script_web(script_type, &command, &params).await {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

// ============================================================================
// NEWS DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_fetch_rss_news(args: Value) -> RpcResponse {
    let sources = args.get("sources")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

    match crate::commands::news::fetch_rss_news_web(&sources, limit).await {
        Ok(news) => RpcResponse::ok(news),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_rss_feed_count() -> RpcResponse {
    RpcResponse::ok(crate::commands::news::get_rss_feed_count_web())
}

// ============================================================================
// SETUP DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_check_setup_status() -> RpcResponse {
    // For web mode, we consider setup complete if the database is initialized
    match crate::database::pool::get_pool() {
        Ok(_) => RpcResponse::ok(serde_json::json!({
            "needs_setup": false,
            "python_installed": true,
            "database_ready": true
        })),
        Err(_) => RpcResponse::ok(serde_json::json!({
            "needs_setup": true,
            "python_installed": false,
            "database_ready": false
        })),
    }
}
