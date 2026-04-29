SELECT
    chain_id,
    raindex_address,
    last_synced_block,
    updated_at
FROM sync_status
WHERE chain_id = ?1
  AND raindex_address = ?2;
