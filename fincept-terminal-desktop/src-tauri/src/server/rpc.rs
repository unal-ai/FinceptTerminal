// RPC Command Dispatcher
// This module maps command names to their handlers, similar to Tauri's invoke_handler.
// It allows reusing all existing command logic without modification.

use super::types::{RpcRequest, RpcResponse, ServerState};
use serde_json::Value;
use std::sync::Arc;

/// Dispatch an RPC request to the appropriate command handler
/// 
/// This function acts as the central router, mapping command names to their
/// implementations. It mirrors the behavior of Tauri's invoke_handler macro.
pub async fn dispatch(state: Arc<ServerState>, request: RpcRequest) -> RpcResponse {
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
        "get_market_quote" => dispatch_market_quote(args).await,
        "get_market_quotes" => dispatch_market_quotes(args).await,
        "get_period_returns" => dispatch_period_returns(args).await,
        "check_market_data_health" => dispatch_market_health().await,
        "get_historical_data" => dispatch_historical_data(args).await,
        "get_stock_info" => dispatch_stock_info(args).await,
        "get_financials" => dispatch_financials(args).await,

        // ============================================================================
        // DATABASE HEALTH & SETTINGS COMMANDS
        // ============================================================================
        "db_check_health" => dispatch_db_health().await,
        "db_get_all_settings" => dispatch_db_get_all_settings().await,
        "db_get_setting" => dispatch_db_get_setting(args).await,
        "db_save_setting" => dispatch_db_save_setting(args).await,

        // ============================================================================
        // CREDENTIALS COMMANDS
        // ============================================================================
        "db_get_credentials" => dispatch_db_get_credentials().await,
        "db_save_credential" => dispatch_db_save_credential(args).await,
        "db_get_credential_by_service" => dispatch_db_get_credential_by_service(args).await,
        "db_delete_credential" => dispatch_db_delete_credential(args).await,

        // ============================================================================
        // LLM CONFIG COMMANDS
        // ============================================================================
        "db_get_llm_configs" => dispatch_db_get_llm_configs().await,
        "db_save_llm_config" => dispatch_db_save_llm_config(args).await,
        "db_get_llm_global_settings" => dispatch_db_get_llm_global_settings().await,
        "db_save_llm_global_settings" => dispatch_db_save_llm_global_settings(args).await,

        // ============================================================================
        // CHAT SESSION COMMANDS
        // ============================================================================
        "db_create_chat_session" => dispatch_db_create_chat_session(args).await,
        "db_get_chat_sessions" => dispatch_db_get_chat_sessions(args).await,
        "db_add_chat_message" => dispatch_db_add_chat_message(args).await,
        "db_get_chat_messages" => dispatch_db_get_chat_messages(args).await,
        "db_delete_chat_session" => dispatch_db_delete_chat_session(args).await,

        // ============================================================================
        // DATA SOURCE COMMANDS
        // ============================================================================
        "db_get_all_data_sources" => dispatch_db_get_all_data_sources().await,
        "db_save_data_source" => dispatch_db_save_data_source(args).await,
        "db_delete_data_source" => dispatch_db_delete_data_source(args).await,

        // ============================================================================
        // PORTFOLIO COMMANDS
        // ============================================================================
        "db_list_portfolios" => dispatch_db_list_portfolios().await,
        "db_get_portfolio" => dispatch_db_get_portfolio(args).await,
        "db_create_portfolio" => dispatch_db_create_portfolio(args).await,
        "db_delete_portfolio" => dispatch_db_delete_portfolio(args).await,

        // ============================================================================
        // WATCHLIST COMMANDS
        // ============================================================================
        "db_get_watchlists" => dispatch_db_get_watchlists().await,
        "db_create_watchlist" => dispatch_db_create_watchlist(args).await,
        "db_get_watchlist_stocks" => dispatch_db_get_watchlist_stocks(args).await,
        "db_add_watchlist_stock" => dispatch_db_add_watchlist_stock(args).await,
        "db_remove_watchlist_stock" => dispatch_db_remove_watchlist_stock(args).await,
        "db_delete_watchlist" => dispatch_db_delete_watchlist(args).await,

        // ============================================================================
        // SETUP & UTILITY COMMANDS
        // ============================================================================
        "check_setup_status" => dispatch_check_setup_status().await,
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
        // WEBSOCKET COMMANDS
        // ============================================================================
        "ws_set_config" => dispatch_ws_set_config(&state.ws_state, args).await,
        "ws_connect" => dispatch_ws_connect(&state.ws_state, args).await,
        "ws_disconnect" => dispatch_ws_disconnect(&state.ws_state, args).await,
        "ws_subscribe" => dispatch_ws_subscribe(&state.ws_state, args).await,
        "ws_unsubscribe" => dispatch_ws_unsubscribe(&state.ws_state, args).await,
        "ws_get_metrics" => dispatch_ws_get_metrics(&state.ws_state, args).await,
        "ws_get_all_metrics" => dispatch_ws_get_all_metrics(&state.ws_state).await,
        "ws_reconnect" => dispatch_ws_reconnect(&state.ws_state, args).await,

        // ============================================================================
        // CATCH-ALL FOR UNIMPLEMENTED COMMANDS
        // ============================================================================
        _ => {
            RpcResponse::err(format!(
                "Command '{}' is not yet available in web mode. \
                See / for API documentation and available commands.",
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
        Ok(data) => RpcResponse::ok(data),
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
// DATABASE HEALTH & SETTINGS DISPATCH FUNCTIONS
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
    match crate::database::operations::get_all_settings() {
        Ok(settings) => RpcResponse::ok(settings),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_setting(args: Value) -> RpcResponse {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return RpcResponse::err("Missing 'key' parameter"),
    };

    match crate::database::operations::get_setting(&key) {
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
    let category = args.get("category").and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::operations::save_setting(&key, &value, category.as_deref()) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// CREDENTIALS DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_get_credentials() -> RpcResponse {
    match crate::database::operations::get_credentials() {
        Ok(creds) => RpcResponse::ok(creds),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_credential(args: Value) -> RpcResponse {
    let cred: crate::database::types::Credential = match serde_json::from_value(args.clone()) {
        Ok(c) => c,
        Err(e) => return RpcResponse::err(format!("Invalid credential data: {}", e)),
    };

    match crate::database::operations::save_credential(&cred) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_credential_by_service(args: Value) -> RpcResponse {
    let service_name = match args.get("serviceName").or(args.get("service_name")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'serviceName' parameter"),
    };

    match crate::database::operations::get_credential_by_service(&service_name) {
        Ok(cred) => RpcResponse::ok(cred),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_credential(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_i64()) {
        Some(i) => i,
        None => return RpcResponse::err("Missing 'id' parameter"),
    };

    match crate::database::operations::delete_credential(id) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// LLM CONFIG DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_get_llm_configs() -> RpcResponse {
    match crate::database::operations::get_llm_configs() {
        Ok(configs) => RpcResponse::ok(configs),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_llm_config(args: Value) -> RpcResponse {
    let config: crate::database::types::LLMConfig = match serde_json::from_value(args.clone()) {
        Ok(c) => c,
        Err(e) => return RpcResponse::err(format!("Invalid LLM config data: {}", e)),
    };

    match crate::database::operations::save_llm_config(&config) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_llm_global_settings() -> RpcResponse {
    match crate::database::operations::get_llm_global_settings() {
        Ok(settings) => RpcResponse::ok(settings),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_llm_global_settings(args: Value) -> RpcResponse {
    let settings: crate::database::types::LLMGlobalSettings = match serde_json::from_value(args.clone()) {
        Ok(s) => s,
        Err(e) => return RpcResponse::err(format!("Invalid LLM global settings: {}", e)),
    };

    match crate::database::operations::save_llm_global_settings(&settings) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"saved": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// CHAT SESSION DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_create_chat_session(args: Value) -> RpcResponse {
    let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("New Chat").to_string();

    match crate::database::operations::create_chat_session(&title) {
        Ok(session) => RpcResponse::ok(session),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_chat_sessions(args: Value) -> RpcResponse {
    let limit = args.get("limit").and_then(|v| v.as_i64());

    match crate::database::operations::get_chat_sessions(limit) {
        Ok(sessions) => RpcResponse::ok(sessions),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_add_chat_message(args: Value) -> RpcResponse {
    let message: crate::database::types::ChatMessage = match serde_json::from_value(args.clone()) {
        Ok(m) => m,
        Err(e) => return RpcResponse::err(format!("Invalid chat message: {}", e)),
    };

    match crate::database::operations::add_chat_message(&message) {
        Ok(msg) => RpcResponse::ok(msg),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_chat_messages(args: Value) -> RpcResponse {
    let session_uuid = match args.get("sessionUuid").or(args.get("session_uuid")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'sessionUuid' parameter"),
    };

    match crate::database::operations::get_chat_messages(&session_uuid) {
        Ok(messages) => RpcResponse::ok(messages),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_chat_session(args: Value) -> RpcResponse {
    let session_uuid = match args.get("sessionUuid").or(args.get("session_uuid")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'sessionUuid' parameter"),
    };

    match crate::database::operations::delete_chat_session(&session_uuid) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// DATA SOURCE DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_get_all_data_sources() -> RpcResponse {
    match crate::database::operations::get_all_data_sources() {
        Ok(sources) => RpcResponse::ok(sources),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_save_data_source(args: Value) -> RpcResponse {
    let source: crate::database::types::DataSource = match serde_json::from_value(args.clone()) {
        Ok(s) => s,
        Err(e) => return RpcResponse::err(format!("Invalid data source: {}", e)),
    };

    match crate::database::operations::save_data_source(&source) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_data_source(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };

    match crate::database::operations::delete_data_source(&id) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// PORTFOLIO DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_list_portfolios() -> RpcResponse {
    match crate::database::operations::get_all_portfolios() {
        Ok(portfolios) => RpcResponse::ok(portfolios),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_portfolio(args: Value) -> RpcResponse {
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };

    match crate::database::operations::get_portfolio_by_id(&portfolio_id) {
        Ok(portfolio) => RpcResponse::ok(portfolio),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_create_portfolio(args: Value) -> RpcResponse {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return RpcResponse::err("Missing 'name' parameter"),
    };
    let owner = args.get("owner").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let currency = args.get("currency").and_then(|v| v.as_str()).unwrap_or("USD").to_string();
    let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::operations::create_portfolio(&id, &name, &owner, &currency, description.as_deref()) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"id": id, "created": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_portfolio(args: Value) -> RpcResponse {
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };

    match crate::database::operations::delete_portfolio(&portfolio_id) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// WATCHLIST DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_get_watchlists() -> RpcResponse {
    match crate::database::queries::get_watchlists() {
        Ok(watchlists) => RpcResponse::ok(watchlists),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_create_watchlist(args: Value) -> RpcResponse {
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return RpcResponse::err("Missing 'name' parameter"),
    };
    let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let color = args.get("color").and_then(|v| v.as_str()).unwrap_or("#3b82f6").to_string();

    match crate::database::queries::create_watchlist(&name, description.as_deref(), &color) {
        Ok(watchlist) => RpcResponse::ok(watchlist),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_watchlist_stocks(args: Value) -> RpcResponse {
    let watchlist_id = match args.get("watchlistId").or(args.get("watchlist_id")).and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'watchlistId' parameter"),
    };

    match crate::database::queries::get_watchlist_stocks(&watchlist_id) {
        Ok(stocks) => RpcResponse::ok(stocks),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_add_watchlist_stock(args: Value) -> RpcResponse {
    let watchlist_id = match args.get("watchlistId").or(args.get("watchlist_id")).and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'watchlistId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let notes = args.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::queries::add_watchlist_stock(&watchlist_id, &symbol, notes.as_deref()) {
        Ok(stock) => RpcResponse::ok(stock),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_remove_watchlist_stock(args: Value) -> RpcResponse {
    let watchlist_id = match args.get("watchlistId").or(args.get("watchlist_id")).and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'watchlistId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };

    match crate::database::queries::remove_watchlist_stock(&watchlist_id, &symbol) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"removed": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_watchlist(args: Value) -> RpcResponse {
    let watchlist_id = match args.get("watchlistId").or(args.get("watchlist_id")).or(args.get("id")).and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'watchlistId' or 'id' parameter"),
    };

    match crate::database::queries::delete_watchlist(&watchlist_id) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

// ============================================================================
// SETUP DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_check_setup_status() -> RpcResponse {
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

// ============================================================================
// WEBSOCKET DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_ws_set_config(state: &crate::WebSocketState, args: Value) -> RpcResponse {
    let config_value = args.get("config").cloned().unwrap_or(args);
    let config: crate::websocket::types::ProviderConfig = match serde_json::from_value(config_value) {
        Ok(config) => config,
        Err(e) => return RpcResponse::err(format!("Invalid config: {}", e)),
    };

    let manager = state.manager.read().await;
    manager.set_config(config);
    RpcResponse::ok(serde_json::json!({"saved": true}))
}

async fn dispatch_ws_connect(state: &crate::WebSocketState, args: Value) -> RpcResponse {
    let provider = match args.get("provider").and_then(|v| v.as_str()) {
        Some(provider) => provider.to_string(),
        None => return RpcResponse::err("Missing 'provider' parameter"),
    };

    let manager = state.manager.read().await;
    match manager.connect(&provider).await {
        Ok(_) => RpcResponse::ok(serde_json::json!({"connected": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_ws_disconnect(state: &crate::WebSocketState, args: Value) -> RpcResponse {
    let provider = match args.get("provider").and_then(|v| v.as_str()) {
        Some(provider) => provider.to_string(),
        None => return RpcResponse::err("Missing 'provider' parameter"),
    };

    let manager = state.manager.read().await;
    match manager.disconnect(&provider).await {
        Ok(_) => RpcResponse::ok(serde_json::json!({"disconnected": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_ws_subscribe(state: &crate::WebSocketState, args: Value) -> RpcResponse {
    let provider = match args.get("provider").and_then(|v| v.as_str()) {
        Some(provider) => provider.to_string(),
        None => return RpcResponse::err("Missing 'provider' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(symbol) => symbol.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let channel = match args.get("channel").and_then(|v| v.as_str()) {
        Some(channel) => channel.to_string(),
        None => return RpcResponse::err("Missing 'channel' parameter"),
    };
    let params = args.get("params").cloned();

    let topic = format!("{}.{}.{}", provider, channel, symbol);
    state.router.write().await.subscribe_frontend(&topic);

    let manager = state.manager.read().await;
    match manager.subscribe(&provider, &symbol, &channel, params).await {
        Ok(_) => RpcResponse::ok(serde_json::json!({"subscribed": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_ws_unsubscribe(state: &crate::WebSocketState, args: Value) -> RpcResponse {
    let provider = match args.get("provider").and_then(|v| v.as_str()) {
        Some(provider) => provider.to_string(),
        None => return RpcResponse::err("Missing 'provider' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(symbol) => symbol.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let channel = match args.get("channel").and_then(|v| v.as_str()) {
        Some(channel) => channel.to_string(),
        None => return RpcResponse::err("Missing 'channel' parameter"),
    };

    state.router.write().await.unsubscribe_frontend(&format!("{}.{}.{}", provider, channel, symbol));

    let manager = state.manager.read().await;
    match manager.unsubscribe(&provider, &symbol, &channel).await {
        Ok(_) => RpcResponse::ok(serde_json::json!({"unsubscribed": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_ws_get_metrics(state: &crate::WebSocketState, args: Value) -> RpcResponse {
    let provider = match args.get("provider").and_then(|v| v.as_str()) {
        Some(provider) => provider.to_string(),
        None => return RpcResponse::err("Missing 'provider' parameter"),
    };

    let manager = state.manager.read().await;
    RpcResponse::ok(manager.get_metrics(&provider))
}

async fn dispatch_ws_get_all_metrics(state: &crate::WebSocketState) -> RpcResponse {
    let manager = state.manager.read().await;
    RpcResponse::ok(manager.get_all_metrics())
}

async fn dispatch_ws_reconnect(state: &crate::WebSocketState, args: Value) -> RpcResponse {
    let provider = match args.get("provider").and_then(|v| v.as_str()) {
        Some(provider) => provider.to_string(),
        None => return RpcResponse::err("Missing 'provider' parameter"),
    };

    let manager = state.manager.read().await;
    match manager.reconnect(&provider).await {
        Ok(_) => RpcResponse::ok(serde_json::json!({"reconnected": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_ws_state() -> crate::WebSocketState {
        let router = Arc::new(tokio::sync::RwLock::new(crate::websocket::MessageRouter::new()));
        let manager = Arc::new(tokio::sync::RwLock::new(crate::websocket::WebSocketManager::new(router.clone())));
        let services = Arc::new(tokio::sync::RwLock::new(crate::WebSocketServices {
            paper_trading: crate::websocket::services::PaperTradingService::new(),
            arbitrage: crate::websocket::services::ArbitrageService::new(),
            portfolio: crate::websocket::services::PortfolioService::new(),
            // Use default() for tests since we're testing RPC parameter validation,
            // not monitoring service functionality. Production code initializes with DB path.
            monitoring: crate::websocket::services::MonitoringService::default(),
        }));
        
        crate::WebSocketState {
            manager,
            router,
            services,
        }
    }

    #[tokio::test]
    async fn test_dispatch_ws_connect_missing_provider() {
        let ws_state = create_test_ws_state();
        let args = serde_json::json!({});
        
        let response = dispatch_ws_connect(&ws_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'provider' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_ws_disconnect_missing_provider() {
        let ws_state = create_test_ws_state();
        let args = serde_json::json!({});
        
        let response = dispatch_ws_disconnect(&ws_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'provider' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_ws_subscribe_missing_parameters() {
        let ws_state = create_test_ws_state();
        
        // Missing provider
        let args = serde_json::json!({"symbol": "BTC/USD", "channel": "ticker"});
        let response = dispatch_ws_subscribe(&ws_state, args).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'provider' parameter");
        
        // Missing symbol
        let args = serde_json::json!({"provider": "binance", "channel": "ticker"});
        let response = dispatch_ws_subscribe(&ws_state, args).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'symbol' parameter");
        
        // Missing channel
        let args = serde_json::json!({"provider": "binance", "symbol": "BTC/USD"});
        let response = dispatch_ws_subscribe(&ws_state, args).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'channel' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_ws_unsubscribe_missing_parameters() {
        let ws_state = create_test_ws_state();
        
        // Missing provider
        let args = serde_json::json!({"symbol": "BTC/USD", "channel": "ticker"});
        let response = dispatch_ws_unsubscribe(&ws_state, args).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'provider' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_ws_get_metrics_missing_provider() {
        let ws_state = create_test_ws_state();
        let args = serde_json::json!({});
        
        let response = dispatch_ws_get_metrics(&ws_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'provider' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_ws_get_all_metrics() {
        let ws_state = create_test_ws_state();
        
        let response = dispatch_ws_get_all_metrics(&ws_state).await;
        
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_dispatch_ws_reconnect_missing_provider() {
        let ws_state = create_test_ws_state();
        let args = serde_json::json!({});
        
        let response = dispatch_ws_reconnect(&ws_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'provider' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_ws_set_config_invalid_config() {
        let ws_state = create_test_ws_state();
        let args = serde_json::json!({"invalid": "data"});
        
        let response = dispatch_ws_set_config(&ws_state, args).await;
        
        // Should return an error for invalid config format
        assert!(response.error.is_some());
    }
}
