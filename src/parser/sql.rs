use anyhow::{Context, Result};
use duckdb::Connection;
use serde_json::Value;
use sqlparser::ast::{
    Expr, Ident, Query, Select, SelectItem, 
    SetExpr, SetOperator, SetQuantifier, Statement, TableFactor, Value as SqlValue, GroupByExpr
};
use sqlparser::dialect::{DuckDbDialect, GenericDialect};
use sqlparser::parser::Parser;
use std::collections::HashSet;

/// Parse SQL string into AST
///
/// # Arguments
///
/// * `sql` - SQL string to parse
/// * `dialect_name` - SQL dialect to use
///
/// # Returns
///
/// * `Vec<Statement>` - Vector of parsed SQL statements
pub fn parse_sql(sql: &str, dialect_name: &str) -> Result<Vec<Statement>> {
    // If using DuckDB dialect, attempt to use DuckDB's built-in parser first
    if dialect_name.to_lowercase() == "duckdb" {
        match parse_with_duckdb_and_convert(sql) {
            Ok(statements) => return Ok(statements),
            Err(e) => {
                tracing::debug!("DuckDB parser failed, falling back to sqlparser: {}", e);
                // Fall back to sqlparser
            }
        }
    }
    
    // Parse SQL with sqlparser
    let statements = if dialect_name.to_lowercase() == "duckdb" {
        let dialect = DuckDbDialect {};
        Parser::parse_sql(&dialect, sql)
    } else {
        let dialect = GenericDialect {};
        Parser::parse_sql(&dialect, sql)
    }.context("Failed to parse SQL")?;
    
    Ok(statements)
}

/// Parse SQL using DuckDB's built-in parser and convert to sqlparser Statements
///
/// # Arguments
///
/// * `sql` - SQL string to parse
///
/// # Returns
///
/// * `Result<Vec<Statement>>` - Vector of parsed SQL statements or error
pub fn parse_with_duckdb_and_convert(sql: &str) -> Result<Vec<Statement>> {
    let duckdb_ast = parse_with_duckdb(sql)?;
    
    // Print the AST for debugging
    tracing::debug!("DuckDB AST: {}", serde_json::to_string_pretty(&duckdb_ast)?);
    
    // Convert DuckDB AST to sqlparser Statement objects
    convert_duckdb_ast_to_statements(&duckdb_ast)
}


/// Convert DuckDB AST to sqlparser Statement objects
///
/// # Arguments
///
/// * `duckdb_ast` - The JSON AST from DuckDB
///
/// # Returns
///
/// * `Result<Vec<Statement>>` - Vector of parsed SQL statements or error
fn convert_duckdb_ast_to_statements(duckdb_ast: &Value) -> Result<Vec<Statement>> {
    // Save AST to a file for debugging
    let ast_string = serde_json::to_string_pretty(duckdb_ast)?;
    let debug_file_path = "duckdb_ast_debug.json";
    std::fs::write(debug_file_path, &ast_string)?;
    tracing::info!("Saved DuckDB AST to {}", debug_file_path);
    
    // Check if we have a valid AST structure
    if !duckdb_ast.is_object() {
        return Err(anyhow::anyhow!("DuckDB AST is not a valid JSON object"));
    }
    
    // Check for errors
    if let Some(error) = duckdb_ast.get("error") {
        if error.as_bool() == Some(true) {
            if let Some(message) = duckdb_ast.get("error_message") {
                return Err(anyhow::anyhow!("DuckDB parser error: {}", message));
            }
            return Err(anyhow::anyhow!("DuckDB parser error"));
        }
    }
    
    // Get statements array
    let statements = match duckdb_ast.get("statements") {
        Some(statements) if statements.is_array() => statements.as_array().unwrap(),
        _ => return Err(anyhow::anyhow!("DuckDB AST does not contain statements array")),
    };
    
    let mut result = Vec::new();
    
    for stmt in statements {
        // Get the node object from the statement
        let node = match stmt.get("node") {
            Some(node) if node.is_object() => node,
            _ => {
                tracing::warn!("Statement missing node object: {:?}", stmt);
                continue;
            }
        };
        
        // Check the node type
        let node_type = node.get("type").and_then(|t| t.as_str());
        
        match node_type {
            Some("SET_OPERATION_NODE") => {
                // Convert set operation (e.g., UNION, UNION ALL, etc.)
                let sqlparser_stmt = convert_set_operation_node(node)?;
                result.push(sqlparser_stmt);
            },
            Some("SELECT_NODE") => {
                // Convert simple SELECT
                let sqlparser_stmt = convert_select_node(node)?;
                result.push(sqlparser_stmt);
            },
            Some(other_type) => {
                tracing::warn!("Unsupported node type: {}", other_type);
                return Err(anyhow::anyhow!("Unsupported DuckDB AST node type: {}", other_type));
            },
            None => {
                tracing::warn!("Node missing type field: {:?}", node);
                return Err(anyhow::anyhow!("DuckDB AST node missing type field"));
            }
        }
    }
    
    Ok(result)
}

