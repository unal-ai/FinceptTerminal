// Database Operations - All CRUD operations with optimized queries
// Part 1: Core operations (Settings, Credentials, LLM, Chat, DataSources)

use crate::database::{pool::get_pool, types::*};
use anyhow::Result;
use rusqlite::{params, OptionalExtension};

// ============================================================================
// Settings Operations
// ============================================================================

pub fn save_setting(key: &str, value: &str, category: Option<&str>) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (setting_key, setting_value, category, updated_at)
         VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
        params![key, value, category],
    )?;

    Ok(())
}

pub fn get_setting(key: &str) -> Result<Option<String>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let result = conn
        .query_row(
            "SELECT setting_value FROM settings WHERE setting_key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;

    Ok(result)
}

pub fn get_all_settings() -> Result<Vec<Setting>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare("SELECT setting_key, setting_value, category, updated_at FROM settings")?;
    let settings = stmt
        .query_map([], |row| {
            Ok(Setting {
                setting_key: row.get(0)?,
                setting_value: row.get(1)?,
                category: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(settings)
}

// ============================================================================
// Credentials Operations
// ============================================================================

pub fn save_credential(cred: &Credential) -> Result<OperationResult> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "INSERT OR REPLACE INTO credentials
         (service_name, username, password, api_key, api_secret, additional_data, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)",
        params![
            cred.service_name,
            cred.username,
            cred.password,
            cred.api_key,
            cred.api_secret,
            cred.additional_data,
        ],
    )?;

    Ok(OperationResult {
        success: true,
        message: "Credential saved successfully".to_string(),
    })
}

pub fn get_credentials() -> Result<Vec<Credential>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, service_name, username, password, api_key, api_secret, additional_data, created_at, updated_at
         FROM credentials ORDER BY service_name"
    )?;

    let credentials = stmt
        .query_map([], |row| {
            Ok(Credential {
                id: row.get(0)?,
                service_name: row.get(1)?,
                username: row.get(2)?,
                password: row.get(3)?,
                api_key: row.get(4)?,
                api_secret: row.get(5)?,
                additional_data: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(credentials)
}

pub fn get_credential_by_service(service_name: &str) -> Result<Option<Credential>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let result = conn
        .query_row(
            "SELECT id, service_name, username, password, api_key, api_secret, additional_data, created_at, updated_at
             FROM credentials WHERE service_name = ?1",
            params![service_name],
            |row| {
                Ok(Credential {
                    id: row.get(0)?,
                    service_name: row.get(1)?,
                    username: row.get(2)?,
                    password: row.get(3)?,
                    api_key: row.get(4)?,
                    api_secret: row.get(5)?,
                    additional_data: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .optional()?;

    Ok(result)
}

pub fn delete_credential(id: i64) -> Result<OperationResult> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute("DELETE FROM credentials WHERE id = ?1", params![id])?;

    Ok(OperationResult {
        success: true,
        message: "Credential deleted successfully".to_string(),
    })
}

// ============================================================================
// LLM Config Operations
// ============================================================================

pub fn get_llm_configs() -> Result<Vec<LLMConfig>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT provider, api_key, base_url, model, is_active, created_at, updated_at
         FROM llm_configs"
    )?;

    let configs = stmt
        .query_map([], |row| {
            Ok(LLMConfig {
                provider: row.get(0)?,
                api_key: row.get(1)?,
                base_url: row.get(2)?,
                model: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(configs)
}

pub fn save_llm_config(config: &LLMConfig) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "INSERT OR REPLACE INTO llm_configs (provider, api_key, base_url, model, is_active, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)",
        params![
            config.provider,
            config.api_key,
            config.base_url,
            config.model,
            if config.is_active { 1 } else { 0 },
        ],
    )?;

    Ok(())
}

pub fn get_llm_global_settings() -> Result<LLMGlobalSettings> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let result = conn.query_row(
        "SELECT temperature, max_tokens, system_prompt FROM llm_global_settings WHERE id = 1",
        [],
        |row| {
            Ok(LLMGlobalSettings {
                temperature: row.get(0)?,
                max_tokens: row.get(1)?,
                system_prompt: row.get(2)?,
            })
        },
    )?;

    Ok(result)
}

pub fn save_llm_global_settings(settings: &LLMGlobalSettings) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "UPDATE llm_global_settings SET temperature = ?1, max_tokens = ?2, system_prompt = ?3 WHERE id = 1",
        params![settings.temperature, settings.max_tokens, settings.system_prompt],
    )?;

    Ok(())
}

// ============================================================================
// Chat Operations
// ============================================================================

pub fn create_chat_session(title: &str) -> Result<ChatSession> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let session_uuid = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO chat_sessions (session_uuid, title) VALUES (?1, ?2)",
        params![session_uuid, title],
    )?;

    let session = conn.query_row(
        "SELECT session_uuid, title, message_count, created_at, updated_at
         FROM chat_sessions WHERE session_uuid = ?1",
        params![session_uuid],
        |row| {
            Ok(ChatSession {
                session_uuid: row.get(0)?,
                title: row.get(1)?,
                message_count: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )?;

    Ok(session)
}

pub fn get_chat_sessions(limit: Option<i64>) -> Result<Vec<ChatSession>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let query = if let Some(lim) = limit {
        format!(
            "SELECT session_uuid, title, message_count, created_at, updated_at
             FROM chat_sessions ORDER BY updated_at DESC LIMIT {}",
            lim
        )
    } else {
        "SELECT session_uuid, title, message_count, created_at, updated_at
         FROM chat_sessions ORDER BY updated_at DESC"
            .to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let sessions = stmt
        .query_map([], |row| {
            Ok(ChatSession {
                session_uuid: row.get(0)?,
                title: row.get(1)?,
                message_count: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(sessions)
}

pub fn add_chat_message(msg: &ChatMessage) -> Result<ChatMessage> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "INSERT INTO chat_messages (id, session_uuid, role, content, provider, model, tokens_used)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            msg.id,
            msg.session_uuid,
            msg.role,
            msg.content,
            msg.provider,
            msg.model,
            msg.tokens_used,
        ],
    )?;

    // Update message count
    conn.execute(
        "UPDATE chat_sessions SET message_count = message_count + 1, updated_at = CURRENT_TIMESTAMP
         WHERE session_uuid = ?1",
        params![msg.session_uuid],
    )?;

    let result = conn.query_row(
        "SELECT id, session_uuid, role, content, timestamp, provider, model, tokens_used
         FROM chat_messages WHERE id = ?1",
        params![msg.id],
        |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                session_uuid: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
                provider: row.get(5)?,
                model: row.get(6)?,
                tokens_used: row.get(7)?,
            })
        },
    )?;

    Ok(result)
}

pub fn get_chat_messages(session_uuid: &str) -> Result<Vec<ChatMessage>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, session_uuid, role, content, timestamp, provider, model, tokens_used
         FROM chat_messages WHERE session_uuid = ?1 ORDER BY timestamp ASC"
    )?;

    let messages = stmt
        .query_map(params![session_uuid], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                session_uuid: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
                provider: row.get(5)?,
                model: row.get(6)?,
                tokens_used: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(messages)
}

