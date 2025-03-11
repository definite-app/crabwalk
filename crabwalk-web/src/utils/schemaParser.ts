interface Column {
  name: string;
  type: string;
  isPrimaryKey: boolean;
  sourceTable?: string;
  sourceColumn?: string;
  description?: string;
}

interface Table {
  name: string;
  description: string;
  columns: Column[];
  dependencies: string[];
}

/**
 * Parse a Crabwalk XML schema into a structured format
 */
export function parseSchema(xmlContent: string): Table[] {
  try {
    const parser = new DOMParser();
    const xmlDoc = parser.parseFromString(xmlContent, "text/xml");
    
    // Check for parsing errors
    const parserError = xmlDoc.querySelector('parsererror');
    if (parserError) {
      throw new Error('XML parsing error: ' + parserError.textContent);
    }
    
    const tables: Table[] = [];
    const tableElements = xmlDoc.querySelectorAll('table');
    
    tableElements.forEach(tableEl => {
      const name = tableEl.getAttribute('name') || 'Unknown';
      
      // Get table description
      const descriptionEl = tableEl.querySelector('description');
      const description = descriptionEl ? descriptionEl.textContent || '' : '';
      
      // Get columns
      const columns: Column[] = [];
      const columnElements = tableEl.querySelectorAll('column');
      
      columnElements.forEach(colEl => {
        const column: Column = {
          name: colEl.getAttribute('name') || 'Unknown',
          type: colEl.getAttribute('type') || 'unknown',
          isPrimaryKey: colEl.getAttribute('primary_key') === 'true',
          description: '',
        };
        
        // Get column description
        const colDescEl = colEl.querySelector('description');
        if (colDescEl) {
          column.description = colDescEl.textContent || '';
        }
        
        // Get source information
        const sourceEl = colEl.querySelector('source');
        if (sourceEl) {
          const sourceTableEl = sourceEl.querySelector('table');
          const sourceColumnEl = sourceEl.querySelector('column');
          
          column.sourceTable = sourceTableEl ? sourceTableEl.textContent || undefined : undefined;
          column.sourceColumn = sourceColumnEl ? sourceColumnEl.textContent || undefined : undefined;
        }
        
        columns.push(column);
      });
      
      // Get dependencies
      const dependencies: string[] = [];
      const dependencyElements = tableEl.querySelectorAll('source_dependencies dependency');
      
      dependencyElements.forEach(depEl => {
        const tableName = depEl.getAttribute('table');
        if (tableName) {
          dependencies.push(tableName);
        }
      });
      
      tables.push({
        name,
        description,
        columns,
        dependencies
      });
    });
    
    return tables;
  } catch (error) {
    console.error('Error parsing schema:', error);
    throw error;
  }
}

/**
 * Generate a simplified schema representation as text
 */
export function generateSchemaText(tables: Table[]): string {
  let text = '';
  
  tables.forEach(table => {
    text += `TABLE: ${table.name}\n`;
    text += `${'-'.repeat(table.name.length + 7)}\n`;
    text += `${table.description}\n\n`;
    
    // Columns
    text += 'COLUMNS:\n';
    if (table.columns.length > 0) {
      // Find the longest column name for formatting
      const maxNameLength = Math.max(...table.columns.map(col => col.name.length));
      const maxTypeLength = Math.max(...table.columns.map(col => col.type.length));
      
      table.columns.forEach(col => {
        const pkMarker = col.isPrimaryKey ? 'PK' : '  ';
        const name = col.name.padEnd(maxNameLength + 2);
        const type = col.type.padEnd(maxTypeLength + 2);
        
        text += `  ${pkMarker} ${name} ${type}`;
        
        if (col.sourceTable && col.sourceColumn) {
          text += `FROM ${col.sourceTable}.${col.sourceColumn}`;
        }
        
        text += '\n';
      });
    } else {
      text += '  No columns defined\n';
    }
    
    // Dependencies
    text += '\nDEPENDENCIES:\n';
    if (table.dependencies.length > 0) {
      table.dependencies.forEach(dep => {
        text += `  ${dep}\n`;
      });
    } else {
      text += '  No dependencies\n';
    }
    
    text += '\n\n';
  });
  
  return text;
}

/**
 * Generate a tabular schema representation as HTML
 */
export function generateSchemaHtml(tables: Table[]): string {
  // Inline styles as string literals
  const styles = {
    schemaTable: 'margin-bottom: 2rem; border: 1px solid #e5e7eb; border-radius: 8px; overflow: hidden; background-color: #ffffff;',
    tableHeader: 'background-color: #2563eb; color: white; padding: 0.75rem 1rem; margin: 0; font-size: 1.25rem;',
    tableDesc: 'padding: 0.75rem 1rem; margin: 0; border-bottom: 1px solid #e5e7eb; color: #6b7280;',
    columnTable: 'width: 100%; border-collapse: collapse;',
    tableHeaderCell: 'text-align: left; padding: 0.75rem 1rem; background-color: #f3f4f6; color: #1f2937; font-weight: 600; font-size: 0.875rem;',
    tableCell: 'padding: 0.75rem 1rem; border-top: 1px solid #e5e7eb; font-size: 0.875rem;',
    dependencies: 'padding: 0.75rem 1rem; border-top: 1px solid #e5e7eb;',
    dependenciesHeader: 'margin: 0 0 0.5rem 0; font-size: 1rem; color: #1f2937;',
    dependenciesList: 'margin: 0; padding-left: 1.5rem; color: #6b7280;'
  };
  
  let html = '';
  
  tables.forEach(table => {
    html += `<div style="${styles.schemaTable}">`;
    html += `<h3 style="${styles.tableHeader}">${table.name}</h3>`;
    html += `<p style="${styles.tableDesc}">${table.description}</p>`;
    
    // Columns
    html += `<table style="${styles.columnTable}">`;
    html += `<thead><tr>
              <th style="${styles.tableHeaderCell}">PK</th>
              <th style="${styles.tableHeaderCell}">Name</th>
              <th style="${styles.tableHeaderCell}">Type</th>
              <th style="${styles.tableHeaderCell}">Source</th>
             </tr></thead>`;
    html += `<tbody>`;
    
    if (table.columns.length > 0) {
      table.columns.forEach(col => {
        html += `<tr>`;
        html += `<td style="${styles.tableCell}">${col.isPrimaryKey ? '✓' : ''}</td>`;
        html += `<td style="${styles.tableCell}">${col.name}</td>`;
        html += `<td style="${styles.tableCell}">${col.type}</td>`;
        html += `<td style="${styles.tableCell}">${col.sourceTable && col.sourceColumn ? `${col.sourceTable}.${col.sourceColumn}` : ''}</td>`;
        html += `</tr>`;
      });
    } else {
      html += `<tr><td style="${styles.tableCell}" colspan="4">No columns defined</td></tr>`;
    }
    
    html += `</tbody></table>`;
    
    // Dependencies
    html += `<div style="${styles.dependencies}">`;
    html += `<h4 style="${styles.dependenciesHeader}">Dependencies:</h4>`;
    html += `<ul style="${styles.dependenciesList}">`;
    
    if (table.dependencies.length > 0) {
      table.dependencies.forEach(dep => {
        html += `<li>${dep}</li>`;
      });
    } else {
      html += `<li>No dependencies</li>`;
    }
    
    html += `</ul></div>`;
    html += `</div>`;
  });
  
  return html;
}

export default {
  parseSchema,
  generateSchemaText,
  generateSchemaHtml
};