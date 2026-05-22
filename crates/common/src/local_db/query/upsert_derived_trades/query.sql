WITH
params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS raindex_address,
    ?3 AS start_block,
    ?4 AS end_block
),
matching_take_orders AS (
  SELECT
    'take' AS trade_kind,
    'take' AS trade_side,
    t.chain_id,
    t.raindex_address,
    t.order_owner,
    t.order_nonce,
    t.transaction_hash,
    t.log_index,
    t.block_number,
    t.block_timestamp,
    t.sender AS transaction_sender,
    t.input_io_index,
    t.output_io_index,
    t.taker_output AS input_delta,
    FLOAT_NEGATE(t.taker_input) AS output_delta
  FROM take_orders t
  JOIN params p
    ON p.chain_id = t.chain_id
   AND p.raindex_address = t.raindex_address
  WHERE t.block_number BETWEEN p.start_block AND p.end_block
),
matching_clears AS (
  SELECT
    c.chain_id,
    c.raindex_address,
    c.transaction_hash,
    c.log_index,
    c.block_number,
    c.block_timestamp,
    c.sender,
    c.alice_order_hash,
    c.bob_order_hash,
    c.alice_input_io_index,
    c.alice_output_io_index,
    c.alice_input_vault_id,
    c.alice_output_vault_id,
    c.bob_input_io_index,
    c.bob_output_io_index,
    c.bob_input_vault_id,
    c.bob_output_vault_id
  FROM clear_v3_events c
  JOIN params p
    ON p.chain_id = c.chain_id
   AND p.raindex_address = c.raindex_address
  WHERE c.block_number BETWEEN p.start_block AND p.end_block
),
take_trades AS (
  SELECT
    mt.trade_kind,
    mt.trade_side,
    mt.chain_id,
    mt.raindex_address,
    oe.order_hash,
    mt.order_owner,
    mt.order_nonce,
    mt.transaction_hash,
    mt.log_index,
    mt.block_number,
    mt.block_timestamp,
    mt.transaction_sender,
    io_in.vault_id AS input_vault_id,
    io_in.token AS input_token,
    mt.input_delta,
    io_out.vault_id AS output_vault_id,
    io_out.token AS output_token,
    mt.output_delta
  FROM matching_take_orders mt
  JOIN order_events oe
    ON oe.chain_id = mt.chain_id
   AND oe.raindex_address = mt.raindex_address
   AND oe.order_owner = mt.order_owner
   AND oe.order_nonce = mt.order_nonce
   AND oe.event_type = 'AddOrderV3'
   AND (
        oe.block_number < mt.block_number
     OR (oe.block_number = mt.block_number AND oe.log_index <= mt.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events newer
     WHERE newer.chain_id = oe.chain_id
       AND newer.raindex_address = oe.raindex_address
       AND newer.order_owner = oe.order_owner
       AND newer.order_nonce = oe.order_nonce
       AND newer.event_type = 'AddOrderV3'
       AND (
            newer.block_number < mt.block_number
         OR (newer.block_number = mt.block_number AND newer.log_index <= mt.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND io_in.raindex_address = oe.raindex_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = mt.input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.raindex_address = oe.raindex_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = mt.output_io_index
   AND io_out.io_type = 'output'
),
clear_alice AS (
  SELECT DISTINCT
    'clear' AS trade_kind,
    'alice' AS trade_side,
    mc.chain_id,
    mc.raindex_address,
    oe.order_hash,
    oe.order_owner,
    oe.order_nonce,
    mc.transaction_hash,
    mc.log_index,
    mc.block_number,
    mc.block_timestamp,
    mc.sender AS transaction_sender,
    mc.alice_input_vault_id AS input_vault_id,
    io_in.token AS input_token,
    a.alice_input AS input_delta,
    mc.alice_output_vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(a.alice_output) AS output_delta
  FROM matching_clears mc
  JOIN order_events oe
    ON oe.chain_id = mc.chain_id
   AND oe.raindex_address = mc.raindex_address
   AND oe.order_hash = mc.alice_order_hash
   AND oe.event_type = 'AddOrderV3'
   AND (
        oe.block_number < mc.block_number
     OR (oe.block_number = mc.block_number AND oe.log_index <= mc.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events newer
     WHERE newer.chain_id = oe.chain_id
       AND newer.raindex_address = oe.raindex_address
       AND newer.order_hash = oe.order_hash
       AND newer.event_type = 'AddOrderV3'
       AND (
            newer.block_number < mc.block_number
         OR (newer.block_number = mc.block_number AND newer.log_index <= mc.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
  JOIN after_clear_v2_events a
    ON a.chain_id = mc.chain_id
   AND a.raindex_address = mc.raindex_address
   AND a.transaction_hash = mc.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = mc.chain_id
         AND ac.raindex_address = mc.raindex_address
         AND ac.transaction_hash = mc.transaction_hash
         AND ac.log_index > mc.log_index
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND io_in.raindex_address = oe.raindex_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = mc.alice_input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.raindex_address = oe.raindex_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = mc.alice_output_io_index
   AND io_out.io_type = 'output'
),
clear_bob AS (
  SELECT DISTINCT
    'clear' AS trade_kind,
    'bob' AS trade_side,
    mc.chain_id,
    mc.raindex_address,
    oe.order_hash,
    oe.order_owner,
    oe.order_nonce,
    mc.transaction_hash,
    mc.log_index,
    mc.block_number,
    mc.block_timestamp,
    mc.sender AS transaction_sender,
    mc.bob_input_vault_id AS input_vault_id,
    io_in.token AS input_token,
    a.bob_input AS input_delta,
    mc.bob_output_vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(a.bob_output) AS output_delta
  FROM matching_clears mc
  JOIN order_events oe
    ON oe.chain_id = mc.chain_id
   AND oe.raindex_address = mc.raindex_address
   AND oe.order_hash = mc.bob_order_hash
   AND oe.event_type = 'AddOrderV3'
   AND (
        oe.block_number < mc.block_number
     OR (oe.block_number = mc.block_number AND oe.log_index <= mc.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_events newer
     WHERE newer.chain_id = oe.chain_id
       AND newer.raindex_address = oe.raindex_address
       AND newer.order_hash = oe.order_hash
       AND newer.event_type = 'AddOrderV3'
       AND (
            newer.block_number < mc.block_number
         OR (newer.block_number = mc.block_number AND newer.log_index <= mc.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
  JOIN after_clear_v2_events a
    ON a.chain_id = mc.chain_id
   AND a.raindex_address = mc.raindex_address
   AND a.transaction_hash = mc.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = mc.chain_id
         AND ac.raindex_address = mc.raindex_address
         AND ac.transaction_hash = mc.transaction_hash
         AND ac.log_index > mc.log_index
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND io_in.raindex_address = oe.raindex_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = mc.bob_input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.raindex_address = oe.raindex_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = mc.bob_output_io_index
   AND io_out.io_type = 'output'
),
trade_rows AS (
  SELECT * FROM take_trades
  UNION ALL
  SELECT * FROM clear_alice
  UNION ALL
  SELECT * FROM clear_bob
)
INSERT OR REPLACE INTO derived_trades (
  chain_id,
  raindex_address,
  trade_id,
  trade_kind,
  trade_side,
  order_hash,
  order_owner,
  order_nonce,
  transaction_hash,
  log_index,
  block_number,
  block_timestamp,
  transaction_sender,
  input_vault_id,
  input_token,
  input_delta,
  input_running_balance,
  output_vault_id,
  output_token,
  output_delta,
  output_running_balance
)
SELECT
  tr.chain_id,
  tr.raindex_address,
  (
    '0x' ||
    lower(replace(tr.transaction_hash, '0x', '')) ||
    printf('%016x', tr.log_index) ||
    CASE tr.trade_side
      WHEN 'alice' THEN '01'
      WHEN 'bob' THEN '02'
      ELSE ''
    END
  ) AS trade_id,
  tr.trade_kind,
  tr.trade_side,
  tr.order_hash,
  tr.order_owner,
  tr.order_nonce,
  tr.transaction_hash,
  tr.log_index,
  tr.block_number,
  tr.block_timestamp,
  tr.transaction_sender,
  tr.input_vault_id,
  tr.input_token,
  tr.input_delta,
  vbc_input.running_balance AS input_running_balance,
  tr.output_vault_id,
  tr.output_token,
  tr.output_delta,
  vbc_output.running_balance AS output_running_balance
FROM trade_rows tr
LEFT JOIN vault_balance_changes vbc_input
  ON vbc_input.chain_id = tr.chain_id
 AND vbc_input.raindex_address = tr.raindex_address
 AND vbc_input.owner = tr.order_owner
 AND vbc_input.token = tr.input_token
 AND vbc_input.vault_id = tr.input_vault_id
 AND vbc_input.block_number = tr.block_number
 AND vbc_input.log_index = tr.log_index
LEFT JOIN vault_balance_changes vbc_output
  ON vbc_output.chain_id = tr.chain_id
 AND vbc_output.raindex_address = tr.raindex_address
 AND vbc_output.owner = tr.order_owner
 AND vbc_output.token = tr.output_token
 AND vbc_output.vault_id = tr.output_vault_id
 AND vbc_output.block_number = tr.block_number
 AND vbc_output.log_index = tr.log_index;
