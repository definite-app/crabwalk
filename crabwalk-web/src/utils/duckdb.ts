import * as duckdb from '@duckdb/duckdb-wasm';
import {
  AsyncDuckDB,
  AsyncDuckDBConnection,
  ConsoleLogger,
  LogLevel,
} from '@duckdb/duckdb-wasm';

// Initialize DuckDB Web Assembly
let db: AsyncDuckDB | null = null;
let initializing = false;
let initializationError: Error | null = null;

// Table metadata cache
export interface TableInfo {
  name: string;         // Quoted name for SQL queries
  displayName?: string; // Unquoted name for UI display
  rowCount: number;
  columnCount: number;
  columns: ColumnInfo[];
  sizeBytes?: number;
  description?: string;
}

export interface ColumnInfo {
  name: string;
  type: string;
  nullable: boolean;
}

const tableCache = new Map<string, TableInfo>();

// Initialize DuckDB
export const initDuckDB = async (): Promise<AsyncDuckDB> => {
  // If already initialized successfully, return the instance
  if (db) return db;
  
  // If there was a previous initialization error, throw it again
  if (initializationError) {
    console.error('Returning previous initialization error:', initializationError);
    throw initializationError;
  }
  
  // If currently initializing, wait for completion
  if (initializing) {
    console.log('Waiting for ongoing initialization...');
    return new Promise((resolve, reject) => {
      const checkInterval = setInterval(() => {
        if (db) {
          clearInterval(checkInterval);
          resolve(db);
        } else if (initializationError) {
          clearInterval(checkInterval);
          reject(initializationError);
        }
      }, 100);
    });
  }

  initializing = true;
  console.log('Starting DuckDB initialization...');

  try {
    // Get bundles from jsDelivr CDN
    const JSDELIVR_BUNDLES = await duckdb.getJsDelivrBundles();
    const bundle = await duckdb.selectBundle(JSDELIVR_BUNDLES);
    
    console.log('Selected bundle:', bundle);

    // Create a logger for DuckDB messages
    const logger = new ConsoleLogger(LogLevel.WARNING);
    
    // Create a Web Worker using a Blob URL to avoid CORS issues
    const workerUrl = URL.createObjectURL(
      new Blob([`importScripts("${bundle.mainWorker}");`], { type: 'text/javascript' })
    );
    
    // Create the Web Worker
    const worker = new Worker(workerUrl);
    
    // Create DuckDB instance with the worker
    db = new AsyncDuckDB(logger, worker);
    
    // Instantiate DuckDB with the WASM module
    await db.instantiate(bundle.mainModule, bundle.pthreadWorker);
    
    // Clean up the Blob URL
    URL.revokeObjectURL(workerUrl);
    
    console.log('DuckDB instantiated successfully');

    // Open the database
    await db.open({
      path: ':memory:',
      query: {
        castTimestampToDate: true,
      },
    });

    // Register HTTPFS to load files from URLs
    const conn = await db.connect();
    await conn.query(`INSTALL httpfs; LOAD httpfs;`);
    await conn.close();

    initializing = false;
    console.log('DuckDB initialized and configured successfully');
    return db;
  } catch (error: any) {
    initializing = false;
    initializationError = error instanceof Error 
      ? error 
      : new Error(String(error));
    
    console.error('Error initializing DuckDB:', error);
    throw error;
  }
};

// Helper function to get a database connection
export const getConnection = async (): Promise<AsyncDuckDBConnection> => {
  const duckdb = await initDuckDB();
  return duckdb.connect();
};

// Load SQL from a URL into DuckDB
export const loadSqlFromUrl = async (
  url: string,
): Promise<void> => {
  try {
    // Initialize DuckDB if not already done
    const conn = await getConnection();
    
    try {
      // Fetch the SQL file content
      const response = await fetch(url);
      const sqlContent = await response.text();
      
      // Execute the SQL query
      await conn.query(sqlContent);
      
      // Update the table cache after loading
      await refreshTableCache(conn);
    } finally {
      // Always close the connection
      await conn.close();
    }
  } catch (error) {
    console.error(`Error loading SQL from ${url}:`, error);
    throw error;
  }
};

