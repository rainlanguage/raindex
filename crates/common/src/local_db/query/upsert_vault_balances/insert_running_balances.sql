WITH latest_blocks AS (
  SELECT
    chain_id,
    raindex_address,
    owner,
    token,
    vault_id,
    MAX(block_number) AS last_block
  FROM vault_deltas vd
  WHERE vd.chain_id = ?1
    AND vd.raindex_address = ?2
    AND vd.block_number BETWEEN ?3 AND ?4
  GROUP BY chain_id, raindex_address, owner, token, vault_id
),
delta_batches AS (
  SELECT
    vd.chain_id,
    vd.raindex_address,
    vd.owner,
    vd.token,
    vd.vault_id,
    COALESCE(
      FLOAT_SUM(vd.delta ORDER BY vd.block_number, vd.log_index),
      FLOAT_ZERO_HEX()
    ) AS balance_delta,
    lb.last_block,
    (
      SELECT MAX(vd2.log_index)
      FROM vault_deltas vd2
      WHERE vd2.chain_id = vd.chain_id
        AND vd2.raindex_address = vd.raindex_address
        AND vd2.owner = vd.owner
        AND vd2.token = vd.token
        AND vd2.vault_id = vd.vault_id
        AND vd2.block_number = lb.last_block
    ) AS last_log_index
  FROM vault_deltas vd
  JOIN latest_blocks lb
    ON lb.chain_id = vd.chain_id
   AND lb.raindex_address = vd.raindex_address
   AND lb.owner = vd.owner
   AND lb.token = vd.token
   AND lb.vault_id = vd.vault_id
  WHERE vd.chain_id = ?1
    AND vd.raindex_address = ?2
    AND vd.block_number BETWEEN ?3 AND ?4
  GROUP BY vd.chain_id, vd.raindex_address, vd.owner, vd.token, vd.vault_id, lb.last_block
),
existing_matching AS (
  SELECT
    mvb.chain_id,
    mvb.raindex_address,
    mvb.owner,
    mvb.token,
    mvb.vault_id,
    mvb.balance AS balance_value,
    mvb.last_block,
    mvb.last_log_index
  FROM running_vault_balances mvb
  JOIN delta_batches db
    ON db.chain_id = mvb.chain_id
   AND db.raindex_address = mvb.raindex_address
   AND db.owner = mvb.owner
   AND db.token = mvb.token
   AND db.vault_id = mvb.vault_id
),
combined AS (
  SELECT
    chain_id,
    raindex_address,
    owner,
    token,
    vault_id,
    balance_delta AS contribution,
    last_block,
    last_log_index
  FROM delta_batches
  UNION ALL
  SELECT
    chain_id,
    raindex_address,
    owner,
    token,
    vault_id,
    balance_value AS contribution,
    last_block,
    last_log_index
  FROM existing_matching
),
aggregated AS (
  SELECT
    chain_id,
    raindex_address,
    owner,
    token,
    vault_id,
    COALESCE(FLOAT_SUM(contribution), FLOAT_ZERO_HEX()) AS balance,
    MAX(last_block) AS last_block
  FROM combined
  GROUP BY chain_id, raindex_address, owner, token, vault_id
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
  (
    SELECT c.last_log_index
    FROM combined c
    WHERE c.chain_id = a.chain_id
      AND c.raindex_address = a.raindex_address
      AND c.owner = a.owner
      AND c.token = a.token
      AND c.vault_id = a.vault_id
      AND c.last_block = a.last_block
    ORDER BY c.last_log_index DESC
    LIMIT 1
  ) AS last_log_index,
  (CAST(strftime('%s', 'now') AS INTEGER) * 1000) AS updated_at
FROM aggregated a;
