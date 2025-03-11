#!/usr/bin/env node

// CLI entry point for crabwalk-web
// This allows users to run 'crabwalk-web' from any directory
// to visualize their Crabwalk project

import { spawn } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

console.log('🦀 Starting Crabwalk Web Visualizer...');
console.log('Scanning for project files in current directory...');

// Build the app if dist directory doesn't exist
if (!fs.existsSync(path.join(rootDir, 'dist'))) {
  console.log('Building application (one-time process)...');
  
  const buildProcess = spawn('npm', ['run', 'build'], {
    cwd: rootDir,
    stdio: 'inherit',
  });
  
  buildProcess.on('close', (code) => {
    if (code !== 0) {
      console.error('Error building application. Exiting.');
      process.exit(1);
    }
    
    startServer();
  });
} else {
  startServer();
}

function startServer() {
  console.log('Starting server...');
  
  // For production use, we should directly run the JS file in dist folder
  const serverProcess = spawn('node', ['dist/server/index.js'], {
    cwd: rootDir,
    stdio: 'inherit',
  });
  
  // Handle process termination
  process.on('SIGINT', () => {
    serverProcess.kill('SIGINT');
    process.exit(0);
  });
  
  process.on('SIGTERM', () => {
    serverProcess.kill('SIGTERM');
    process.exit(0);
  });
  
  serverProcess.on('close', (code) => {
    console.log(`Server process exited with code ${code}`);
    process.exit(code || 0);
  });
}