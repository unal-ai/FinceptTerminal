// RPC Command Dispatcher
// This module maps command names to their handlers, similar to Tauri's invoke_handler.
// It allows reusing all existing command logic without modification.

use super::types::{RpcRequest, RpcResponse, ServerState};
use serde_json::Value;
use std::collections::HashMap;
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
        // NEWS COMMANDS
        // ============================================================================
        "fetch_all_rss_news" => dispatch_fetch_all_rss_news().await,
        "get_rss_feed_count" => dispatch_get_rss_feed_count().await,
        "get_active_sources" => dispatch_get_active_sources().await,

        // ============================================================================
        // PYTHON DATA SOURCES
        // ============================================================================
        "execute_polygon_command" => dispatch_execute_polygon_command(args).await,
        "execute_yfinance_command" => dispatch_execute_yfinance_command(args).await,
        "execute_edgar_command" => dispatch_execute_edgar_command(args).await,
        "execute_alphavantage_command" => dispatch_execute_alphavantage_command(args).await,
        "get_alphavantage_quote" => dispatch_get_alphavantage_quote(args).await,
        "get_alphavantage_daily" => dispatch_get_alphavantage_daily(args).await,
        "get_alphavantage_intraday" => dispatch_get_alphavantage_intraday(args).await,
        "get_alphavantage_overview" => dispatch_get_alphavantage_overview(args).await,
        "search_alphavantage_symbols" => dispatch_search_alphavantage_symbols(args).await,
        "get_alphavantage_comprehensive" => dispatch_get_alphavantage_comprehensive(args).await,
        "get_alphavantage_market_movers" => dispatch_get_alphavantage_market_movers().await,

        // ============================================================================
        // PMDARIMA COMMANDS
        // ============================================================================
        "pmdarima_fit_auto_arima" => dispatch_pmdarima_fit_auto_arima(args).await,
        "pmdarima_forecast_auto_arima" => dispatch_pmdarima_forecast_auto_arima(args).await,
        "pmdarima_forecast_arima" => dispatch_pmdarima_forecast_arima(args).await,
        "pmdarima_boxcox_transform" => dispatch_pmdarima_boxcox_transform(args).await,
        "pmdarima_inverse_boxcox" => dispatch_pmdarima_inverse_boxcox(args).await,
        "pmdarima_calculate_acf" => dispatch_pmdarima_calculate_acf(args).await,
        "pmdarima_calculate_pacf" => dispatch_pmdarima_calculate_pacf(args).await,
        "pmdarima_decompose_timeseries" => dispatch_pmdarima_decompose_timeseries(args).await,
        "pmdarima_cross_validate" => dispatch_pmdarima_cross_validate(args).await,

        // ============================================================================
        // GOVERNMENT & MACRO COMMANDS
        // ============================================================================
        "execute_government_us_command" => dispatch_execute_government_us_command(args).await,
        "get_treasury_prices" => dispatch_get_treasury_prices(args).await,
        "get_treasury_auctions" => dispatch_get_treasury_auctions(args).await,
        "get_comprehensive_treasury_data" => dispatch_get_comprehensive_treasury_data(args).await,
        "get_treasury_summary" => dispatch_get_treasury_summary(args).await,
        "execute_congress_gov_command" => dispatch_execute_congress_gov_command(args).await,
        "get_congress_bills" => dispatch_get_congress_bills(args).await,
        "get_bill_info" => dispatch_get_bill_info(args).await,
        "get_bill_text" => dispatch_get_bill_text(args).await,
        "download_bill_text" => dispatch_download_bill_text(args).await,
        "get_comprehensive_bill_data" => dispatch_get_comprehensive_bill_data(args).await,
        "get_bill_summary_by_congress" => dispatch_get_bill_summary_by_congress(args).await,
        "execute_oecd_command" => dispatch_execute_oecd_command(args).await,
        "get_oecd_gdp_real" => dispatch_get_oecd_gdp_real(args).await,
        "get_oecd_consumer_price_index" => dispatch_get_oecd_consumer_price_index(args).await,
        "get_oecd_gdp_forecast" => dispatch_get_oecd_gdp_forecast(args).await,
        "get_oecd_unemployment" => dispatch_get_oecd_unemployment(args).await,
        "get_oecd_economic_summary" => dispatch_get_oecd_economic_summary(args).await,
        "get_oecd_country_list" => dispatch_get_oecd_country_list().await,
        "execute_imf_command" => dispatch_execute_imf_command(args).await,
        "get_imf_economic_indicators" => dispatch_get_imf_economic_indicators(args).await,
        "get_imf_direction_of_trade" => dispatch_get_imf_direction_of_trade(args).await,
        "get_imf_available_indicators" => dispatch_get_imf_available_indicators().await,
        "get_imf_comprehensive_economic_data" => dispatch_get_imf_comprehensive_economic_data(args).await,
        "get_imf_reserves_data" => dispatch_get_imf_reserves_data(args).await,
        "get_imf_trade_summary" => dispatch_get_imf_trade_summary(args).await,

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
        "db_update_portfolio_balance" => dispatch_db_update_portfolio_balance(args).await,

        // ============================================================================
        // PAPER TRADING - POSITIONS
        // ============================================================================
        "db_create_position" => dispatch_db_create_position(args).await,
        "db_get_portfolio_positions" => dispatch_db_get_portfolio_positions(args).await,
        "db_get_position" => dispatch_db_get_position(args).await,
        "db_get_position_by_symbol" => dispatch_db_get_position_by_symbol(args).await,
        "db_get_position_by_symbol_and_side" => dispatch_db_get_position_by_symbol_and_side(args).await,
        "db_update_position" => dispatch_db_update_position(args).await,
        "db_delete_position" => dispatch_db_delete_position(args).await,

        // ============================================================================
        // PAPER TRADING - ORDERS
        // ============================================================================
        "db_create_order" => dispatch_db_create_order(args).await,
        "db_get_order" => dispatch_db_get_order(args).await,
        "db_get_portfolio_orders" => dispatch_db_get_portfolio_orders(args).await,
        "db_get_pending_orders" => dispatch_db_get_pending_orders(args).await,
        "db_update_order" => dispatch_db_update_order(args).await,
        "db_delete_order" => dispatch_db_delete_order(args).await,

        // ============================================================================
        // PAPER TRADING - TRADES
        // ============================================================================
        "db_create_trade" => dispatch_db_create_trade(args).await,
        "db_get_trade" => dispatch_db_get_trade(args).await,
        "db_get_portfolio_trades" => dispatch_db_get_portfolio_trades(args).await,
        "db_get_order_trades" => dispatch_db_get_order_trades(args).await,
        "db_delete_trade" => dispatch_db_delete_trade(args).await,

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
        "cleanup_running_workflows" => dispatch_cleanup_running_workflows().await,
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
        // MONITORING COMMANDS
        // ============================================================================
        "monitor_add_condition" => dispatch_monitor_add_condition(&state.ws_state, args).await,
        "monitor_get_conditions" => dispatch_monitor_get_conditions().await,
        "monitor_delete_condition" => dispatch_monitor_delete_condition(&state.ws_state, args).await,
        "monitor_get_alerts" => dispatch_monitor_get_alerts(args).await,
        "monitor_load_conditions" => dispatch_monitor_load_conditions(&state.ws_state).await,

        // ============================================================================
        // MCP COMMANDS
        // ============================================================================
        "spawn_mcp_server" => dispatch_spawn_mcp_server(&state.mcp_state, args).await,
        "send_mcp_request" => dispatch_send_mcp_request(&state.mcp_state, args).await,
        "send_mcp_notification" => dispatch_send_mcp_notification(&state.mcp_state, args).await,
        "ping_mcp_server" => dispatch_ping_mcp_server(&state.mcp_state, args).await,
        "kill_mcp_server" => dispatch_kill_mcp_server(&state.mcp_state, args).await,

        // ============================================================================
        // CATCH-ALL FOR UNIMPLEMENTED COMMANDS
        // ============================================================================
        _ => {
            if crate::command_registry::is_known_command(request.cmd.as_str()) {
                RpcResponse::err(format!(
                    "Command '{}' is not yet available in web mode. \
                    See / for API documentation and available commands.",
                    request.cmd
                ))
            } else {
                RpcResponse::err(format!(
                    "Command '{}' is not recognized. \
                    See / for API documentation and available commands.",
                    request.cmd
                ))
            }
        }
    }
}

