use anyhow::{Context, Result};
use regex::Regex;
use crate::config::ModelConfig;

/// Extract model-level configuration from SQL comments with @config directive
///
/// Configuration should be in YAML format:
/// -- @config: {output: {type: "view"}}
///
/// # Arguments
///
/// * `sql` - SQL content with possible @config comments
///
/// # Returns
///
/// * `Result<Option<ModelConfig>>` - Model configuration if present
pub fn extract_config_from_sql(sql: &str) -> Result<Option<ModelConfig>> {
    // Match lines starting with -- @config: followed by any text
    let re = Regex::new(r"^\s*--\s*@config:\s*(.+)$").context("Failed to compile regex")?;
    
    let mut config = ModelConfig::default();
    let mut has_config = false;
    
    for line in sql.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(yaml_text) = captures.get(1) {
                let yaml_str = yaml_text.as_str();
                match serde_yaml::from_str::<ModelConfig>(yaml_str) {
                    Ok(model_config) => {
                        // Merge configs, with later configs potentially overriding earlier ones
                        if let Some(output) = &model_config.output {
                            config.output = Some(output.clone());
                        }
                        has_config = true;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse YAML config: {}", e);
                        // Continue to next line, don't fail the whole function
                    }
                }
            }
        }
    }
    
    if has_config {
        Ok(Some(config))
    } else {
        Ok(None)
    }
}