// Load a local database file
export const loadDatabaseFile = async (file: File): Promise<void> => {
  try {
    const duckdb = await initDuckDB();
    const arrayBuffer = await file.arrayBuffer();
    
    // Register the file with DuckDB
    await duckdb.registerFileBuffer(
      file.name,
      new Uint8Array(arrayBuffer)
    );
    
    // Attach the database
    const conn = await duckdb.connect();
    try {
      await conn.query(`ATTACH '${file.name}' AS imported;`);
      console.log(`Database ${file.name} attached as 'imported'`);
      
      // Update the table cache after loading
      await refreshTableCache(conn);
    } finally {
      await conn.close();
    }
  } catch (error) {
    console.error(`Error loading database file ${file.name}:`, error);
    throw error;
  }
};

// Helper function to convert BigInt values to regular numbers
const convertBigIntToNumber = (value: any): any => {
  if (typeof value === 'bigint') {
    // Convert BigInt to Number if it's within safe integer range
    if (value <= Number.MAX_SAFE_INTEGER && value >= Number.MIN_SAFE_INTEGER) {
      return Number(value);
    }
    // For values outside safe range, convert to string to preserve precision
    return value.toString();
  }
  
  if (Array.isArray(value)) {
    return value.map(convertBigIntToNumber);
  }
  
  if (value !== null && typeof value === 'object') {
    const result: Record<string, any> = {};
    for (const key in value) {
      result[key] = convertBigIntToNumber(value[key]);
    }
    return result;
  }
  
  return value;
};

