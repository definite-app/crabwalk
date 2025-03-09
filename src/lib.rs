pub mod cli;
pub mod config;
pub mod executor;
pub mod parser;
pub mod storage;

use anyhow::Result;
use parser::dependencies::Dependency;

/// Crabwalk is the main struct for the SQL transformation orchestrator
pub struct Crabwalk {
    /// Path to the DuckDB database file
    database_path: String,
    /// Path to the SQL folder
    sql_folder: String,
    /// SQL dialect (currently only DuckDB is supported)
    dialect: String,
    /// Schema name in the DuckDB database
    schema: String,
    /// Default output configuration
    default_output: config::OutputConfig,
    /// S3 configuration for backup/restore (optional)
    s3_config: Option<storage::S3Config>,
}

impl Crabwalk {
    /// Create a new Crabwalk instance
    pub fn new(
        database_path: String,
        sql_folder: String,
        dialect: String,
        schema: String,
        default_output: Option<config::OutputConfig>,
        s3_config: Option<storage::S3Config>,
    ) -> Self {
        Self {
            database_path,
            sql_folder,
            dialect,
            schema,
            default_output: default_output.unwrap_or_default(),
            s3_config,
        }
    }

    /// Run the transformation pipeline
    pub fn run(&self) -> Result<()> {
        // Initialize tracing for logging
        tracing::info!("Starting Crabwalk transformation pipeline");
        
        // Connect to DuckDB
        let conn = executor::connect_to_duckdb(&self.database_path)?;
        
        // Create context
        let context = executor::RunContext::new(conn);
        
        // Get dependencies
        let dependencies = parser::dependencies::get_dependencies(&self.sql_folder, &self.dialect)?;
        
        // Get execution order
        let execution_order = parser::dependencies::get_execution_order(&dependencies)?;
        
        // Run pre-queries (create schema)
        self.run_pre_queries(&context)?;
        
        // Run objects in order
        self.run_objects(execution_order, &dependencies, &context)?;
        
        // Generate lineage diagram
        parser::lineage::generate_mermaid_diagram(&self.sql_folder, &dependencies)?;
        
        tracing::info!("Crabwalk transformation pipeline completed successfully");
        
        Ok(())
    }
    
