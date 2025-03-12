use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::io::Write;

use crate::parser::dependencies::Dependency;

/// Generate a visualization of the database schema
///
/// # Arguments
///
/// * `dependencies` - Map of model names to their dependencies
/// * `format` - Output format (html, svg, png)
/// * `output_path` - Path to save the visualization
/// * `include_columns` - Whether to include column-level details
///
/// # Returns
///
/// * `Result<()>` - Success or error
pub fn visualize_database_schema(
    dependencies: &HashMap<String, Dependency>,
    format: &str,
    output_path: Option<&str>,
    include_columns: bool,
) -> Result<()> {
    // Determine output path
    let actual_output_path = output_path.unwrap_or("crabwalk_schema.html");
    
    match format.to_lowercase().as_str() {
        "html" => generate_html_visualization(dependencies, actual_output_path, include_columns)?,
        "svg" => {
            // For now, we'll generate HTML and embed instructions to save as SVG
            generate_html_visualization(dependencies, actual_output_path, include_columns)?;
            println!("To save as SVG: Open the HTML file in a browser and use the browser's save as SVG option");
        },
        "png" => {
            // For now, we'll generate HTML and embed instructions to save as PNG
            generate_html_visualization(dependencies, actual_output_path, include_columns)?;
            println!("To save as PNG: Open the HTML file in a browser and use the browser's save as PNG option");
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported visualization format: {}", format));
        }
    }
    
    Ok(())
}

