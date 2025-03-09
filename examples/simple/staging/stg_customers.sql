-- Create a simple customers staging table
SELECT 
  1 as customer_id,
  'John Smith' as name,
  'john@example.com' as email
UNION ALL SELECT
  2 as customer_id,
  'Jane Doe' as name,
  'jane@example.com' as email