// ============================================================================
// MCP DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_spawn_mcp_server(
    mcp_state: &Arc<crate::MCPState>,
    args: Value,
) -> RpcResponse {
    let server_id = match args.get("serverId").or(args.get("server_id")).and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'serverId' parameter"),
    };
    let command = match args.get("command").and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'command' parameter"),
    };
    let command_args: Vec<String> = match args.get("args") {
        Some(value) => match serde_json::from_value(value.clone()) {
            Ok(v) => v,
            Err(e) => return RpcResponse::err(format!("Invalid 'args' parameter: {}", e)),
        },
        None => Vec::new(),
    };
    let env: HashMap<String, String> = match args.get("env") {
        Some(value) => match serde_json::from_value(value.clone()) {
            Ok(v) => v,
            Err(e) => return RpcResponse::err(format!("Invalid 'env' parameter: {}", e)),
        },
        None => HashMap::new(),
    };

    match crate::spawn_mcp_server_internal(
        None,
        mcp_state.as_ref(),
        server_id,
        command,
        command_args,
        env,
    ) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_send_mcp_request(
    mcp_state: &Arc<crate::MCPState>,
    args: Value,
) -> RpcResponse {
    let server_id = match args.get("serverId").or(args.get("server_id")).and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'serverId' parameter"),
    };
    let request = match args.get("request").and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'request' parameter"),
    };

    match crate::send_mcp_request_internal(mcp_state.as_ref(), server_id, request) {
        Ok(response) => RpcResponse::ok(response),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_send_mcp_notification(
    mcp_state: &Arc<crate::MCPState>,
    args: Value,
) -> RpcResponse {
    let server_id = match args.get("serverId").or(args.get("server_id")).and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'serverId' parameter"),
    };
    let notification = match args.get("notification").and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'notification' parameter"),
    };

    match crate::send_mcp_notification_internal(mcp_state.as_ref(), server_id, notification) {
        Ok(()) => RpcResponse::ok(true),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_ping_mcp_server(
    mcp_state: &Arc<crate::MCPState>,
    args: Value,
) -> RpcResponse {
    let server_id = match args.get("serverId").or(args.get("server_id")).and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'serverId' parameter"),
    };

    match crate::ping_mcp_server_internal(mcp_state.as_ref(), server_id) {
        Ok(is_alive) => RpcResponse::ok(is_alive),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_kill_mcp_server(
    mcp_state: &Arc<crate::MCPState>,
    args: Value,
) -> RpcResponse {
    let server_id = match args.get("serverId").or(args.get("server_id")).and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => return RpcResponse::err("Missing 'serverId' parameter"),
    };

    match crate::kill_mcp_server_internal(mcp_state.as_ref(), server_id) {
        Ok(()) => RpcResponse::ok(true),
        Err(e) => RpcResponse::err(e),
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
// NEWS DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_fetch_all_rss_news() -> RpcResponse {
    match crate::commands::news::fetch_all_rss_news().await {
        Ok(articles) => RpcResponse::ok(articles),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_rss_feed_count() -> RpcResponse {
    RpcResponse::ok(crate::commands::news::get_rss_feed_count())
}

async fn dispatch_get_active_sources() -> RpcResponse {
    RpcResponse::ok(crate::commands::news::get_active_sources())
}

// ============================================================================
// PYTHON SCRIPT DISPATCH HELPERS
// ============================================================================

fn execute_python_script_runtime(script_name: &str, args: Vec<String>) -> Result<String, String> {
    let script_path = crate::utils::python::get_script_path_for_runtime(None, script_name)?;
    crate::python_runtime::execute_python_script(&script_path, args)
}

fn execute_python_command_runtime(
    script_name: &str,
    command: &str,
    args: Vec<String>,
) -> Result<String, String> {
    let mut cmd_args = vec![command.to_string()];
    cmd_args.extend(args);
    execute_python_script_runtime(script_name, cmd_args)
}

fn get_required_string(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing '{}' parameter", key))
}

fn get_optional_string(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn get_optional_i32(args: &Value, key: &str) -> Option<i32> {
    args.get(key).and_then(|v| v.as_i64()).map(|v| v as i32)
}

fn get_optional_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| v.as_bool())
}

fn get_string_list(args: &Value, key: &str) -> Result<Vec<String>, String> {
    match args.get(key) {
        Some(value) => serde_json::from_value(value.clone())
            .map_err(|e| format!("Invalid '{}' parameter: {}", key, e)),
        None => Ok(Vec::new()),
    }
}

// ============================================================================
// PYTHON DATA SOURCE DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_execute_polygon_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };
    if let Some(api_key) = get_optional_string(&args, "apiKey").or_else(|| get_optional_string(&args, "api_key")) {
        std::env::set_var("POLYGON_API_KEY", api_key);
    }

    match execute_python_command_runtime("polygon_data.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_execute_yfinance_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("yfinance_data.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_execute_edgar_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("edgar_tools.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_execute_alphavantage_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("alphavantage_data.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_alphavantage_quote(args: Value) -> RpcResponse {
    let symbol = match get_required_string(&args, "symbol") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("alphavantage_data.py", "quote", vec![symbol]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_alphavantage_daily(args: Value) -> RpcResponse {
    let symbol = match get_required_string(&args, "symbol") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let mut cmd_args = vec![symbol];
    if let Some(outputsize) = get_optional_string(&args, "outputsize") {
        cmd_args.push(outputsize);
    }

    match execute_python_command_runtime("alphavantage_data.py", "daily", cmd_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_alphavantage_intraday(args: Value) -> RpcResponse {
    let symbol = match get_required_string(&args, "symbol") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let mut cmd_args = vec![symbol];
    if let Some(interval) = get_optional_string(&args, "interval") {
        cmd_args.push(interval);
    }

    match execute_python_command_runtime("alphavantage_data.py", "intraday", cmd_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_alphavantage_overview(args: Value) -> RpcResponse {
    let symbol = match get_required_string(&args, "symbol") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("alphavantage_data.py", "overview", vec![symbol]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_search_alphavantage_symbols(args: Value) -> RpcResponse {
    let keywords = match get_required_string(&args, "keywords") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("alphavantage_data.py", "search", vec![keywords]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_alphavantage_comprehensive(args: Value) -> RpcResponse {
    let symbol = match get_required_string(&args, "symbol") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("alphavantage_data.py", "comprehensive", vec![symbol]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_alphavantage_market_movers() -> RpcResponse {
    match execute_python_command_runtime("alphavantage_data.py", "market_movers", Vec::new()) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

// ============================================================================
// PMDARIMA DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_pmdarima_fit_auto_arima(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_fit_auto_arima(
        data,
        get_optional_bool(&args, "seasonal"),
        get_optional_i32(&args, "m"),
        get_optional_i32(&args, "max_p"),
        get_optional_i32(&args, "max_q"),
    )
    .await
    {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_forecast_auto_arima(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };
    let n_periods = match get_optional_i32(&args, "n_periods") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'n_periods' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_forecast_auto_arima(
        data,
        n_periods,
        get_optional_bool(&args, "seasonal"),
        get_optional_bool(&args, "return_conf_int"),
        args.get("alpha").and_then(|v| v.as_f64()),
    )
    .await
    {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_forecast_arima(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };
    let p = match get_optional_i32(&args, "p") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'p' parameter"),
    };
    let d = match get_optional_i32(&args, "d") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'd' parameter"),
    };
    let q = match get_optional_i32(&args, "q") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'q' parameter"),
    };
    let n_periods = match get_optional_i32(&args, "n_periods") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'n_periods' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_forecast_arima(
        data,
        p,
        d,
        q,
        n_periods,
        get_optional_bool(&args, "return_conf_int"),
        args.get("alpha").and_then(|v| v.as_f64()),
    )
    .await
    {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_boxcox_transform(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_boxcox_transform(data).await {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_inverse_boxcox(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };
    let lambda = match args.get("lambda").and_then(|v| v.as_f64()) {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'lambda' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_inverse_boxcox(data, lambda).await {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_calculate_acf(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_calculate_acf(data, get_optional_i32(&args, "nlags")).await {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_calculate_pacf(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_calculate_pacf(data, get_optional_i32(&args, "nlags")).await {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_decompose_timeseries(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };
    let decomp_type = match get_required_string(&args, "decomp_type") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let period = match get_optional_i32(&args, "period") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'period' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_decompose_timeseries(data, decomp_type, period).await {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_pmdarima_cross_validate(args: Value) -> RpcResponse {
    let data: Vec<f64> = match args.get("data").cloned() {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => return RpcResponse::err("Missing 'data' parameter"),
    };
    let p = match get_optional_i32(&args, "p") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'p' parameter"),
    };
    let d = match get_optional_i32(&args, "d") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'd' parameter"),
    };
    let q = match get_optional_i32(&args, "q") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'q' parameter"),
    };
    let cv_splits = match get_optional_i32(&args, "cv_splits") {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'cv_splits' parameter"),
    };

    match crate::commands::pmdarima::pmdarima_cross_validate(data, p, d, q, cv_splits).await {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

// ============================================================================
// GOVERNMENT & MACRO DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_execute_government_us_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("government_us_data.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_treasury_prices(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "target_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "cusip") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "security_type") {
        command_args.push(value);
    }

    match execute_python_command_runtime("government_us_data.py", "treasury_prices", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_treasury_auctions(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "security_type") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_i32(&args, "page_size") {
        command_args.push(value.to_string());
    }
    if let Some(value) = get_optional_i32(&args, "page_num") {
        command_args.push(value.to_string());
    }

    match execute_python_command_runtime("government_us_data.py", "treasury_auctions", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_comprehensive_treasury_data(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "target_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "security_type") {
        command_args.push(value);
    }

    match execute_python_command_runtime("government_us_data.py", "comprehensive", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_treasury_summary(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "target_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("government_us_data.py", "summary", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_execute_congress_gov_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("congress_gov_data.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_congress_bills(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_i32(&args, "congress") {
        command_args.push(value.to_string());
    }
    if let Some(value) = get_optional_string(&args, "bill_type") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_i32(&args, "limit") {
        command_args.push(value.to_string());
    }
    if let Some(value) = get_optional_i32(&args, "offset") {
        command_args.push(value.to_string());
    }
    if let Some(value) = get_optional_string(&args, "sort_by") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_bool(&args, "get_all") {
        command_args.push(value.to_string());
    }

    match execute_python_command_runtime("congress_gov_data.py", "congress_bills", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_bill_info(args: Value) -> RpcResponse {
    let bill_url = match get_required_string(&args, "bill_url") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("congress_gov_data.py", "bill_info", vec![bill_url]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_bill_text(args: Value) -> RpcResponse {
    let bill_url = match get_required_string(&args, "bill_url") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("congress_gov_data.py", "bill_text", vec![bill_url]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_download_bill_text(args: Value) -> RpcResponse {
    let text_url = match get_required_string(&args, "text_url") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("congress_gov_data.py", "download_text", vec![text_url]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_comprehensive_bill_data(args: Value) -> RpcResponse {
    let bill_url = match get_required_string(&args, "bill_url") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("congress_gov_data.py", "comprehensive", vec![bill_url]) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_bill_summary_by_congress(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_i32(&args, "congress") {
        command_args.push(value.to_string());
    }
    if let Some(value) = get_optional_i32(&args, "limit") {
        command_args.push(value.to_string());
    }

    match execute_python_command_runtime("congress_gov_data.py", "summary", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_execute_oecd_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("oecd_data.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_oecd_gdp_real(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "countries") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "frequency") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("oecd_data.py", "gdp_real", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_oecd_consumer_price_index(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "countries") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "expenditure") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "frequency") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "units") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_bool(&args, "harmonized") {
        command_args.push(value.to_string());
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("oecd_data.py", "cpi", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_oecd_gdp_forecast(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "countries") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("oecd_data.py", "gdp_forecast", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_oecd_unemployment(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "countries") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "frequency") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("oecd_data.py", "unemployment", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_oecd_economic_summary(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "country") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("oecd_data.py", "economic_summary", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_oecd_country_list() -> RpcResponse {
    match execute_python_command_runtime("oecd_data.py", "country_list", Vec::new()) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_execute_imf_command(args: Value) -> RpcResponse {
    let command = match get_required_string(&args, "command") {
        Ok(value) => value,
        Err(e) => return RpcResponse::err(e),
    };
    let command_args = match get_string_list(&args, "args") {
        Ok(list) => list,
        Err(e) => return RpcResponse::err(e),
    };

    match execute_python_command_runtime("imf_data.py", &command, command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_imf_economic_indicators(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "country") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "indicator") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("imf_data.py", "economic_indicators", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_imf_direction_of_trade(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "country") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "partner") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("imf_data.py", "direction_of_trade", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_imf_available_indicators() -> RpcResponse {
    match execute_python_command_runtime("imf_data.py", "available_indicators", Vec::new()) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_imf_comprehensive_economic_data(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "country") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("imf_data.py", "comprehensive", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_imf_reserves_data(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "country") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("imf_data.py", "reserves", command_args) {
        Ok(result) => RpcResponse::ok(result),
        Err(e) => RpcResponse::err(e),
    }
}

async fn dispatch_get_imf_trade_summary(args: Value) -> RpcResponse {
    let mut command_args = Vec::new();
    if let Some(value) = get_optional_string(&args, "country") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "start_date") {
        command_args.push(value);
    }
    if let Some(value) = get_optional_string(&args, "end_date") {
        command_args.push(value);
    }

    match execute_python_command_runtime("imf_data.py", "trade_summary", command_args) {
        Ok(result) => RpcResponse::ok(result),
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
    match crate::database::paper_trading::list_portfolios() {
        Ok(portfolios) => RpcResponse::ok(portfolios),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_portfolio(args: Value) -> RpcResponse {
    let portfolio_id = match args.get("id").or(args.get("portfolioId")).or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'id' or 'portfolioId' parameter"),
    };

    match crate::database::paper_trading::get_portfolio(&portfolio_id) {
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
    let provider = match args.get("provider").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return RpcResponse::err("Missing 'provider' parameter"),
    };
    let initial_balance = match args
        .get("initialBalance")
        .or(args.get("initial_balance"))
        .and_then(|v| v.as_f64())
    {
        Some(value) => value,
        None => return RpcResponse::err("Missing 'initialBalance' parameter"),
    };
    let currency = args.get("currency").and_then(|v| v.as_str()).unwrap_or("USD").to_string();
    let margin_mode = args
        .get("marginMode")
        .or(args.get("margin_mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("cross")
        .to_string();
    let leverage = args.get("leverage").and_then(|v| v.as_f64()).unwrap_or(1.0);

    match crate::database::paper_trading::create_portfolio(
        &id,
        &name,
        &provider,
        initial_balance,
        &currency,
        &margin_mode,
        leverage,
    ) {
        Ok(portfolio) => RpcResponse::ok(portfolio),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_portfolio(args: Value) -> RpcResponse {
    let portfolio_id = match args
        .get("id")
        .or(args.get("portfolioId"))
        .or(args.get("portfolio_id"))
        .and_then(|v| v.as_str())
    {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'id' or 'portfolioId' parameter"),
    };

    match crate::database::paper_trading::delete_portfolio(&portfolio_id) {
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
// PAPER TRADING DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_db_update_portfolio_balance(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) => i.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    let new_balance = match args.get("newBalance").or(args.get("new_balance")).and_then(|v| v.as_f64()) {
        Some(b) => b,
        None => return RpcResponse::err("Missing 'newBalance' parameter"),
    };

    match crate::database::paper_trading::update_portfolio_balance(&id, new_balance) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"updated": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_create_position(args: Value) -> RpcResponse {
    let id = args.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let side = match args.get("side").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'side' parameter"),
    };
    let entry_price = match args.get("entryPrice").or(args.get("entry_price")).and_then(|v| v.as_f64()) {
        Some(f) => f,
        None => return RpcResponse::err("Missing 'entryPrice' parameter"),
    };
    let quantity = match args.get("quantity").and_then(|v| v.as_f64()) {
        Some(f) => f,
        None => return RpcResponse::err("Missing 'quantity' parameter"),
    };
    let leverage = args.get("leverage").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let margin_mode = args.get("marginMode").or(args.get("margin_mode")).and_then(|v| v.as_str()).unwrap_or("cross").to_string();

    match crate::database::paper_trading::create_position(&id, &portfolio_id, &symbol, &side, entry_price, quantity, leverage, &margin_mode) {
         Ok(_) => RpcResponse::ok(serde_json::json!({"created": true})),
         Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_portfolio_positions(args: Value) -> RpcResponse {
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let status = args.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::paper_trading::get_portfolio_positions(&portfolio_id, status.as_deref()) {
        Ok(positions) => RpcResponse::ok(positions),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_position(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    match crate::database::paper_trading::get_position(&id) {
        Ok(pos) => RpcResponse::ok(pos),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_position_by_symbol(args: Value) -> RpcResponse {
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let status = args.get("status").and_then(|v| v.as_str()).unwrap_or("open").to_string();

    match crate::database::paper_trading::get_position_by_symbol(&portfolio_id, &symbol, &status) {
        Ok(pos) => RpcResponse::ok(pos),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_position_by_symbol_and_side(args: Value) -> RpcResponse {
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let side = match args.get("side").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'side' parameter"),
    };
    let status = args.get("status").and_then(|v| v.as_str()).unwrap_or("open").to_string();

    match crate::database::paper_trading::get_position_by_symbol_and_side(&portfolio_id, &symbol, &side, &status) {
        Ok(pos) => RpcResponse::ok(pos),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_update_position(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    let quantity = args.get("quantity").and_then(|v| v.as_f64());
    let entry_price = args.get("entryPrice").or(args.get("entry_price")).and_then(|v| v.as_f64());
    let current_price = args.get("currentPrice").or(args.get("current_price")).and_then(|v| v.as_f64());
    let unrealized_pnl = args.get("unrealizedPnl").or(args.get("unrealized_pnl")).and_then(|v| v.as_f64());
    let realized_pnl = args.get("realizedPnl").or(args.get("realized_pnl")).and_then(|v| v.as_f64());
    let liquidation_price = args.get("liquidationPrice").or(args.get("liquidation_price")).and_then(|v| v.as_f64());
    let status = args.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
    let closed_at = args.get("closedAt").or(args.get("closed_at")).and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::paper_trading::update_position(&id, quantity, entry_price, current_price, unrealized_pnl, realized_pnl, liquidation_price, status.as_deref(), closed_at.as_deref()) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"updated": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_position(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    match crate::database::paper_trading::delete_position(&id) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_create_order(args: Value) -> RpcResponse {
     let id = args.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let side = match args.get("side").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'side' parameter"),
    };
    let order_type = match args.get("orderType").or(args.get("order_type")).or(args.get("type")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'orderType' parameter"),
    };
    let quantity = match args.get("quantity").and_then(|v| v.as_f64()) {
        Some(f) => f,
        None => return RpcResponse::err("Missing 'quantity' parameter"),
    };
    let price = args.get("price").and_then(|v| v.as_f64());
    let time_in_force = args.get("timeInForce").or(args.get("time_in_force")).and_then(|v| v.as_str()).unwrap_or("GTC").to_string();

    match crate::database::paper_trading::create_order(&id, &portfolio_id, &symbol, &side, &order_type, quantity, price, &time_in_force) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"created": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_order(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    match crate::database::paper_trading::get_order(&id) {
        Ok(order) => RpcResponse::ok(order),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_portfolio_orders(args: Value) -> RpcResponse {
     let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let status = args.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::paper_trading::get_portfolio_orders(&portfolio_id, status.as_deref()) {
        Ok(orders) => RpcResponse::ok(orders),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_pending_orders(args: Value) -> RpcResponse {
    let portfolio_id = args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()).map(|s| s.to_string());
    match crate::database::paper_trading::get_pending_orders(portfolio_id.as_deref()) {
        Ok(orders) => RpcResponse::ok(orders),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_update_order(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    let filled_quantity = args.get("filledQuantity").or(args.get("filled_quantity")).and_then(|v| v.as_f64());
    let avg_fill_price = args.get("avgFillPrice").or(args.get("avg_fill_price")).and_then(|v| v.as_f64());
    let status = args.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
    let filled_at = args.get("filledAt").or(args.get("filled_at")).and_then(|v| v.as_str()).map(|s| s.to_string());

    match crate::database::paper_trading::update_order(&id, filled_quantity, avg_fill_price, status.as_deref(), filled_at.as_deref()) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"updated": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_order(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    match crate::database::paper_trading::delete_order(&id) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_create_trade(args: Value) -> RpcResponse {
    let id = args.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let order_id = match args.get("orderId").or(args.get("order_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'orderId' parameter"),
    };
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'symbol' parameter"),
    };
    let side = match args.get("side").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'side' parameter"),
    };
    let price = match args.get("price").and_then(|v| v.as_f64()) {
        Some(f) => f,
        None => return RpcResponse::err("Missing 'price' parameter"),
    };
    let quantity = match args.get("quantity").and_then(|v| v.as_f64()) {
        Some(f) => f,
        None => return RpcResponse::err("Missing 'quantity' parameter"),
    };
    let fee = args.get("fee").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let fee_rate = args.get("feeRate").or(args.get("fee_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let is_maker = args.get("isMaker").or(args.get("is_maker")).and_then(|v| v.as_bool()).unwrap_or(false);

    match crate::database::paper_trading::create_trade(&id, &portfolio_id, &order_id, &symbol, &side, price, quantity, fee, fee_rate, is_maker) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"created": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_portfolio_trades(args: Value) -> RpcResponse {
    let portfolio_id = match args.get("portfolioId").or(args.get("portfolio_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'portfolioId' parameter"),
    };
    let limit = args.get("limit").and_then(|v| v.as_i64());

    match crate::database::paper_trading::get_portfolio_trades(&portfolio_id, limit) {
        Ok(trades) => RpcResponse::ok(trades),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_trade(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    match crate::database::paper_trading::get_trade(&id) {
        Ok(trade) => RpcResponse::ok(trade),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_get_order_trades(args: Value) -> RpcResponse {
    let order_id = match args.get("orderId").or(args.get("order_id")).and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'orderId' parameter"),
    };
    match crate::database::paper_trading::get_order_trades(&order_id) {
        Ok(trades) => RpcResponse::ok(trades),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_db_delete_trade(args: Value) -> RpcResponse {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return RpcResponse::err("Missing 'id' parameter"),
    };
    match crate::database::paper_trading::delete_trade(&id) {
        Ok(_) => RpcResponse::ok(serde_json::json!({"deleted": true})),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_cleanup_running_workflows() -> RpcResponse {
    RpcResponse::ok(serde_json::Value::Null)
}

// ============================================================================
// MONITORING DISPATCH FUNCTIONS
// ============================================================================

async fn dispatch_monitor_add_condition(
    state: &crate::WebSocketState,
    args: Value,
) -> RpcResponse {
    use rusqlite::params;

    let condition_value = args.get("condition").cloned().unwrap_or(args);
    let condition: crate::websocket::services::monitoring::MonitorCondition =
        match serde_json::from_value(condition_value) {
            Ok(condition) => condition,
            Err(e) => return RpcResponse::err(format!("Invalid 'condition' parameter: {}", e)),
        };

    let pool = match crate::database::pool::get_pool() {
        Ok(pool) => pool,
        Err(e) => return RpcResponse::err(e.to_string()),
    };
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    if let Err(e) = conn.execute(
        "INSERT INTO monitor_conditions (provider, symbol, field, operator, value, value2, enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &condition.provider,
            &condition.symbol,
            condition.field.as_str(),
            condition.operator.as_str(),
            condition.value,
            condition.value2,
            if condition.enabled { 1 } else { 0 },
        ],
    ) {
        return RpcResponse::err(e.to_string());
    }

    let id = conn.last_insert_rowid();

    let services = state.services.read().await;
    let _ = services.monitoring.load_conditions().await;

    RpcResponse::ok(id)
}

async fn dispatch_monitor_get_conditions() -> RpcResponse {
    let pool = match crate::database::pool::get_pool() {
        Ok(pool) => pool,
        Err(e) => return RpcResponse::err(e.to_string()),
    };
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, provider, symbol, field, operator, value, value2, enabled
         FROM monitor_conditions
         ORDER BY created_at DESC",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    let conditions = match stmt.query_map([], |row| {
        Ok(crate::websocket::services::monitoring::MonitorCondition {
            id: Some(row.get(0)?),
            provider: row.get(1)?,
            symbol: row.get(2)?,
            field: crate::websocket::services::monitoring::MonitorField::from_str(
                &row.get::<_, String>(3)?,
            )
            .unwrap(),
            operator: crate::websocket::services::monitoring::MonitorOperator::from_str(
                &row.get::<_, String>(4)?,
            )
            .unwrap(),
            value: row.get(5)?,
            value2: row.get(6)?,
            enabled: row.get::<_, i32>(7)? == 1,
        })
    }) {
        Ok(rows) => rows.collect::<Result<Vec<_>, _>>(),
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    match conditions {
        Ok(conditions) => RpcResponse::ok(conditions),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_monitor_delete_condition(
    state: &crate::WebSocketState,
    args: Value,
) -> RpcResponse {
    use rusqlite::params;

    let id = match args.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return RpcResponse::err("Missing 'id' parameter"),
    };

    let pool = match crate::database::pool::get_pool() {
        Ok(pool) => pool,
        Err(e) => return RpcResponse::err(e.to_string()),
    };
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    if let Err(e) = conn.execute("DELETE FROM monitor_conditions WHERE id = ?1", params![id]) {
        return RpcResponse::err(e.to_string());
    }

    let services = state.services.read().await;
    if let Err(e) = services.monitoring.load_conditions().await {
        return RpcResponse::err(e.to_string());
    }

    RpcResponse::ok(serde_json::json!({"deleted": true}))
}

async fn dispatch_monitor_get_alerts(args: Value) -> RpcResponse {
    use rusqlite::params;

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);

    let pool = match crate::database::pool::get_pool() {
        Ok(pool) => pool,
        Err(e) => return RpcResponse::err(e.to_string()),
    };
    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, condition_id, provider, symbol, field, triggered_value, triggered_at
         FROM monitor_alerts
         ORDER BY triggered_at DESC
         LIMIT ?1",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    let alerts = match stmt.query_map(params![limit], |row| {
        Ok(crate::websocket::services::monitoring::MonitorAlert {
            id: Some(row.get(0)?),
            condition_id: row.get(1)?,
            provider: row.get(2)?,
            symbol: row.get(3)?,
            field: crate::websocket::services::monitoring::MonitorField::from_str(
                &row.get::<_, String>(4)?,
            )
            .unwrap(),
            triggered_value: row.get(5)?,
            triggered_at: row.get::<_, i64>(6)? as u64,
        })
    }) {
        Ok(rows) => rows.collect::<Result<Vec<_>, _>>(),
        Err(e) => return RpcResponse::err(e.to_string()),
    };

    match alerts {
        Ok(alerts) => RpcResponse::ok(alerts),
        Err(e) => RpcResponse::err(e.to_string()),
    }
}

async fn dispatch_monitor_load_conditions(state: &crate::WebSocketState) -> RpcResponse {
    let services = state.services.read().await;
    match services.monitoring.load_conditions().await {
        Ok(_) => RpcResponse::ok(serde_json::json!({"loaded": true})),
        Err(e) => RpcResponse::err(e.to_string()),
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
        
        // Missing symbol
        let args = serde_json::json!({"provider": "binance", "channel": "ticker"});
        let response = dispatch_ws_unsubscribe(&ws_state, args).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'symbol' parameter");
        
        // Missing channel
        let args = serde_json::json!({"provider": "binance", "symbol": "BTC/USD"});
        let response = dispatch_ws_unsubscribe(&ws_state, args).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'channel' parameter");
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

    // ============================================================================
    // MCP DISPATCH FUNCTION TESTS
    // ============================================================================

    fn create_test_mcp_state() -> Arc<crate::MCPState> {
        Arc::new(crate::MCPState {
            processes: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    #[tokio::test]
    async fn test_dispatch_spawn_mcp_server_missing_server_id() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({"command": "npx"});
        
        let response = dispatch_spawn_mcp_server(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'serverId' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_spawn_mcp_server_missing_command() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({"serverId": "test-server"});
        
        let response = dispatch_spawn_mcp_server(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'command' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_spawn_mcp_server_invalid_args_parameter() {
        let mcp_state = create_test_mcp_state();
        // Pass invalid args - should be an array, not a string
        let args = serde_json::json!({
            "serverId": "test-server",
            "command": "npx",
            "args": "not-an-array"
        });
        
        let response = dispatch_spawn_mcp_server(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        let error_msg = response.error.unwrap();
        assert!(error_msg.contains("Invalid 'args' parameter"));
    }

    #[tokio::test]
    async fn test_dispatch_spawn_mcp_server_invalid_env_parameter() {
        let mcp_state = create_test_mcp_state();
        // Pass invalid env - should be an object/map, not an array
        let args = serde_json::json!({
            "serverId": "test-server",
            "command": "npx",
            "env": ["not", "a", "map"]
        });
        
        let response = dispatch_spawn_mcp_server(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        let error_msg = response.error.unwrap();
        assert!(error_msg.contains("Invalid 'env' parameter"));
    }

    #[tokio::test]
    async fn test_dispatch_spawn_mcp_server_supports_snake_case() {
        let mcp_state = create_test_mcp_state();
        // Test that server_id (snake_case) works in addition to serverId (camelCase)
        let args = serde_json::json!({
            "server_id": "test-server",
            "command": "npx"
        });
        
        let response = dispatch_spawn_mcp_server(&mcp_state, args).await;
        
        // Should not fail due to missing serverId parameter
        // (may fail for other reasons like missing binary, but that's expected)
        if let Some(err) = response.error {
            assert!(!err.contains("Missing 'serverId' parameter"));
        }
    }

    #[tokio::test]
    async fn test_dispatch_send_mcp_request_missing_server_id() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({"request": "test-request"});
        
        let response = dispatch_send_mcp_request(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'serverId' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_send_mcp_request_missing_request() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({"serverId": "test-server"});
        
        let response = dispatch_send_mcp_request(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'request' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_send_mcp_notification_missing_server_id() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({"notification": "test-notification"});
        
        let response = dispatch_send_mcp_notification(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'serverId' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_send_mcp_notification_missing_notification() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({"serverId": "test-server"});
        
        let response = dispatch_send_mcp_notification(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'notification' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_ping_mcp_server_missing_server_id() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({});
        
        let response = dispatch_ping_mcp_server(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'serverId' parameter");
    }

    #[tokio::test]
    async fn test_dispatch_kill_mcp_server_missing_server_id() {
        let mcp_state = create_test_mcp_state();
        let args = serde_json::json!({});
        
        let response = dispatch_kill_mcp_server(&mcp_state, args).await;
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Missing 'serverId' parameter");
    }
}
