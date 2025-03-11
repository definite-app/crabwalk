// Utility to automatically load Crabwalk project files from the current directory

import { FileType } from '../types';

interface ProjectFile {
  name: string;
  type: FileType;
  content: string;
}

interface FilePattern {
  regex: RegExp;
  type: FileType;
}

// Define patterns to identify file types
const FILE_PATTERNS: FilePattern[] = [
  { regex: /database_schema\.xml$/i, type: 'schema' },
  { regex: /lineage\.mmd$/i, type: 'lineage' },
  { regex: /\.sql$/i, type: 'sql' },
];

/**
 * Scan for project files in the current directory or provided path
 */
export const scanProjectFiles = async (basePath: string = '.'): Promise<ProjectFile[]> => {
  try {
    // Fetch a listing of files from the server
    const response = await fetch(`${basePath}/api/files`);
    if (!response.ok) {
      throw new Error(`Failed to fetch file listing: ${response.statusText}`);
    }
    
    const fileList = await response.json();
    
    // Load detected files in parallel
    const filePromises = fileList.map(async (filePath: string) => {
      // Determine file type based on patterns
      const fileName = filePath.split('/').pop() || '';
      const filePattern = FILE_PATTERNS.find(p => p.regex.test(fileName));
      
      if (!filePattern) return null; // Skip files that don't match our patterns
      
      try {
        // Use the dedicated API endpoint to read file contents
        const fileResponse = await fetch(`${basePath}/api/file/${encodeURIComponent(filePath)}`);
        if (!fileResponse.ok) return null;
        
        const content = await fileResponse.text();
        
        return {
          name: fileName,
          type: filePattern.type,
          content,
        };
      } catch (err) {
        console.error(`Error loading file ${filePath}:`, err);
        return null;
      }
    });
    
    const loadedFiles = await Promise.all(filePromises);
    
    // Filter out any null values (failed loads)
    return loadedFiles.filter((file): file is ProjectFile => file !== null);
    
  } catch (error) {
    console.error('Error scanning project files:', error);
    return [];
  }
};

/**
 * Check if we're running in a Crabwalk project directory
 */
export const isProjectDirectory = async (): Promise<boolean> => {
  try {
    // Look for key indicators like schema files, lineage diagrams, or SQL files
    const response = await fetch('./api/check-project');
    if (!response.ok) return false;
    
    const result = await response.json();
    return result.isProject === true;
  } catch (error) {
    return false;
  }
};

/**
 * Load all project files from the current directory
 */
export const loadProjectFiles = async (): Promise<ProjectFile[]> => {
  try {
    // First check if we're in a project directory
    const isProject = await isProjectDirectory();
    if (!isProject) {
      return [];
    }
    
    // Then scan for files
    return await scanProjectFiles();
  } catch (error) {
    console.error('Error loading project files:', error);
    return [];
  }
};

export default {
  scanProjectFiles,
  isProjectDirectory,
  loadProjectFiles
};