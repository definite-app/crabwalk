-- @config: {output: {type: "view"}}
-- Join customers and orders to create a customer orders view
SELECT
  c.customer_id,
  c.name as customer_name,
  c.email,
  o.order_id,
  o.order_date,
  o.amount
FROM stg_customers c
JOIN stg_orders o ON c.customer_id = o.customer_id