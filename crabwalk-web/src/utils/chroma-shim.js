/**
 * Shim for chroma-js to provide a default export
 * This fixes the "does not provide an export named 'default'" error
 */

// Import chroma-js directly as a namespace
import * as chromaNamespace from 'chroma-js';

// Create a function that has all the properties of the namespace
const chroma = function(...args) {
  return chromaNamespace.chroma(...args);
};

// Copy all properties from the namespace to our function
Object.assign(chroma, chromaNamespace);

// Export as default
export default chroma;

// Don't re-export all named exports to avoid duplicate declarations
// export * from 'chroma-js'; 