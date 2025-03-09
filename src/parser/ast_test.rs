use anyhow::{Context, Result};
use crate::parser::sql;
use duckdb::Connection;
use std::fs;

/// Test tool for exploring DuckDB's AST output
pub fn test_duckdb_ast(sql_file: &str) -> Result<()> {
    // Read SQL file
    println!("Reading SQL file: {}", sql_file);
    let sql_content = fs::read_to_string(sql_file)?;
    
    // Print DuckDB version information
    let conn = Connection::open_in_memory().context("Failed to open DuckDB connection")?;
    
    // Print DuckDB version
    if let Ok(mut stmt) = conn.prepare("SELECT version()") {
        if let Ok(mut rows) = stmt.query([]) {
            if let Ok(Some(row)) = rows.next() {
                let version: String = row.get(0)?;
                println!("DuckDB version: {}", version);
            }
        }
    }
    
    // Try to install JSON extension
    println!("Attempting to install JSON extension...");
    if let Ok(_) = conn.execute("INSTALL 'json'; LOAD 'json';", []) {
        println!("Successfully installed and loaded JSON extension");
        
        // Try direct test of json_serialize_sql 
        println!("Testing json_serialize_sql with literal SQL...");
        if let Ok(mut stmt) = conn.prepare("SELECT json_serialize_sql('SELECT 1 AS test')") {
            if let Ok(mut rows) = stmt.query([]) {
                if let Ok(Some(row)) = rows.next() {
                    let result: String = row.get(0)?;
                    println!("Direct json_serialize_sql test succeeded");
                    println!("Result: {}", result);
                    
                    // Save the result to a file
                    let output_file = format!("{}_direct_test.json", sql_file);
                    fs::write(&output_file, &result)?;
                    println!("Saved result to: {}", output_file);
                } else {
                    println!("Direct json_serialize_sql test: no results");
                }
            } else {
                println!("Direct json_serialize_sql test query failed");
            }
        } else {
            println!("Direct json_serialize_sql test prepare failed");
        }
    } else {
        println!("Failed to install JSON extension. This function might not be available in your DuckDB version.");
    }
    
    // Try to parse with sqlparser
    println!("\nParsing with sqlparser:");
    match sql::parse_sql(&sql_content, "duckdb") {
        Ok(statements) => {
            println!("Successfully parsed with sqlparser:");
            for (i, stmt) in statements.iter().enumerate() {
                println!("Statement {}: {}", i + 1, stmt);
            }
        },
        Err(e) => {
            println!("Failed with sqlparser: {}", e);
            return Err(e);
        }
    }
    
    println!("\nImplementing DuckDB AST parsing may require a newer version of DuckDB with the json_serialize_sql function.");
    println!("You should be able to see the output format in the examples you shared.");
    
    Ok(())
}