/// Convert a DuckDB SET_OPERATION_NODE to a sqlparser Statement
fn convert_set_operation_node(node: &Value) -> Result<Statement> {
    // Get the set operation type
    let setop_type = node.get("setop_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("SET_OPERATION_NODE missing setop_type"))?;
    
    // Get the "all" flag
    let setop_all = node.get("setop_all")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Get left and right nodes
    let left_node = node.get("left")
        .ok_or_else(|| anyhow::anyhow!("SET_OPERATION_NODE missing left node"))?;
    
    let right_node = node.get("right")
        .ok_or_else(|| anyhow::anyhow!("SET_OPERATION_NODE missing right node"))?;
    
    // Convert left and right nodes to SetExpr
    let left_expr = convert_node_to_set_expr(left_node)?;
    let right_expr = convert_node_to_set_expr(right_node)?;
    
    // Map DuckDB set operation type to sqlparser SetOperator
    let set_operator = match setop_type {
        "UNION" => SetOperator::Union,
        "EXCEPT" => SetOperator::Except,
        "INTERSECT" => SetOperator::Intersect,
        _ => return Err(anyhow::anyhow!("Unsupported set operation type: {}", setop_type)),
    };
    
    // Create SetExpr::SetOperation
    let set_expr = SetExpr::SetOperation {
        op: set_operator,
        set_quantifier: if setop_all { SetQuantifier::All } else { SetQuantifier::Distinct },
        left: Box::new(left_expr),
        right: Box::new(right_expr),
    };
    
    // Create Query with the SetExpr
    let query = Query {
        with: None,
        body: Box::new(set_expr),
        order_by: None,
        limit: None,
        offset: None,
        fetch: None,
        locks: Vec::new(),
        limit_by: Vec::new(),
        for_clause: None,
        format_clause: None,
        settings: None,
    };
    
    // Create Statement::Query
    Ok(Statement::Query(Box::new(query)))
}

/// Convert a DuckDB SELECT_NODE to a sqlparser SetExpr
fn convert_select_node(node: &Value) -> Result<Statement> {
    let set_expr = convert_node_to_set_expr(node)?;
    
    // Create Query with the SetExpr
    let query = Query {
        with: None,
        body: Box::new(set_expr),
        order_by: None,
        limit: None,
        offset: None,
        fetch: None,
        locks: Vec::new(),
        limit_by: Vec::new(),
        for_clause: None,
        format_clause: None,
        settings: None,
    };
    
    // Create Statement::Query
    Ok(Statement::Query(Box::new(query)))
}

/// Convert a DuckDB node to a sqlparser SetExpr
fn convert_node_to_set_expr(node: &Value) -> Result<SetExpr> {
    // Check node type
    let node_type = node.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Node missing type field"))?;
    
    match node_type {
        "SELECT_NODE" => {
            // Get select list
            let select_list = node.get("select_list")
                .and_then(|sl| sl.as_array())
                .ok_or_else(|| anyhow::anyhow!("SELECT_NODE missing select_list array"))?;
            
            // Convert select list items
            let mut projection = Vec::new();
            for item in select_list {
                let select_item = convert_select_list_item(item)?;
                projection.push(select_item);
            }
            
            // Create Select
            let select = Select {
                distinct: None,
                top: None,
                projection,
                into: None,
                from: Vec::new(), // DuckDB often uses empty FROM clause for constant selects
                lateral_views: Vec::new(),
                selection: None,
                group_by: GroupByExpr::Expressions(Vec::new(), Vec::new()),
                cluster_by: Vec::new(),
                distribute_by: Vec::new(),
                sort_by: Vec::new(),
                having: None,
                qualify: None,
                named_window: Vec::new(),
                connect_by: None,
                window_before_qualify: false,
                prewhere: None,
                value_table_mode: None,
            };
            
            // Check if we need to add a FROM clause
            if let Some(from_table) = node.get("from_table") {
                let from_type = from_table.get("type").and_then(|t| t.as_str());
                if from_type != Some("EMPTY") {
                    // This would be where we add tables to the FROM clause
                    // For now, we don't fully implement this as it's complex
                    tracing::debug!("Non-empty FROM clause found: {:?}", from_table);
                }
            }
            
            Ok(SetExpr::Select(Box::new(select)))
        },
        "SET_OPERATION_NODE" => {
            // For nested set operations, convert recursively
            let inner_stmt = convert_set_operation_node(node)?;
            if let Statement::Query(query) = inner_stmt {
                Ok(*query.body)
            } else {
                Err(anyhow::anyhow!("Expected Query from set operation conversion"))
            }
        },
        _ => Err(anyhow::anyhow!("Unsupported node type for SetExpr conversion: {}", node_type)),
    }
}

/// Convert a DuckDB select list item to a sqlparser SelectItem
fn convert_select_list_item(item: &Value) -> Result<SelectItem> {
    // Get item alias
    let alias = item.get("alias")
        .and_then(|a| a.as_str())
        .unwrap_or("");
    
    // Get item class
    let item_class = item.get("class")
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::anyhow!("Select list item missing class"))?;
    
    // Get item type
    let item_type = item.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Select list item missing type"))?;
    
    match (item_class, item_type) {
        ("CONSTANT", "VALUE_CONSTANT") => {
            // Get value
            let value = item.get("value")
                .ok_or_else(|| anyhow::anyhow!("VALUE_CONSTANT missing value"))?;
            
            // Create expression from value
            let expr = convert_value_to_expr(value)?;
            
            // Create SelectItem
            if !alias.is_empty() {
                Ok(SelectItem::ExprWithAlias {
                    expr,
                    alias: Ident::new(alias),
                })
            } else {
                Ok(SelectItem::UnnamedExpr(expr))
            }
        },
        // Add more cases for different item types
        _ => {
            tracing::warn!("Unsupported select list item: class={}, type={}", item_class, item_type);
            // For now, create a placeholder expression
            let expr = Expr::Value(SqlValue::Null);
            
            if !alias.is_empty() {
                Ok(SelectItem::ExprWithAlias {
                    expr,
                    alias: Ident::new(alias),
                })
            } else {
                Ok(SelectItem::UnnamedExpr(expr))
            }
        }
    }
}

