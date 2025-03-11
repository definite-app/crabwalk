// Common type definitions for the application

export type FileType = 'schema' | 'lineage' | 'sql';

export interface ProjectFile {
  name: string;
  type: FileType;
  content: string;
}

export interface Table {
  name: string;
  description: string;
  columns: {
    name: string;
    type: string;
    isPrimaryKey: boolean;
    sourceTable?: string;
    sourceColumn?: string;
    description?: string;
  }[];
  dependencies: string[];
}