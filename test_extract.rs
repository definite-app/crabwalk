use anyhow::Result;
use crabwalk::parser::sql::{parse_sql, extract_tables};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("debug"))
        .init();
    
    // Simple SQL query to test table extraction
    let sql = "SELECT c.name, o.order_id FROM customers c JOIN orders o ON c.id = o.customer_id WHERE o.amount > 100";
    
    // Parse SQL
    println!("Parsing SQL: {}", sql);
    let statements = parse_sql(sql, "duckdb")?;
    
    println!("Found {} statements", statements.len());
    
    // Extract tables from each statement
    for (i, stmt) in statements.iter().enumerate() {
        println!("Statement {}: {:?}", i, stmt);
        let tables = extract_tables(stmt);
        println!("Extracted tables: {:?}", tables);
    }
    
    Ok(())
}