#!/bin/bash

# This script will start the crabwalk web viewer with database integration
# It will look for .duckdb or .db files in the current directory

echo "🦀 Starting Crabwalk Web with DuckDB Integration"
echo "==============================================="

# Check if a DuckDB file exists in the current directory
DB_FILES=$(find . -maxdepth 1 -type f \( -name "*.db" -o -name "*.duckdb" -o -name "*.sqlite" \))

if [ -n "$DB_FILES" ]; then
  echo "Found database files in current directory:"
  echo "$DB_FILES"
  echo ""
  echo "These will be accessible from the Tables tab."
fi

# Start the web server
echo "Starting web interface. Press Ctrl+C to exit."
crabwalk-web