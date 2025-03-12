use anyhow::{Context, Result};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::ModelConfig;
use crate::parser::config::extract_config_from_sql;
use crate::parser::sql::{extract_tables, parse_sql, extract_columns, extract_column_lineage};

use crate::parser::sql::{ColumnInfo, TableColumnRelationship};

/// Represents a dependency for a model/query
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Dependencies of this model (table names)
    pub deps: HashSet<String>,
    /// File path of the model
    pub filename: String,
    /// Model configuration from SQL comments
    pub config: Option<ModelConfig>,
    /// Column information for this model
    pub columns: Vec<ColumnInfo>,
    /// Column-level lineage relationships
    pub column_lineage: Vec<TableColumnRelationship>,
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
    
    tracing::info!("Looking for SQL files in folder: {}", folder);
    
    let walker = WalkDir::new(folder).follow_links(true);
    let files: Vec<_> = walker.into_iter().filter_map(|e| e.ok()).collect();
    
    tracing::info!("Found {} entries in folder", files.len());
    
    for entry in files {
        let path = entry.path();
        tracing::info!("Examining entry: {}", path.display());
        
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let extension_str = extension.to_string_lossy().to_lowercase();
                tracing::info!("Entry is a file with extension: {}", extension_str);
                
                if extension_str == "sql" {
                    tracing::info!("Processing SQL file: {}", path.display());
                    process_sql_file(path, dialect, &mut dependencies)?;
                } else if extension_str == "py" {
                    // Python support would be handled here
                    // For now, we'll skip Python files
                    tracing::info!("Python support not yet implemented, skipping: {}", path.display());
                }
            } else {
                tracing::info!("File has no extension: {}", path.display());
            }
        } else {
            tracing::info!("Entry is not a file: {}", path.display());
        }
    }
    
    tracing::info!("Dependency processing complete, found {} models", dependencies.len());
    
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
    let statements = parse_sql(&sql, dialect)?;
    
    // Log the number of statements parsed
    tracing::info!("Parsed {} statements from file: {}", statements.len(), path.display());
    
    // For storing column information and lineage
    let mut columns = Vec::new();
    let mut column_lineage = Vec::new();
    
    for statement in &statements {
        // Log the statement type
        tracing::info!("Processing statement: {:?}", statement);
        
        // Extract tables and add to deps
        let tables = extract_tables(statement);
        tracing::info!("Extracted tables: {:?}", tables);
        
        deps.extend(tables);
        
        // Extract column information
        if let Ok(cols) = extract_columns(statement) {
            tracing::info!("Extracted {} columns from statement", cols.len());
            columns.extend(cols);
        }
        
        // Extract column lineage
        if let Ok(lineage) = extract_column_lineage(statement, &model_name) {
            tracing::info!("Extracted {} column lineage relationships", lineage.len());
            column_lineage.extend(lineage);
        }
    }
    
    // Remove self-dependencies (CTE references)
    deps.remove(&model_name);
    
    tracing::info!("Final dependencies for {}: {:?}", model_name, deps);
    tracing::info!("Column count for {}: {}", model_name, columns.len());
    tracing::info!("Column lineage count for {}: {}", model_name, column_lineage.len());
    
    // Add dependency to the map
    dependencies.insert(model_name, Dependency {
        deps,
        filename: path.to_string_lossy().to_string(),
        config,
        columns,
        column_lineage,
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
        tracing::info!("Processing edges for model '{}'", name);
        let from_idx = *node_map.get(name).context("Node not found in graph")?;
        
        for dep in &dependency.deps {
            tracing::info!("Checking dependency: '{}' -> '{}'", dep, name);
            // If the dependency exists in our models, add an edge
            if let Some(to_idx) = node_map.get(dep) {
                tracing::info!("Adding edge: '{}' -> '{}'", dep, name);
                graph.add_edge(*to_idx, from_idx, ());
            } else {
                tracing::info!("Skipping edge for external dependency: '{}'", dep);
            }
        }
    }
    
    // Log graph structure for debugging
    tracing::info!("Dependency graph structure before sorting:");
    for (name, dependency) in dependencies {
        tracing::info!("Model '{}' depends on: {:?}", name, dependency.deps);
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
    
    // Log the final execution order
    tracing::info!("Final execution order after topological sort: {:?}", execution_order);
    
    Ok(execution_order)
}