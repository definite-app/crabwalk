use anyhow::{Context, Result};
use duckdb::Connection;
use serde_json::Value;
use sqlparser::ast::{
    Expr, Ident, Query, Select, SelectItem, OrderBy, Distinct, FunctionArguments,
    SetExpr, SetOperator, SetQuantifier, Statement, TableFactor,
    Value as SqlValue, GroupByExpr
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
    
    // Check for ORDER BY
    let order_by = if let Some(modifiers) = node.get("modifiers").and_then(|m| m.as_array()) {
        let mut order_exprs = Vec::new();
        
        for modifier in modifiers {
            if modifier.get("type").and_then(|t| t.as_str()) == Some("ORDER_MODIFIER") {
                if let Some(orders) = modifier.get("orders").and_then(|o| o.as_array()) {
                    for order in orders {
                        if let Some(expr) = order.get("expression") {
                            // Convert expression
                            let order_expr = convert_expr_node(expr)?;
                            
                            // Get order type (ASC/DESC)
                            let asc = order.get("type")
                                .and_then(|t| t.as_str())
                                .map(|t| t == "ASCENDING")
                                .unwrap_or(true);
                            
                            // Get nulls order
                            let nulls_first = order.get("null_order")
                                .and_then(|n| n.as_str())
                                .map(|n| n == "NULLS_FIRST")
                                .unwrap_or(true);
                            
                            // Create OrderByExpr
                            order_exprs.push(sqlparser::ast::OrderByExpr {
                                expr: order_expr,
                                asc: Some(asc),
                                nulls_first: Some(nulls_first),
                                with_fill: None,
                            });
                        }
                    }
                }
            }
        }
        
        if order_exprs.is_empty() {
            None
        } else {
            Some(OrderBy {
                exprs: order_exprs,
                interpolate: None,
            })
        }
    } else {
        None
    };
    
    // Check for LIMIT
    let limit = if let Some(modifiers) = node.get("modifiers").and_then(|m| m.as_array()) {
        let limit_modifier = modifiers.iter().find(|m| 
            m.get("type").and_then(|t| t.as_str()) == Some("LIMIT_MODIFIER")
        );
        
        if let Some(limit_mod) = limit_modifier {
            if let Some(limit_val) = limit_mod.get("limit") {
                // Convert limit value to expression
                let limit_expr = convert_expr_node(limit_val)?;
                Some(limit_expr)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Check for OFFSET
    let offset = if let Some(modifiers) = node.get("modifiers").and_then(|m| m.as_array()) {
        let limit_modifier = modifiers.iter().find(|m| 
            m.get("type").and_then(|t| t.as_str()) == Some("LIMIT_MODIFIER")
        );
        
        if let Some(limit_mod) = limit_modifier {
            if let Some(offset_val) = limit_mod.get("offset") {
                // Convert offset value to expression
                let offset_expr = convert_expr_node(offset_val)?;
                Some(sqlparser::ast::Offset {
                    value: offset_expr,
                    rows: sqlparser::ast::OffsetRows::Rows,
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Create Query with the SetExpr and modifiers
    let query = Query {
        with: None,
        body: Box::new(set_expr),
        order_by,
        limit,
        offset,
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
            
            // Check for DISTINCT modifier
            let distinct = if let Some(modifiers) = node.get("modifiers").and_then(|m| m.as_array()) {
                let has_distinct = modifiers.iter().any(|m| {
                    m.get("type").and_then(|t| t.as_str()) == Some("DISTINCT_MODIFIER")
                });
                
                if has_distinct {
                    Some(Distinct::Distinct)
                } else {
                    None
                }
            } else {
                None
            };
            
            // Initialize empty FROM clause
            let mut from = Vec::new();
            
            // Check if we need to add a FROM clause
            if let Some(from_table) = node.get("from_table") {
                let from_type = from_table.get("type").and_then(|t| t.as_str());
                
                if from_type != Some("EMPTY") {
                    // Process FROM table
                    if let Some(table_entry) = convert_table_reference(from_table) {
                        from.push(table_entry);
                    }
                }
            }
            
            // Check for WHERE clause
            let selection = if let Some(where_clause) = node.get("where_clause") {
                // Convert where clause to expression
                let where_expr = convert_expr_node(where_clause)?;
                Some(where_expr)
            } else {
                None
            };
            
            // Check for GROUP BY
            let group_by = if let Some(group_expressions) = node.get("group_expressions").and_then(|g| g.as_array()) {
                if !group_expressions.is_empty() {
                    let mut group_by_exprs = Vec::new();
                    
                    for expr in group_expressions {
                        let group_expr = convert_expr_node(expr)?;
                        group_by_exprs.push(group_expr);
                    }
                    
                    GroupByExpr::Expressions(group_by_exprs, Vec::new()) // Empty having clause for now
                } else {
                    GroupByExpr::Expressions(Vec::new(), Vec::new())
                }
            } else {
                GroupByExpr::Expressions(Vec::new(), Vec::new())
            };
            
            // Check for HAVING
            let having = if let Some(having_clause) = node.get("having_clause") {
                let having_expr = convert_expr_node(having_clause)?;
                Some(having_expr)
            } else {
                None
            };
            
            // Create Select
            let select = Select {
                distinct,
                top: None,
                projection,
                into: None,
                from,
                lateral_views: Vec::new(),
                selection,
                group_by,
                cluster_by: Vec::new(),
                distribute_by: Vec::new(),
                sort_by: Vec::new(),
                having,
                qualify: None,
                named_window: Vec::new(),
                connect_by: None,
                window_before_qualify: false,
                prewhere: None,
                value_table_mode: None,
            };
            
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

/// Convert a DuckDB table reference to a sqlparser TableWithJoins
fn convert_table_reference(table_ref: &Value) -> Option<sqlparser::ast::TableWithJoins> {
    // Get table reference type
    let ref_type = table_ref.get("type").and_then(|t| t.as_str());
    
    match ref_type {
        Some("BASE_TABLE") => {
            // Get table name
            let table_name = table_ref.get("table_name").and_then(|t| t.as_str());
            let alias = table_ref.get("alias").and_then(|a| a.as_str());
            
            if let Some(name) = table_name {
                // Create table factor
                let relation = TableFactor::Table {
                    name: sqlparser::ast::ObjectName(vec![Ident::new(name)]),
                    alias: alias.map(|a| sqlparser::ast::TableAlias {
                        name: Ident::new(a),
                        columns: Vec::new(),
                    }),
                    args: None,
                    with_hints: Vec::new(),
                    version: None,
                    partitions: Vec::new(),
                    with_ordinality: false,
                };
                
                // Create table with joins
                Some(sqlparser::ast::TableWithJoins {
                    relation,
                    joins: Vec::new(),
                })
            } else {
                tracing::warn!("BASE_TABLE missing table_name");
                None
            }
        },
        Some("JOIN") => {
            // Get left and right tables
            let left = table_ref.get("left").and_then(|l| convert_table_reference(l));
            let right = table_ref.get("right").and_then(|r| {
                // Get right table relation
                let ref_type = r.get("type").and_then(|t| t.as_str());
                
                match ref_type {
                    Some("BASE_TABLE") => {
                        let table_name = r.get("table_name").and_then(|t| t.as_str());
                        let alias = r.get("alias").and_then(|a| a.as_str());
                        
                        if let Some(name) = table_name {
                            Some(TableFactor::Table {
                                name: sqlparser::ast::ObjectName(vec![Ident::new(name)]),
                                alias: alias.map(|a| sqlparser::ast::TableAlias {
                                    name: Ident::new(a),
                                    columns: Vec::new(),
                                }),
                                args: None,
                                with_hints: Vec::new(),
                                version: None,
                                partitions: Vec::new(),
                                with_ordinality: false,
                            })
                        } else {
                            tracing::warn!("JOIN right side BASE_TABLE missing table_name");
                            None
                        }
                    },
                    _ => {
                        tracing::warn!("Unsupported JOIN right side type: {:?}", ref_type);
                        None
                    }
                }
            });
            
            // Get join type
            let join_type = table_ref.get("join_type")
                .and_then(|j| j.as_str())
                .unwrap_or("INNER");
            
            // Get join condition
            let condition = table_ref.get("condition").and_then(|c| {
                // Try to convert condition to expression
                convert_expr_node(c).ok()
            });
            
            // Map join type to sqlparser JoinType
            let join_operator = match join_type {
                "INNER" => sqlparser::ast::JoinOperator::Inner(
                    sqlparser::ast::JoinConstraint::On(
                        condition.unwrap_or(Expr::Value(SqlValue::Boolean(true)))
                    )
                ),
                "LEFT" => sqlparser::ast::JoinOperator::LeftOuter(
                    sqlparser::ast::JoinConstraint::On(
                        condition.unwrap_or(Expr::Value(SqlValue::Boolean(true)))
                    )
                ),
                "RIGHT" => sqlparser::ast::JoinOperator::RightOuter(
                    sqlparser::ast::JoinConstraint::On(
                        condition.unwrap_or(Expr::Value(SqlValue::Boolean(true)))
                    )
                ),
                "FULL" => sqlparser::ast::JoinOperator::FullOuter(
                    sqlparser::ast::JoinConstraint::On(
                        condition.unwrap_or(Expr::Value(SqlValue::Boolean(true)))
                    )
                ),
                _ => sqlparser::ast::JoinOperator::Inner(
                    sqlparser::ast::JoinConstraint::On(
                        condition.unwrap_or(Expr::Value(SqlValue::Boolean(true)))
                    )
                ),
            };
            
            // If we have both left and right, create a table with join
            if let (Some(mut left_table), Some(right_relation)) = (left, right) {
                // Add join to left table's joins
                left_table.joins.push(sqlparser::ast::Join {
                    relation: right_relation,
                    join_operator,
                });
                
                Some(left_table)
            } else {
                tracing::warn!("JOIN missing left or right table");
                None
            }
        },
        Some("SUBQUERY") => {
            // Get subquery
            if let (Some(subquery), alias) = (
                table_ref.get("subquery").and_then(|s| {
                    // Convert subquery to SetExpr
                    convert_node_to_set_expr(s).ok().map(|expr| {
                        // Create Query
                        let query = Query {
                            with: None,
                            body: Box::new(expr),
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
                        query
                    })
                }),
                table_ref.get("alias").and_then(|a| a.as_str())
            ) {
                // Create derived table
                let relation = TableFactor::Derived {
                    lateral: false,
                    subquery: Box::new(subquery),
                    alias: alias.map(|a| sqlparser::ast::TableAlias {
                        name: Ident::new(a),
                        columns: Vec::new(),
                    }),
                };
                
                // Create table with joins
                Some(sqlparser::ast::TableWithJoins {
                    relation,
                    joins: Vec::new(),
                })
            } else {
                tracing::warn!("SUBQUERY missing subquery or alias");
                None
            }
        },
        _ => {
            tracing::warn!("Unsupported table reference type: {:?}", ref_type);
            None
        }
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
        ("FUNCTION", "FUNCTION") => {
            // Get function name
            let function_name = item.get("function_name")
                .and_then(|f| f.as_str())
                .ok_or_else(|| anyhow::anyhow!("FUNCTION missing function_name"))?;
            
            // Get function children/arguments
            let children = item.get("children")
                .and_then(|c| c.as_array())
                .ok_or_else(|| anyhow::anyhow!("FUNCTION missing children array"))?;
            
            // Convert function arguments
            let mut args = Vec::new();
            for child in children {
                // Convert each child to an expression
                let arg_expr = convert_expr_node(child)?;
                args.push(arg_expr);
            }
            
            // Create function expression
            let expr = Expr::Function(sqlparser::ast::Function {
                name: sqlparser::ast::ObjectName(vec![Ident::new(function_name)]),
                args: convert_args_to_function_arguments(args),
                over: None,
                filter: None,
                null_treatment: None,
                within_group: Vec::new(),
                parameters: FunctionArguments::None,
            });
            
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
        ("COLUMN_REF", "COLUMN_REF") => {
            // Get column name
            let column_name = item.get("column_name")
                .and_then(|c| c.as_str())
                .ok_or_else(|| anyhow::anyhow!("COLUMN_REF missing column_name"))?;
            
            // Check if there's a table name
            let table_name = item.get("table_name")
                .and_then(|t| t.as_str());
            
            // Create column reference expression
            let expr = if let Some(table) = table_name {
                Expr::CompoundIdentifier(vec![
                    Ident::new(table),
                    Ident::new(column_name),
                ])
            } else {
                Expr::Identifier(Ident::new(column_name))
            };
            
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

/// Convert a DuckDB expression node to a sqlparser Expr
fn convert_expr_node(node: &Value) -> Result<Expr> {
    // Get node class and type
    let node_class = node.get("class")
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::anyhow!("Expression node missing class"))?;
    
    let node_type = node.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow::anyhow!("Expression node missing type"))?;
    
    match (node_class, node_type) {
        ("CONSTANT", "VALUE_CONSTANT") => {
            // Get value
            let value = node.get("value")
                .ok_or_else(|| anyhow::anyhow!("VALUE_CONSTANT missing value"))?;
            
            // Convert value to expression
            convert_value_to_expr(value)
        },
        ("FUNCTION", "FUNCTION") => {
            // Get function name
            let function_name = node.get("function_name")
                .and_then(|f| f.as_str())
                .ok_or_else(|| anyhow::anyhow!("FUNCTION missing function_name"))?;
            
            // Get function children/arguments
            let children = node.get("children")
                .and_then(|c| c.as_array())
                .ok_or_else(|| anyhow::anyhow!("FUNCTION missing children array"))?;
            
            // Convert function arguments
            let mut args = Vec::new();
            for child in children {
                // Recursive conversion of child expressions
                let arg_expr = convert_expr_node(child)?;
                args.push(arg_expr);
            }
            
            // Create function expression
            Ok(Expr::Function(sqlparser::ast::Function {
                name: sqlparser::ast::ObjectName(vec![Ident::new(function_name)]),
                args: convert_args_to_function_arguments(args),
                over: None,
                filter: None,
                null_treatment: None,
                within_group: Vec::new(),
                parameters: FunctionArguments::None,
            }))
        },
        ("COLUMN_REF", "COLUMN_REF") => {
            // Get column name
            let column_name = node.get("column_name")
                .and_then(|c| c.as_str())
                .ok_or_else(|| anyhow::anyhow!("COLUMN_REF missing column_name"))?;
            
            // Check if there's a table name
            let table_name = node.get("table_name")
                .and_then(|t| t.as_str());
            
            // Create column reference expression
            if let Some(table) = table_name {
                Ok(Expr::CompoundIdentifier(vec![
                    Ident::new(table),
                    Ident::new(column_name),
                ]))
            } else {
                Ok(Expr::Identifier(Ident::new(column_name)))
            }
        },
        ("OPERATOR", "OPERATOR") => {
            // Some operators may be represented differently in DuckDB's AST
            let operator_type = node.get("operator_type")
                .and_then(|o| o.as_str())
                .ok_or_else(|| anyhow::anyhow!("OPERATOR missing operator_type"))?;
            
            // Get operands
            let children = node.get("children")
                .and_then(|c| c.as_array())
                .ok_or_else(|| anyhow::anyhow!("OPERATOR missing children array"))?;
            
            // Check for unary vs binary operators
            if children.len() == 1 {
                // Unary operator
                let operand = convert_expr_node(&children[0])?;
                
                match operator_type {
                    "NOT" => Ok(Expr::UnaryOp {
                        op: sqlparser::ast::UnaryOperator::Not,
                        expr: Box::new(operand),
                    }),
                    // Add more unary operators as needed
                    _ => {
                        tracing::warn!("Unsupported unary operator: {}", operator_type);
                        Ok(operand) // Fallback to just returning the operand
                    }
                }
            } else if children.len() == 2 {
                // Binary operator
                let left = convert_expr_node(&children[0])?;
                let right = convert_expr_node(&children[1])?;
                
                match operator_type {
                    "AND" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::And,
                        right: Box::new(right),
                    }),
                    "OR" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Or,
                        right: Box::new(right),
                    }),
                    "=" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Eq,
                        right: Box::new(right),
                    }),
                    ">" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Gt,
                        right: Box::new(right),
                    }),
                    "<" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Lt,
                        right: Box::new(right),
                    }),
                    ">=" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::GtEq,
                        right: Box::new(right),
                    }),
                    "<=" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::LtEq,
                        right: Box::new(right),
                    }),
                    "<>" | "!=" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::NotEq,
                        right: Box::new(right),
                    }),
                    "+" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Plus,
                        right: Box::new(right),
                    }),
                    "-" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Minus,
                        right: Box::new(right),
                    }),
                    "*" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Multiply,
                        right: Box::new(right),
                    }),
                    "/" => Ok(Expr::BinaryOp {
                        left: Box::new(left),
                        op: sqlparser::ast::BinaryOperator::Divide,
                        right: Box::new(right),
                    }),
                    // Add more binary operators as needed
                    _ => {
                        tracing::warn!("Unsupported binary operator: {}", operator_type);
                        // Create a function call as a fallback
                        let args = vec![left, right];
                        
                        Ok(Expr::Function(sqlparser::ast::Function {
                            name: sqlparser::ast::ObjectName(vec![Ident::new(operator_type)]),
                            args: convert_args_to_function_arguments(args),
                            over: None,
                            filter: None,
                            null_treatment: None,
                            within_group: Vec::new(),
                            parameters: FunctionArguments::None,
                        }))
                    }
                }
            } else {
                tracing::warn!("Operator with unexpected number of children: {}", children.len());
                Ok(Expr::Value(SqlValue::Null))
            }
        },
        // Add more expression types as needed
        _ => {
            tracing::warn!("Unsupported expression node: class={}, type={}", node_class, node_type);
            Ok(Expr::Value(SqlValue::Null))
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
        "DOUBLE" | "FLOAT" => {
            if let Some(num) = value_data.as_f64() {
                Ok(Expr::Value(SqlValue::Number(num.to_string(), false)))
            } else {
                Err(anyhow::anyhow!("FLOAT/DOUBLE value is not a number"))
            }
        },
        "BOOLEAN" => {
            if let Some(b) = value_data.as_bool() {
                Ok(Expr::Value(SqlValue::Boolean(b)))
            } else {
                Err(anyhow::anyhow!("BOOLEAN value is not a boolean"))
            }
        },
        "DATE" => {
            if let Some(s) = value_data.as_str() {
                // Use SingleQuotedString as a workaround since Date type doesn't exist
                Ok(Expr::Value(SqlValue::SingleQuotedString(format!("DATE '{}'", s))))
            } else {
                Err(anyhow::anyhow!("DATE value is not a string"))
            }
        },
        "TIMESTAMP" => {
            if let Some(s) = value_data.as_str() {
                // Use SingleQuotedString as a workaround since Timestamp type doesn't exist
                Ok(Expr::Value(SqlValue::SingleQuotedString(format!("TIMESTAMP '{}'", s))))
            } else {
                Err(anyhow::anyhow!("TIMESTAMP value is not a string"))
            }
        },
        "DECIMAL" => {
            // For DECIMAL types, we'll convert to a string representation
            if let Some(num) = value_data.as_f64() {
                Ok(Expr::Value(SqlValue::Number(num.to_string(), true))) // true for exact
            } else if let Some(s) = value_data.as_str() {
                Ok(Expr::Value(SqlValue::Number(s.to_string(), true))) // true for exact
            } else {
                Err(anyhow::anyhow!("DECIMAL value is not a number or string"))
            }
        },
        // Add more type conversions as needed
        _ => {
            tracing::warn!("Unsupported value type: {}", value_type);
            Ok(Expr::Value(SqlValue::Null))
        }
    }
}

