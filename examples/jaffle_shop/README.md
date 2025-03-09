# Jaffle Shop Example for Crabwalk

This is a Crabwalk implementation of the popular "Jaffle Shop" example, which demonstrates a simple ELT workflow processing customer orders for a fictional restaurant.

## Structure

The example is organized in three layers:

1. **Sources** - Raw data loaded from CSV files:
   - `raw_customers.sql` - Customer information
   - `raw_orders.sql` - Order details
   - `raw_products.sql` - Product catalog
   - `raw_stores.sql` - Store locations
   - `raw_supplies.sql` - Supplies inventory
   - `raw_items.sql` - Order items

2. **Staging** - Lightly transformed data with renamed columns and improved types:
   - `stg_customers.sql` - Cleaned customer data
   - `stg_orders.sql` - Cleaned order data
   - `stg_products.sql` - Cleaned product data
   - `stg_locations.sql` - Cleaned store location data
   - `stg_supplies.sql` - Cleaned supplies data
   - `stg_order_items.sql` - Cleaned order items

3. **Marts** - Business-focused models combining multiple sources:
   - `customers.sql` - Customer profile with order history
   - `orders.sql` - Order details with customer information
   - `products.sql` - Product details
   - `locations.sql` - Store locations
   - `supplies.sql` - Supply inventory
   - `order_items.sql` - Order items with product details

## Running the Example

To run the Jaffle Shop example:

```bash
./run-jaffle
```

This script will:
1. Create a fresh database
2. Process source files (loading from CSVs)
3. Process staging files (transforming raw data)
4. Process mart files (creating business models)
5. Display a summary of all created tables

## Exploring the Data

After running the example, you can explore the data using DuckDB:

```bash
duckdb crabwalk.db
```

Example queries:

```sql
-- View all customers
SELECT * FROM customers;

-- View orders with customer details
SELECT o.order_id, o.order_date, c.customer_name
FROM orders o
JOIN customers c ON o.customer_id = c.customer_id
LIMIT 10;

-- View order items with product details
SELECT oi.order_id, oi.product_id, p.product_name, oi.quantity
FROM order_items oi
JOIN products p ON oi.product_id = p.product_id
LIMIT 10;
```

## Notes

- This example includes some circular dependencies between models to demonstrate how to handle them in Crabwalk.
- The lineage feature may show errors for file paths, but this doesn't affect the data processing.
- All tables are created in the `crabwalk.db` DuckDB database.