use anyhow::{Context, Result};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::ModelConfig;
use crate::parser::config::extract_config_from_sql;
use crate::parser::sql::{extract_tables, parse_sql};

/// Represents a dependency for a model/query
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Dependencies of this model (table names)
    pub deps: HashSet<String>,
    /// File path of the model
    pub filename: String,
    /// Model configuration from SQL comments
    pub config: Option<ModelConfig>,
}

/// Get dependencies for all SQL files in a folder
///
/// # Arguments
///
/// * `folder` - Folder containing SQL files
/// * `dialect` - SQL dialect to use for parsing
///
/// # Returns
///
/// * `HashMap<String, Dependency>` - Map of model names to their dependencies
pub fn get_dependencies(folder: &str, dialect: &str) -> Result<HashMap<String, Dependency>> {
    let mut dependencies = HashMap::new();
    
    for entry in WalkDir::new(folder).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let extension_str = extension.to_string_lossy().to_lowercase();
                
                if extension_str == "sql" {
                    process_sql_file(path, dialect, &mut dependencies)?;
                } else if extension_str == "py" {
                    // Python support would be handled here
                    // For now, we'll skip Python files
                    tracing::info!("Python support not yet implemented, skipping: {}", path.display());
                }
            }
        }
    }
    
    Ok(dependencies)
}

/// Process a SQL file to extract dependencies
fn process_sql_file(path: &Path, dialect: &str, dependencies: &mut HashMap<String, Dependency>) -> Result<()> {
    // Get the model name from the filename (without extension)
    let model_name = path.file_stem()
        .context("Failed to get file stem")?
        .to_string_lossy()
        .to_string();
    
    // Read the SQL file
    let sql = std::fs::read_to_string(path)
        .context(format!("Failed to read SQL file: {}", path.display()))?;
    
    // Extract config from SQL comments
    let config = extract_config_from_sql(&sql)?;
    
    // Parse SQL and extract tables
    let mut deps = HashSet::new();
    for statement in parse_sql(&sql, dialect)? {
        deps.extend(extract_tables(&statement));
    }
    
    // Remove self-dependencies (CTE references)
    deps.remove(&model_name);
    
    // Add dependency to the map
    dependencies.insert(model_name, Dependency {
        deps,
        filename: path.to_string_lossy().to_string(),
        config,
    });
    
    Ok(())
}

/// Get execution order for dependencies using topological sort
///
/// # Arguments
///
/// * `dependencies` - Map of model names to their dependencies
///
/// # Returns
///
/// * `Vec<String>` - Ordered list of model names to execute
pub fn get_execution_order(dependencies: &HashMap<String, Dependency>) -> Result<Vec<String>> {
    // Create a directed graph for topological sorting
    let mut graph = DiGraph::<String, ()>::new();
    let mut node_map = HashMap::new();
    
    // Add nodes for all models
    for name in dependencies.keys() {
        let node_idx = graph.add_node(name.clone());
        node_map.insert(name.clone(), node_idx);
    }
    
    // Add edges for dependencies
    for (name, dependency) in dependencies {
        let from_idx = *node_map.get(name).context("Node not found in graph")?;
        
        for dep in &dependency.deps {
            // If the dependency exists in our models, add an edge
            if let Some(to_idx) = node_map.get(dep) {
                graph.add_edge(*to_idx, from_idx, ());
            }
        }
    }
    
    // Perform topological sort
    let sorted = match toposort(&graph, None) {
        Ok(nodes) => nodes,
        Err(_) => return Err(anyhow::anyhow!("Cycle detected in dependency graph")),
    };
    
    // Map indices back to model names
    let execution_order = sorted.into_iter()
        .map(|idx| graph[idx].clone())
        .collect();
    
    Ok(execution_order)
}