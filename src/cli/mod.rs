use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;

use crate::config::{OutputType, OutputConfig};

/// Command line arguments for crabwalk
#[derive(Parser, Debug)]
#[command(name = "crabwalk")]
#[command(author = "Crabwalk Team")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// SQL file or directory to process
    #[arg(help = "SQL file or directory to process")]
    path: Option<String>,

    /// Database filename to use
    #[arg(short, long, default_value = "crabwalk.db")]
    database: String,

    /// Schema name to use
    #[arg(short, long, default_value = "transform")]
    schema: String,

    /// Output type (table, view, parquet, csv, json)
    #[arg(short, long, default_value = "table")]
    output: OutputType,

    /// Output directory for exports
    #[arg(short, long)]
    output_dir: Option<String>,

    /// Keep temporary tables when generating files
    #[arg(short, long)]
    keep_tables: bool,
    
    /// Generate lineage diagrams only (no SQL execution)
    #[arg(long)]
    lineage_only: bool,
    
    /// Force execution even with dependency issues
    #[arg(short, long)]
    force: bool,
    
    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the jaffle shop example
    Jaffle {
        /// Output type (table, view, parquet, csv, json)
        #[arg(short, long, default_value = "table")]
        output: OutputType,
        
        /// Force execution even with dependency issues
        #[arg(short, long)]
        force: bool,
    },
    
    /// Run the simple example
    Simple {
        /// Output type (table, view, parquet, csv, json)
        #[arg(short, long, default_value = "table")]
        output: OutputType,
    },
    
    /// Generate instructions for LLMs to create a Crabwalk project
    Init {
        /// Output format for LLM instructions (markdown or json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
}

/// Improved CLI implementation
pub fn run() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Process commands if present
    if let Some(command) = cli.command {
        match command {
            Command::Jaffle { output, force } => {
                let crabwalk = crate::Crabwalk::new(
                    "crabwalk.db".to_string(),
                    "./examples/jaffle_shop".to_string(),
                    "duckdb".to_string(),
                    "transform".to_string(),
                    Some(OutputConfig {
                        output_type: output,
                        location: cli.output_dir,
                        keep_table: cli.keep_tables,
                    }),
                    None,
                );
                println!("Running jaffle shop example...");
                
                if force {
                    println!("Running in force mode (ignoring dependency cycles)...");
                    crabwalk.run_force()?;
                } else {
                    crabwalk.run()?;
                }
                
                println!("Example completed successfully!");
                return Ok(());
            },
            Command::Simple { output } => {
                let crabwalk = crate::Crabwalk::new(
                    "crabwalk.db".to_string(),
                    "./examples/simple".to_string(),
                    "duckdb".to_string(),
                    "transform".to_string(),
                    Some(OutputConfig {
                        output_type: output,
                        location: cli.output_dir,
                        keep_table: cli.keep_tables,
                    }),
                    None,
                );
                println!("Running simple example...");
                crabwalk.run()?;
                println!("Example completed successfully!");
                return Ok(());
            },
            Command::Init { format } => {
                print_llm_instructions(&format);
                return Ok(());
            }
        }
    }
    
    // Get SQL path or use default
    let sql_path = cli.path.unwrap_or_else(|| "./examples/simple".to_string());
    
    tracing::info!("Command arguments: {:?}", std::env::args().collect::<Vec<_>>());
    tracing::info!("Using SQL folder: {}", sql_path);
    
    // Create Crabwalk instance
    let crabwalk = crate::Crabwalk::new(
        cli.database,
        sql_path,
        "duckdb".to_string(),
        cli.schema,
        Some(OutputConfig {
            output_type: cli.output,
            location: cli.output_dir,
            keep_table: cli.keep_tables,
        }),
        None,
    );
    
    // Check if lineage-only mode
    if cli.lineage_only {
        println!("Generating lineage diagrams only...");
        crabwalk.generate_lineage()?;
        return Ok(());
    }
    
    // Force mode
    if cli.force {
        println!("Running in force mode (ignoring dependency issues)...");
        crabwalk.run_force()?;
    } else {
        // Run the transformation
        println!("Running Crabwalk transformation...");
        crabwalk.run()?;
    }
    
    Ok(())
}

