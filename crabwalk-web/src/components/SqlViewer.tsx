import { useState, useEffect } from 'react';

interface SqlViewerProps {
  filePath: string;
  fileName: string;
  onClose?: () => void;
}

// Inline styles
const styles = {
  overlay: {
    position: 'fixed' as const,
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 50,
    padding: '1rem',
  },
  modal: {
    backgroundColor: 'white',
    borderRadius: '0.5rem',
    boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
    width: '100%',
    maxWidth: '56rem',
    maxHeight: '90vh',
    display: 'flex',
    flexDirection: 'column' as const,
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    borderBottom: '1px solid #e5e7eb',
    padding: '1rem',
  },
  title: {
    fontSize: '1.125rem',
    fontWeight: 500,
  },
  closeButton: {
    color: '#6b7280',
    border: 'none',
    background: 'none',
    cursor: 'pointer',
  },
  content: {
    flexGrow: 1,
    overflowY: 'auto' as const,
    padding: '1rem',
  },
  loadingContainer: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    height: '16rem',
  },
  spinner: {
    height: '2rem',
    width: '2rem',
    borderRadius: '9999px',
    borderBottom: '2px solid #3b82f6',
    animation: 'spin 1s linear infinite',
  },
  errorMessage: {
    color: '#ef4444',
    padding: '1rem',
  },
  codeBlock: {
    backgroundColor: '#f3f4f6',
    padding: '1rem',
    borderRadius: '0.375rem',
    overflowX: 'auto' as const,
    whiteSpace: 'pre-wrap' as const,
    fontSize: '0.875rem',
    fontFamily: 'monospace',
  },
  footer: {
    borderTop: '1px solid #e5e7eb',
    padding: '1rem',
    display: 'flex',
    justifyContent: 'flex-end',
  },
  button: {
    padding: '0.5rem 1rem',
    backgroundColor: '#e5e7eb',
    color: '#1f2937',
    borderRadius: '0.375rem',
    border: 'none',
    cursor: 'pointer',
  },
};

const SqlViewer = ({ filePath, fileName, onClose }: SqlViewerProps) => {
  const [content, setContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchContent = async () => {
      setIsLoading(true);
      setError(null);
      
      try {
        const response = await fetch(filePath);
        if (!response.ok) {
          throw new Error(`Failed to fetch file: ${response.statusText}`);
        }
        
        const text = await response.text();
        setContent(text);
      } catch (err) {
        console.error('Error loading SQL file:', err);
        setError(err instanceof Error ? err.message : 'Failed to load SQL file');
      } finally {
        setIsLoading(false);
      }
    };
    
    fetchContent();
  }, [filePath]);

  return (
    <div style={styles.overlay}>
      <div style={styles.modal}>
        <div style={styles.header}>
          <h3 style={styles.title}>{fileName}</h3>
          <button 
            onClick={onClose} 
            style={styles.closeButton}
            aria-label="Close"
          >
            <svg width="24" height="24" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <path d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>
        
        <div style={styles.content}>
          {isLoading ? (
            <div style={styles.loadingContainer}>
              <div style={{
                ...styles.spinner,
                animation: 'spin 1s linear infinite',
              }}></div>
            </div>
          ) : error ? (
            <div style={styles.errorMessage}>
              Error: {error}
            </div>
          ) : (
            <pre style={styles.codeBlock}>
              {content}
            </pre>
          )}
        </div>
        
        <div style={styles.footer}>
          <button 
            onClick={onClose}
            style={styles.button}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default SqlViewer;