pub fn delete_chat_session(session_uuid: &str) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "DELETE FROM chat_sessions WHERE session_uuid = ?1",
        params![session_uuid],
    )?;

    Ok(())
}

// ============================================================================
// Data Sources Operations
// ============================================================================

pub fn save_data_source(source: &DataSource) -> Result<OperationResultWithId> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "INSERT OR REPLACE INTO data_sources
         (id, alias, display_name, description, type, provider, category, config, enabled, tags, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)",
        params![
            source.id,
            source.alias,
            source.display_name,
            source.description,
            source.ds_type,
            source.provider,
            source.category,
            source.config,
            if source.enabled { 1 } else { 0 },
            source.tags,
        ],
    )?;

    Ok(OperationResultWithId {
        success: true,
        message: "Data source saved successfully".to_string(),
        id: Some(source.id.clone()),
    })
}

pub fn get_all_data_sources() -> Result<Vec<DataSource>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, alias, display_name, description, type, provider, category, config, enabled, tags, created_at, updated_at
         FROM data_sources ORDER BY display_name"
    )?;

    let sources = stmt
        .query_map([], |row| {
            Ok(DataSource {
                id: row.get(0)?,
                alias: row.get(1)?,
                display_name: row.get(2)?,
                description: row.get(3)?,
                ds_type: row.get(4)?,
                provider: row.get(5)?,
                category: row.get(6)?,
                config: row.get(7)?,
                enabled: row.get::<_, i32>(8)? != 0,
                tags: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(sources)
}

pub fn delete_data_source(id: &str) -> Result<OperationResult> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute("DELETE FROM data_sources WHERE id = ?1", params![id])?;

    Ok(OperationResult {
        success: true,
        message: "Data source deleted successfully".to_string(),
    })
}

// ============================================================================
// WebSocket Provider Config Operations
// ============================================================================

pub fn get_ws_provider_configs() -> Result<Vec<WSProviderConfig>> {
    // what: read all websocket provider configs
    // why: the settings UI needs persisted providers instead of empty stubs
    // how: select every row ordered by provider name and map SQLite booleans to Rust bools
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, provider_name, enabled, api_key, api_secret, endpoint, config_data, created_at, updated_at
         FROM ws_provider_configs ORDER BY provider_name",
    )?;

    let configs = stmt
        .query_map([], |row| {
            Ok(WSProviderConfig {
                id: row.get(0)?,
                provider_name: row.get(1)?,
                enabled: row.get::<_, i32>(2)? != 0,
                api_key: row.get(3)?,
                api_secret: row.get(4)?,
                endpoint: row.get(5)?,
                config_data: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(configs)
}

pub fn get_ws_provider_config(provider_name: &str) -> Result<Option<WSProviderConfig>> {
    // what: fetch a single websocket provider config by its unique name
    // why: connect/disconnect flows require the latest credentials for one provider
    // how: query by provider_name and surface None when not found
    let pool = get_pool()?;
    let conn = pool.get()?;

    let result = conn
        .query_row(
            "SELECT id, provider_name, enabled, api_key, api_secret, endpoint, config_data, created_at, updated_at
             FROM ws_provider_configs WHERE provider_name = ?1",
            params![provider_name],
            |row| {
                Ok(WSProviderConfig {
                    id: row.get(0)?,
                    provider_name: row.get(1)?,
                    enabled: row.get::<_, i32>(2)? != 0,
                    api_key: row.get(3)?,
                    api_secret: row.get(4)?,
                    endpoint: row.get(5)?,
                    config_data: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .optional()?;

    Ok(result)
}

pub fn save_ws_provider_config(config: &WSProviderConfig) -> Result<OperationResult> {
    // what: upsert a websocket provider config keyed by provider_name
    // why: allows the UI to add or edit providers while keeping timestamps accurate
    // how: rely on SQLite's UNIQUE constraint with an ON CONFLICT update and let AUTOINCREMENT handle ids
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "INSERT INTO ws_provider_configs (provider_name, enabled, api_key, api_secret, endpoint, config_data, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
         ON CONFLICT(provider_name) DO UPDATE SET
           enabled = excluded.enabled,
           api_key = excluded.api_key,
           api_secret = excluded.api_secret,
           endpoint = excluded.endpoint,
           config_data = excluded.config_data,
           updated_at = CURRENT_TIMESTAMP",
        params![
            config.provider_name,
            if config.enabled { 1 } else { 0 },
            config.api_key,
            config.api_secret,
            config.endpoint,
            config.config_data,
        ],
    )?;

    Ok(OperationResult {
        success: true,
        message: "WebSocket provider saved successfully".to_string(),
    })
}

pub fn delete_ws_provider_config(provider_name: &str) -> Result<OperationResult> {
    // what: remove a websocket provider config by name
    // why: keeps stale credentials from lingering after the user deletes them
    // how: delete the row scoped by provider_name and return a simple status
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "DELETE FROM ws_provider_configs WHERE provider_name = ?1",
        params![provider_name],
    )?;

    Ok(OperationResult {
        success: true,
        message: "WebSocket provider deleted successfully".to_string(),
    })
}

pub fn toggle_ws_provider_enabled(provider_name: &str) -> Result<ToggleResult> {
    // what: flip the enabled flag for a provider
    // why: mirrors the UI toggle so the backend knows which providers are active
    // how: read the current value, invert it, persist, and return the new state
    let pool = get_pool()?;
    let conn = pool.get()?;

    let current_enabled: bool = conn
        .query_row(
            "SELECT enabled FROM ws_provider_configs WHERE provider_name = ?1",
            params![provider_name],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        )
        .optional()?
        .unwrap_or(false);

    let new_enabled = !current_enabled;

    conn.execute(
        "UPDATE ws_provider_configs SET enabled = ?1, updated_at = CURRENT_TIMESTAMP WHERE provider_name = ?2",
        params![if new_enabled { 1 } else { 0 }, provider_name],
    )?;

    Ok(ToggleResult {
        success: true,
        message: format!(
            "WebSocket provider {}",
            if new_enabled { "enabled" } else { "disabled" }
        ),
        enabled: new_enabled,
    })
}

// ============================================================================
// Portfolio Operations
// ============================================================================

pub fn create_portfolio(id: &str, name: &str, owner: &str, currency: &str, description: Option<&str>) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute(
        "INSERT INTO portfolios (id, name, owner, currency, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        params![id, name, owner, currency, description],
    )?;

    Ok(())
}

pub fn get_all_portfolios() -> Result<Vec<serde_json::Value>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, name, owner, currency, description, created_at, updated_at
         FROM portfolios ORDER BY created_at DESC"
    )?;

    let portfolios = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "owner": row.get::<_, String>(2)?,
                "currency": row.get::<_, String>(3)?,
                "description": row.get::<_, Option<String>>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "updated_at": row.get::<_, String>(6)?
            }))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(portfolios)
}

pub fn get_portfolio_by_id(portfolio_id: &str) -> Result<Option<serde_json::Value>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let result = conn
        .query_row(
            "SELECT id, name, owner, currency, description, created_at, updated_at
             FROM portfolios WHERE id = ?1",
            params![portfolio_id],
            |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "owner": row.get::<_, String>(2)?,
                    "currency": row.get::<_, String>(3)?,
                    "description": row.get::<_, Option<String>>(4)?,
                    "created_at": row.get::<_, String>(5)?,
                    "updated_at": row.get::<_, String>(6)?
                }))
            },
        )
        .optional()?;

    Ok(result)
}

