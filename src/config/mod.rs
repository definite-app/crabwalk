mod output;

pub use output::OutputConfig;
pub use output::OutputType;

use serde::{Deserialize, Serialize};

/// Model configuration settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    /// Output configuration for the model
    #[serde(default)]
    pub output: Option<OutputConfig>,
    // Can be extended with additional configuration options
}

/// Command line arguments for the crabwalk CLI
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// Path to the DuckDB database file
    pub database_path: String,
    /// Path to the SQL folder
    pub sql_folder: String,
    /// Schema name in the DuckDB database
    pub schema: String,
    /// Default output type
    pub output_type: OutputType,
    /// Default output location for file outputs
    pub output_location: Option<String>,
    /// Whether to overwrite existing database during restore
    pub overwrite: bool,
}