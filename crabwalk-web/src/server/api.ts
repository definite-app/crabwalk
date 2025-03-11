import fs from 'fs';
import path from 'path';
import express from 'express';
import { Request, Response } from 'express';

// Create router for API endpoints
const apiRouter = express.Router();

// Common file patterns for Crabwalk projects
const PROJECT_INDICATORS = [
  /database_schema\.xml$/i,
  /lineage\.mmd$/i,
  /\.sql$/i,
];

// API endpoint to list files in current directory
apiRouter.get('/files', (_req: Request, res: Response) => {
  try {
    const currentDir = process.cwd();
    const files: string[] = [];
    
    // Recursive function to scan directories
    const scanDir = (dir: string, relativePath: string = '') => {
      const entries = fs.readdirSync(dir, { withFileTypes: true });
      
      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        const relativeName = path.join(relativePath, entry.name);
        
        // Skip node_modules and other hidden directories
        if (entry.name.startsWith('.') || entry.name === 'node_modules') {
          continue;
        }
        
        if (entry.isDirectory()) {
          scanDir(fullPath, relativeName);
        } else {
          files.push(relativeName);
        }
      }
    };
    
    scanDir(currentDir);
    
    res.json(files);
  } catch (error) {
    console.error('Error scanning directory:', error);
    res.status(500).json({ error: 'Failed to scan directory' });
  }
});

// API endpoint to check if current directory is a Crabwalk project
apiRouter.get('/check-project', (_req: Request, res: Response) => {
  try {
    const currentDir = process.cwd();
    const files = fs.readdirSync(currentDir);
    
    // Check if any of the key project indicators exist
    const isProject = files.some(file => {
      return PROJECT_INDICATORS.some(pattern => pattern.test(file));
    });
    
    res.json({ isProject });
  } catch (error) {
    console.error('Error checking project directory:', error);
    res.status(500).json({ error: 'Failed to check project directory' });
  }
});

// API endpoint to read a file from the project
apiRouter.get('/file/:filename(*)', (req: Request, res: Response) => {
  try {
    const { filename } = req.params;
    const filePath = path.join(process.cwd(), filename);
    
    // Security check - prevent directory traversal
    if (!filePath.startsWith(process.cwd())) {
      return res.status(403).json({ error: 'Access denied' });
    }
    
    // Check if file exists
    if (!fs.existsSync(filePath)) {
      return res.status(404).json({ error: 'File not found' });
    }
    
    // Read file content
    const content = fs.readFileSync(filePath, 'utf8');
    res.send(content);
  } catch (error) {
    console.error('Error reading file:', error);
    res.status(500).json({ error: 'Failed to read file' });
  }
});

export default apiRouter;