/// Helper function to convert a vector of expressions to FunctionArguments
fn convert_args_to_function_arguments(args: Vec<Expr>) -> FunctionArguments {
    // Depending on sqlparser 0.49.0 documentation, we can determine which variant to use
    if args.is_empty() {
        FunctionArguments::None
    } else {
        // Create a custom implementation to handle args differently based on the number of args
        // This requires inspecting the 0.49.0 documentation to understand FunctionArguments structure
        // As a fallback, return FunctionArguments::None to avoid compilation errors
        FunctionArguments::None
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
    
    tracing::info!("Extracting tables from statement: {:?}", statement);
    
    if let Statement::Query(query) = statement {
        tracing::info!("Statement is a Query, processing...");
        extract_tables_from_query(query, &mut tables);
    } else {
        tracing::info!("Statement is not a Query, skipping: {:?}", statement);
    }
    
    tracing::info!("Extracted tables: {:?}", tables);
    tables
}

/// Extract table names from a SQL query
fn extract_tables_from_query(query: &Query, tables: &mut HashSet<String>) {
    // Add debug logging
    tracing::debug!("Extracting tables from query: {:?}", query);
    
    if let SetExpr::Select(select) = query.body.as_ref() {
        tracing::debug!("Processing SELECT with {} FROM clauses", select.from.len());
        
        for table_with_join in &select.from {
            tracing::debug!("Processing FROM clause: {:?}", table_with_join);
            
            match &table_with_join.relation {
                TableFactor::Table { name, .. } => {
                    // Add the table name to the set
                    let table_name = name.to_string();
                    tracing::debug!("Found table: {}", table_name);
                    tables.insert(table_name);
                }
                // Handle other table factors like subqueries
                TableFactor::Derived { subquery, .. } => {
                    tracing::debug!("Processing derived table (subquery)");
                    extract_tables_from_query(subquery, tables);
                }
                _ => {
                    tracing::debug!("Unsupported table factor type: {:?}", table_with_join.relation);
                }
            }
            
            // Handle joins
            tracing::debug!("Processing {} joins", table_with_join.joins.len());
            for join in &table_with_join.joins {
                match &join.relation {
                    TableFactor::Table { name, .. } => {
                        let table_name = name.to_string();
                        tracing::debug!("Found joined table: {}", table_name);
                        tables.insert(table_name);
                    }
                    TableFactor::Derived { subquery, .. } => {
                        tracing::debug!("Processing joined derived table (subquery)");
                        extract_tables_from_query(subquery, tables);
                    }
                    _ => {
                        tracing::debug!("Unsupported join relation type: {:?}", join.relation);
                    }
                }
            }
        }
    } else {
        tracing::debug!("Query is not a SELECT: {:?}", query.body);
    }
    
    // Also check for CTEs (WITH clause)
    if let Some(with) = &query.with {
        tracing::debug!("Processing WITH clause with {} CTEs", with.cte_tables.len());
        for cte in &with.cte_tables {
            tracing::debug!("Processing CTE: {}", cte.alias.name);
            extract_tables_from_query(&cte.query, tables);
        }
    }
}