/// Print instructions for LLMs to help create a Crabwalk project
fn print_llm_instructions(format: &str) {
    if format == "json" {
        // JSON format for programmatic use
        let instructions = json!({
            "crabwalk_project_structure": {
                "directories": [
                    {
                        "name": "staging",
                        "description": "Contains SQL files for initial data transformation from source tables",
                        "file_pattern": "stg_*.sql"
                    },
                    {
                        "name": "marts",
                        "description": "Contains SQL files for business-level aggregations and transformations",
                        "file_pattern": "*.sql"
                    },
                    {
                        "name": "output",
                        "description": "Default location for exported data files (Parquet, CSV, etc.)",
                        "file_pattern": "*.parquet, *.csv, etc."
                    }
                ],
                "file_types": [
                    {
                        "extension": ".sql",
                        "description": "SQL transformation files using DuckDB syntax",
                        "config_format": "SQL comments with @config: {...} JSON format",
                        "example": "-- @config: {output: {type: \"parquet\", location: \"./output/example.parquet\"}}",
                        "config_schema": {
                            "type": "object",
                            "properties": {
                                "output": {
                                    "type": "object",
                                    "properties": {
                                        "type": {
                                            "type": "string",
                                            "enum": ["table", "view", "parquet", "csv", "json"],
                                            "description": "Output type for the SQL transformation"
                                        },
                                        "location": {
                                            "type": "string",
                                            "description": "Output file path (required for file-based outputs like parquet, csv, json)"
                                        },
                                        "keep_table": {
                                            "type": "boolean",
                                            "description": "Keep temporary tables when generating file outputs",
                                            "default": false
                                        }
                                    },
                                    "required": ["type"]
                                }
                            }
                        }
                    },
                    {
                        "extension": ".mmd",
                        "description": "Auto-generated Mermaid lineage diagrams",
                        "location": "Generated at the root and in each subfolder"
                    }
                ]
            },
            "sql_file_template": {
                "header": "-- @config: {output: {type: \"TABLE|VIEW|PARQUET|CSV|JSON\", location: \"./output/filename.ext\"}}\n-- Description of the transformation",
                "body": "SELECT\n  column1,\n  column2\nFROM source_table\nWHERE condition",
                "dependencies": "Dependencies are automatically extracted from the FROM clause"
            },
            "execution_flow": {
                "dependency_resolution": "Crabwalk automatically detects dependencies between SQL files",
                "execution_order": "SQL files are executed in topological order based on dependencies",
                "output_generation": "Each SQL file can specify its output type in the @config comment"
            },
            "command_examples": [
                {
                    "command": "crabwalk .",
                    "description": "Process all SQL files in the current directory and subdirectories"
                },
                {
                    "command": "crabwalk --output parquet",
                    "description": "Override output type to parquet for all files"
                },
                {
                    "command": "crabwalk --output-dir ./exports",
                    "description": "Set output directory for all exports"
                },
                {
                    "command": "crabwalk --force",
                    "description": "Force execution even with circular dependencies"
                },
                {
                    "command": "crabwalk --lineage-only",
                    "description": "Generate dependency diagrams without executing SQL"
                }
            ]
        });
        
        // Print the JSON instructions
        println!("{}", serde_json::to_string_pretty(&instructions).unwrap());
    } else {
        // Default to Markdown format for human readability
        println!("# Crabwalk Project Guide for LLMs\n");
        println!("## Project Structure\n");
        println!("Create a directory structure with the following components:\n");
        println!("```");
        println!("project_root/");
        println!("├── staging/               # Initial data transformations");
        println!("│   ├── stg_customers.sql  # Example staging model");
        println!("│   └── stg_orders.sql     # Example staging model");
        println!("├── marts/                 # Business-level models");
        println!("│   ├── customer_orders.sql # Example mart model");
        println!("│   └── order_summary.sql   # Example mart model");
        println!("└── output/                # Generated data files");
        println!("    └── order_summary.parquet # Example output file");
        println!("```\n");
        
        println!("## SQL File Structure\n");
        println!("Each SQL file should follow this template:\n");
        println!("```sql");
        println!("-- @config: {{output: {{type: \"parquet\", location: \"./output/example.parquet\"}}}}");
        println!("-- Description of the transformation");
        println!("");
        println!("SELECT");
        println!("  column1,");
        println!("  column2");
        println!("FROM source_table");
        println!("WHERE condition");
        println!("```\n");
        
        println!("## Configuration Options\n");
        println!("The `@config` comment supports the following output types:\n");
        println!("- `table`: Create a table in the database");
        println!("- `view`: Create a view in the database");
        println!("- `parquet`: Export as Parquet file");
        println!("- `csv`: Export as CSV file");
        println!("- `json`: Export as JSON file\n");
        
        println!("## Configuration JSON Schema\n");
        println!("```json");
        println!("{{");
        println!("  \"type\": \"object\",");
        println!("  \"properties\": {{");
        println!("    \"output\": {{");
        println!("      \"type\": \"object\",");
        println!("      \"properties\": {{");
        println!("        \"type\": {{");
        println!("          \"type\": \"string\",");
        println!("          \"enum\": [\"table\", \"view\", \"parquet\", \"csv\", \"json\"],");
        println!("          \"description\": \"Output type for the SQL transformation\"");
        println!("        }},");
        println!("        \"location\": {{");
        println!("          \"type\": \"string\",");
        println!("          \"description\": \"Output file path (required for file-based outputs)\"");
        println!("        }},");
        println!("        \"keep_table\": {{");
        println!("          \"type\": \"boolean\",");
        println!("          \"description\": \"Keep temporary tables when generating file outputs\",");
        println!("          \"default\": false");
        println!("        }}");
        println!("      }},");
        println!("      \"required\": [\"type\"]");
        println!("    }}");
        println!("  }}");
        println!("}}");
        println!("```\n");
        
        println!("## Dependency Management\n");
        println!("- Dependencies are automatically extracted from SQL `FROM` clauses");
        println!("- Files are executed in the correct order based on dependencies");
        println!("- Circular dependencies can be handled with the `--force` flag");
        println!("- Lineage diagrams are automatically generated\n");
        
        println!("## Example SQL Files\n");
        println!("### staging/stg_customers.sql\n");
        println!("```sql");
        println!("-- Basic staging model for customers");
        println!("SELECT 1 as customer_id, 'John Smith' as name, 'john@example.com' as email");
        println!("UNION ALL");
        println!("SELECT 2 as customer_id, 'Jane Doe' as name, 'jane@example.com' as email");
        println!("```\n");
        
        println!("### staging/stg_orders.sql\n");
        println!("```sql");
        println!("-- Basic staging model for orders");
        println!("SELECT 101 as order_id, 1 as customer_id, '2023-01-15' as order_date, 99.99 as amount");
        println!("UNION ALL");
        println!("SELECT 102 as order_id, 1 as customer_id, '2023-03-10' as order_date, 149.99 as amount");
        println!("UNION ALL");
        println!("SELECT 103 as order_id, 2 as customer_id, '2023-02-22' as order_date, 199.99 as amount");
        println!("```\n");
        
        println!("### marts/customer_orders.sql\n");
        println!("```sql");
        println!("-- @config: {{output: {{type: \"view\"}}}}");
        println!("-- Join customers with their orders");
        println!("SELECT");
        println!("  c.customer_id,");
        println!("  c.name as customer_name,");
        println!("  c.email,");
        println!("  o.order_id,");
        println!("  o.order_date,");
        println!("  o.amount");
        println!("FROM stg_customers c");
        println!("JOIN stg_orders o ON c.customer_id = o.customer_id");
        println!("```\n");
        
        println!("### marts/order_summary.sql\n");
        println!("```sql");
        println!("-- @config: {{output: {{type: \"parquet\", location: \"./output/order_summary.parquet\"}}}}");
        println!("-- Create an order summary with aggregate metrics");
        println!("SELECT");
        println!("  customer_id,");
        println!("  COUNT(*) as order_count,");
        println!("  SUM(amount) as total_spent,");
        println!("  MIN(order_date) as first_order_date,");
        println!("  MAX(order_date) as last_order_date,");
        println!("  AVG(amount) as average_order_value");
        println!("FROM stg_orders");
        println!("GROUP BY customer_id");
        println!("```\n");
        
        println!("## Common Commands\n");
        println!("```bash");
        println!("# Run all transformations in the current directory");
        println!("crabwalk .");
        println!("");
        println!("# Override output type to parquet for all files");
        println!("crabwalk --output parquet .");
        println!("");
        println!("# Set output directory for all exports");
        println!("crabwalk --output-dir ./exports .");
        println!("");
        println!("# Force execution even with circular dependencies");
        println!("crabwalk --force .");
        println!("");
        println!("# Generate dependency diagrams without executing SQL");
        println!("crabwalk --lineage-only .");
        println!("```\n");
    }
}