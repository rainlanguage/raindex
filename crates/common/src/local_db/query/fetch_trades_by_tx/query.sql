WITH
params AS (
  SELECT
    ?1 AS transaction_hash
),
matching_take_orders AS (
  SELECT
    'take' AS trade_kind,
    t.chain_id,
    t.orderbook_address,
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
    ON t.transaction_hash = p.transaction_hash
  WHERE 1 = 1
  /*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/
  /*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/
),
matching_clears AS (
  SELECT
    c.chain_id,
    c.orderbook_address,
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
    ON c.transaction_hash = p.transaction_hash
  WHERE 1 = 1
  /*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/
  /*CLEAR_EVENTS_ORDERBOOKS_CLAUSE*/
),
take_trades AS (
  SELECT
    mt.trade_kind,
    mt.chain_id,
    mt.orderbook_address,
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
   AND oe.orderbook_address = mt.orderbook_address
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
      AND newer.orderbook_address = oe.orderbook_address
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
   AND io_in.orderbook_address = oe.orderbook_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = mt.input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.orderbook_address = oe.orderbook_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = mt.output_io_index
   AND io_out.io_type = 'output'
),
clear_alice AS (
  SELECT DISTINCT
    'clear' AS trade_kind,
    mc.chain_id,
    mc.orderbook_address,
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
   AND oe.orderbook_address = mc.orderbook_address
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
      AND newer.orderbook_address = oe.orderbook_address
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
   AND a.orderbook_address = mc.orderbook_address
   AND a.transaction_hash = mc.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = mc.chain_id
         AND ac.orderbook_address = mc.orderbook_address
         AND ac.transaction_hash = mc.transaction_hash
         AND ac.log_index > mc.log_index
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND io_in.orderbook_address = oe.orderbook_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = mc.alice_input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.orderbook_address = oe.orderbook_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = mc.alice_output_io_index
   AND io_out.io_type = 'output'
),
clear_bob AS (
  SELECT DISTINCT
    'clear' AS trade_kind,
    mc.chain_id,
    mc.orderbook_address,
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
   AND oe.orderbook_address = mc.orderbook_address
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
      AND newer.orderbook_address = oe.orderbook_address
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
   AND a.orderbook_address = mc.orderbook_address
   AND a.transaction_hash = mc.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = mc.chain_id
         AND ac.orderbook_address = mc.orderbook_address
         AND ac.transaction_hash = mc.transaction_hash
         AND ac.log_index > mc.log_index
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND io_in.orderbook_address = oe.orderbook_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = mc.bob_input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.orderbook_address = oe.orderbook_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = mc.bob_output_io_index
   AND io_out.io_type = 'output'
),
clear_trades AS (
  SELECT * FROM clear_alice
  UNION ALL
  SELECT * FROM clear_bob
),
unioned_trades AS (
  SELECT * FROM take_trades
  UNION ALL
  SELECT * FROM clear_trades
),
trade_rows AS (
  SELECT
    ut.trade_kind,
    ut.chain_id,
    ut.orderbook_address,
    ut.order_hash,
    ut.order_owner,
    ut.order_nonce,
    ut.transaction_hash,
    ut.log_index,
    ut.block_number,
    ut.block_timestamp,
    ut.transaction_sender,
    ut.input_vault_id,
    ut.input_token,
    ut.input_delta,
    ut.output_vault_id,
    ut.output_token,
    ut.output_delta
  FROM unioned_trades ut
),
trade_with_snapshots AS (
  SELECT
    tr.*,
    mvb_in.balance AS input_base_balance,
    mvb_in.last_block AS input_base_block,
    mvb_in.last_log_index AS input_base_log_index,
    mvb_out.balance AS output_base_balance,
    mvb_out.last_block AS output_base_block,
    mvb_out.last_log_index AS output_base_log_index
  FROM trade_rows tr
  LEFT JOIN running_vault_balances mvb_in
    ON mvb_in.chain_id = tr.chain_id
   AND mvb_in.orderbook_address = tr.orderbook_address
   AND mvb_in.owner = tr.order_owner
   AND mvb_in.token = tr.input_token
   AND mvb_in.vault_id = tr.input_vault_id
  LEFT JOIN running_vault_balances mvb_out
    ON mvb_out.chain_id = tr.chain_id
   AND mvb_out.orderbook_address = tr.orderbook_address
   AND mvb_out.owner = tr.order_owner
   AND mvb_out.token = tr.output_token
   AND mvb_out.vault_id = tr.output_vault_id
)
SELECT
  tws.chain_id,
  tws.trade_kind,
  tws.orderbook_address AS orderbook,
  tws.order_hash,
  tws.order_owner,
  tws.order_nonce,
  tws.transaction_hash,
  tws.log_index,
  tws.block_number,
  tws.block_timestamp,
  tws.transaction_sender,
  tws.input_vault_id,
  tws.input_token,
  tok_in.name AS input_token_name,
  tok_in.symbol AS input_token_symbol,
  tok_in.decimals AS input_token_decimals,
  tws.input_delta,
  vbc_input.running_balance AS input_running_balance,
  tws.output_vault_id,
  tws.output_token,
  tok_out.name AS output_token_name,
  tok_out.symbol AS output_token_symbol,
  tok_out.decimals AS output_token_decimals,
  tws.output_delta,
  vbc_output.running_balance AS output_running_balance,
  (
    '0x' ||
    lower(replace(tws.transaction_hash, '0x', '')) ||
    printf('%016x', tws.log_index)
  ) AS trade_id
FROM trade_with_snapshots tws
LEFT JOIN vault_balance_changes vbc_input
  ON vbc_input.chain_id = tws.chain_id
 AND vbc_input.orderbook_address = tws.orderbook_address
 AND vbc_input.owner = tws.order_owner
 AND vbc_input.token = tws.input_token
 AND vbc_input.vault_id = tws.input_vault_id
 AND vbc_input.block_number = tws.block_number
 AND vbc_input.log_index = tws.log_index
LEFT JOIN vault_balance_changes vbc_output
  ON vbc_output.chain_id = tws.chain_id
 AND vbc_output.orderbook_address = tws.orderbook_address
 AND vbc_output.owner = tws.order_owner
 AND vbc_output.token = tws.output_token
 AND vbc_output.vault_id = tws.output_vault_id
 AND vbc_output.block_number = tws.block_number
 AND vbc_output.log_index = tws.log_index
LEFT JOIN erc20_tokens tok_in
  ON tok_in.chain_id = tws.chain_id
 AND tok_in.orderbook_address = tws.orderbook_address
 AND tok_in.token_address = tws.input_token
LEFT JOIN erc20_tokens tok_out
  ON tok_out.chain_id = tws.chain_id
 AND tok_out.orderbook_address = tws.orderbook_address
 AND tok_out.token_address = tws.output_token
ORDER BY tws.block_timestamp DESC, tws.block_number DESC, tws.log_index DESC, tws.trade_kind;
