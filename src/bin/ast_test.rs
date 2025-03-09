use anyhow::Result;
use tracing_subscriber::EnvFilter;
use crabwalk::parser::sql::{parse_sql, extract_tables};
use std::fs;

fn main() -> Result<()> {
    // Initialize tracing with filter to show all debug logs
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::new("debug,duckdb=error")
        )
        .init();
    
    // Get the SQL file from command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <sql_file>", args[0]);
        std::process::exit(1);
    }
    
    let sql_file = &args[1];
    
    // Run the AST test for DuckDB parser
    crabwalk::parser::ast_test::test_duckdb_ast(sql_file)?;
    
    // Additionally, test table extraction
    println!("\nTesting table extraction:");
    let sql_content = fs::read_to_string(sql_file)?;
    
    // Parse the SQL and extract tables
    let statements = parse_sql(&sql_content, "duckdb")?;
    
    // Extract tables from each statement
    for (i, stmt) in statements.iter().enumerate() {
        println!("Extracting tables from statement {}:", i + 1);
        let tables = extract_tables(stmt);
        
        println!("Extracted tables: {:?}", tables);
        if tables.is_empty() {
            println!("WARNING: No tables extracted!");
        }
    }
    
    Ok(())
}