use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::parser::dependencies::Dependency;

/// Encode a Mermaid diagram string for use in Mermaid Live Editor URL
pub fn encode_mermaid_diagram(diagram: &str) -> Result<String> {
    // Create the state object that Mermaid Live Editor expects
    let state = json!({
        "code": diagram,
        "mermaid": {"theme": "default"},
        "autoSync": true,
        "updateDiagram": true
    });
    
    // Convert to JSON string
    let json_state = serde_json::to_string(&state)?;
    
    // Compress with zlib (similar to pako in JS)
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    std::io::Write::write_all(&mut encoder, json_state.as_bytes())?;
    let compressed = encoder.finish()?;
    
    // Encode to Base64 (URL-safe)
    let encoded = general_purpose::URL_SAFE.encode(&compressed);
    
    Ok(encoded)
}

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
    
    tracing::info!("Generating lineage diagram with {} dependencies", dependencies.len());
    
    // Write diagram header
    writeln!(file, "graph TD")?;
    
    // Write nodes
    for (name, _) in dependencies {
        writeln!(file, "    {}", name)?;
        tracing::info!("Added node: {}", name);
    }
    
    // Write edges
    for (name, dependency) in dependencies {
        tracing::info!("Processing edges for {}", name);
        for dep in &dependency.deps {
            tracing::info!("Checking dependency: {} -> {}", dep, name);
            if dependencies.contains_key(dep) {
                writeln!(file, "    {} --> {}", dep, name)?;
                tracing::info!("Added edge: {} --> {}", dep, name);
            } else {
                tracing::info!("Skipping edge for external dependency: {}", dep);
            }
        }
    }
    
    tracing::info!("Generated lineage diagram at {}", output_path.display());
    
    // Also generate a Mermaid Live Editor URL for easy visualization
    let diagram_contents = std::fs::read_to_string(&output_path)
        .context(format!("Failed to read generated diagram from {}", output_path.display()))?;
    
    // Encode the diagram for use in a Mermaid Live Editor URL
    let encoded_diagram = encode_mermaid_diagram(&diagram_contents)?;
    let mermaid_url = format!("https://mermaid.live/edit#pako:{}", encoded_diagram);
    
    println!("\n🔍 View your lineage diagram online:");
    println!("Mermaid Live Editor URL: {}\n", mermaid_url);
    
    Ok(())
}