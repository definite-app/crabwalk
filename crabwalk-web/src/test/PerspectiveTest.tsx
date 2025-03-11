import { useEffect, useRef, useState } from 'react';

// Test component for Perspective WebAssembly loading via CDN
export default function PerspectiveTest() {
  const [status, setStatus] = useState<string>('Initializing...');
  const [error, setError] = useState<string | null>(null);
  const viewerRef = useRef<any>(null);
  const [isLoaded, setIsLoaded] = useState<boolean>(false);

  // Load scripts in the head once when the component mounts
  useEffect(() => {
    // Only load scripts once
    if (document.querySelector('script[data-perspective-cdn]')) {
      console.log('Perspective CDN scripts already loaded');
      setIsLoaded(true);
      return;
    }

    const scripts = [
      { src: 'https://cdn.jsdelivr.net/npm/@finos/perspective/dist/cdn/perspective.js', id: 'perspective-core' },
      { src: 'https://cdn.jsdelivr.net/npm/@finos/perspective-viewer/dist/cdn/perspective-viewer.js', id: 'perspective-viewer' },
      { src: 'https://cdn.jsdelivr.net/npm/@finos/perspective-viewer-datagrid/dist/cdn/perspective-viewer-datagrid.js', id: 'perspective-datagrid' },
      { src: 'https://cdn.jsdelivr.net/npm/@finos/perspective-viewer-d3fc/dist/cdn/perspective-viewer-d3fc.js', id: 'perspective-d3fc' }
    ];

    // Add CSS for Perspective
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = 'https://cdn.jsdelivr.net/npm/@finos/perspective-viewer/dist/css/themes.css';
    link.id = 'perspective-css';
    document.head.appendChild(link);

    const loadScript = (scriptInfo: { src: string, id: string }) => {
      return new Promise<void>((resolve, reject) => {
        // Check if script already exists
        if (document.getElementById(scriptInfo.id)) {
          resolve();
          return;
        }

        const script = document.createElement('script');
        script.id = scriptInfo.id;
        script.src = scriptInfo.src;
        script.setAttribute('data-perspective-cdn', 'true');
        script.async = true;
        script.onload = () => {
          console.log(`Loaded ${scriptInfo.id}`);
          resolve();
        };
        script.onerror = () => reject(new Error(`Failed to load ${scriptInfo.src}`));
        document.head.appendChild(script);
      });
    };

    // Load scripts sequentially
    const loadAllScripts = async () => {
      try {
        setStatus('Loading Perspective libraries from CDN...');
        for (const scriptInfo of scripts) {
          await loadScript(scriptInfo);
        }
        console.log('All Perspective CDN scripts loaded successfully');
        setIsLoaded(true);
        setStatus('Perspective libraries loaded');
      } catch (err) {
        console.error('Failed to load Perspective scripts:', err);
        setError(`Error loading scripts: ${err instanceof Error ? err.message : String(err)}`);
        setStatus('Failed to load scripts');
      }
    };

    loadAllScripts();

    // No cleanup needed - we want to keep the scripts loaded for other components
  }, []);

  // Initialize Perspective and load data once scripts are loaded
  useEffect(() => {
    if (!isLoaded) return;

    const initPerspective = async () => {
      try {
        setStatus('Initializing Perspective...');
        
        // Access the perspective object from the window
        // @ts-ignore - perspective is loaded globally
        if (!window.perspective) {
          throw new Error('Perspective not loaded correctly');
        }
        
        // @ts-ignore - perspective is loaded globally
        const worker = await window.perspective.worker();
        setStatus('Perspective worker initialized');
        
        // Fetch sample data from Superstore Arrow dataset
        setStatus('Fetching sample data...');
        const WASM_URL = "https://cdn.jsdelivr.net/npm/superstore-arrow/superstore.lz4.arrow";
        
        const table = await fetch(WASM_URL)
          .then((x) => x.arrayBuffer())
          .then((x) => worker.table(x));
        
        setStatus('Data loaded successfully');
        
        // Load into viewer
        if (viewerRef.current) {
          await viewerRef.current.load(table);
          setStatus('Data loaded into viewer successfully');
        }
      } catch (err) {
        console.error('Perspective test failed:', err);
        setError(`Error: ${err instanceof Error ? err.message : String(err)}`);
        setStatus('Failed');
      }
    };
    
    initPerspective();
  }, [isLoaded]);
  
  return (
    <div style={{ padding: '20px' }}>
      <h1>Perspective WebAssembly Test (CDN)</h1>
      <div style={{ border: '1px solid #ccc', padding: '10px', height: '400px', width: '100%' }}>
        {/* @ts-ignore */}
        <perspective-viewer style={{ width: '100%', height: '100%' }} ref={viewerRef}></perspective-viewer>
      </div>
      
      <div style={{ marginBottom: '20px' }}>
        <strong>Status:</strong> {status}
      </div>
      
      {error && (
        <div style={{ color: 'red', marginBottom: '20px' }}>
          <strong>Error:</strong> {error}
        </div>
      )}
    </div>
  );
}