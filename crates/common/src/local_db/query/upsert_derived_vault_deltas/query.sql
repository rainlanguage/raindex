INSERT OR REPLACE INTO derived_vault_deltas (
  chain_id,
  raindex_address,
  transaction_hash,
  log_index,
  block_number,
  block_timestamp,
  owner,
  kind,
  token,
  vault_id,
  delta
)
SELECT
  vd.chain_id,
  vd.raindex_address,
  vd.transaction_hash,
  vd.log_index,
  vd.block_number,
  vd.block_timestamp,
  vd.owner,
  vd.kind,
  vd.token,
  vd.vault_id,
  vd.delta
FROM vault_deltas vd
WHERE vd.chain_id = ?1
  AND vd.raindex_address = ?2
  AND vd.block_number BETWEEN ?3 AND ?4;
