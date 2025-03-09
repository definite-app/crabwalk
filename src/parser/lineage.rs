use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::parser::dependencies::Dependency;

/// Generate a Mermaid diagram of the dependencies
///
/// # Arguments
///
/// * `sql_folder` - Folder containing SQL files
/// * `dependencies` - Map of model names to their dependencies
///
/// # Returns
///
/// * `Result<()>` - Success or error
pub fn generate_mermaid_diagram(sql_folder: &str, dependencies: &HashMap<String, Dependency>) -> Result<()> {
    let output_path = Path::new(sql_folder).join("lineage.mmd");
    let mut file = File::create(&output_path)
        .context(format!("Failed to create lineage file: {}", output_path.display()))?;
    
    // Write diagram header
    writeln!(file, "graph TD")?;
    
    // Write nodes
    for (name, _) in dependencies {
        writeln!(file, "    {}", name)?;
    }
    
    // Write edges
    for (name, dependency) in dependencies {
        for dep in &dependency.deps {
            if dependencies.contains_key(dep) {
                writeln!(file, "    {} --> {}", dep, name)?;
            }
        }
    }
    
    tracing::info!("Generated lineage diagram at {}", output_path.display());
    
    Ok(())
}