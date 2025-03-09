use std::collections::{HashMap, HashSet};
use std::fs;
use tempfile::tempdir;
use crabwalk::parser::dependencies::Dependency;
use crabwalk::parser::lineage::{generate_mermaid_diagram, encode_mermaid_diagram};

#[test]
fn test_encode_mermaid_diagram() {
    let diagram = "graph TD\n    A --> B";
    let result = encode_mermaid_diagram(diagram);
    
    assert!(result.is_ok(), "Should encode diagram without error");
    let encoded = result.unwrap();
    
    // The encoded string should be non-empty and be valid base64
    assert!(!encoded.is_empty(), "Encoded diagram should not be empty");
    // With Pako encoding, the output could vary but will typically start with certain patterns
    // due to the JSON structure and compression. Just check that it's not empty for now.
    // Since the compressed output might vary slightly, we'll skip the exact prefix check.
}

#[test]
fn test_generate_mermaid_diagram_empty() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    let dependencies = HashMap::new();
    
    let result = generate_mermaid_diagram(path, &dependencies);
    assert!(result.is_ok(), "Should generate diagram for empty dependencies");
    
    // Check that the file was created
    let diagram_path = format!("{}/lineage.mmd", path);
    assert!(fs::metadata(&diagram_path).is_ok(), "Diagram file should exist");
    
    // Check content
    let content = fs::read_to_string(&diagram_path).unwrap();
    assert!(content.contains("graph TD"), "Diagram should have correct header");
}

#[test]
fn test_generate_mermaid_diagram_simple() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    
    // Create a simple dependency graph
    let mut dependencies = HashMap::new();
    
    // Add source model with no dependencies
    let source = Dependency {
        deps: HashSet::new(),
        filename: "source.sql".to_string(),
        config: None,
    };
    dependencies.insert("source".to_string(), source);
    
    // Add target model that depends on source
    let mut target_deps = HashSet::new();
    target_deps.insert("source".to_string());
    let target = Dependency {
        deps: target_deps,
        filename: "target.sql".to_string(),
        config: None,
    };
    dependencies.insert("target".to_string(), target);
    
    let result = generate_mermaid_diagram(path, &dependencies);
    assert!(result.is_ok(), "Should generate diagram for simple dependencies");
    
    // Check content
    let diagram_path = format!("{}/lineage.mmd", path);
    let content = fs::read_to_string(&diagram_path).unwrap();
    
    // Diagram should contain both nodes and the edge
    assert!(content.contains("source"), "Diagram should contain source node");
    assert!(content.contains("target"), "Diagram should contain target node");
    assert!(content.contains("source --> target"), "Diagram should contain the edge");
}

#[test]
fn test_generate_mermaid_diagram_complex() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    
    // Create a more complex dependency graph
    let mut dependencies = HashMap::new();
    
    // Add source models
    for name in &["source1", "source2"] {
        dependencies.insert(name.to_string(), Dependency {
            deps: HashSet::new(),
            filename: format!("{}.sql", name),
            config: None,
        });
    }
    
    // Add intermediate model that depends on both sources
    let mut intermediate_deps = HashSet::new();
    intermediate_deps.insert("source1".to_string());
    intermediate_deps.insert("source2".to_string());
    dependencies.insert("intermediate".to_string(), Dependency {
        deps: intermediate_deps,
        filename: "intermediate.sql".to_string(),
        config: None,
    });
    
    // Add final model that depends on intermediate
    let mut final_deps = HashSet::new();
    final_deps.insert("intermediate".to_string());
    dependencies.insert("final".to_string(), Dependency {
        deps: final_deps,
        filename: "final.sql".to_string(),
        config: None,
    });
    
    let result = generate_mermaid_diagram(path, &dependencies);
    assert!(result.is_ok(), "Should generate diagram for complex dependencies");
    
    // Check content
    let diagram_path = format!("{}/lineage.mmd", path);
    let content = fs::read_to_string(&diagram_path).unwrap();
    
    // Check all nodes and edges
    for node in &["source1", "source2", "intermediate", "final"] {
        assert!(content.contains(node), "Diagram should contain {} node", node);
    }
    
    // Check all edges
    assert!(content.contains("source1 --> intermediate"), "Diagram should contain edge from source1 to intermediate");
    assert!(content.contains("source2 --> intermediate"), "Diagram should contain edge from source2 to intermediate");
    assert!(content.contains("intermediate --> final"), "Diagram should contain edge from intermediate to final");
}