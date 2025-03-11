#!/usr/bin/env node

// This script copies WebAssembly files needed by perspective.js to the public directory
// so they can be served by the web server and loaded by the browser

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const WASM_SOURCE_DIRS = [
  path.resolve(__dirname, '../node_modules/@finos/perspective/dist/wasm'),
  path.resolve(__dirname, '../node_modules/@finos/perspective-viewer/dist/wasm')
];

// Also copy Javascript files
const JS_FILES = [
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective/dist/cdn/perspective.js'),
    dest: path.resolve(__dirname, '../public/wasm/perspective.js')
  },
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective/dist/cdn/perspective-server.worker.js'),
    dest: path.resolve(__dirname, '../public/wasm/perspective-server.worker.js')
  },
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective/dist/esm/perspective.js'),
    dest: path.resolve(__dirname, '../public/wasm/perspective.esm.js')
  }
];

// Create aliases for WebAssembly files that may be required by Perspective with different names
const WASM_ALIASES = [
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective/dist/wasm/perspective-js.wasm'),
    dest: path.resolve(__dirname, '../public/wasm/perspective-client.wasm')
  },
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective/dist/wasm/perspective-js.wasm'),
    dest: path.resolve(__dirname, '../public/wasm/perspective.wasm')
  },
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective-viewer/dist/wasm/perspective-viewer.wasm'),
    dest: path.resolve(__dirname, '../public/wasm/perspective-view.wasm')
  }
];

// Copy essential worker files - different formats for browser compatibility
const WORKER_FILES = [
  // UMD format - easier to use directly in browser
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective/dist/umd/perspective.js'),
    dest: path.resolve(__dirname, '../public/wasm/perspective-umd.js')
  },
  {
    src: path.resolve(__dirname, '../node_modules/@finos/perspective/dist/umd/perspective.worker.js'),
    dest: path.resolve(__dirname, '../public/wasm/perspective.worker.js')
  }
];

const WASM_DEST_DIR = path.resolve(__dirname, '../public/wasm');

// Create destination directory if it doesn't exist
if (!fs.existsSync(WASM_DEST_DIR)) {
  fs.mkdirSync(WASM_DEST_DIR, { recursive: true });
  console.log(`Created directory: ${WASM_DEST_DIR}`);
}

// Copy all .wasm files
let copiedFiles = 0;
for (const sourceDir of WASM_SOURCE_DIRS) {
  if (fs.existsSync(sourceDir)) {
    const files = fs.readdirSync(sourceDir);
    for (const file of files) {
      if (file.endsWith('.wasm')) {
        const sourcePath = path.join(sourceDir, file);
        const destPath = path.join(WASM_DEST_DIR, file);
        fs.copyFileSync(sourcePath, destPath);
        copiedFiles++;
        console.log(`Copied: ${sourcePath} -> ${destPath}`);
      }
    }
  } else {
    console.warn(`Source directory not found: ${sourceDir}`);
  }
}

console.log(`Copied ${copiedFiles} WebAssembly files to ${WASM_DEST_DIR}`);

// Copy JS files
let copiedJsFiles = 0;
for (const file of JS_FILES) {
  if (fs.existsSync(file.src)) {
    fs.copyFileSync(file.src, file.dest);
    copiedJsFiles++;
    console.log(`Copied: ${file.src} -> ${file.dest}`);
  } else {
    console.warn(`Source file not found: ${file.src}`);
  }
}

console.log(`Copied ${copiedJsFiles} JavaScript files to ${WASM_DEST_DIR}`);

// Copy WebAssembly aliases
let copiedAliases = 0;
for (const file of WASM_ALIASES) {
  if (fs.existsSync(file.src)) {
    fs.copyFileSync(file.src, file.dest);
    copiedAliases++;
    console.log(`Created alias: ${file.src} -> ${file.dest}`);
  } else {
    console.warn(`Source file for alias not found: ${file.src}`);
  }
}

console.log(`Created ${copiedAliases} WebAssembly file aliases in ${WASM_DEST_DIR}`);

// Copy worker files
let copiedWorkerFiles = 0;
for (const file of WORKER_FILES) {
  if (fs.existsSync(file.src)) {
    try {
      fs.copyFileSync(file.src, file.dest);
      copiedWorkerFiles++;
      console.log(`Copied worker file: ${file.src} -> ${file.dest}`);
    } catch (err) {
      console.warn(`Failed to copy worker file ${file.src}: ${err}`);
    }
  } else {
    console.warn(`Worker file not found: ${file.src}`);
  }
}

console.log(`Copied ${copiedWorkerFiles} WebWorker files to ${WASM_DEST_DIR}`);