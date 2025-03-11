// Custom WebAssembly worker for Perspective.js
// This file is loaded by Perspective when creating a worker

// Set the paths to WebAssembly files
const paths = {
  wasmBinary: '/wasm/perspective-js.wasm',
  wasmPath: '/wasm/',
};

// Listen for messages from the main thread
self.addEventListener('message', async function(event) {
  if (event.data && event.data.cmd === 'init') {
    // Respond with the initialized state
    self.postMessage({
      id: event.data.id || 0,
      data: {
        initialized: true
      }
    });
  } else {
    // Forward other messages to the actual worker implementation
    try {
      // Process the message (should be implemented by the actual worker)
      // ...
      
      // Send a response (even if empty)
      self.postMessage({
        id: event.data.id || 0,
        data: {}
      });
    } catch (e) {
      // Send error message
      self.postMessage({
        id: event.data.id || 0,
        error: e.message
      });
    }
  }
});