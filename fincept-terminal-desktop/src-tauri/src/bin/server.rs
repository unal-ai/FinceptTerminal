// Fincept Terminal Web Server
// This binary starts the Axum HTTP server that exposes Fincept Terminal
// functionality via a JSON-RPC API.
//
// Usage:
//   cargo run --bin fincept-server --features web
//
// Or with release build:
//   cargo build --release --bin fincept-server --features web
//   ./target/release/fincept-server
//
// Environment Variables:
//   FINCEPT_HOST - Server host (default: 0.0.0.0)
//   FINCEPT_PORT - Server port (default: 3000)
//   FINCEPT_PYTHON_PATH - Path to Python executable
//   FINCEPT_SCRIPTS_PATH - Path to Python scripts directory

#[cfg(feature = "web")]
fn main() {
    use fincept_terminal_desktop_lib::server::types::ServerConfig;
    
    // Parse command line args or environment variables
    let host = std::env::var("FINCEPT_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("FINCEPT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    
    let config = ServerConfig {
        host,
        port,
        cors_enabled: true,
        cors_origins: vec!["*".to_string()],
    };
    
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║         FINCEPT TERMINAL WEB SERVER v{}              ║", env!("CARGO_PKG_VERSION"));
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Professional-grade financial analysis platform           ║");
    println!("║  Now accessible via web interface!                        ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    
    // Start the async runtime and server
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime");
    
    rt.block_on(async {
        if let Err(e) = fincept_terminal_desktop_lib::server::axum_server::run_server(config).await {
            eprintln!("❌ Server error: {}", e);
            std::process::exit(1);
        }
    });
}

#[cfg(not(feature = "web"))]
fn main() {
    eprintln!("❌ Error: Web server feature is not enabled.");
    eprintln!();
    eprintln!("To run the web server, build with the 'web' feature:");
    eprintln!("  cargo run --bin fincept-server --features web");
    eprintln!();
    std::process::exit(1);
}
