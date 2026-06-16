INSERT INTO target_watermarks (chain_id, raindex_address, last_block, last_hash)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(chain_id, raindex_address) DO UPDATE SET
  last_block = excluded.last_block,
  last_hash = excluded.last_hash,
  updated_at = (CAST(strftime('%s', 'now') AS INTEGER) * 1000);
