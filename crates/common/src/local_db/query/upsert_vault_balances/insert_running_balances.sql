WITH affected_keys AS (
  SELECT DISTINCT
    chain_id,
    raindex_address,
    owner,
    token,
    vault_id
  FROM derived_vault_deltas vd
  WHERE vd.chain_id = ?1
    AND vd.raindex_address = ?2
    AND vd.block_number BETWEEN ?3 AND ?4
),
latest_blocks AS (
  SELECT
    vd.chain_id,
    vd.raindex_address,
    vd.owner,
    vd.token,
    vd.vault_id,
    MAX(vd.block_number) AS last_block
  FROM derived_vault_deltas vd
  JOIN affected_keys ak
    ON ak.chain_id = vd.chain_id
   AND ak.raindex_address = vd.raindex_address
   AND ak.owner = vd.owner
   AND ak.token = vd.token
   AND ak.vault_id = vd.vault_id
  WHERE vd.block_number <= ?4
  GROUP BY vd.chain_id, vd.raindex_address, vd.owner, vd.token, vd.vault_id
),
aggregated AS (
  SELECT
    vd.chain_id,
    vd.raindex_address,
    vd.owner,
    vd.token,
    vd.vault_id,
    COALESCE(
      FLOAT_SUM(vd.delta ORDER BY vd.block_number, vd.log_index),
      FLOAT_ZERO_HEX()
    ) AS balance,
    lb.last_block,
    (
      SELECT MAX(vd2.log_index)
      FROM derived_vault_deltas vd2
      WHERE vd2.chain_id = vd.chain_id
        AND vd2.raindex_address = vd.raindex_address
        AND vd2.owner = vd.owner
        AND vd2.token = vd.token
        AND vd2.vault_id = vd.vault_id
        AND vd2.block_number = lb.last_block
    ) AS last_log_index
  FROM derived_vault_deltas vd
  JOIN latest_blocks lb
    ON lb.chain_id = vd.chain_id
   AND lb.raindex_address = vd.raindex_address
   AND lb.owner = vd.owner
   AND lb.token = vd.token
   AND lb.vault_id = vd.vault_id
  WHERE vd.block_number <= ?4
  GROUP BY vd.chain_id, vd.raindex_address, vd.owner, vd.token, vd.vault_id, lb.last_block
)
INSERT OR REPLACE INTO running_vault_balances (
  chain_id,
  raindex_address,
  owner,
  token,
  vault_id,
  balance,
  last_block,
  last_log_index,
  updated_at
)
SELECT
  a.chain_id,
  a.raindex_address,
  a.owner,
  a.token,
  a.vault_id,
  a.balance,
  a.last_block,
  a.last_log_index,
  (CAST(strftime('%s', 'now') AS INTEGER) * 1000) AS updated_at
FROM aggregated a;
