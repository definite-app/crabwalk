-- Test SQL statement for table extraction
SELECT 
  c.customer_id,
  c.name as customer_name,
  o.order_id,
  o.amount
FROM stg_customers c
JOIN stg_orders o ON c.customer_id = o.customer_id
WHERE o.amount > 50;