/// Convert a DuckDB value to a sqlparser Expr
fn convert_value_to_expr(value: &Value) -> Result<Expr> {
    // Check if value is null
    let is_null = value.get("is_null")
        .and_then(|n| n.as_bool())
        .unwrap_or(false);
    
    if is_null {
        return Ok(Expr::Value(SqlValue::Null));
    }
    
    // Get value type
    let value_type = value.get("type")
        .and_then(|t| t.get("id"))
        .and_then(|id| id.as_str())
        .ok_or_else(|| anyhow::anyhow!("Value missing type id"))?;
    
    // Get actual value
    let value_data = value.get("value")
        .ok_or_else(|| anyhow::anyhow!("Missing value"))?;
    
    // Convert value based on type
    match value_type {
        "INTEGER" => {
            if let Some(num) = value_data.as_i64() {
                Ok(Expr::Value(SqlValue::Number(num.to_string(), false)))
            } else {
                Err(anyhow::anyhow!("INTEGER value is not a number"))
            }
        },
        "VARCHAR" => {
            if let Some(s) = value_data.as_str() {
                Ok(Expr::Value(SqlValue::SingleQuotedString(s.to_string())))
            } else {
                Err(anyhow::anyhow!("VARCHAR value is not a string"))
            }
        },
        // Add more type conversions as needed
        _ => {
            tracing::warn!("Unsupported value type: {}", value_type);
            Ok(Expr::Value(SqlValue::Null))
        }
    }
}

/// Parse SQL using DuckDB's built-in parser
///
/// # Arguments
///
/// * `sql` - SQL string to parse
///
/// # Returns
///
/// * `Result<Value>` - JSON representation of the AST or error
pub fn parse_with_duckdb(sql: &str) -> Result<Value> {
    // This feature requires DuckDB 0.9.0 or higher
    // Connect to an in-memory DuckDB database
    let conn = Connection::open_in_memory().context("Failed to open DuckDB connection")?;
    
    // Check DuckDB version
    if let Ok(mut version_stmt) = conn.prepare("SELECT version()") {
        if let Ok(mut version_rows) = version_stmt.query([]) {
            if let Ok(Some(row)) = version_rows.next() {
                let version: String = row.get(0)?;
                tracing::debug!("DuckDB version: {}", version);
            }
        }
    }
    
    // Try different approaches
    // Method 1: Directly using json_serialize_sql with parameter
    let result = try_json_serialize_sql_param(&conn, sql);
    if result.is_ok() {
        return result;
    }
    
    // Method 2: Using json_serialize_sql with literal SQL
    let result = try_json_serialize_sql_literal(&conn, sql);
    if result.is_ok() {
        return result;
    }
    
    // Method 3: First parsing the SQL, then using json_serialize_sql
    let result = try_via_prepare_then_serialize(&conn, sql);
    if result.is_ok() {
        return result;
    }
    
    // If all methods fail, return an informative error
    Err(anyhow::anyhow!("Failed to parse SQL with DuckDB's json_serialize_sql. This may be due to using an unsupported DuckDB version or feature limitation."))
}

