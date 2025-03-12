import React, { useEffect, useRef, useState } from 'react';
import { executeQuery } from '../utils/duckdb';
import './TableViewer.css';

// Add TypeScript declaration for Perspective
declare global {
  interface Window {
    perspective: {
      worker: () => Promise<any>;
      [key: string]: any;
    };
  }
}

// Reference the PerspectiveViewerElement type from the types file
// This is defined in src/types/perspective.d.ts
type PerspectiveViewerElement = any;

interface TableViewerProps {
  tableName: string;
  sqlQuery?: string;
  onClose?: () => void;
}

const styles = {
  container: {
    position: 'fixed' as const,
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    zIndex: 1000,
  },
  content: {
    position: 'relative' as const,
    width: '90%',
    height: '90%',
    backgroundColor: 'white',
    borderRadius: '8px',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.2)',
    display: 'flex',
    flexDirection: 'column' as const,
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    padding: '16px 24px',
    borderBottom: '1px solid #e5e7eb',
  },
  title: {
    margin: 0,
    fontSize: '1.25rem',
    fontWeight: 500,
  },
  stats: {
    marginLeft: 'auto',
    fontSize: '0.875rem',
    color: '#6b7280',
  },
  closeButton: {
    marginLeft: '16px',
    background: 'none',
    border: 'none',
    fontSize: '24px',
    cursor: 'pointer',
    color: '#6b7280',
  },
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

const TableViewer: React.FC<TableViewerProps> = ({ tableName, sqlQuery, onClose }) => {
  const viewerRef = useRef<PerspectiveViewerElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rowCount, setRowCount] = useState<number | null>(null);
  const [columnCount, setColumnCount] = useState<number | null>(null);
  
  // Default data to use when the dataset is empty
  const defaultData = {
    id: [1, 2, 3],
    name: ['Default Item 1', 'Default Item 2', 'Default Item 3'],
    value: [100, 200, 300],
    created_at: [new Date().toISOString(), new Date().toISOString(), new Date().toISOString()]
  };
  
  // Handle clicks outside the content area
  const handleContainerClick = (e: React.MouseEvent<HTMLDivElement>) => {
    // Only close if clicking the backdrop (container) and not the content
    if (onClose && contentRef.current && !contentRef.current.contains(e.target as Node)) {
      onClose();
    }
  };
  
  // Load data when component mounts
  useEffect(() => {
    const loadData = async () => {
      setLoading(false);
      setError(null);
      
      try {
        // Check if perspective is available (it should be loaded from index.html)
        if (!window.perspective) {
          throw new Error(
            'Perspective library is not available. This might be due to loading issues. ' +
            'Try refreshing the page or check the console for more details.'
          );
        }
        
        // Create or use a query to get the data
        let query: string;
        
        if (sqlQuery) {
          query = sqlQuery;
        } else {
          query = `SELECT * FROM ${tableName}`;
        }
        
        // Execute the query using our configured database driver
        console.log(`Executing query: ${query}`);
        const rows = await executeQuery(query);
        
        // Convert any BigInt values to regular numbers
        const convertedRows = convertBigIntToNumber(rows);
        
        // Get the result data for perspective
        let table;
        try {
          // Create column-oriented data structure for Perspective
          let columnData: Record<string, any[]> = {};
          
          // If we have results
          if (convertedRows.length > 0) {
            // Get column names from the first row
            const columnNames = Object.keys(convertedRows[0]);
            
            // Initialize the column arrays
            columnNames.forEach(colName => {
              columnData[colName] = convertedRows.map((row: Record<string, any>) => row[colName]);
            });
          } else {
            // Use default data if the result set is empty
            console.log('No data returned from query, using default data');
            columnData = defaultData;
          }
          
          // Create the table using the globally loaded perspective
          console.log('Creating Perspective table with data');
          const worker = await window.perspective.worker();
          table = await worker.table(columnData);
        } catch (error) {
          console.error('Error creating Perspective table:', error);
          throw new Error(`Failed to create Perspective table: ${error instanceof Error ? error.message : String(error)}`);
        }
        
        // Calculate row and column count
        const schema = await table.schema();
        setColumnCount(Object.keys(schema).length);
        
        const tableSize = await table.size();
        setRowCount(tableSize);
        
        // Load the data into the viewer
        if (viewerRef.current) {
          // await viewerRef.current.reset();
          viewerRef.current.toggleConfig();
          await viewerRef.current.load(table);
          
          // Configure the viewer
          viewerRef.current.toggleConfig();
        }
        
        setLoading(false);
      } catch (err) {
        console.error('Error loading data:', err);
        setError(`Failed to load data: ${err instanceof Error ? err.message : String(err)}`);
        setLoading(false);
      }
    };
    
    loadData();
  }, [tableName, sqlQuery]);
  
  return (
    <div style={styles.container} onClick={handleContainerClick}>
      <div style={styles.content} ref={contentRef}>
        <div style={styles.header}>
          <h2 style={styles.title}>{tableName}</h2>
          {rowCount !== null && columnCount !== null && (
            <div style={styles.stats}>
              {rowCount.toLocaleString()} rows × {columnCount} columns
            </div>
          )}
          <button style={styles.closeButton} onClick={onClose}>
            ×
          </button>
        </div>
        
        {error && (
          <div style={{ 
            padding: '20px', 
            color: 'red', 
            backgroundColor: '#ffeeee', 
            borderRadius: '4px',
            margin: '10px 0',
            border: '1px solid #ffcccc'
          }}>
            <h3>Error</h3>
            <p>{error}</p>
            <p>
              <button 
                onClick={() => window.location.reload()} 
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#f44336',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer'
                }}
              >
                Reload Page
              </button>
            </p>
          </div>
        )}
        
        {loading && !error && (
          <div style={{ padding: '20px', textAlign: 'center' }}>
            <p>Loading data...</p>
          </div>
        )}
        
        <div style={{ 
          display: loading || error ? 'none' : 'block',
          width: '100%', 
          height: '500px' 
        }}>
          {/* @ts-ignore - Using custom element */}
          <perspective-viewer 
            ref={viewerRef} 
            style={{ marginTop: '68px' }}
          ></perspective-viewer>
        </div>
      </div>
    </div>
  );
};

export default TableViewer;