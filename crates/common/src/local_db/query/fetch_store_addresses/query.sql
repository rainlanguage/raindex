SELECT DISTINCT store_address AS store_address
FROM order_events
WHERE chain_id = ?1
  AND raindex_address = ?2
  AND store_address IS NOT NULL
  AND store_address != ''
UNION
SELECT DISTINCT store_address AS store_address
FROM interpreter_store_sets
WHERE chain_id = ?1
  AND raindex_address = ?2
  AND store_address IS NOT NULL
  AND store_address != '';
