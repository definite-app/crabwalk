-- Raw orders data
SELECT 
  1 as id, 
  1 as user_id, 
  10 as order_amount, 
  '2018-01-01' as order_date, 
  'returned' as status
UNION ALL SELECT 
  2, 3, 20, '2018-01-02', 'completed'
UNION ALL SELECT 
  3, 5, 30, '2018-01-03', 'completed'
UNION ALL SELECT 
  4, 6, 40, '2018-01-04', 'returned'
UNION ALL SELECT 
  5, 7, 50, '2018-01-05', 'completed'
UNION ALL SELECT 
  6, 8, 60, '2018-01-06', 'completed'
UNION ALL SELECT 
  7, 9, 70, '2018-01-07', 'completed'
UNION ALL SELECT 
  8, 10, 80, '2018-01-08', 'completed'
UNION ALL SELECT 
  9, 2, 90, '2018-01-09', 'returned'
UNION ALL SELECT 
  10, 4, 100, '2018-01-10', 'completed'
UNION ALL SELECT 
  11, 1, 110, '2018-01-11', 'completed'
UNION ALL SELECT 
  12, 3, 120, '2018-01-12', 'completed'
UNION ALL SELECT 
  13, 5, 130, '2018-01-13', 'completed'
UNION ALL SELECT 
  14, 7, 140, '2018-01-14', 'returned'
UNION ALL SELECT 
  15, 9, 150, '2018-01-15', 'completed'