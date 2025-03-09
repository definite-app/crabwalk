use crabwalk::config::{OutputType, OutputConfig, ModelConfig};
use crabwalk::parser::config::extract_config_from_sql;

#[test]
fn test_output_type_default() {
    // Default value should be Table
    let output_config = OutputConfig::default();
    assert!(matches!(output_config.output_type, OutputType::Table), "Default output type should be Table");
}

#[test]
fn test_model_config_default() {
    // Default ModelConfig should have None for output
    let model_config = ModelConfig::default();
    assert!(model_config.output.is_none(), "Default model config should have None for output");
}

#[test]
fn test_extract_config_from_sql_empty() {
    let sql = "SELECT * FROM test";
    let config = extract_config_from_sql(sql).unwrap();
    assert!(config.is_none(), "SQL without config comment should return None");
}

#[test]
fn test_extract_config_from_sql_with_config() {
    // SQL with a config comment for view output
    let sql = "-- @config: {output: {type: \"view\"}}\nSELECT * FROM test";
    let config = extract_config_from_sql(sql).unwrap();
    
    assert!(config.is_some(), "SQL with config comment should parse successfully");
    
    let model_config = config.unwrap();
    assert!(model_config.output.is_some(), "Config should contain output section");
    
    let output_config = model_config.output.unwrap();
    assert!(matches!(output_config.output_type, OutputType::View), "Output type should be View");
    assert!(output_config.location.is_none(), "Location should be None");
}

#[test]
fn test_extract_config_with_location() {
    // SQL with a config comment for parquet output with location
    let sql = "-- @config: {output: {type: \"parquet\", location: \"./output/test.parquet\"}}\nSELECT * FROM test";
    let config = extract_config_from_sql(sql).unwrap();
    
    assert!(config.is_some(), "SQL with config comment should parse successfully");
    
    let model_config = config.unwrap();
    assert!(model_config.output.is_some(), "Config should contain output section");
    
    let output_config = model_config.output.unwrap();
    assert!(matches!(output_config.output_type, OutputType::Parquet), "Output type should be Parquet");
    assert_eq!(output_config.location, Some("./output/test.parquet".to_string()), "Location should match");
}

#[test]
fn test_extract_config_with_multiple_comments() {
    // SQL with multiple comments, only the @config one should be parsed
    let sql = "-- This is a normal comment\n-- @config: {output: {type: \"csv\"}}\n-- Another normal comment\nSELECT * FROM test";
    let config = extract_config_from_sql(sql).unwrap();
    
    assert!(config.is_some(), "SQL with config comment should parse successfully");
    
    let model_config = config.unwrap();
    assert!(model_config.output.is_some(), "Config should contain output section");
    
    let output_config = model_config.output.unwrap();
    assert!(matches!(output_config.output_type, OutputType::Csv), "Output type should be CSV");
}

#[test]
fn test_extract_config_invalid_json() {
    // SQL with invalid JSON in config comment
    let sql = "-- @config: {output: {type: \"view\", invalid_json}\nSELECT * FROM test";
    let config = extract_config_from_sql(sql).unwrap();
    
    // Should return None for invalid JSON
    assert!(config.is_none(), "Invalid JSON should return None");
}

#[test]
fn test_extract_config_invalid_structure() {
    // SQL with valid JSON but invalid structure (missing output.type)
    let sql = "-- @config: {other_field: \"value\"}\nSELECT * FROM test";
    let config = extract_config_from_sql(sql).unwrap();
    
    // This should parse but the output field would be None
    assert!(config.is_some(), "Valid JSON with invalid structure should parse");
    let model_config = config.unwrap();
    assert!(model_config.output.is_none(), "Output field should be None for invalid structure");
}