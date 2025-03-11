// Simple server to serve the app and APIs
import path from 'path';
import express from 'express';
import { fileURLToPath } from 'url';
import apiRouter from './api.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Create Express app
const app = express();
const PORT = process.env.PORT || 3000;

// Serve static files from the dist directory
app.use(express.static(path.resolve(__dirname, '../../dist')));

// Serve test directory for debugging
app.use('/test', express.static(path.resolve(__dirname, '../../src/test')));

// Mount API routes
app.use('/api', apiRouter);

// Serve the index.html for any other route (SPA)
app.get('*', (_req, res) => {
  res.sendFile(path.resolve(__dirname, '../../dist/index.html'));
});

// Function to open browser
const openBrowser = async (url: string) => {
  // Use dynamic import for ES modules compatibility
  const { spawn } = await import('child_process');
  let command;
  let args;
  
  switch (process.platform) {
    case 'darwin': // macOS
      command = 'open';
      args = [url];
      break;
    case 'win32': // Windows
      command = 'cmd';
      args = ['/c', 'start', url];
      break;
    default: // Linux and others
      command = 'xdg-open';
      args = [url];
      break;
  }
  
  spawn(command, args, { stdio: 'ignore' });
};

// Start the server
app.listen(PORT, () => {
  const url = `http://localhost:${PORT}`;
  console.log(`Crabwalk Web server running at ${url}`);
  
  // Open browser automatically
  setTimeout(async () => {
    console.log('Opening web browser...');
    await openBrowser(url);
  }, 500);
});

export default app;