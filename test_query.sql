-- A complex SQL query to test the DuckDB AST parser with various node types
SELECT 
    c.customer_id,
    c.name AS customer_name,
    COUNT(o.order_id) AS order_count,
    SUM(o.amount) AS total_spent,
    AVG(o.amount) AS avg_order_value,
    MAX(o.order_date) AS last_order_date,
    CASE
        WHEN COUNT(o.order_id) > 10 THEN 'VIP'
        WHEN COUNT(o.order_id) > 5 THEN 'Regular'
        ELSE 'New'
    END AS customer_status
FROM 
    customers c
LEFT JOIN 
    orders o ON c.customer_id = o.customer_id
WHERE 
    c.is_active = TRUE
    AND o.order_date >= DATE '2023-01-01'
GROUP BY 
    c.customer_id, c.name
HAVING 
    COUNT(o.order_id) > 0
ORDER BY 
    total_spent DESC
LIMIT 
    100;