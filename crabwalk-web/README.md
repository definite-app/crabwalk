# Crabwalk Web

A web interface for the Crabwalk SQL transformation orchestrator.

## Getting Started

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Start production server
npm run start
```

## Build and Run After Making Changes

When you make changes to the codebase, follow these steps to build and run the application:

```bash
# Compile TypeScript and build the application
npm run build

# Start the server with the updated build
npm run server

# Or, build and start in one command
npm run start
```

The build process will:
1. Compile TypeScript (`tsc -b`)
2. Build the frontend with Vite (`vite build`)
3. Compile server TypeScript (`tsc -p tsconfig.server.json`)

After building, the application will be available at http://localhost:3000 (or the configured port).

## Relationship with Cargo/Rust App

This web interface is a companion to the main Crabwalk CLI tool, which is built with Rust/Cargo and located in the parent directory. To build and use both components:

### Building the Rust CLI

Navigate to the parent directory and build the Rust application:

```bash
# From the crabwalk-web directory
cd ..

# Build the Rust CLI
cargo build --release

# Run examples with the Rust CLI
cargo run
```

### Using Both Together

The web application can visualize projects created by the Rust CLI. A typical workflow:

1. Use the Rust CLI to process SQL files and generate schema/lineage information:
   ```bash
   cargo run -- run ./path/to/sql/files
   ```

2. Run the web application to visualize the output:
   ```bash
   npm run start
   ```

3. Or use the CLI command to launch the web interface directly:
   ```bash
   cargo run -- app --open
   ```

## Troubleshooting

### Perspective WebAssembly Setup

The application uses Perspective.js for data visualization, which requires WebAssembly files. We've implemented a robust solution to ensure all WebAssembly files are correctly loaded:

1. **WebAssembly File Management**:
   - A script (`scripts/setup-wasm.js`) copies necessary WebAssembly files from node_modules to the `public/wasm` directory
   - The script also creates aliases for the WebAssembly files with alternative names that Perspective might look for
   - This includes specific handling for `perspective-client.wasm` which is required but not directly provided

2. **Path Configuration**:
   - We inject WebAssembly paths into the window object in the HTML files
   - This ensures Perspective can find the WebAssembly files even when using different naming conventions
   - We use `window.PERSPECTIVE_ASSETS` to specify exact paths for each WebAssembly file

3. **Testing Perspective**:
   - A dedicated test component (`/src/test/PerspectiveTest.tsx`) verifies WebAssembly loading
   - Run `npm run test:perspective` to check if Perspective is working correctly
   - This helps diagnose WebAssembly loading issues independently of the main application

If you encounter errors like "Missing perspective-client.wasm":

1. Check that all WebAssembly files and aliases were created:
   ```bash
   npm run setup-wasm
   ls -la public/wasm
   ```

2. Make sure your server has the correct CORS headers:
   ```
   Cross-Origin-Opener-Policy: same-origin
   Cross-Origin-Embedder-Policy: require-corp
   ```

3. Try clearing browser cache and storage:
   - Clear browser cache
   - Clear IndexedDB and WebAssembly storage
   - Restart your browser

4. Check for console errors about disallowed WebAssembly features:
   - Some browsers restrict WebAssembly features
   - Ensure SharedArrayBuffer is available and allowed

### DuckDB WebAssembly Implementation

The application uses DuckDB-wasm to provide SQL database capabilities directly in the browser. Here's how it works:

1. **WebAssembly Loading**: DuckDB is compiled to WebAssembly, which runs in the browser with near-native performance.

2. **Web Worker**: DuckDB runs in a dedicated Web Worker thread to avoid freezing the UI during intensive operations.

3. **Blob URL Creation**: We use a Blob URL to create the worker, which resolves cross-origin issues and provides better compatibility across browsers.

4. **Memory Database**: By default, an in-memory database is created, and you can load external database files.

If you encounter any issues:

1. **Clear Browser Cache**: Clear your browser cache and reload the application.

2. **Use a Modern Browser**: Ensure you're using a recent version of Chrome, Firefox, Edge, or Safari.

3. **Check Console Logs**: Open your browser developer tools (F12) to check for error messages.

4. **WebAssembly Support**: Your browser must support WebAssembly. All modern browsers support this feature.

5. **Cross-Origin Issues**: When running locally, use a proper web server (like the Vite dev server) rather than opening the HTML file directly.

### Using Example Files

Example database files are available in the `examples` directory of the Crabwalk project. Try loading these files first to ensure the application is working correctly.