use anyhow::Result;
use tracing_subscriber::EnvFilter;

/// Main entry point for the crabwalk CLI
fn main() -> Result<()> {
    // Initialize tracing with filter to hide DuckDB success messages
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::new("info,duckdb=error")
        )
        .init();
    
    // Run the CLI
    crabwalk::cli::run()
}
