import { useState } from 'react';
import { executeQuery } from '../utils/duckdb';

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100%',
    padding: '1rem',
  },
  header: {
    marginBottom: '1rem',
  },
  title: {
    fontSize: '1.25rem',
    fontWeight: 600,
    margin: '0 0 0.5rem 0',
  },
  description: {
    color: '#6b7280',
    margin: '0 0 1rem 0',
    fontSize: '0.875rem',
  },
  queryArea: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '0.5rem',
    marginBottom: '1rem',
  },
  queryInput: {
    fontFamily: 'monospace',
    fontSize: '0.875rem',
    padding: '0.75rem',
    borderRadius: '0.375rem',
    border: '1px solid #d1d5db',
    minHeight: '120px',
    resize: 'vertical' as const,
    backgroundColor: '#f9fafb',
  },
  buttonRow: {
    display: 'flex',
    gap: '0.5rem',
    justifyContent: 'flex-end',
  },
  runButton: {
    backgroundColor: '#2563eb',
    color: 'white',
    border: 'none',
    borderRadius: '0.375rem',
    padding: '0.5rem 1rem',
    fontSize: '0.875rem',
    fontWeight: 500,
    cursor: 'pointer',
  },
  clearButton: {
    backgroundColor: '#f3f4f6',
    color: '#374151',
    border: '1px solid #d1d5db',
    borderRadius: '0.375rem',
    padding: '0.5rem 1rem',
    fontSize: '0.875rem',
    fontWeight: 500,
    cursor: 'pointer',
  },
  resultContainer: {
    flex: 1,
    overflow: 'auto',
    border: '1px solid #e5e7eb',
    borderRadius: '0.375rem',
    backgroundColor: 'white',
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
  noResults: {
    textAlign: 'center' as const,
    padding: '2rem',
    color: '#6b7280',
  },
  resultInfo: {
    fontSize: '0.875rem',
    color: '#6b7280',
    padding: '0.5rem',
    borderBottom: '1px solid #e5e7eb',
  },
  tableContainer: {
    flex: 1,
    overflow: 'auto',
    minHeight: '300px',
  },
  sampleQuery: {
    display: 'inline-block',
    fontFamily: 'monospace',
    fontSize: '0.875rem',
    backgroundColor: '#f3f4f6',
    padding: '0.25rem 0.5rem',
    borderRadius: '0.25rem',
    cursor: 'pointer',
    marginRight: '0.5rem',
    marginBottom: '0.5rem',
  }
};

const SqlQueryPanel: React.FC = () => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<any[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rowCount, setRowCount] = useState(0);
  
  // Sample queries that users can click on
  const sampleQueries = [
    'SELECT * FROM information_schema.tables',
    'SELECT * FROM imported.races LIMIT 10',
    'SELECT COUNT(*) FROM imported.drivers',
    'SELECT name, description FROM imported.circuits LIMIT 5',
  ];

  const runQuery = async () => {
    if (!query.trim()) {
      setError('Please enter a SQL query');
      return;
    }

    setLoading(true);
    setError(null);
    setResults(null);

    try {
      const data = await executeQuery(query);
      console.log('Query results:', data);
      
      // Diagnostic logging to check result structure
      if (data.length > 0) {
        console.log('First row:', data[0]);
        console.log('First row keys:', Object.keys(data[0]));
        console.log('First row values:', Object.values(data[0]));
        console.log('Raw first row:', JSON.stringify(data[0]));
      }
      
      setResults(data);
      setRowCount(data.length);
    } catch (err) {
      console.error('Error executing SQL query:', err);
      setError(`Error executing query: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const clearQuery = () => {
    setQuery('');
    setResults(null);
    setError(null);
  };

  const handleSampleQuery = (sampleQuery: string) => {
    setQuery(sampleQuery);
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={styles.title}>SQL Query</h2>
        <p style={styles.description}>
          Run SQL queries against the loaded database
        </p>
        
        <div style={{ marginBottom: '0.75rem' }}>
          <div style={{ marginBottom: '0.25rem', fontSize: '0.875rem', fontWeight: 500 }}>Sample queries:</div>
          {sampleQueries.map((sampleQuery, index) => (
            <span 
              key={index} 
              style={styles.sampleQuery}
              onClick={() => handleSampleQuery(sampleQuery)}
            >
              {sampleQuery}
            </span>
          ))}
        </div>
      </div>

      <div style={styles.queryArea}>
        <textarea
          style={styles.queryInput}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Enter SQL query here..."
          spellCheck={false}
        />
        <div style={styles.buttonRow}>
          <button
            style={styles.clearButton}
            onClick={clearQuery}
          >
            Clear
          </button>
          <button
            style={styles.runButton}
            onClick={runQuery}
            disabled={loading}
          >
            {loading ? 'Running...' : 'Run Query'}
          </button>
        </div>
      </div>

      {error && (
        <div style={styles.error}>{error}</div>
      )}

      <div style={styles.resultContainer}>
        {loading ? (
          <div style={styles.loadingIndicator}>
            <div style={styles.loadingSpinner}></div>
            <span>Running query...</span>
          </div>
        ) : results ? (
          <div style={styles.tableContainer}>
            <div style={styles.resultInfo}>
              {rowCount} {rowCount === 1 ? 'row' : 'rows'} returned
            </div>
            {/* We'll create a custom table for displaying results */}
            <div style={{ padding: '1rem' }}>
              {results.length > 0 ? (
                <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                  <thead>
                    <tr>
                      {Object.keys(results[0]).map((column, idx) => (
                        <th key={idx} style={{ 
                          textAlign: 'left', 
                          padding: '0.5rem', 
                          borderBottom: '2px solid #e5e7eb',
                          fontWeight: 600,
                          color: '#374151'
                        }}>
                          {column}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {results.map((row, rowIdx) => {
                      // Debugging: log each row as we render it
                      if (rowIdx < 3) {
                        console.log(`Rendering row ${rowIdx}:`, row);
                        console.log(`Row ${rowIdx} values:`, Object.values(row));
                        console.log(`Row ${rowIdx} keys:`, Object.keys(row));
                      }
                      
                      return (
                        <tr key={rowIdx} style={{ 
                          backgroundColor: rowIdx % 2 === 0 ? 'white' : '#f9fafb' 
                        }}>
                          {Object.entries(row).map(([key, cell], cellIdx) => {
                            // Debug each cell
                            if (rowIdx < 2 && cellIdx < 2) {
                              console.log(`Cell [${rowIdx},${cellIdx}] key=${key}, value:`, cell);
                              console.log(`Cell [${rowIdx},${cellIdx}] type:`, typeof cell);
                            }
                            
                            return (
                              <td key={cellIdx} style={{ 
                                padding: '0.5rem', 
                                borderBottom: '1px solid #e5e7eb',
                                maxWidth: '300px',
                                overflow: 'hidden',
                                textOverflow: 'ellipsis',
                                whiteSpace: 'nowrap',
                                fontSize: '0.875rem'
                              }}>
                                {cell === null || cell === undefined 
                                  ? <span style={{ color: '#9ca3af', fontStyle: 'italic' }}>NULL</span> 
                                  : String(cell)}
                              </td>
                            );
                          })}
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              ) : (
                <div style={styles.noResults}>Query executed successfully, but no rows were returned</div>
              )}
            </div>
          </div>
        ) : (
          <div style={styles.noResults}>
            Enter a SQL query and click "Run Query" to see results
          </div>
        )}
      </div>
    </div>
  );
};

export default SqlQueryPanel;