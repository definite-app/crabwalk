import React from 'react';
import { createRoot } from 'react-dom/client';
import mermaid from 'mermaid';

// Simple test component for Mermaid
const MermaidTest = () => {
  const [svg, setSvg] = React.useState<string>('');
  const [error, setError] = React.useState<string>('');

  // Test samples
  const samples = [
    {
      name: 'Simple Graph',
      content: `graph TD
        A[Client] --> B[Load Balancer]
        B --> C[Server1]
        B --> D[Server2]`
    },
    {
      name: 'Simple Table List',
      content: `graph TD
        driver_fact
        races
        race_summary`
    },
    {
      name: 'Auto-generated connections',
      content: `graph TD
        driver_fact
        races
        race_summary
        drivers`
    },
    {
      name: 'Invalid content',
      content: 'This is not valid mermaid'
    }
  ];

  const renderDiagram = async (content: string) => {
    try {
      setError('');
      
      // Initialize mermaid
      mermaid.initialize({
        startOnLoad: false,
        theme: 'default',
        securityLevel: 'loose',
      });
      
      // Generate SVG
      const { svg } = await mermaid.render('mermaid-test', content);
      setSvg(svg);
    } catch (err) {
      console.error('Error rendering diagram:', err);
      setError(String(err));
      setSvg('');
    }
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui, sans-serif' }}>
      <h1>Mermaid Rendering Test</h1>
      
      <div style={{ display: 'flex', gap: '20px' }}>
        <div style={{ width: '300px' }}>
          <h2>Select Test Case</h2>
          {samples.map((sample, index) => (
            <div key={index} style={{ marginBottom: '10px' }}>
              <button 
                onClick={() => renderDiagram(sample.content)}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#2563eb',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                  width: '100%',
                  textAlign: 'left'
                }}
              >
                {sample.name}
              </button>
            </div>
          ))}
        </div>
        
        <div style={{ flex: 1 }}>
          <h2>Output</h2>
          {error ? (
            <div style={{ 
              padding: '16px',
              backgroundColor: '#fee2e2',
              color: '#dc2626',
              borderRadius: '4px',
              marginBottom: '20px'
            }}>
              <h3>Error:</h3>
              <pre>{error}</pre>
            </div>
          ) : null}
          
          <div 
            style={{ 
              border: '1px solid #e5e7eb',
              borderRadius: '4px',
              padding: '16px',
              backgroundColor: 'white',
              minHeight: '400px'
            }}
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        </div>
      </div>
    </div>
  );
};

// Only render in browser, not during SSR
if (typeof window !== 'undefined') {
  const rootElement = document.createElement('div');
  document.body.appendChild(rootElement);
  createRoot(rootElement).render(<MermaidTest />);
}

export default MermaidTest;