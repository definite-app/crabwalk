# Crabwalk: An Easy SQL Orchestration Tool

Crabwalk is a lightweight SQL orchestrator built on top of DuckDB. It processes SQL files in a folder, determines dependencies, and runs them in the correct order.

## Features

- **SQL Orchestration**: Automatically determine the execution order of SQL queries based on dependencies
- **Flexible Output Types**: Configure outputs as tables, views, or files (Parquet, CSV, JSON)
- **Model-level Configuration**: Set output types and other options at the model level using SQL comments
- **S3 Integration**: Backup and restore your DuckDB database to/from S3 (optional)
- **Lightweight**: Minimal dependencies, fast execution
- **Environment Variables**: Support for environment variables in SQL queries

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/definite-app/crabwalk.git
cd crabwalk

# Build the project
cargo build --release

# Optional: Build with S3 support
cargo build --release --features s3
```

## Usage

### Getting Started with the Example

The project includes a simple example you can run to see Crabwalk in action:

```bash
# Clone the repository
git clone https://github.com/definite-app/crabwalk.git
cd crabwalk

# Build and run the example
cargo run

# Examine the output lineage diagram
cat examples/simple/lineage.mmd
```

The example processes these files:
- `examples/simple/staging/stg_customers.sql` - A simple customers table
- `examples/simple/staging/stg_orders.sql` - A simple orders table
- `examples/simple/marts/customer_orders.sql` - Joins customers and orders with model-level config: `@config: {output: {type: "view"}}`
- `examples/simple/marts/order_summary.sql` - Creates order metrics by customer with model-level config: `@config: {output: {type: "parquet", location: "./output/order_summary.parquet"}}`

This example demonstrates several key features:
1. Automatic dependency resolution (Crabwalk figures out the correct execution order)
2. Model-level configuration through SQL comments
3. Support for both tables and views
4. Parquet file output
5. Lineage diagram generation

### Basic Usage

```bash
# Run SQL transformations
crabwalk run ./sql --db my_database.duckdb --schema transform

# Use different output types
crabwalk run ./sql --output-type view
crabwalk run ./sql --output-type parquet --output-location ./data/parquet
```

### Backup and Restore (with S3 support)

```bash
# Backup the database to S3
crabwalk backup --db my_database.duckdb --bucket my-bucket --access-key XXX --secret-key YYY

# Restore the database from S3
crabwalk restore --db my_database.duckdb --bucket my-bucket --access-key XXX --secret-key YYY
```

## Model Configuration

You can configure models directly in SQL files using comments:

```sql
-- @config: {output: {type: "view"}}
SELECT * FROM source_table

-- Or for file outputs:
-- @config: {output: {type: "parquet", location: "./output/custom_{table_name}.parquet"}}
SELECT * FROM source_table
```

## How It Works

1. Crabwalk analyzes SQL files in the specified folder
2. It parses the SQL syntax to extract table dependencies
3. It builds a directed graph of dependencies and performs a topological sort
4. It executes the SQL files in the correct order
5. It creates outputs based on configuration (tables, views, or files)

## Limitations

- Only DuckDB is supported as the backend/dialect
- Python transformations are not yet supported
- Jinja templating is not supported (environment variables are available)

## License

MIT License