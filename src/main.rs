use anyhow::Result;

/// Main entry point for the crabwalk CLI
fn main() -> Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    // Run the CLI
    crabwalk::cli::run()
}
