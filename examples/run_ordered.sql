-- This is a wrapper script to ensure proper execution order

-- First, create the staging tables
CREATE OR REPLACE TABLE stg_customers AS
SELECT 
  1 as customer_id,
  'John Smith' as name,
  'john@example.com' as email
UNION ALL SELECT
  2 as customer_id,
  'Jane Doe' as name,
  'jane@example.com' as email;

CREATE OR REPLACE TABLE stg_orders AS
SELECT 
  101 as order_id,
  1 as customer_id,
  '2023-01-15' as order_date,
  99.99 as amount
UNION ALL SELECT
  102 as order_id,
  1 as customer_id,
  '2023-03-10' as order_date,
  149.99 as amount
UNION ALL SELECT
  103 as order_id,
  2 as customer_id,
  '2023-02-22' as order_date,
  199.99 as amount;

-- Now run marts queries

-- Create customer_orders view
-- @config: {output: {type: "view"}}
CREATE OR REPLACE VIEW customer_orders AS
SELECT
  c.customer_id,
  c.name as customer_name,
  c.email,
  o.order_id,
  o.order_date,
  o.amount
FROM stg_customers c
JOIN stg_orders o ON c.customer_id = o.customer_id;

-- Create order_summary 
-- @config: {output: {type: "parquet", location: "./examples/simple/output/order_summary.parquet"}}
CREATE OR REPLACE TABLE temp_order_summary AS
SELECT
  customer_id,
  COUNT(*) as order_count,
  SUM(amount) as total_spent,
  MIN(order_date) as first_order_date,
  MAX(order_date) as last_order_date,
  AVG(amount) as average_order_value
FROM stg_orders
GROUP BY customer_id;

-- Export to parquet
COPY (SELECT * FROM temp_order_summary) TO './examples/simple/output/order_summary.parquet' (FORMAT PARQUET);