pub fn delete_portfolio(portfolio_id: &str) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    conn.execute("DELETE FROM portfolios WHERE id = ?1", params![portfolio_id])?;

    Ok(())
}

pub fn add_portfolio_asset(
    id: &str,
    portfolio_id: &str,
    symbol: &str,
    quantity: f64,
    price: f64,
) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    // Check if asset exists
    let existing: Option<(String, f64, f64)> = conn
        .query_row(
            "SELECT id, quantity, avg_buy_price FROM portfolio_assets
             WHERE portfolio_id = ?1 AND symbol = ?2",
            params![portfolio_id, symbol],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    if let Some((existing_id, existing_qty, existing_avg_price)) = existing {
        // Update existing asset with weighted average
        let total_qty = existing_qty + quantity;
        let new_avg_price = ((existing_avg_price * existing_qty) + (price * quantity)) / total_qty;

        conn.execute(
            "UPDATE portfolio_assets
             SET quantity = ?1, avg_buy_price = ?2, last_updated = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![total_qty, new_avg_price, existing_id],
        )?;
    } else {
        // Insert new asset
        conn.execute(
            "INSERT INTO portfolio_assets (id, portfolio_id, symbol, quantity, avg_buy_price, first_purchase_date, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![id, portfolio_id, symbol, quantity, price],
        )?;
    }

    Ok(())
}

