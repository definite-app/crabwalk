// Type definitions for @finos/perspective
declare module '@finos/perspective' {
  export function worker(): {
    table: (data: any, options?: any) => Promise<Table>;
  };
  
  export interface Table {
    schema(): Promise<Record<string, string>>;
    size(): Promise<number>;
    view(config?: any): Promise<View>;
    delete(): void;
  }
  
  export interface View {
    to_columns(): Promise<Record<string, any[]>>;
    to_json(): Promise<any[]>;
    delete(): void;
  }
}

// Type definitions for perspective web components
interface PerspectiveViewerElement extends HTMLElement {
  load(table: any): Promise<void>;
  toggleConfig(): void;
  restore(config: any): Promise<void>;
  save(): Promise<any>;
  table: any;
}

declare namespace JSX {
  interface IntrinsicElements {
    'perspective-viewer': React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
  }
}