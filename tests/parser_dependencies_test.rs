use std::fs;
use std::io::Write;
use tempfile::tempdir;
use crabwalk::parser::dependencies::{get_dependencies, Dependency};

#[test]
fn test_process_empty_folder() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    
    let result = get_dependencies(path, "duckdb");
    assert!(result.is_ok(), "Should handle empty folder gracefully");
    
    let dependencies = result.unwrap();
    assert_eq!(dependencies.len(), 0, "Empty folder should yield no dependencies");
}

#[test]
fn test_process_single_file_without_dependencies() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    
    // Create a simple SQL file
    let file_path = format!("{}/simple.sql", path);
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(file, "SELECT 1 as test").unwrap();
    
    let result = get_dependencies(path, "duckdb");
    assert!(result.is_ok(), "Should process single file without error");
    
    let dependencies = result.unwrap();
    assert_eq!(dependencies.len(), 1, "Should have one model");
    assert!(dependencies.contains_key("simple"), "Model name should be derived from filename");
    
    let deps = dependencies.get("simple").unwrap();
    assert_eq!(deps.deps.len(), 0, "Simple query should have no dependencies");
}

#[test]
fn test_process_file_with_dependencies() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    
    // Create the first SQL file (will be a dependency)
    let dep_file_path = format!("{}/source.sql", path);
    let mut file = fs::File::create(&dep_file_path).unwrap();
    writeln!(file, "SELECT 1 as id, 'test' as name").unwrap();
    
    // Create the second SQL file (depends on the first)
    let file_path = format!("{}/dependent.sql", path);
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(file, "SELECT * FROM source WHERE id > 0").unwrap();
    
    let result = get_dependencies(path, "duckdb");
    assert!(result.is_ok(), "Should process files with dependencies");
    
    let dependencies = result.unwrap();
    assert_eq!(dependencies.len(), 2, "Should have two models");
    assert!(dependencies.contains_key("source"), "Source model should exist");
    assert!(dependencies.contains_key("dependent"), "Dependent model should exist");
    
    // Check the dependencies are correct
    let source_deps = dependencies.get("source").unwrap();
    assert_eq!(source_deps.deps.len(), 0, "Source should have no dependencies");
    
    let dependent_deps = dependencies.get("dependent").unwrap();
    assert_eq!(dependent_deps.deps.len(), 1, "Dependent should have one dependency");
    assert!(dependent_deps.deps.contains(&"source".to_string()), "Dependent should depend on source");
}

#[test]
fn test_process_files_with_complex_dependencies() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    
    // Create several SQL files with interdependencies
    let files = [
        ("source1.sql", "SELECT 1 as id, 'test1' as name"),
        ("source2.sql", "SELECT 2 as id, 'test2' as name"),
        ("intermediate.sql", "SELECT * FROM source1 JOIN source2 ON source1.id = source2.id"),
        ("final.sql", "SELECT * FROM intermediate WHERE name LIKE '%test%'")
    ];
    
    for (filename, content) in files.iter() {
        let file_path = format!("{}/{}", path, filename);
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
    }
    
    let result = get_dependencies(path, "duckdb");
    assert!(result.is_ok(), "Should process complex dependencies");
    
    let dependencies = result.unwrap();
    assert_eq!(dependencies.len(), 4, "Should have four models");
    
    // Check each model has the correct dependencies
    let source1_deps = dependencies.get("source1").unwrap();
    assert_eq!(source1_deps.deps.len(), 0, "source1 should have no dependencies");
    
    let source2_deps = dependencies.get("source2").unwrap();
    assert_eq!(source2_deps.deps.len(), 0, "source2 should have no dependencies");
    
    let intermediate_deps = dependencies.get("intermediate").unwrap();
    assert_eq!(intermediate_deps.deps.len(), 2, "intermediate should have two dependencies");
    assert!(intermediate_deps.deps.contains(&"source1".to_string()), "intermediate should depend on source1");
    assert!(intermediate_deps.deps.contains(&"source2".to_string()), "intermediate should depend on source2");
    
    let final_deps = dependencies.get("final").unwrap();
    assert_eq!(final_deps.deps.len(), 1, "final should have one dependency");
    assert!(final_deps.deps.contains(&"intermediate".to_string()), "final should depend on intermediate");
}