/// Generate an HTML visualization of the database schema
fn generate_html_visualization(
    dependencies: &HashMap<String, Dependency>,
    output_path: &str,
    include_columns: bool,
) -> Result<()> {
    // Create base HTML structure
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("  <title>Crabwalk Database Schema</title>\n");
    
    // Add CSS styles
    html.push_str("  <style>\n");
    html.push_str("    body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("    h1 { color: #333; }\n");
    html.push_str("    .schema-container { margin-top: 20px; }\n");
    html.push_str("    .table { border: 1px solid #ccc; margin-bottom: 20px; border-radius: 5px; overflow: hidden; }\n");
    html.push_str("    .table-header { background-color: #f0f0f0; padding: 10px; border-bottom: 1px solid #ccc; font-weight: bold; }\n");
    html.push_str("    .table-body { padding: 10px; }\n");
    html.push_str("    .column { margin-bottom: 5px; padding: 5px; border-bottom: 1px solid #eee; }\n");
    html.push_str("    .column-name { font-weight: bold; }\n");
    html.push_str("    .column-type { color: #666; margin-left: 10px; }\n");
    html.push_str("    .column-source { color: #888; font-size: 0.9em; margin-top: 3px; }\n");
    html.push_str("    .dependencies { margin-top: 10px; color: #555; }\n");
    html.push_str("    .lineage-container { margin-top: 30px; }\n");
    html.push_str("    .lineage-title { color: #333; margin-bottom: 10px; }\n");
    html.push_str("    .lineage-diagram { border: 1px solid #ddd; padding: 20px; background-color: #f9f9f9; }\n");
    html.push_str("    #schema-viz { width: 100%; height: 600px; border: 1px solid #ccc; }\n");
    html.push_str("    .controls { margin-bottom: 20px; }\n");
    html.push_str("    button { padding: 5px 10px; margin-right: 10px; }\n");
    html.push_str("  </style>\n");
    
    // Add Mermaid.js for diagram visualization
    html.push_str("  <script src=\"https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js\"></script>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str("  <h1>Crabwalk Database Schema</h1>\n");
    
    // Add controls
    html.push_str("  <div class=\"controls\">\n");
    html.push_str("    <button onclick=\"saveAsSVG()\">Save as SVG</button>\n");
    html.push_str("    <button onclick=\"saveAsPNG()\">Save as PNG</button>\n");
    html.push_str("    <button onclick=\"toggleView('tables')\">Show Tables</button>\n");
    html.push_str("    <button onclick=\"toggleView('diagram')\">Show Diagram</button>\n");
    html.push_str("  </div>\n");
    
    // Add tables section
    html.push_str("  <div id=\"tables-view\" class=\"schema-container\">\n");
    
    // Sort dependencies by name for consistent output
    let mut sorted_deps: Vec<(&String, &Dependency)> = dependencies.iter().collect();
    sorted_deps.sort_by_key(|a| a.0);
    
    // Generate table details
    for (table_name, dependency) in &sorted_deps {
        html.push_str(&format!("    <div class=\"table\">\n"));
        html.push_str(&format!("      <div class=\"table-header\">{}</div>\n", table_name));
        html.push_str(&format!("      <div class=\"table-body\">\n"));
        
        // Add columns if requested
        if include_columns {
            if dependency.columns.is_empty() {
                html.push_str("        <div class=\"column\">\n");
                html.push_str("          <span class=\"column-name\">id</span>\n");
                html.push_str("          <span class=\"column-type\">(unknown)</span>\n");
                html.push_str("          <div class=\"column-source\">Primary key (automatically inferred)</div>\n");
                html.push_str("        </div>\n");
            } else {
                for column in &dependency.columns {
                    html.push_str("        <div class=\"column\">\n");
                    html.push_str(&format!("          <span class=\"column-name\">{}</span>\n", column.name));
                    html.push_str(&format!("          <span class=\"column-type\">({})</span>\n", column.data_type));
                    
                    // Add source information if available
                    if let (Some(table), Some(col)) = (&column.source_table, &column.source_column) {
                        html.push_str(&format!("          <div class=\"column-source\">From {}.{}</div>\n", table, col));
                    } else if column.is_derived {
                        html.push_str("          <div class=\"column-source\">Derived from expression</div>\n");
                    }
                    
                    html.push_str("        </div>\n");
                }
            }
        }
        
        // Add dependencies
        if !dependency.deps.is_empty() {
            html.push_str("        <div class=\"dependencies\">\n");
            html.push_str("          <strong>Dependencies:</strong> ");
            
            let deps_list: Vec<_> = dependency.deps.iter().cloned().collect();
            html.push_str(&format!("{}\n", deps_list.join(", ")));
            
            html.push_str("        </div>\n");
        }
        
        html.push_str("      </div>\n"); // Close table-body
        html.push_str("    </div>\n"); // Close table
    }
    
    html.push_str("  </div>\n"); // Close tables-view
    
    // Add diagram section
    html.push_str("  <div id=\"diagram-view\" class=\"lineage-container\" style=\"display:none;\">\n");
    html.push_str("    <h2 class=\"lineage-title\">Database Schema Diagram</h2>\n");
    html.push_str("    <div class=\"lineage-diagram\">\n");
    html.push_str("      <div class=\"mermaid\" id=\"schema-viz\">\n");
    
    // Generate Mermaid diagram
    html.push_str("erDiagram\n");
    
    // Add entities (tables)
    for (table_name, dependency) in &sorted_deps {
        // Start table definition
        html.push_str(&format!("    {} {{\n", table_name));
        
        // Add columns
        if dependency.columns.is_empty() {
            html.push_str("        int id PK \"Primary key\"\n");
        } else {
            for (i, column) in dependency.columns.iter().enumerate() {
                let pk_marker = if i == 0 { "PK" } else { "" };
                html.push_str(&format!("        {} {} {} \"{}\"\n", 
                    column.data_type, 
                    column.name,
                    pk_marker,
                    if column.is_derived { "Derived" } else { "Column" }
                ));
            }
        }
        
        // End table definition
        html.push_str("    }\n");
    }
    
    // Add relationships
    for (table_name, dependency) in &sorted_deps {
        for dep in &dependency.deps {
            if dependencies.contains_key(dep) {
                html.push_str(&format!("    {} }}|--|| {} : depends_on\n", dep, table_name));
            }
        }
        
        // Add column-level relationships if requested
        if include_columns && !dependency.column_lineage.is_empty() {
            for lineage in &dependency.column_lineage {
                // We only add relationships to tables we know about
                if dependencies.contains_key(&lineage.source_table) {
                    html.push_str(&format!("    {} ||--|| {} : \"{} -> {}\"\n", 
                        lineage.source_table, 
                        table_name,
                        lineage.source_column,
                        lineage.target_column
                    ));
                }
            }
        }
    }
    
    html.push_str("      </div>\n"); // Close mermaid
    html.push_str("    </div>\n"); // Close lineage-diagram
    html.push_str("  </div>\n"); // Close diagram-view
    
    // Add JavaScript functions
    html.push_str("<script>\n");
    html.push_str("  // Initialize Mermaid\n");
    html.push_str("  mermaid.initialize({ startOnLoad: true, theme: 'default' });\n");
    
    // Function to toggle views
    html.push_str("  function toggleView(view) {\n");
    html.push_str("    if (view === 'tables') {\n");
    html.push_str("      document.getElementById('tables-view').style.display = 'block';\n");
    html.push_str("      document.getElementById('diagram-view').style.display = 'none';\n");
    html.push_str("    } else {\n");
    html.push_str("      document.getElementById('tables-view').style.display = 'none';\n");
    html.push_str("      document.getElementById('diagram-view').style.display = 'block';\n");
    html.push_str("    }\n");
    html.push_str("  }\n");
    
    // Function to save as SVG
    html.push_str("  function saveAsSVG() {\n");
    html.push_str("    const svgEl = document.querySelector('#schema-viz svg');\n");
    html.push_str("    if (svgEl) {\n");
    html.push_str("      const svgData = new XMLSerializer().serializeToString(svgEl);\n");
    html.push_str("      const svgBlob = new Blob([svgData], {type: 'image/svg+xml;charset=utf-8'});\n");
    html.push_str("      const svgUrl = URL.createObjectURL(svgBlob);\n");
    html.push_str("      const downloadLink = document.createElement('a');\n");
    html.push_str("      downloadLink.href = svgUrl;\n");
    html.push_str("      downloadLink.download = 'crabwalk_schema.svg';\n");
    html.push_str("      document.body.appendChild(downloadLink);\n");
    html.push_str("      downloadLink.click();\n");
    html.push_str("      document.body.removeChild(downloadLink);\n");
    html.push_str("    } else {\n");
    html.push_str("      alert('Please view the diagram first');\n");
    html.push_str("      toggleView('diagram');\n");
    html.push_str("    }\n");
    html.push_str("  }\n");
    
    // Function to save as PNG
    html.push_str("  function saveAsPNG() {\n");
    html.push_str("    const svgEl = document.querySelector('#schema-viz svg');\n");
    html.push_str("    if (svgEl) {\n");
    html.push_str("      const canvas = document.createElement('canvas');\n");
    html.push_str("      const ctx = canvas.getContext('2d');\n");
    html.push_str("      const svgData = new XMLSerializer().serializeToString(svgEl);\n");
    html.push_str("      const img = new Image();\n");
    html.push_str("      img.onload = function() {\n");
    html.push_str("        canvas.width = img.width;\n");
    html.push_str("        canvas.height = img.height;\n");
    html.push_str("        ctx.drawImage(img, 0, 0);\n");
    html.push_str("        const pngUrl = canvas.toDataURL('image/png');\n");
    html.push_str("        const downloadLink = document.createElement('a');\n");
    html.push_str("        downloadLink.href = pngUrl;\n");
    html.push_str("        downloadLink.download = 'crabwalk_schema.png';\n");
    html.push_str("        document.body.appendChild(downloadLink);\n");
    html.push_str("        downloadLink.click();\n");
    html.push_str("        document.body.removeChild(downloadLink);\n");
    html.push_str("      };\n");
    html.push_str("      img.src = 'data:image/svg+xml;base64,' + btoa(unescape(encodeURIComponent(svgData)));\n");
    html.push_str("    } else {\n");
    html.push_str("      alert('Please view the diagram first');\n");
    html.push_str("      toggleView('diagram');\n");
    html.push_str("    }\n");
    html.push_str("  }\n");
    
    html.push_str("</script>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");
    
    // Write to file
    let output_dir = Path::new(output_path).parent().unwrap_or(Path::new("."));
    fs::create_dir_all(output_dir)?;
    let mut file = fs::File::create(output_path)?;
    file.write_all(html.as_bytes())?;
    
    println!("Generated HTML schema visualization at {}", output_path);
    
    Ok(())
}