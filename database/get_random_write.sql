SELECT id, created_by, content
FROM main.writes
ORDER BY RANDOM()
LIMIT 1