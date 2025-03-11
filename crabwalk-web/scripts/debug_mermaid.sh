#!/bin/bash

# Kill any existing processes on port 3000
echo "Stopping any existing web servers..."
kill $(lsof -t -i:3000) 2>/dev/null || true

# Change to the crabwalk-web directory
cd "$(dirname "$0")/.."

# Start the Mermaid test server
echo "Starting Mermaid testing server..."
npm run test:mermaid