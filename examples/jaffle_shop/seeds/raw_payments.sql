-- Raw payments data
SELECT 
  1 as id, 
  1 as order_id, 
  'credit_card' as payment_method, 
  10 as amount
UNION ALL SELECT 
  2, 2, 'credit_card', 20
UNION ALL SELECT 
  3, 3, 'coupon', 30
UNION ALL SELECT 
  4, 4, 'bank_transfer', 40
UNION ALL SELECT 
  5, 5, 'credit_card', 50
UNION ALL SELECT 
  6, 6, 'credit_card', 60
UNION ALL SELECT 
  7, 7, 'coupon', 70
UNION ALL SELECT 
  8, 8, 'credit_card', 80
UNION ALL SELECT 
  9, 9, 'bank_transfer', 90
UNION ALL SELECT 
  10, 10, 'bank_transfer', 100
UNION ALL SELECT 
  11, 11, 'credit_card', 110
UNION ALL SELECT 
  12, 12, 'credit_card', 120
UNION ALL SELECT 
  13, 13, 'credit_card', 130
UNION ALL SELECT 
  14, 14, 'coupon', 140
UNION ALL SELECT 
  15, 15, 'bank_transfer', 150