import initSqlJs, { Database, SqlJsStatic } from 'sql.js';

// Types to match DuckDB interface
import { TableInfo, ColumnInfo } from './duckdb';

let SQL: SqlJsStatic | null = null;
let db: Database | null = null;
const tableCache = new Map<string, TableInfo>();

// Load SQL.js
export const initSqlite = async (): Promise<SqlJsStatic> => {
  if (SQL) return SQL;
  
  try {
    console.log('Initializing SQL.js fallback...');
    SQL = await initSqlJs({
      // Attempt to load from CDN if local fails
      locateFile: (file: string) => `https://cdnjs.cloudflare.com/ajax/libs/sql.js/1.8.0/${file}`
    });
    console.log('SQL.js initialized successfully');
    return SQL;
  } catch (error) {
    console.error('Failed to initialize SQL.js:', error);
    throw error;
  }
};

// Load database file
export const loadDatabaseFile = async (file: File): Promise<void> => {
  try {
    // Initialize SQL.js
    const SQL = await initSqlite();
    
    // Read file as array buffer
    const arrayBuffer = await file.arrayBuffer();
    const uInt8Array = new Uint8Array(arrayBuffer);
    
    // Create database from file
    if (db) {
      db.close();
    }
    
    db = new SQL.Database(uInt8Array);
    console.log(`Database ${file.name} loaded successfully with SQL.js`);
    
    // Update table cache
    await refreshTableCache();
  } catch (error) {
    console.error(`Error loading database with SQL.js:`, error);
    throw error;
  }
};

// Execute a SQL query
export const executeQuery = async (query: string): Promise<any[]> => {
  if (!db) {
    throw new Error('No database loaded. Please load a database file first.');
  }
  
  try {
    console.log(`Executing query with SQL.js: ${query}`);
    const results = db.exec(query);
    
    if (results.length === 0) {
      return [];
    }
    
    // Convert SQL.js format to our format
    const rows = results[0].values.map((row: any[]) => {
      const obj: Record<string, any> = {};
      results[0].columns.forEach((col: string, i: number) => {
        obj[col] = row[i];
      });
      return obj;
    });
    
    return rows;
  } catch (error) {
    console.error(`Error executing query: ${query}`, error);
    throw error;
  }
};

// List all tables
export const listTables = async (): Promise<TableInfo[]> => {
  if (!db) {
    return [];
  }
  
  try {
    // Refresh the cache before returning
    await refreshTableCache();
    
    // Return the cached tables
    return Array.from(tableCache.values());
  } catch (error) {
    console.error('Error listing tables:', error);
    throw error;
  }
};

// Get table statistics
export const getTableStats = async (tableName: string): Promise<TableInfo> => {
  if (tableCache.has(tableName)) {
    return tableCache.get(tableName)!;
  }
  
  if (!db) {
    throw new Error('No database loaded');
  }
  
  try {
    // Get column information
    const pragma = db.exec(`PRAGMA table_info(${tableName})`);
    
    if (!pragma.length || !pragma[0].values.length) {
      throw new Error(`Table ${tableName} not found`);
    }
    
    const columns: ColumnInfo[] = pragma[0].values.map((row: any[]) => ({
      name: row[1],
      type: row[2],
      nullable: row[3] === 0, // notnull is 1 when NOT NULL, 0 when nullable
    }));
    
    // Get row count
    const countResult = db.exec(`SELECT COUNT(*) FROM ${tableName}`);
    const rowCount = Number(countResult[0].values[0][0] || 0);
    
    // Create table info
    const tableInfo: TableInfo = {
      name: tableName,
      rowCount,
      columnCount: columns.length,
      columns,
    };
    
    // Cache the info
    tableCache.set(tableName, tableInfo);
    
    return tableInfo;
  } catch (error) {
    console.error(`Error getting stats for table ${tableName}:`, error);
    throw error;
  }
};

// Get columns for a table
export const getTableColumns = async (tableName: string): Promise<ColumnInfo[]> => {
  const tableInfo = await getTableStats(tableName);
  return tableInfo.columns;
};

// Helper to refresh table cache
async function refreshTableCache(): Promise<void> {
  if (!db) return;
  
  try {
    // Clear existing cache
    tableCache.clear();
    
    // Get list of all tables
    const tablesQuery = `
      SELECT name FROM sqlite_master 
      WHERE type='table' AND name NOT LIKE 'sqlite_%'
    `;
    
    const tablesResult = db.exec(tablesQuery);
    
    if (!tablesResult.length) {
      return;
    }
    
    const tables = tablesResult[0].values.map((row: any[]) => row[0]);
    
    // Process each table
    for (const tableName of tables) {
      try {
        // Get column information
        const pragma = db.exec(`PRAGMA table_info(${tableName})`);
        
        const columns: ColumnInfo[] = pragma[0].values.map((row: any[]) => ({
          name: row[1],
          type: row[2],
          nullable: row[3] === 0,
        }));
        
        // Get row count
        const countResult = db.exec(`SELECT COUNT(*) FROM ${tableName}`);
        const rowCount = Number(countResult[0].values[0][0] || 0);
        
        // Create the table info
        const tableInfo: TableInfo = {
          name: tableName,
          rowCount,
          columnCount: columns.length,
          columns,
        };
        
        // Cache the info
        tableCache.set(tableName, tableInfo);
      } catch (err) {
        console.warn(`Error processing table ${tableName}:`, err);
      }
    }
  } catch (error) {
    console.error('Error refreshing table cache:', error);
  }
}