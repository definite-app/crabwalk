-- @config: {output: {type: "parquet", location: "./output/order_summary.parquet"}}
-- Create an order summary with aggregate metrics
SELECT
  customer_id,
  COUNT(*) as order_count,
  SUM(amount) as total_spent,
  MIN(order_date) as first_order_date,
  MAX(order_date) as last_order_date,
  AVG(amount) as average_order_value
FROM stg_orders
GROUP BY customer_id