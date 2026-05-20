WITH RECURSIVE params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS raindex_address,
    ?3 AS start_block,
    ?4 AS end_block
),
filtered AS (
  SELECT
    vd.chain_id,
    vd.raindex_address,
    vd.transaction_hash,
    vd.owner,
    vd.token,
    vd.vault_id,
    vd.block_number,
    vd.block_timestamp,
    vd.log_index,
    vd.kind AS change_type,
    vd.delta,
    ROW_NUMBER() OVER (
      PARTITION BY vd.chain_id, vd.raindex_address, vd.owner, vd.token, vd.vault_id
      ORDER BY vd.block_number, vd.log_index
    ) AS rn,
    COALESCE(rvb.balance, FLOAT_ZERO_HEX()) AS prefix_balance
  FROM vault_deltas vd
  JOIN params p
    ON p.chain_id = vd.chain_id
   AND p.raindex_address = vd.raindex_address
  LEFT JOIN running_vault_balances rvb
    ON rvb.chain_id = vd.chain_id
   AND rvb.raindex_address = vd.raindex_address
   AND rvb.owner = vd.owner
   AND rvb.token = vd.token
   AND rvb.vault_id = vd.vault_id
  WHERE vd.block_number BETWEEN p.start_block AND p.end_block
),
ordered AS (
  SELECT
    f.chain_id,
    f.raindex_address,
    f.transaction_hash,
    f.owner,
    f.token,
    f.vault_id,
    f.block_number,
    f.block_timestamp,
    f.log_index,
    f.change_type,
    f.delta,
    f.rn,
    (
      SELECT COALESCE(
        FLOAT_SUM(val ORDER BY ord),
        FLOAT_ZERO_HEX()
      )
      FROM (
        SELECT f.prefix_balance AS val, 0 AS ord
        UNION ALL
        SELECT f.delta AS val, 1 AS ord
      )
    ) AS running_balance
  FROM filtered f
  WHERE f.rn = 1

  UNION ALL

  SELECT
    next.chain_id,
    next.raindex_address,
    next.transaction_hash,
    next.owner,
    next.token,
    next.vault_id,
    next.block_number,
    next.block_timestamp,
    next.log_index,
    next.change_type,
    next.delta,
    next.rn,
    (
      SELECT COALESCE(
        FLOAT_SUM(val ORDER BY ord),
        FLOAT_ZERO_HEX()
      )
      FROM (
        SELECT ordered.running_balance AS val, 0 AS ord
        UNION ALL
        SELECT next.delta AS val, 1 AS ord
      )
    ) AS running_balance
  FROM ordered
  JOIN filtered next
    ON next.chain_id = ordered.chain_id
   AND next.raindex_address = ordered.raindex_address
   AND next.owner = ordered.owner
   AND next.token = ordered.token
   AND next.vault_id = ordered.vault_id
   AND next.rn = ordered.rn + 1
)
INSERT OR IGNORE INTO vault_balance_changes (
  chain_id,
  raindex_address,
  transaction_hash,
  owner,
  token,
  vault_id,
  block_number,
  block_timestamp,
  log_index,
  change_type,
  delta,
  running_balance
)
SELECT
  chain_id,
  raindex_address,
  transaction_hash,
  owner,
  token,
  vault_id,
  block_number,
  block_timestamp,
  log_index,
  change_type,
  delta,
  running_balance
FROM ordered;
