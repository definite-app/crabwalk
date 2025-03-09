use serde::{Deserialize, Serialize};
use std::fmt;

/// Output type for the model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    /// Create a DuckDB table
    Table,
    /// Create a DuckDB view
    View,
    /// Export to Parquet file
    Parquet,
    /// Export to CSV file
    Csv,
    /// Export to JSON file
    Json,
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::Table
    }
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputType::Table => write!(f, "table"),
            OutputType::View => write!(f, "view"),
            OutputType::Parquet => write!(f, "parquet"),
            OutputType::Csv => write!(f, "csv"),
            OutputType::Json => write!(f, "json"),
        }
    }
}

impl std::str::FromStr for OutputType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(OutputType::Table),
            "view" => Ok(OutputType::View),
            "parquet" => Ok(OutputType::Parquet),
            "csv" => Ok(OutputType::Csv),
            "json" => Ok(OutputType::Json),
            _ => Err(format!("Unknown output type: {}", s)),
        }
    }
}

/// Output configuration for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Type of output (table, view, parquet, csv, json)
    #[serde(default)]
    #[serde(alias = "type")]
    pub output_type: OutputType,
    /// Location for file outputs (parquet, csv, json)
    pub location: Option<String>,
    /// Whether to keep temporary tables for file outputs
    #[serde(default)]
    pub keep_table: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            output_type: OutputType::default(),
            location: None,
            keep_table: false,
        }
    }
}

impl OutputConfig {
    /// Create a new output configuration
    pub fn new(output_type: OutputType, location: Option<String>, keep_table: bool) -> Self {
        Self {
            output_type,
            location,
            keep_table,
        }
    }

    /// Update this config from another one, only changing non-None values
    pub fn update_from(&mut self, other: &OutputConfig) {
        self.output_type = other.output_type.clone();
        if other.location.is_some() {
            self.location = other.location.clone();
        }
        self.keep_table = other.keep_table;
    }

    /// Get the location, replacing {table_name} placeholder if present
    pub fn get_location(&self, table_name: &str) -> Option<String> {
        self.location.as_ref().map(|loc| loc.replace("{table_name}", table_name))
    }

    /// Get default location for a given output type and table name
    pub fn default_location(&self, table_name: &str) -> String {
        match self.output_type {
            OutputType::Parquet => format!("./output/{}.parquet", table_name),
            OutputType::Csv => format!("./output/{}.csv", table_name),
            OutputType::Json => format!("./output/{}.json", table_name),
            _ => String::new(),
        }
    }
}