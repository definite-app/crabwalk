import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
  ],
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        test: resolve(__dirname, 'src/test/test.html'),
        perspectiveTest: resolve(__dirname, 'src/test/perspective-test.html'),
        perspectiveTestFixed: resolve(__dirname, 'src/test/perspective-test-fixed.html'),
        perspectiveDirect: resolve(__dirname, 'src/test/perspective-direct.html'),
        perspectiveSimple: resolve(__dirname, 'src/test/perspective-simple.html'),
      },
      // Add external dependencies that should be excluded from the bundle
      external: [],
      // Configure output to handle ESM modules better
      output: {
        // Preserve modules to avoid bundling issues
        preserveModules: false,
        // Ensure ESM format
        format: 'es',
        // Avoid mangling exports which can cause issues with named exports
        exports: 'named',
      }
    },
    assetsInlineLimit: 0, // Don't inline WebAssembly files
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  optimizeDeps: {
    exclude: [],
    include: [],
    esbuildOptions: {
      // Fix for modules that use Node.js globals
      define: {
        global: 'globalThis',
        'process.env.NODE_ENV': '"development"'
      },
    },
  },
  // Allow importing .wasm files directly
  assetsInclude: ['**/*.wasm'],
  resolve: {
    alias: {},
  },
})
