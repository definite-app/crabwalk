import { useEffect, useState } from 'react';
import { listTables, loadDatabaseFile, TableInfo } from '../utils/duckdb';
import TableViewer from './TableViewer';

interface DatabaseExplorerProps {
  className?: string;
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100%',
    padding: '1rem',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '1rem',
  },
  title: {
    fontSize: '1.25rem',
    fontWeight: 600,
    margin: 0,
  },
  uploadButton: {
    backgroundColor: '#2563eb',
    color: 'white',
    border: 'none',
    borderRadius: '0.375rem',
    padding: '0.5rem 1rem',
    fontSize: '0.875rem',
    fontWeight: 500,
    cursor: 'pointer',
  },
  tableList: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
    gap: '1rem',
    flex: 1,
    overflowY: 'auto' as const,
  },
  tableCard: {
    backgroundColor: 'white',
    borderRadius: '0.5rem',
    border: '1px solid #e5e7eb',
    padding: '1rem',
    cursor: 'pointer',
    transition: 'transform 0.1s, box-shadow 0.1s',
    ':hover': {
      transform: 'translateY(-2px)',
      boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
    },
  },
  tableName: {
    fontSize: '1rem',
    fontWeight: 600,
    marginBottom: '0.5rem',
  },
  tableInfo: {
    fontSize: '0.875rem',
    color: '#6b7280',
  },
  loadingIndicator: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '200px',
    color: '#6b7280',
  },
  loadingSpinner: {
    border: '4px solid #e5e7eb',
    borderTopColor: '#3b82f6',
    borderRadius: '50%',
    width: '24px',
    height: '24px',
    animation: 'spin 1s linear infinite',
    marginRight: '0.5rem',
  },
  error: {
    color: '#ef4444',
    backgroundColor: '#fee2e2',
    padding: '1rem',
    borderRadius: '0.5rem',
    marginTop: '1rem',
  },
  noTables: {
    textAlign: 'center' as const,
    padding: '2rem',
    color: '#6b7280',
  },
  fileInput: {
    display: 'none',
  },
  badge: (schema: string) => ({
    fontSize: '0.75rem',
    fontWeight: 500,
    padding: '0.125rem 0.375rem',
    borderRadius: '0.25rem',
    backgroundColor: schema === 'main' ? '#e0f2fe' : '#f0fdf4',
    color: schema === 'main' ? '#0369a1' : '#166534',
    marginLeft: '0.5rem',
  }),
};

const DatabaseExplorer: React.FC<DatabaseExplorerProps> = ({ className }) => {
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedTable, setSelectedTable] = useState<string | null>(null);
  const [refreshCounter, setRefreshCounter] = useState<number>(0);
  
  // Function to trigger a refresh of the table list
  const refreshTables = () => {
    setRefreshCounter(prev => prev + 1);
  };
  
  // Load the list of tables
  useEffect(() => {
    const fetchTables = async () => {
      setLoading(true);
      setError(null);
      
      try {
        const tablesList = await listTables();
        setTables(tablesList);
      } catch (err) {
        console.error('Error fetching tables:', err);
        setError(`Failed to fetch tables: ${err instanceof Error ? err.message : String(err)}`);
      } finally {
        setLoading(false);
      }
    };
    
    fetchTables();
  }, [refreshCounter]);
  
  // This function was removed as we now handle database file uploads through the main App component
  
  return (
    <div style={styles.container} className={className}>
      <div style={styles.header}>
        <h2 style={styles.title}>Database Tables</h2>
      </div>
      
      {error && (
        <div style={styles.error}>{error}</div>
      )}
      
      {loading ? (
        <div style={styles.loadingIndicator}>
          <div style={styles.loadingSpinner}></div>
          <span>Loading tables...</span>
        </div>
      ) : tables.length === 0 ? (
        <div style={styles.noTables}>
          <p>No tables found. Click "Upload Files" in the top bar to upload a database file (.db, .sqlite, or .duckdb).</p>
        </div>
      ) : (
        <div style={styles.tableList}>
          {tables.map((table) => {
            // Use the displayName from the table info object if available
            // Otherwise fall back to the old behavior
            let tableName = table.displayName || table.name;
            let schema = 'main';
            let database = null;
            
            // Parse the full identifier to extract database, schema, and table parts
            const parts = tableName.split('.');
            if (parts.length === 3) {
              // Format: database.schema.table
              database = parts[0];
              schema = parts[1];
              tableName = parts[2];
            } else if (parts.length === 2) {
              // Format: schema.table
              schema = parts[0];
              tableName = parts[1];
            }
            
            return (
              <div
                key={table.name}
                style={styles.tableCard}
                onClick={() => setSelectedTable(table.name)}
                role="button"
                tabIndex={0}
              >
                <div style={styles.tableName}>
                  {tableName}
                  {schema !== 'main' && <span style={styles.badge(schema)}>{schema}</span>}
                  {database && <span style={{...styles.badge(database), backgroundColor: '#4a6da7', marginLeft: '4px'}}>{database}</span>}
                </div>
                <div style={styles.tableInfo}>
                  {table.rowCount.toLocaleString()} rows • {table.columnCount} columns
                </div>
              </div>
            );
          })}
        </div>
      )}
      
      {selectedTable && (
        <TableViewer
          tableName={selectedTable}
          onClose={() => setSelectedTable(null)}
        />
      )}
    </div>
  );
};

export default DatabaseExplorer;