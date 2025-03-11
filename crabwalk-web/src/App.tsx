import { useState, useRef, useEffect } from 'react';
import MermaidDiagram from './components/MermaidDiagram';
import DatabaseExplorer from './components/DatabaseExplorer';
import SqlQueryPanel from './components/SqlQueryPanel';
import { parseSchema, generateSchemaHtml } from './utils/schemaParser';
import { loadProjectFiles } from './utils/projectLoader';
import { ProjectFile, Table, FileType } from './types';
import PerspectiveTest from './test/PerspectiveTest';

// Inline styles object with explicit React CSS types
const styles = {
  app: {
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    lineHeight: 1.5,
    minHeight: '100vh',
    display: 'flex',
    flexDirection: 'column' as const,
    margin: 0,
    padding: 0,
    backgroundColor: '#f9fafb',
    color: '#1f2937',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '1rem 2rem',
    backgroundColor: '#2563eb',
    color: 'white',
    boxShadow: '0 2px 4px rgba(0, 0, 0, 0.1)',
  },
  h1: {
    fontSize: '1.5rem',
    fontWeight: 600,
  },
  button: {
    backgroundColor: 'white',
    color: '#2563eb',
    border: 'none',
    borderRadius: '4px',
    padding: '0.5rem 1rem',
    fontSize: '1rem',
    fontWeight: 500,
    cursor: 'pointer',
  },
  tabs: {
    display: 'flex',
    padding: '0 2rem',
    borderBottom: '1px solid #e5e7eb',
  },
  tab: (isActive: boolean) => ({
    padding: '1rem 1.5rem',
    border: 'none',
    background: 'none',
    fontSize: '1rem',
    fontWeight: 500,
    color: isActive ? '#2563eb' : '#6b7280',
    borderBottom: isActive ? '2px solid #2563eb' : '2px solid transparent',
    cursor: 'pointer',
  }),
  content: {
    flex: 1,
    padding: '2rem',
    overflowY: 'auto' as const,
  },
  emptyState: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    minHeight: '300px',
    textAlign: 'center' as const,
  },
  card: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '1.5rem',
    border: '1px solid #e5e7eb',
    marginBottom: '1rem',
    boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
  },
  fileCard: {
    backgroundColor: 'white',
    borderRadius: '8px',
    padding: '1.5rem',
    border: '1px solid #e5e7eb',
    marginBottom: '1rem',
    cursor: 'pointer',
    transition: 'transform 0.2s, box-shadow 0.2s',
  },
  fileGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))',
    gap: '1.5rem',
  },
  modal: {
    position: 'fixed' as const,
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  modalContent: {
    backgroundColor: 'white',
    width: '90%',
    maxWidth: '900px',
    maxHeight: '90vh',
    borderRadius: '8px',
    boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
    display: 'flex',
    flexDirection: 'column' as const,
  },
  modalHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '1rem 1.5rem',
    borderBottom: '1px solid #e5e7eb',
  },
  modalBody: {
    flex: 1,
    padding: '1.5rem',
    overflowY: 'auto' as const,
  },
  footer: {
    padding: '1rem 2rem',
    borderTop: '1px solid #e5e7eb',
    textAlign: 'center' as const,
    color: '#6b7280',
    fontSize: '0.875rem',
  },
  codeBlock: {
    backgroundColor: '#f3f4f6',
    borderRadius: '4px',
    padding: '1rem',
    overflowX: 'auto' as const,
    whiteSpace: 'pre-wrap' as const,
    fontFamily: 'monospace',
    fontSize: '0.875rem',
  },
};