// Execute a SQL query and return the results as JSON objects
export const executeQuery = async (
  query: string
): Promise<any[]> => {
  try {
    // Get a connection
    const conn = await getConnection();
    
    try {
      console.log(`Executing query: ${query}`);
      const result = await conn.query(query);
      
      // Debug result structure
      console.log('DuckDB result object:', result);
      console.log('DuckDB schema:', result.schema);
      console.log('DuckDB fields:', result.schema.fields);
      
      // Try different ways to get the data
      let resultArray: any[] = [];
      
      try {
        // Try to see if we have access to an arrow batches internal array 
        if (result.batches && Array.isArray(result.batches) && result.batches.length > 0) {
          console.log('Detected Arrow batches format, attempting to extract directly');
          
          // Extract data from arrow batches - specific to DuckDB-wasm
          const allRows: any[] = [];
          const fieldCount = result.schema.fields.length;
          
          for (const batch of result.batches) {
            if (batch.data && Array.isArray(batch.data)) {
              // Each batch has column data
              for (let rowIdx = 0; rowIdx < batch.numRows; rowIdx++) {
                const rowObj: Record<string, any> = {};
                
                for (let colIdx = 0; colIdx < fieldCount; colIdx++) {
                  const field = result.schema.fields[colIdx];
                  const colData = batch.data[colIdx];
                  
                  if (colData && colData.values) {
                    rowObj[field.name] = colData.values[rowIdx];
                  } else {
                    rowObj[field.name] = null;
                  }
                }
                
                allRows.push(rowObj);
              }
            }
          }
          
          if (allRows.length > 0) {
            console.log('Successfully extracted from Arrow batches');
            resultArray = allRows;
          } else {
            // Fall back to standard method
            resultArray = result.toArray();
            console.log('Using standard toArray() method');
          }
        } else {
          // Standard toArray method
          resultArray = result.toArray();
          console.log('Using standard toArray() method');
        }
      } catch (err) {
        console.warn('Error using toArray():', err);
        
        try {
          // Try accessing through vectorLength and getChildAt using type assertion
          // @ts-ignore - Using dynamic property access for compatibility
          if (typeof result.vectorLength === 'number' && typeof result.getChildAt === 'function') {
            console.log('Using vectorLength/getChildAt approach');
            // @ts-ignore - Using dynamic property access for compatibility
            const len = result.vectorLength;
            resultArray = Array(len);
            
            for (let i = 0; i < len; i++) {
              // @ts-ignore - Using dynamic property access for compatibility
              resultArray[i] = result.getChildAt(i);
            }
          // @ts-ignore - Using dynamic property access for compatibility
          } else if (typeof result.getChild === 'function' && typeof result.numChildren === 'function') {
            console.log('Using getChild/numChildren approach');
            // @ts-ignore - Using dynamic property access for compatibility
            const len = result.numChildren();
            resultArray = Array(len);
            
            for (let i = 0; i < len; i++) {
              // @ts-ignore - Using dynamic property access for compatibility
              resultArray[i] = result.getChild(i);
            }
          } else if (result.data && Array.isArray(result.data)) {
            console.log('Using direct data property');
            resultArray = result.data;
          }
        } catch (err2) {
          console.error('Error with alternative data extraction:', err2);
        }
      }
      
      console.log('DuckDB raw rows:', resultArray);
      
      if (resultArray.length > 0) {
        console.log('First raw row type:', typeof resultArray[0]);
        console.log('First raw row is array?', Array.isArray(resultArray[0]));
        console.log('First raw row:', resultArray[0]);
        
        if (typeof resultArray[0] === 'object' && resultArray[0] !== null) {
          console.log('First row properties:', Object.getOwnPropertyNames(resultArray[0]));
          console.log('First row prototype:', Object.getPrototypeOf(resultArray[0]));
          
          // Try to get methods
          const methods: string[] = [];
          for (const prop in resultArray[0]) {
            if (typeof (resultArray[0] as any)[prop] === 'function') {
              methods.push(prop);
            }
          }
          console.log('First row methods:', methods);
        }
      }
      
      // Try to determine if we're using an Arrow format or direct format
      const isArrowFormat = typeof result.getChild === 'function' || 
                          typeof result.toArray !== 'function' ||
                          (resultArray.length > 0 && Array.isArray(resultArray[0]) === false);
                          
      console.log('Is Arrow format?', isArrowFormat);
      
      // Try different approaches to convert data
      let mappedResults: Record<string, any>[] = [];
      
      try {
        // First approach: standard mapping
        mappedResults = resultArray.map((row: any) => {
          const obj: Record<string, any> = {};
          
          // Handle both array and object formats
          if (Array.isArray(row)) {
            // Standard format - array of values
            for (let i = 0; i < result.schema.fields.length; i++) {
              const fieldName = result.schema.fields[i].name;
              const value = row[i];
              obj[fieldName] = value;
              
              // Debug first row mapping
              if (row === resultArray[0]) {
                console.log(`Mapping field[${i}] ${fieldName} = ${value}, type: ${typeof value}`);
              }
            }
          } else if (typeof row === 'object' && row !== null) {
            // Object format - may already have property names
            const keys = Object.keys(row);
            if (keys.length > 0) {
              // If row already has named properties, use them directly
              return row;
            } else {
              // Try accessing by field name
              for (let i = 0; i < result.schema.fields.length; i++) {
                const fieldName = result.schema.fields[i].name;
                const value = (row as any)[fieldName];
                obj[fieldName] = value;
                
                // Debug first row mapping
                if (row === resultArray[0]) {
                  console.log(`Direct mapping field ${fieldName} = ${value}, type: ${typeof value}`);
                }
              }
            }
          }
          
          return obj;
        });
        
        // Check if we got data properly
        if (mappedResults.length > 0 && Object.values(mappedResults[0]).every(v => v === null || v === undefined)) {
          console.warn('First approach produced all null values, trying alternative approach');
          throw new Error('All null values');
        }
      } catch (err) {
        console.warn('Error in first mapping approach:', err);
        
        // Second approach: try with field indices and Arrow format handling
        try {
          // Check if we might be dealing with an Arrow format where columns are separate arrays
          // Safer dynamic property checking with type casting
          const dynamicResult = result as any;
          if (result.schema && result.schema.fields && result.schema.fields.length > 0 &&
              typeof dynamicResult.getColumn === 'function') {
            
            console.log('Detected possible Arrow column-wise format, trying column extraction');
            
            // Extract all columns first
            const columns: any[][] = [];
            for (let i = 0; i < result.schema.fields.length; i++) {
              try {
                // Use the dynamic result variable
                const col = dynamicResult.getColumn(i);
                columns.push(col);
                console.log(`Column ${i} (${result.schema.fields[i].name}) length:`, col.length);
                if (col.length > 0) {
                  console.log(`Column ${i} first value:`, col[0]);
                }
              } catch (colErr) {
                console.warn(`Error getting column ${i}:`, colErr);
                columns.push([]);
              }
            }
            
            // Now create row objects from the columns
            const rowCount = columns[0]?.length || 0;
            mappedResults = [];
            
            for (let rowIdx = 0; rowIdx < rowCount; rowIdx++) {
              const obj: Record<string, any> = {};
              
              for (let colIdx = 0; colIdx < result.schema.fields.length; colIdx++) {
                const fieldName = result.schema.fields[colIdx].name;
                const colArray = columns[colIdx] || [];
                obj[fieldName] = colIdx < colArray.length ? colArray[rowIdx] : null;
                
                // Debug the first row
                if (rowIdx === 0) {
                  console.log(`Column-wise extraction: row[0].${fieldName} = ${obj[fieldName]}`);
                }
              }
              
              mappedResults.push(obj);
            }
            
          } else {
            // Row-wise approach
            mappedResults = resultArray.map((row: any, rowIndex: number) => {
              const obj: Record<string, any> = {};
              
              // For each field in the schema
              for (let i = 0; i < result.schema.fields.length; i++) {
                const fieldName = result.schema.fields[i].name;
                
                // Try different ways to access the value
                let value: any = null;
                
                if (typeof row.get === 'function') {
                  // Try Arrow-style get method
                  value = row.get(i);
                } else if (typeof row.getChildAt === 'function') {
                  // Try Arrow-style getChildAt method
                  value = row.getChildAt(i);
                } else if (Array.isArray(row)) {
                  // Direct array access
                  value = row[i];
                } else if (typeof row === 'object' && row !== null) {
                  // Try direct property access
                  value = row[fieldName];
                }
                
                obj[fieldName] = value;
                
                // Debug the mapping for first row
                if (rowIndex === 0) {
                  console.log(`Alternative mapping field[${i}] ${fieldName} = ${value}, type: ${typeof value}`);
                }
              }
              
              return obj;
            });
          }
        } catch (err2) {
          console.error('Error in alternative mapping approach:', err2);
          // Fallback to original array
          mappedResults = resultArray;
        }
      }
      
      // Debug mapped results
      console.log('Mapped results:', mappedResults);
      if (mappedResults.length > 0) {
        console.log('First mapped row:', mappedResults[0]);
        console.log('First mapped row keys:', Object.keys(mappedResults[0]));
        console.log('First mapped row values:', Object.values(mappedResults[0]));
      }
      
      // Convert any BigInt values to regular numbers before returning
      const convertedResults = convertBigIntToNumber(mappedResults);
      return convertedResults;
    } finally {
      await conn.close();
    }
  } catch (error) {
    console.error(`Error executing query: ${query}`, error);
    throw error;
  }
};