    /// Run the transformation pipeline in force mode (ignoring dependency cycles)
    pub fn run_force(&self) -> Result<()> {
        // Initialize tracing for logging
        tracing::info!("Starting Crabwalk transformation pipeline in force mode");
        
        // Connect to DuckDB
        let conn = executor::connect_to_duckdb(&self.database_path)?;
        
        // Create context
        let context = executor::RunContext::new(conn);
        
        // Get dependencies
        let dependencies = parser::dependencies::get_dependencies(&self.sql_folder, &self.dialect)?;
        
        // Run pre-queries (create schema)
        self.run_pre_queries(&context)?;
        
        // In force mode, we run each file directly without worrying about dependencies
        let mut file_count = 0;
        
        // Check if this is a single file or a directory
        if self.sql_folder.ends_with(".sql") {
            // Single file mode
            let file_path = &self.sql_folder;
            let file_name = std::path::Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
                
            tracing::info!("Running single SQL file in force mode: {}", file_path);
            
            // Run the SQL file directly
            match self.run_sql_query(file_path, file_name, &context, None) {
                Ok(_) => {
                    file_count += 1;
                    tracing::info!("Successfully executed: {}", file_path);
                },
                Err(e) => {
                    tracing::error!("Error executing {}: {}", file_path, e);
                }
            }
        } else {
            // Directory mode - process each SQL file
            if let Ok(entries) = std::fs::read_dir(&self.sql_folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sql") {
                        if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                            tracing::info!("Running SQL file in force mode: {}", path.display());
                            
                            // Run the SQL file directly
                            match self.run_sql_query(&path.to_string_lossy(), file_name, &context, None) {
                                Ok(_) => {
                                    file_count += 1;
                                    tracing::info!("Successfully executed: {}", path.display());
                                },
                                Err(e) => {
                                    tracing::error!("Error executing {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Generate lineage diagram if possible
        if let Err(e) = parser::lineage::generate_mermaid_diagram(&self.sql_folder, &dependencies) {
            tracing::warn!("Could not generate lineage diagram: {}", e);
        }
        
        tracing::info!("Crabwalk force mode completed, processed {} files", file_count);
        
        Ok(())
    }
    
    /// Generate lineage diagrams only without executing SQL
    pub fn generate_lineage(&self) -> Result<()> {
        // Get dependencies
        let dependencies = parser::dependencies::get_dependencies(&self.sql_folder, &self.dialect)?;
        
        // Generate lineage diagram
        parser::lineage::generate_mermaid_diagram(&self.sql_folder, &dependencies)?;
        
        tracing::info!("Lineage diagram generation completed");
        
        Ok(())
    }

    /// Run pre-queries to set up the environment
    fn run_pre_queries(&self, context: &executor::RunContext) -> Result<()> {
        // Create schema if it doesn't exist
        context.execute(&format!("CREATE SCHEMA IF NOT EXISTS {}", self.schema))?;
        // Set schema as default
        context.execute(&format!("USE {}", self.schema))?;
        
        Ok(())
    }

    /// Run all objects in the execution order
    fn run_objects(&self, execution_order: Vec<String>, dependencies: &std::collections::HashMap<String, Dependency>, context: &executor::RunContext) -> Result<()> {
        tracing::info!("Running {} objects", execution_order.len());
        tracing::info!("Execution order: {:?}", execution_order);
        
        for object_name in execution_order {
            if let Some(dependency) = dependencies.get(&object_name) {
                let filename = &dependency.filename;
                if filename.ends_with(".sql") {
                    tracing::info!("Running SQL {}", object_name);
                    self.run_sql_query(filename, &object_name, context, dependency.config.as_ref())?;
                    tracing::info!("{} completed", object_name);
                } else if filename.ends_with(".py") {
                    // Python execution will be handled differently in Rust, possibly via subprocess
                    tracing::warn!("Python execution not yet implemented: {}", object_name);
                }
            } else {
                tracing::info!("Identified {} as a source", object_name);
            }
        }
        
        Ok(())
    }

    /// Run a SQL query and handle the output based on configuration
    fn run_sql_query(&self, filename: &str, table_name: &str, context: &executor::RunContext, model_config: Option<&config::ModelConfig>) -> Result<()> {
        // Read SQL file
        let sql = std::fs::read_to_string(filename)?;
        
        // Extract config from SQL comments
        let sql_config = parser::config::extract_config_from_sql(&sql)?;
        
        // Merge configs with precedence: SQL config > model_config > default_output
        let output_config = self.get_output_config(sql_config.as_ref().or(model_config));
        
        // Parse SQL
        let trees = parser::sql::parse_sql(&sql, &self.dialect)?;
        
        tracing::info!("SQL config for {}: {:?}", table_name, sql_config);
        tracing::info!("Merged output config for {}: {:?}", table_name, output_config);
        
        if trees.len() > 1 {
            for tree in trees {
                if parser::sql::is_select_tree(&tree) {
                    // Handle output for SELECT statements
                    executor::output::handle_output(table_name, &tree.to_string(), &output_config, &self.schema, context)?;
                } else {
                    // Execute non-SELECT statements directly
                    context.execute(&tree.to_string())?;
                }
            }
        } else if !trees.is_empty() {
            // Handle output for the single SQL statement
            executor::output::handle_output(table_name, &sql, &output_config, &self.schema, context)?;
        }
        
        Ok(())
    }

    /// Get the output configuration for a model, merging model-specific config with defaults
    fn get_output_config(&self, model_config: Option<&config::ModelConfig>) -> config::OutputConfig {
        let mut output_config = self.default_output.clone();
        
        if let Some(config) = model_config {
            if let Some(ref model_output) = config.output {
                // Update with model-specific output config
                output_config.update_from(model_output);
            }
        }
        
        output_config
    }

    /// Backup the DuckDB database to S3
    pub fn backup(&self) -> Result<()> {
        if let Some(ref s3_config) = self.s3_config {
            tracing::info!("Backing up DuckDB database to S3");
            storage::backup(&self.database_path, s3_config)?;
        } else {
            tracing::warn!("S3 configuration not provided, skipping backup");
        }
        Ok(())
    }

    /// Restore the DuckDB database from S3
    pub fn restore(&self, overwrite: bool) -> Result<()> {
        if let Some(ref s3_config) = self.s3_config {
            tracing::info!("Restoring DuckDB database from S3");
            storage::restore(&self.database_path, s3_config, overwrite)?;
        } else {
            tracing::warn!("S3 configuration not provided, skipping restore");
        }
        Ok(())
    }
}