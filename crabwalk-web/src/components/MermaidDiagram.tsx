import { useEffect, useState } from 'react';
import mermaid from 'mermaid';
import { processLineageDiagram } from '../utils/lineageProcessor';

interface MermaidDiagramProps {
  content: string;
}

// Initialize mermaid once to prevent multiple initializations
mermaid.initialize({
  startOnLoad: false,
  theme: 'default',
  securityLevel: 'loose',
  fontFamily: 'system-ui, sans-serif',
});

// Inline styles for MermaidDiagram
const styles = {
  container: {
    backgroundColor: 'white',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    padding: '1.5rem',
    overflow: 'auto',
    marginBottom: '2rem',
  },
  errorMessage: {
    color: '#dc2626',
    backgroundColor: '#fee2e2',
    border: '1px solid #fecaca',
    borderRadius: '4px',
    padding: '1rem',
    marginTop: '1rem',
  },
  errorPre: {
    marginTop: '1rem',
    whiteSpace: 'pre-wrap' as const,
    fontSize: '0.75rem',
    backgroundColor: 'rgba(0, 0, 0, 0.05)',
    padding: '0.5rem',
    borderRadius: '4px',
  },
  toggleContainer: {
    marginBottom: '1rem', 
    display: 'flex', 
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#f0f9ff',
    border: '1px solid #bae6fd',
    borderRadius: '4px',
    padding: '0.75rem 1rem'
  },
  toggleBtn: (active: boolean) => ({
    backgroundColor: active ? '#0ea5e9' : '#e0f2fe',
    color: active ? 'white' : '#0369a1',
    border: 'none',
    borderRadius: '4px',
    padding: '0.5rem 0.75rem',
    fontSize: '0.875rem',
    cursor: 'pointer'
  }),
  diagramContent: {
    width: '100%',
    minHeight: '200px',
  }
};

const MermaidDiagram: React.FC<MermaidDiagramProps> = ({ content }) => {
  const [svg, setSvg] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [processedContent, setProcessedContent] = useState<string>(content);
  const [hasConnections, setHasConnections] = useState<boolean>(false);
  const [showEnhanced, setShowEnhanced] = useState<boolean>(true);
  const [isProcessing, setIsProcessing] = useState<boolean>(true);

  // Process the content to add connections if needed
  useEffect(() => {
    try {
      if (!content || typeof content !== 'string') {
        setProcessedContent('');
        setIsProcessing(false);
        return;
      }
      
      // Check if the diagram already has connections
      const hasExistingConnections = 
        content.includes('-->') || 
        content.includes('->') || 
        content.includes('---');
      
      setHasConnections(hasExistingConnections);
      
      // Process the content to add connections if none exist
      const processed = processLineageDiagram(content);
      setProcessedContent(processed);
      setIsProcessing(false);
    } catch (err) {
      console.error('Error processing diagram content:', err);
      setProcessedContent(content); // Fallback to original
      setIsProcessing(false);
    }
  }, [content]);

  // Render the mermaid diagram when content changes
  useEffect(() => {
    const renderDiagram = async () => {
      if (isProcessing) return;
      
      setError('');
      setSvg('');
      
      try {
        // Get the content to display (original or processed)
        const displayContent = showEnhanced ? processedContent : content;
        
        if (!displayContent || typeof displayContent !== 'string') {
          throw new Error('No valid diagram content to render');
        }
        
        // Generate a unique ID to avoid conflicts
        const id = `mermaid-${Date.now()}-${Math.floor(Math.random() * 10000)}`;
        
        // Render the diagram
        const { svg } = await mermaid.render(id, displayContent);
        setSvg(svg);
      } catch (err) {
        console.error('Error rendering Mermaid diagram:', err);
        setError(String(err));
      }
    };
    
    renderDiagram();
  }, [content, processedContent, showEnhanced, isProcessing]);

  return (
    <div style={styles.container}>
      {!hasConnections && processedContent !== content && (
        <div style={styles.toggleContainer}>
          <div>
            <div style={{ fontWeight: 500, color: '#0369a1' }}>
              Enhanced Diagram
            </div>
            <div style={{ fontSize: '0.875rem', color: '#0c4a6e' }}>
              Connections between tables have been automatically generated.
            </div>
          </div>
          <button 
            onClick={() => setShowEnhanced(!showEnhanced)}
            style={styles.toggleBtn(showEnhanced)}
          >
            {showEnhanced ? 'Show Original' : 'Show Enhanced'}
          </button>
        </div>
      )}
      
      {error && (
        <div style={styles.errorMessage}>
          <p>Error rendering diagram</p>
          <pre style={styles.errorPre}>{error}</pre>
          <pre style={styles.errorPre}>{showEnhanced ? processedContent : content}</pre>
        </div>
      )}
      
      {isProcessing ? (
        <div style={{ 
          textAlign: 'center', 
          padding: '2rem',
          color: '#6b7280' 
        }}>
          Processing diagram...
        </div>
      ) : !error && (
        <div 
          style={styles.diagramContent}
          dangerouslySetInnerHTML={{ __html: svg }}
        />
      )}
    </div>
  );
};

export default MermaidDiagram;