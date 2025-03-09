use anyhow::Result;
use tracing_subscriber::EnvFilter;

/// Main entry point for the crabwalk CLI
fn main() -> Result<()> {
    // Initialize tracing with filter to show info level logs by default
    // Get logging level from environment or use a less verbose default
    let env_filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,sqlparser=warn,duckdb=error".to_string());
        
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(env_filter))
        .init();
    
    // Run the CLI
    crabwalk::cli::run()
}
