use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::{OutputConfig, OutputType};
use crate::executor::RunContext;

/// Handle different output types based on configuration
///
/// # Arguments
///
/// * `table_name` - Name of the model
/// * `sql_query` - SQL query string
/// * `output_config` - Output configuration
/// * `schema` - Database schema
/// * `context` - RunContext for SQL execution
///
/// # Returns
///
/// * `Result<()>` - Success or error
#[allow(unused_variables)]
pub fn handle_output(
    table_name: &str,
    sql_query: &str,
    output_config: &OutputConfig,
    _schema: &str,
    context: &RunContext,
) -> Result<()> {
    tracing::info!("Handling output for {}, type: {}", table_name, output_config.output_type);
    
    match output_config.output_type {
        OutputType::Table => {
            // Default behavior - create a table
            let create_table_sql = format!("CREATE OR REPLACE TABLE {}.{} AS {}", _schema, table_name, sql_query);
            context.execute(&create_table_sql)?;
        }
        OutputType::View => {
            // Create a view instead of a table
            let create_view_sql = format!("CREATE OR REPLACE VIEW {}.{} AS {}", _schema, table_name, sql_query);
            context.execute(&create_view_sql)?;
        }
        OutputType::Parquet => {
            // Write to a Parquet file
            tracing::info!("Output type is Parquet for {}", table_name);
            handle_file_output(table_name, sql_query, output_config, _schema, context, "parquet")?;
        }
        OutputType::Csv => {
            // Write to a CSV file
            handle_file_output(table_name, sql_query, output_config, _schema, context, "csv")?;
        }
        OutputType::Json => {
            // Write to a JSON file
            handle_file_output(table_name, sql_query, output_config, _schema, context, "json")?;
        }
    }
    
    Ok(())
}

/// Handle file outputs (Parquet, CSV, JSON)
fn handle_file_output(
    table_name: &str,
    sql_query: &str,
    output_config: &OutputConfig,
    _schema: &str,
    context: &RunContext,
    format: &str,
) -> Result<()> {
    // Get location, with fallback to default
    let location = output_config
        .get_location(table_name)
        .unwrap_or_else(|| output_config.default_location(table_name));
    
    tracing::info!("File output location: {}", location);
    
    // Ensure output directory exists
    if let Some(parent) = Path::new(&location).parent() {
        if !parent.exists() {
            tracing::info!("Creating directory: {}", parent.display());
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }
    }
    
    // First create a temporary table
    let temp_table = format!("temp_{}", table_name);
    let create_temp_table_sql = format!("CREATE OR REPLACE TABLE {} AS {}", temp_table, sql_query);
    tracing::info!("Creating temp table with SQL: {}", create_temp_table_sql);
    context.execute(&create_temp_table_sql)?;
    
    // Then export to file
    let format_options = match format {
        "csv" => "(FORMAT CSV, HEADER)",
        "json" => "(FORMAT JSON)",
        "parquet" => "(FORMAT PARQUET)",
        _ => "(FORMAT PARQUET)",
    };
    
    let export_sql = format!("COPY (SELECT * FROM {}) TO '{}' {}", temp_table, location, format_options);
    tracing::info!("Export SQL: {}", export_sql);
    let result = context.execute(&export_sql);
    
    if let Err(ref e) = result {
        tracing::error!("Error exporting data: {}", e);
    }
    
    result?;
    
    // Clean up the temporary table if not keeping it
    if !output_config.keep_table {
        let drop_sql = format!("DROP TABLE IF EXISTS {}", temp_table);
        context.execute(&drop_sql)?;
    }
    
    tracing::info!("Wrote {} file to {}", format, location);
    
    Ok(())
}