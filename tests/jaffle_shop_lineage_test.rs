use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crabwalk::parser::dependencies::{get_dependencies, Dependency, get_execution_order};
use crabwalk::parser::lineage::generate_mermaid_diagram;
use crabwalk::Crabwalk;

/// Test to verify that lineage is properly extracted from the Jaffle Shop example
#[test]
fn test_jaffle_shop_lineage() {
    // Path to the Jaffle Shop example
    let jaffle_shop_path = Path::new("examples/jaffle_shop");
    
    // Make sure the Jaffle Shop example exists
    assert!(jaffle_shop_path.exists(), "Jaffle Shop example directory should exist");
    
    // Extract dependencies from SQL files
    let dialect = "duckdb";
    let dependencies_result = get_dependencies(jaffle_shop_path.to_str().unwrap(), dialect);
    assert!(dependencies_result.is_ok(), "Should extract dependencies without error");
    
    let dependencies = dependencies_result.unwrap();
    
    // Verify we found all the models from the Jaffle Shop example
    let expected_models = vec![
        "stg_customers", "stg_orders", "stg_products", "stg_locations", 
        "stg_supplies", "stg_order_items", "customers", "orders", 
        "order_items", "products", "locations", "supplies"
    ];
    
    for model in &expected_models {
        assert!(dependencies.contains_key(*model), "Dependencies should include model: {}", model);
    }
    
    // Check specific dependency relationships
    verify_dependency(&dependencies, "stg_customers", "raw_customers");
    verify_dependency(&dependencies, "stg_orders", "raw_orders");
    verify_dependency(&dependencies, "stg_products", "raw_products");
    verify_dependency(&dependencies, "stg_order_items", "raw_items");
    
    // Check more complex dependencies for marts
    verify_dependency(&dependencies, "customers", "stg_customers");
    verify_dependency(&dependencies, "customers", "stg_orders");
    verify_dependency(&dependencies, "customers", "orders");
    
    verify_dependency(&dependencies, "orders", "stg_orders");
    verify_dependency(&dependencies, "orders", "order_items");
    
    verify_dependency(&dependencies, "order_items", "stg_order_items");
    verify_dependency(&dependencies, "order_items", "stg_orders");
    verify_dependency(&dependencies, "order_items", "stg_products");
    verify_dependency(&dependencies, "order_items", "stg_supplies");
    
    // Generate a lineage diagram in a temporary directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    
    // Copy all SQL files to the temp directory to preserve the original Jaffle Shop example
    for entry in walkdir::WalkDir::new(jaffle_shop_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "sql") {
            let rel_path = entry.path().strip_prefix(jaffle_shop_path).unwrap();
            let target_path = Path::new(temp_path).join(rel_path);
            
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            
            fs::copy(entry.path(), &target_path).unwrap();
        }
    }
    
    // Generate lineage diagram
    let result = generate_mermaid_diagram(temp_path, &dependencies);
    assert!(result.is_ok(), "Should generate lineage diagram without error");
    
    // Check that the lineage file was created
    let lineage_path = format!("{}/lineage.mmd", temp_path);
    assert!(fs::metadata(&lineage_path).is_ok(), "Lineage diagram file should exist");
    
    // Read the generated lineage diagram
    let lineage_content = fs::read_to_string(&lineage_path).unwrap();
    
    // Verify that the diagram contains expected nodes and edges
    assert!(lineage_content.contains("graph TD"), "Diagram should have the correct header");
    
    // Check for nodes
    for model in &expected_models {
        assert!(lineage_content.contains(model), "Diagram should contain node: {}", model);
    }
    
    // Check for specific edges in the diagram
    assert_edge_in_diagram(&lineage_content, "raw_customers", "stg_customers");
    assert_edge_in_diagram(&lineage_content, "raw_orders", "stg_orders");
    assert_edge_in_diagram(&lineage_content, "raw_products", "stg_products");
    
    assert_edge_in_diagram(&lineage_content, "stg_customers", "customers");
    assert_edge_in_diagram(&lineage_content, "stg_orders", "orders");
    assert_edge_in_diagram(&lineage_content, "stg_orders", "customers");
    assert_edge_in_diagram(&lineage_content, "orders", "customers");
    
    assert_edge_in_diagram(&lineage_content, "stg_order_items", "order_items");
    assert_edge_in_diagram(&lineage_content, "stg_products", "order_items");
    assert_edge_in_diagram(&lineage_content, "stg_orders", "order_items");
    assert_edge_in_diagram(&lineage_content, "stg_supplies", "order_items");
    
    println!("✅ Jaffle Shop lineage test passed successfully!");
}

/// Test to verify that circular dependencies are detected in the Jaffle Shop example
#[test]
fn test_jaffle_shop_circular_dependencies() {
    // Path to the Jaffle Shop example
    let jaffle_shop_path = Path::new("examples/jaffle_shop");
    
    // Extract dependencies from SQL files
    let dialect = "duckdb";
    let dependencies = get_dependencies(jaffle_shop_path.to_str().unwrap(), dialect).unwrap();
    
    // Try to get execution order - this should fail due to circular dependencies
    let execution_order_result = get_execution_order(&dependencies);
    
    // Verify that we correctly detected a cycle
    assert!(execution_order_result.is_err(), "Should detect circular dependency");
    let error = execution_order_result.unwrap_err();
    assert!(
        error.to_string().contains("Cycle"), 
        "Error should mention cycle: {}", error
    );
    
    // Verify the specific circular dependency between orders and order_items
    println!("Checking for circular dependency between orders and order_items");
    verify_dependency(&dependencies, "orders", "order_items");
    verify_dependency(&dependencies, "order_items", "orders");
    
    println!("✅ Jaffle Shop circular dependency test passed successfully!");
}

/// Test to verify that force mode can handle circular dependencies
#[test]
fn test_jaffle_shop_force_mode() {
    // Create a temporary directory for running the force mode test
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    
    // Create the Crabwalk instance with force mode
    let crabwalk = Crabwalk::new(
        format!("{}/test.db", temp_path),
        "examples/jaffle_shop".to_string(),
        "duckdb".to_string(),
        "transform".to_string(),
        None,
        None,
    );
    
    // Run in force mode to handle circular dependencies
    let result = crabwalk.run_force();
    
    // The operation should succeed even with circular dependencies
    assert!(result.is_ok(), "Force mode should succeed even with circular dependencies: {:?}", result);
    println!("✅ Jaffle Shop force mode test passed successfully!");
}

// Remove unused function

/// Helper function to verify that a model depends on a specific dependency
fn verify_dependency(dependencies: &HashMap<String, Dependency>, model: &str, dependency: &str) {
    if let Some(model_dep) = dependencies.get(model) {
        assert!(
            model_dep.deps.contains(dependency),
            "Model {} should depend on {}", model, dependency
        );
    } else {
        panic!("Model {} not found in dependencies", model);
    }
}

/// Helper function to check if an edge exists in the diagram
fn assert_edge_in_diagram(diagram: &str, from: &str, to: &str) {
    let edge = format!("{} --> {}", from, to);
    assert!(
        diagram.contains(&edge),
        "Diagram should contain edge: {}",
        edge
    );
}