SELECT chain_id, raindex_address, last_block, last_hash, updated_at
FROM target_watermarks
WHERE chain_id = ?1 AND raindex_address = ?2;