/// Try parsing with json_serialize_sql with a parameter
fn try_json_serialize_sql_param(conn: &Connection, sql: &str) -> Result<Value> {
    tracing::debug!("Trying json_serialize_sql with parameter");
    let query = "SELECT json_serialize_sql(?)";
    let mut stmt = conn.prepare(query)?;
    
    let mut rows = stmt.query([sql])?;
    if let Some(row) = rows.next()? {
        let ast_json: String = row.get(0)?;
        let parsed_json: Value = serde_json::from_str(&ast_json)
            .context(format!("Failed to parse DuckDB AST JSON: {}", ast_json))?;
        
        return Ok(parsed_json);
    }
    
    Err(anyhow::anyhow!("No result from json_serialize_sql with parameter"))
}

/// Try parsing with json_serialize_sql with a literal SQL string
fn try_json_serialize_sql_literal(conn: &Connection, sql: &str) -> Result<Value> {
    tracing::debug!("Trying json_serialize_sql with literal");
    // Escape single quotes in SQL
    let escaped_sql = sql.replace('\'', "''");
    let query = format!("SELECT json_serialize_sql('{}')", escaped_sql);
    
    let mut stmt = conn.prepare(&query)?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let ast_json: String = row.get(0)?;
        let parsed_json: Value = serde_json::from_str(&ast_json)
            .context(format!("Failed to parse DuckDB AST JSON: {}", ast_json))?;
        
        return Ok(parsed_json);
    }
    
    Err(anyhow::anyhow!("No result from json_serialize_sql with literal"))
}

/// Try first preparing the SQL, then serializing
fn try_via_prepare_then_serialize(conn: &Connection, sql: &str) -> Result<Value> {
    tracing::debug!("Trying prepare then serialize approach");
    
    // First try to prepare the statement to validate SQL
    let _stmt = match conn.prepare(sql) {
        Ok(stmt) => stmt,
        Err(e) => {
            tracing::debug!("Failed to prepare SQL: {}", e);
            return Err(anyhow::anyhow!("Failed to prepare SQL: {}", e));
        }
    };
    
    // Then try to get the serialized form
    // This is a placeholder implementation and may need adjustment
    let query = format!("SELECT json_serialize_sql(q) FROM (SELECT '{}' as q) t", sql.replace('\'', "''"));
    let mut stmt = conn.prepare(&query)?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let ast_json: String = row.get(0)?;
        let parsed_json: Value = serde_json::from_str(&ast_json)
            .context(format!("Failed to parse DuckDB AST JSON: {}", ast_json))?;
        
        return Ok(parsed_json);
    }
    
    Err(anyhow::anyhow!("No result from prepare-then-serialize approach"))
}

/// Check if a statement is a SELECT query
///
/// # Arguments
///
/// * `statement` - SQL statement to check
///
/// # Returns
///
/// * `bool` - True if the statement is a SELECT query
pub fn is_select_tree(statement: &Statement) -> bool {
    match statement {
        Statement::Query(query) => {
            match query.body.as_ref() {
                SetExpr::Select(_) => true,
                SetExpr::Query(_) => true,
                _ => false,
            }
        }
        _ => false,
    }
}

/// Extract table names from a SQL query
///
/// # Arguments
///
/// * `statement` - SQL statement to extract tables from
///
/// # Returns
///
/// * `HashSet<String>` - Set of table names
pub fn extract_tables(statement: &Statement) -> HashSet<String> {
    let mut tables = HashSet::new();
    
    if let Statement::Query(query) = statement {
        extract_tables_from_query(query, &mut tables);
    }
    
    tables
}

/// Extract table names from a SQL query
fn extract_tables_from_query(query: &Query, tables: &mut HashSet<String>) {
    if let SetExpr::Select(select) = query.body.as_ref() {
        for table_with_join in &select.from {
            match &table_with_join.relation {
                TableFactor::Table { name, .. } => {
                    // Add the table name to the set
                    tables.insert(name.to_string());
                }
                // Handle other table factors like subqueries
                TableFactor::Derived { subquery, .. } => {
                    extract_tables_from_query(subquery, tables);
                }
                _ => {}
            }
            
            // Handle joins
            for join in &table_with_join.joins {
                match &join.relation {
                    TableFactor::Table { name, .. } => {
                        tables.insert(name.to_string());
                    }
                    TableFactor::Derived { subquery, .. } => {
                        extract_tables_from_query(subquery, tables);
                    }
                    _ => {}
                }
            }
        }
    }
}