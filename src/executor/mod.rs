pub mod output;

use anyhow::{Context, Result};
use duckdb::Connection;
use std::path::Path;

/// Connect to DuckDB database
///
/// # Arguments
///
/// * `database_path` - Path to the DuckDB database file
///
/// # Returns
///
/// * `Result<Connection>` - DuckDB connection
pub fn connect_to_duckdb(database_path: &str) -> Result<Connection> {
    let path = Path::new(database_path);
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }
    }
    
    // Connect to DuckDB
    let conn = Connection::open(path)
        .context(format!("Failed to connect to DuckDB database: {}", database_path))?;
    
    Ok(conn)
}

/// Runtime context for SQL execution
pub struct RunContext {
    /// DuckDB connection
    conn: Connection,
}

impl RunContext {
    /// Create a new run context
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    
    /// Execute a SQL statement with environment variable replacement
    pub fn execute(&self, sql: &str) -> Result<()> {
        // Replace environment variables
        let sql_with_env = replace_env_vars(sql)?;
        
        // Execute the SQL without displaying the "error code: 0" messages
        let result = std::panic::catch_unwind(|| {
            // Temporarily redirect stderr to suppress DuckDB's "error code: 0" messages
            let old_stderr = unsafe { libc::dup(libc::STDERR_FILENO) };
            let dev_null = unsafe { libc::open("/dev/null\0".as_ptr() as *const i8, libc::O_WRONLY) };
            unsafe { libc::dup2(dev_null, libc::STDERR_FILENO) };
            
            // Execute the SQL
            let exec_result = self.conn.execute(&sql_with_env, []);
            
            // Restore stderr
            unsafe {
                libc::dup2(old_stderr, libc::STDERR_FILENO);
                libc::close(old_stderr);
                libc::close(dev_null);
            }
            
            exec_result
        });
        
        // Check if the panic occurred
        match result {
            Ok(exec_result) => {
                exec_result.context(format!("Failed to execute SQL: {}", sql_with_env))?;
                Ok(())
            },
            Err(_) => {
                Err(anyhow::anyhow!("SQL execution panicked: {}", sql_with_env))
            }
        }
    }
    
    /// Get the DuckDB connection
    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

/// Replace environment variables in SQL
///
/// # Arguments
///
/// * `sql` - SQL with potential environment variables in the format {{VAR_NAME}}
///
/// # Returns
///
/// * `Result<String>` - SQL with environment variables replaced
fn replace_env_vars(sql: &str) -> Result<String> {
    let re = regex::Regex::new(r"\{\{\s*(\w+)\s*\}\}")
        .context("Failed to compile environment variable regex")?;
    
    let result = re.replace_all(sql, |caps: &regex::Captures| {
        let var_name = &caps[1];
        match std::env::var(var_name) {
            Ok(value) => value,
            Err(_) => {
                tracing::warn!("Environment variable not set: {}", var_name);
                format!("{{{{{}}}}}", var_name) // Return original if not set
            }
        }
    });
    
    Ok(result.to_string())
}