// Main App component
function App() {
  const [files, setFiles] = useState<ProjectFile[]>([]);
  const [activeTab, setActiveTab] = useState<'schema' | 'lineage' | 'sql' | 'tables' | 'query' | 'perspective'>('schema');
  const [selectedFile, setSelectedFile] = useState<ProjectFile | null>(null);
  const [parsedTables, setParsedTables] = useState<Table[]>([]);
  const [schemaHtml, setSchemaHtml] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [autoDetected, setAutoDetected] = useState<boolean>(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  
  // Auto-detect and load project files on startup
  useEffect(() => {
    const detectProjectFiles = async () => {
      setIsLoading(true);
      try {
        const projectFiles = await loadProjectFiles();
        if (projectFiles.length > 0) {
          setFiles(projectFiles);
          setAutoDetected(true);
          console.log(`Auto-detected ${projectFiles.length} files from project directory`);
        }
      } catch (error) {
        console.error('Error auto-detecting project files:', error);
      } finally {
        setIsLoading(false);
      }
    };
    
    detectProjectFiles();
  }, []);
  
  // Parse schema when a schema file is found
  useEffect(() => {
    const schemaFile = files.find(f => f.type === 'schema');
    if (schemaFile?.content) {
      try {
        const tables = parseSchema(schemaFile.content);
        setParsedTables(tables);
        setSchemaHtml(generateSchemaHtml(tables));
      } catch (error) {
        console.error('Error parsing schema:', error);
      }
    }
  }, [files]);

  // Handle file upload
  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!e.target.files?.length) return;
    
    // Process each uploaded file
    Array.from(e.target.files).forEach(file => {
      const reader = new FileReader();
      
      reader.onload = (event) => {
        const content = event.target?.result as string;
        if (!content) return;
        
        let fileType: FileType = 'sql';
        if (file.name.includes('schema') && file.name.endsWith('.xml')) {
          fileType = 'schema';
        } else if (file.name.endsWith('.mmd') || file.name.includes('lineage')) {
          fileType = 'lineage';
        }
        
        setFiles(prev => [
          ...prev,
          {
            name: file.name,
            type: fileType,
            content
          }
        ]);
      };
      
      reader.readAsText(file);
    });
  };

  // Trigger file input click
  const handleUploadClick = () => {
    fileInputRef.current?.click();
  };

  // Render different content based on active tab
  const renderContent = () => {
    if (isLoading) {
      return (
        <div style={styles.emptyState}>
          <h2>Loading Project Files...</h2>
          <p>Scanning for Crabwalk project files in the current directory.</p>
          <div style={{
            border: '4px solid #e5e7eb',
            borderTopColor: '#3b82f6',
            borderRadius: '50%',
            width: '40px',
            height: '40px',
            animation: 'spin 1s linear infinite',
            margin: '20px auto',
          }}></div>
        </div>
      );
    }
    
    if (files.length === 0) {
      return (
        <div style={styles.emptyState}>
          <h2>No files found</h2>
          <p>{autoDetected 
            ? "No Crabwalk project files were found in this directory." 
            : "Upload your Crabwalk project files to get started."}</p>
          <button 
            style={styles.button} 
            onClick={handleUploadClick}
          >
            Upload Files
          </button>
        </div>
      );
    }

    // If we're on the perspective tab, show the CDN test component
    if (activeTab === 'perspective') {
      return <PerspectiveTest />;
    }
    
    switch (activeTab) {
      case 'schema':
        return (
          <div>
            <h2>Database Schema</h2>
            {parsedTables.length > 0 ? (
              <div dangerouslySetInnerHTML={{ __html: schemaHtml }} />
            ) : files.some(f => f.type === 'schema') ? (
              <div>Parsing schema...</div>
            ) : (
              <p>No schema files found. Upload a database_schema.xml file.</p>
            )}
          </div>
        );
      
      case 'lineage':
        return (
          <div>
            <h2>Data Lineage</h2>
            {files.some(f => f.type === 'lineage') ? (
              <MermaidDiagram content={files.find(f => f.type === 'lineage')?.content || ''} />
            ) : (
              <p>No lineage files found. Upload a lineage.mmd file.</p>
            )}
          </div>
        );
      
      case 'sql':
        return (
          <div>
            <h2>SQL Files</h2>
            <div style={styles.fileGrid}>
              {files.filter(f => f.type === 'sql').map((file, index) => (
                <div 
                  key={index} 
                  style={styles.fileCard} 
                  onClick={() => setSelectedFile(file)}
                >
                  <h3>{file.name}</h3>
                  <p>SQL File</p>
                </div>
              ))}
              {files.filter(f => f.type === 'sql').length === 0 && (
                <p>No SQL files found. Upload your .sql files.</p>
              )}
            </div>
          </div>
        );
        
      case 'tables':
        return <DatabaseExplorer />;
        
      case 'query':
        return <SqlQueryPanel />;
    }
  };

  return (
    <div style={styles.app}>
      <header style={styles.header}>
        <h1 style={styles.h1}>Crabwalk SQL Explorer</h1>
        <button style={styles.button} onClick={handleUploadClick}>
          Upload Files
        </button>
        <input
          type="file"
          ref={fileInputRef}
          style={{ display: 'none' }}
          onChange={handleFileUpload}
          multiple
        />
      </header>
      
      <div style={styles.tabs}>
        <button
          style={styles.tab(activeTab === 'schema')}
          onClick={() => setActiveTab('schema')}
        >
          Schema
        </button>
        <button
          style={styles.tab(activeTab === 'lineage')}
          onClick={() => setActiveTab('lineage')}
        >
          Lineage
        </button>
        <button
          style={styles.tab(activeTab === 'sql')}
          onClick={() => setActiveTab('sql')}
        >
          SQL
        </button>
        <button
          style={styles.tab(activeTab === 'tables')}
          onClick={() => setActiveTab('tables')}
        >
          Tables
        </button>
        <button
          style={styles.tab(activeTab === 'query')}
          onClick={() => setActiveTab('query')}
        >
          Query
        </button>
        <button
          style={styles.tab(activeTab === 'perspective')}
          onClick={() => setActiveTab('perspective')}
        >
          Perspective
        </button>
      </div>
      
      <main style={styles.content}>
        {renderContent()}
      </main>
      
      <footer style={styles.footer}>
        <p>Crabwalk SQL Explorer &copy; {new Date().getFullYear()}</p>
      </footer>

      {selectedFile && (
        <div style={styles.modal}>
          <div style={styles.modalContent}>
            <div style={styles.modalHeader}>
              <h2>{selectedFile.name}</h2>
              <button 
                onClick={() => setSelectedFile(null)}
                style={{ background: 'none', border: 'none', fontSize: '1.5rem', cursor: 'pointer' }}
              >
                &times;
              </button>
            </div>
            <div style={styles.modalBody}>
              <pre style={styles.codeBlock}>
                {selectedFile.content}
              </pre>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;