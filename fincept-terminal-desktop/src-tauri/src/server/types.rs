// RPC Types for Web Server
// These types mirror the JSON-RPC protocol used by Tauri's invoke system

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
use std::time::Instant;

/// RPC Request - mirrors Tauri's invoke pattern
#[derive(Debug, Clone, Deserialize)]
pub struct RpcRequest {
    /// Command name (e.g., "get_market_quote", "get_historical_data")
    pub cmd: String,
    /// Command arguments as JSON value
    #[serde(default)]
    pub args: serde_json::Value,
}

/// RPC Response - standardized response format
#[derive(Debug, Clone, Serialize)]
pub struct RpcResponse {
    /// Whether the command succeeded
    pub success: bool,
    /// Result data (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RpcResponse {
    /// Create a successful response
    pub fn ok<T: Serialize>(data: T) -> Self {
        match serde_json::to_value(data) {
            Ok(data_value) => Self {
                success: true,
                data: Some(data_value),
                error: None,
            },
            Err(e) => {
                // Serialization failed - return error response instead of masking the error
                tracing::error!(
                    error = %e,
                    "RpcResponse::ok: failed to serialize response data"
                );
                Self {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to serialize response: {}", e)),
                }
            }
        }
    }

    /// Create an error response
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to (default: 0.0.0.0)
    pub host: String,
    /// Port to listen on (default: 3000)
    pub port: u16,
    /// Enable CORS for web clients
    pub cors_enabled: bool,
    /// Allowed origins for CORS
    pub cors_origins: Vec<String>,
}

/// Server state shared across handlers
pub struct ServerState {
    pub start_time: Instant,
    pub config: ServerConfig,
    pub request_count: AtomicU64,
    pub ws_state: crate::WebSocketState,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            cors_enabled: true,
            cors_origins: vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:3000".to_string(),
            ],
        }
    }
}