pub fn sell_portfolio_asset(
    portfolio_id: &str,
    symbol: &str,
    quantity: f64,
) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let existing: Option<(String, f64)> = conn
        .query_row(
            "SELECT id, quantity FROM portfolio_assets
             WHERE portfolio_id = ?1 AND symbol = ?2",
            params![portfolio_id, symbol],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    if let Some((asset_id, existing_qty)) = existing {
        if quantity >= existing_qty {
            // Sell all - delete asset
            conn.execute(
                "DELETE FROM portfolio_assets WHERE id = ?1",
                params![asset_id],
            )?;
        } else {
            // Partial sell - update quantity
            let new_qty = existing_qty - quantity;
            conn.execute(
                "UPDATE portfolio_assets
                 SET quantity = ?1, last_updated = CURRENT_TIMESTAMP
                 WHERE id = ?2",
                params![new_qty, asset_id],
            )?;
        }
    } else {
        return Err(anyhow::anyhow!("Asset not found in portfolio"));
    }

    Ok(())
}

pub fn add_portfolio_transaction(
    id: &str,
    portfolio_id: &str,
    symbol: &str,
    transaction_type: &str,
    quantity: f64,
    price: f64,
    notes: Option<&str>,
) -> Result<()> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let total_value = quantity * price;

    conn.execute(
        "INSERT INTO portfolio_transactions (id, portfolio_id, symbol, transaction_type, quantity, price, total_value, notes, transaction_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
        params![id, portfolio_id, symbol, transaction_type, quantity, price, total_value, notes],
    )?;

    Ok(())
}

