use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Initialize tracing with filter to hide DuckDB success messages
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::new("info,duckdb=error")
        )
        .init();
    
    // Get the SQL file from command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <sql_file>", args[0]);
        std::process::exit(1);
    }
    
    let sql_file = &args[1];
    
    // Run the AST test
    crabwalk::parser::ast_test::test_duckdb_ast(sql_file)
}