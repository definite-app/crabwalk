// Simple HTTP server to serve the Perspective test HTML file
import http from 'http';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// Get current directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PORT = 3000;

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
};

const server = http.createServer((req, res) => {
  console.log(`Request: ${req.method} ${req.url}`);
  
  let filePath;
  
  // Handle root path
  if (req.url === '/') {
    filePath = path.join(__dirname, 'src/test/perspective-test-page.html');
  } 
  // Handle direct file requests in the test directory
  else if (req.url.endsWith('.html') && !req.url.includes('/')) {
    // If it's just a filename without a path, look in the test directory
    filePath = path.join(__dirname, 'src/test', req.url);
    console.log(`Looking for HTML file in test directory: ${filePath}`);
  } 
  // Handle all other paths
  else {
    // For other paths, try both with and without src prefix
    const directPath = path.join(__dirname, req.url.startsWith('/') ? req.url.slice(1) : req.url);
    const srcPath = path.join(__dirname, 'src', req.url.startsWith('/') ? req.url.slice(1) : req.url);
    
    // Check if the file exists with src prefix first
    if (fs.existsSync(srcPath)) {
      filePath = srcPath;
      console.log(`Found file with src prefix: ${filePath}`);
    } else {
      filePath = directPath;
      console.log(`Trying direct path: ${filePath}`);
    }
  }
  
  const extname = path.extname(filePath);
  const contentType = MIME_TYPES[extname] || 'text/plain';
  
  fs.readFile(filePath, (err, content) => {
    if (err) {
      if (err.code === 'ENOENT') {
        console.error(`File not found: ${filePath}`);
        
        // If the file wasn't found and it's an HTML file, try in the test directory as a fallback
        if (req.url.endsWith('.html')) {
          const testDirPath = path.join(__dirname, 'src/test', req.url.startsWith('/') ? req.url.slice(1) : req.url);
          console.log(`Trying test directory as fallback: ${testDirPath}`);
          
          fs.readFile(testDirPath, (testErr, testContent) => {
            if (testErr) {
              res.writeHead(404);
              res.end('File not found');
            } else {
              res.writeHead(200, {
                'Content-Type': contentType,
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
                'Access-Control-Allow-Headers': 'Content-Type'
              });
              res.end(testContent, 'utf-8');
            }
          });
        } else {
          res.writeHead(404);
          res.end('File not found');
        }
      } else {
        console.error(`Server error: ${err.code}`);
        res.writeHead(500);
        res.end(`Server Error: ${err.code}`);
      }
    } else {
      // Add CORS headers to allow loading from CDN
      res.writeHead(200, {
        'Content-Type': contentType,
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type'
      });
      res.end(content, 'utf-8');
    }
  });
});

server.listen(PORT, () => {
  console.log(`Server running at http://localhost:${PORT}/`);
  console.log(`Open http://localhost:${PORT}/ to view the Perspective test options`);
}); 