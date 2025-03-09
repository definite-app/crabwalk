-- Create a simple orders staging table
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
  199.99 as amount