pub fn get_portfolio_assets(portfolio_id: &str) -> Result<Vec<serde_json::Value>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT id, portfolio_id, symbol, quantity, avg_buy_price, first_purchase_date, last_updated
         FROM portfolio_assets WHERE portfolio_id = ?1 ORDER BY symbol"
    )?;

    let assets = stmt
        .query_map(params![portfolio_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "portfolio_id": row.get::<_, String>(1)?,
                "symbol": row.get::<_, String>(2)?,
                "quantity": row.get::<_, f64>(3)?,
                "avg_buy_price": row.get::<_, f64>(4)?,
                "first_purchase_date": row.get::<_, String>(5)?,
                "last_updated": row.get::<_, String>(6)?
            }))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(assets)
}

pub fn get_portfolio_transactions(portfolio_id: &str, limit: Option<i32>) -> Result<Vec<serde_json::Value>> {
    let pool = get_pool()?;
    let conn = pool.get()?;

    let query = if let Some(lim) = limit {
        format!(
            "SELECT id, portfolio_id, symbol, transaction_type, quantity, price, total_value, transaction_date, notes
             FROM portfolio_transactions WHERE portfolio_id = ?1 ORDER BY transaction_date DESC LIMIT {}",
            lim
        )
    } else {
        "SELECT id, portfolio_id, symbol, transaction_type, quantity, price, total_value, transaction_date, notes
         FROM portfolio_transactions WHERE portfolio_id = ?1 ORDER BY transaction_date DESC".to_string()
    };

    let mut stmt = conn.prepare(&query)?;

    let transactions = stmt
        .query_map(params![portfolio_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "portfolio_id": row.get::<_, String>(1)?,
                "symbol": row.get::<_, String>(2)?,
                "transaction_type": row.get::<_, String>(3)?,
                "quantity": row.get::<_, f64>(4)?,
                "price": row.get::<_, f64>(5)?,
                "total_value": row.get::<_, f64>(6)?,
                "transaction_date": row.get::<_, String>(7)?,
                "notes": row.get::<_, Option<String>>(8)?
            }))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(transactions)
}
