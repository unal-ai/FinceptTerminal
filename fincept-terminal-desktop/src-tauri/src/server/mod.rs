// Web Server Module for Fincept Terminal
// This module provides an Axum-based HTTP server that exposes the same
// commands as the Tauri desktop app via a JSON-RPC endpoint.
//
// Architecture:
// - Single POST /api/rpc endpoint accepts { cmd: string, args: object }
// - Dispatches to existing command handlers
// - Returns JSON response
//
// This enables running Fincept Terminal as a web service while reusing
// all 930+ existing Rust commands without modification.

pub mod rpc;
pub mod types;

#[cfg(feature = "web")]
pub mod axum_server;

pub use types::*;
