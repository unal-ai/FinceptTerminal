// RPC Command Dispatcher
// This module maps command names to their handlers, similar to Tauri's invoke_handler.
// It allows reusing all existing command logic without modification.

use super::types::{RpcRequest, RpcResponse};
use serde_json::Value;

/// Dispatch an RPC request to the appropriate command handler
/// 
/// This function acts as the central router, mapping command names to their
/// implementations. It mirrors the behavior of Tauri's invoke_handler macro.
pub async fn dispatch(request: RpcRequest) -> RpcResponse {
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
                This command may only be available in the desktop application. \
                Available commands: greet, get_market_quote, get_market_quotes, \
                get_period_returns, check_market_data_health, get_historical_data, \
                get_stock_info, get_financials, db_check_health, check_setup_status, sha256_hash",
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

    // Use the web-compatible data source
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