// Safely quote an SQL identifier (table or column name)
export const quoteIdentifier = (id: string | null | undefined): string => {
  // Handle null/undefined case
  if (id === null || id === undefined) {
    return '""';
  }
  
  // Ensure id is a string
  const idStr = String(id);
  
  // Replace any embedded quotes with double quotes (SQL standard)
  return `"${idStr.replace(/"/g, '""')}"`;
};

// Get all available tables across all schemas
export const listTables = async (): Promise<TableInfo[]> => {
  try {
    const conn = await getConnection();
    
    try {
      // Refresh the cache before returning
      await refreshTableCache(conn);
      
      // Return the cached tables
      return Array.from(tableCache.values());
    } finally {
      await conn.close();
    }
  } catch (error) {
    console.error('Error listing tables:', error);
    throw error;
  }
};

// Get table statistics
export const getTableStats = async (tableName: string): Promise<TableInfo> => {
  // Add debugging to help identify where problematic table names are coming from
  if (tableName.includes('table_name,') || tableName.includes('table_schema,')) {
    console.warn(`Problematic table name detected: "${tableName}"`);
    console.trace('Stack trace for problematic table name');
  }
  
  // Check if the input has the problematic prefixes and clean it
  const { database, schema, table } = parseTableIdentifier(tableName);
  
  // Construct a clean table identifier for cache lookup
  let cleanTableId = table;
  if (schema) {
    cleanTableId = `${schema}.${table}`;
  }
  if (database) {
    cleanTableId = `${database}.${cleanTableId}`;
  }
  
  // If we have cached info, return it
  if (tableCache.has(cleanTableId)) {
    return tableCache.get(cleanTableId)!;
  }
  
  // Also check the original tableName in the cache (for backward compatibility)
  if (tableCache.has(tableName)) {
    return tableCache.get(tableName)!;
  }
  
  try {
    const conn = await getConnection();
    
    try {
      // Get the column information
      let columnsQuery: string;
      
      if (database && schema) {
        // If we have both database and schema, use them in the query
        const safeDatabaseStr = String(database).replace(/'/g, "''");
        const safeSchemaStr = String(schema).replace(/'/g, "''");
        const safeTableStr = String(table).replace(/'/g, "''");
        columnsQuery = `
          SELECT column_name, data_type, is_nullable
          FROM information_schema.columns
          WHERE table_catalog = '${safeDatabaseStr}' AND table_schema = '${safeSchemaStr}' AND table_name = '${safeTableStr}'
          ORDER BY ordinal_position
        `;
      } else if (schema) {
        // If we have just a schema, use it in the query
        const safeSchemaStr = String(schema).replace(/'/g, "''");
        const safeTableStr = String(table).replace(/'/g, "''");
        columnsQuery = `
          SELECT column_name, data_type, is_nullable
          FROM information_schema.columns
          WHERE table_schema = '${safeSchemaStr}' AND table_name = '${safeTableStr}'
          ORDER BY ordinal_position
        `;
      } else {
        // Otherwise just use the table name
        columnsQuery = `
          SELECT column_name, data_type, is_nullable
          FROM information_schema.columns
          WHERE table_name = '${String(table).replace(/'/g, "''")}'
          ORDER BY ordinal_position
        `;
      }
      
      const columnsResult = await conn.query(columnsQuery);
      
      // Convert the result to column info
      const columns: ColumnInfo[] = columnsResult.toArray().map((row: any) => {
        // Handle both array and object formats
        if (Array.isArray(row)) {
          return {
            name: String(row[0]),
            type: String(row[1]),
            nullable: String(row[2]) === 'YES',
          };
        } else if (typeof row === 'object' && row !== null) {
          return {
            name: String(row.column_name || row[0]),
            type: String(row.data_type || row[1]),
            nullable: String(row.is_nullable || row[2]) === 'YES',
          };
        } else {
          // Default fallback
          return {
            name: 'unknown',
            type: 'unknown',
            nullable: false,
          };
        }
      });
      
      // Get row count - use a properly escaped table name
      let queryStr: string;
      if (schema) {
        // Has schema, properly quote both parts
        queryStr = `SELECT COUNT(*) FROM ${quoteIdentifier(schema)}.${quoteIdentifier(table)}`;
      } else {
        // No schema, just quote the table name
        queryStr = `SELECT COUNT(*) FROM ${quoteIdentifier(table)}`;
      }
      
      const countResult = await conn.query(queryStr);
      const countRow = countResult.toArray()[0];
      const rowCount = Number(Array.isArray(countRow) ? countRow[0] : countRow.count);
      
      // Create the table info
      const tableInfo: TableInfo = {
        name: schema ? `${quoteIdentifier(schema)}.${quoteIdentifier(table)}` : quoteIdentifier(table),
        displayName: schema ? `${schema}.${table}` : table,
        rowCount,
        columnCount: columns.length,
        columns,
      };
      
      // Cache the info using the clean table identifier
      tableCache.set(cleanTableId, tableInfo);
      
      return tableInfo;
    } finally {
      await conn.close();
    }
  } catch (error) {
    console.error(`Error getting stats for table ${tableName}:`, error);
    throw error;
  }
};

// Get columns for a table
export const getTableColumns = async (tableName: string): Promise<ColumnInfo[]> => {
  // Add debugging to help identify where problematic table names are coming from
  if (tableName.includes('table_name,') || tableName.includes('table_schema,')) {
    console.warn(`Problematic table name detected in getTableColumns: "${tableName}"`);
    console.trace('Stack trace for problematic table name in getTableColumns');
  }
  
  // Check if the input has the problematic prefixes and clean it
  const { database, schema, table } = parseTableIdentifier(tableName);
  
  // Construct a clean table identifier for cache lookup
  let cleanTableId = table;
  if (schema) {
    cleanTableId = `${schema}.${table}`;
  }
  if (database) {
    cleanTableId = `${database}.${cleanTableId}`;
  }
  
  // If we have cached info, return it
  if (tableCache.has(cleanTableId)) {
    return tableCache.get(cleanTableId)!.columns;
  }
  
  // Also check the original tableName in the cache (for backward compatibility)
  if (tableCache.has(tableName)) {
    return tableCache.get(tableName)!.columns;
  }
  
  try {
    const tableInfo = await getTableStats(tableName);
    return tableInfo.columns;
  } catch (error) {
    console.error(`Error getting columns for table ${tableName}:`, error);
    throw error;
  }
};

// Refresh the table cache
async function refreshTableCache(conn: AsyncDuckDBConnection): Promise<void> {
  // Get all tables, including database information
  const tablesResult = await conn.query(`
    SELECT 
      (CASE WHEN table_catalog <> 'memory' THEN table_catalog ELSE NULL END) as database_name,
      table_schema,
      table_name
    FROM information_schema.tables
    WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
    ORDER BY table_catalog, table_schema, table_name
  `);
  
  // Debug: Log the raw result structure
  console.log('Tables result structure:', tablesResult);
  
  // Get the array of rows
  const tablesArray = tablesResult.toArray();
  console.log('Tables array:', tablesArray);
  
  // Process each table
  // The result might be an array of arrays or an array of objects
  // We need to handle both cases
  for (let i = 0; i < tablesArray.length; i++) {
    const row = tablesArray[i];
    
    // Extract database, schema and table name from the row
    // If row is an array, use indices; if it's an object with properties, use property names
    let database: string | null = null;
    let schema: string;
    let table: string;
    
    if (Array.isArray(row)) {
      // Row is an array [database, schema, table]
      database = row[0] ? String(row[0]) : null;
      schema = String(row[1]);
      table = String(row[2]);
    } else if (typeof row === 'object' && row !== null) {
      // Row is an object, try to extract database, schema and table
      // Check for common patterns in the property names
      if ('table_schema' in row && 'table_name' in row) {
        database = 'database_name' in row && row.database_name ? String(row.database_name) : null;
        schema = String(row.table_schema);
        table = String(row.table_name);
      } else {
        // If we can't determine the schema and table, skip this row
        console.warn('Unknown row format in tables result:', row);
        continue;
      }
    } else {
      // If we can't determine the schema and table, skip this row
      console.warn('Unknown row format in tables result:', row);
      continue;
    }
    
    // Debug: Log each database, schema and table
    console.log('Processing table - database:', database, 'schema:', schema, 'table:', table);
    
    // Store the properly quoted name for SQL queries
    let quotedName: string;
    if (database) {
      quotedName = `${quoteIdentifier(database)}.${quoteIdentifier(schema)}.${quoteIdentifier(table)}`;
    } else if (schema === 'main') {
      quotedName = quoteIdentifier(table);
    } else {
      quotedName = `${quoteIdentifier(schema)}.${quoteIdentifier(table)}`;
    }
    
    // Store the display name for UI (unquoted)
    let displayName: string;
    if (database) {
      displayName = `${database}.${schema}.${table}`;
    } else if (schema === 'main') {
      displayName = table;
    } else {
      displayName = `${schema}.${table}`;
    }
    
    try {
      // Get column information - safely escape single quotes
      const safeDatabaseStr = database ? String(database).replace(/'/g, "''") : null;
      const safeSchemaStr = schema ? String(schema).replace(/'/g, "''") : '';
      const safeTableStr = table ? String(table).replace(/'/g, "''") : '';
      
      let columnsQuery: string;
      if (safeDatabaseStr) {
        columnsQuery = `
          SELECT column_name, data_type, is_nullable
          FROM information_schema.columns
          WHERE table_catalog = '${safeDatabaseStr}' AND table_schema = '${safeSchemaStr}' AND table_name = '${safeTableStr}'
          ORDER BY ordinal_position
        `;
      } else {
        columnsQuery = `
          SELECT column_name, data_type, is_nullable
          FROM information_schema.columns
          WHERE table_schema = '${safeSchemaStr}' AND table_name = '${safeTableStr}'
          ORDER BY ordinal_position
        `;
      }
      
      const columnsResult = await conn.query(columnsQuery);
      
      // Convert the result to column info
      const columns: ColumnInfo[] = columnsResult.toArray().map((row: any) => {
        // Handle both array and object formats
        if (Array.isArray(row)) {
          return {
            name: String(row[0]),
            type: String(row[1]),
            nullable: String(row[2]) === 'YES',
          };
        } else if (typeof row === 'object' && row !== null) {
          return {
            name: String(row.column_name || row[0]),
            type: String(row.data_type || row[1]),
            nullable: String(row.is_nullable || row[2]) === 'YES',
          };
        } else {
          // Default fallback
          return {
            name: 'unknown',
            type: 'unknown',
            nullable: false,
          };
        }
      });
      
      // Get row count (try/catch in case table has issues)
      let rowCount = 0;
      try {
        // Use the properly quoted name for the COUNT query
        const countResult = await conn.query(`SELECT COUNT(*) FROM ${quotedName}`);
        const countRow = countResult.toArray()[0];
        rowCount = Number(Array.isArray(countRow) ? countRow[0] : countRow.count);
      } catch (err) {
        console.warn(`Error getting row count for ${displayName}:`, err);
      }
      
      // Create the table info
      const tableInfo: TableInfo = {
        name: quotedName,    // Store quoted name for SQL queries
        displayName,         // Store unquoted name for UI display
        rowCount,
        columnCount: columns.length,
        columns,
      };
      
      // Cache the info - use displayName as the key
      tableCache.set(displayName, tableInfo);
    } catch (err) {
      console.warn(`Error processing table ${displayName}:`, err);
    }
  }
}

// Helper function to parse table names and handle special cases
export const parseTableIdentifier = (tableName: string): { database: string | null, schema: string | null, table: string } => {
  let database: string | null = null;
  let schema: string | null = null;
  let table: string = tableName;
  
  // Check if the table name has the "table_name," prefix
  if (tableName.startsWith('table_name,')) {
    // Remove the "table_name," prefix
    table = tableName.substring('table_name,'.length);
    console.log(`Removed "table_name," prefix from table: ${table}`);
    return { database, schema, table };
  }
  
  // Check for the pattern "table_schema,X.table_name,Y"
  const compoundPattern = /^table_schema,([^.]+)\.table_name,(.+)$/;
  const match = tableName.match(compoundPattern);
  
  if (match) {
    // Extract the actual schema and table from the compound pattern
    schema = match[1];
    table = match[2];
    console.log(`Extracted from compound pattern - schema: ${schema}, table: ${table}`);
    return { database, schema, table };
  }
  
  // Check if the schema has the "table_schema," prefix
  if (tableName.includes('.')) {
    const parts = tableName.split('.');
    let potentialSchema = parts[0];
    
    if (potentialSchema.startsWith('table_schema,')) {
      // Remove the "table_schema," prefix
      schema = potentialSchema.substring('table_schema,'.length);
      table = parts.slice(1).join('.');
      console.log(`Removed "table_schema," prefix from schema: ${schema}, table: ${table}`);
      return { database, schema, table };
    }
  }
  
  // Handle database.schema.table format (three parts)
  const parts = tableName.split('.');
  if (parts.length === 3) {
    database = parts[0];
    schema = parts[1];
    table = parts[2];
    console.log(`Parsed three-part identifier - database: ${database}, schema: ${schema}, table: ${table}`);
    return { database, schema, table };
  }
  
  // Handle the normal case - check for schema.table format (two parts)
  if (parts.length === 2) {
    schema = parts[0];
    table = parts[1];
    console.log(`Parsed two-part identifier - schema: ${schema}, table: ${table}`);
    return { database, schema, table };
  }
  
  // Single part - just a table name
  return { database, schema, table };
};

// Types are already exported above