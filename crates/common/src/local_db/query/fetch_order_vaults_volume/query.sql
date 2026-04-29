WITH
params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS raindex_address,
    ?3 AS order_hash
),
order_add_events AS (
  SELECT
    oe.chain_id,
    oe.raindex_address,
    oe.transaction_hash,
    oe.log_index,
    oe.block_number,
    oe.block_timestamp,
    oe.order_owner,
    oe.order_nonce,
    oe.order_hash
  FROM order_events oe
  JOIN params p
    ON oe.chain_id = p.chain_id
   AND oe.raindex_address = p.raindex_address
   AND oe.order_hash = p.order_hash
  WHERE oe.event_type = 'AddOrderV3'
),
take_trades AS (
  SELECT
    t.block_timestamp,
    io_in.vault_id AS input_vault_id,
    io_in.token AS input_token,
    t.taker_output AS input_delta,
    io_out.vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(t.taker_input) AS output_delta
  FROM take_orders t
  JOIN params p
    ON t.chain_id = p.chain_id
   AND t.raindex_address = p.raindex_address
  JOIN order_add_events oe
    ON oe.chain_id = t.chain_id
   AND oe.raindex_address = t.raindex_address
   AND oe.order_owner = t.order_owner
   AND oe.order_nonce = t.order_nonce
   AND (
        oe.block_number < t.block_number
     OR (oe.block_number = t.block_number AND oe.log_index <= t.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_add_events newer
     WHERE newer.chain_id = oe.chain_id
      AND newer.raindex_address = oe.raindex_address
      AND newer.order_owner = oe.order_owner
       AND newer.order_nonce = oe.order_nonce
       AND (
            newer.block_number < t.block_number
         OR (newer.block_number = t.block_number AND newer.log_index <= t.log_index)
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
   AND io_in.io_index = t.input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.raindex_address = oe.raindex_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = t.output_io_index
   AND io_out.io_type = 'output'
),
clear_alice AS (
  SELECT DISTINCT
    c.block_timestamp,
    c.alice_input_vault_id AS input_vault_id,
    io_in.token AS input_token,
    a.alice_input AS input_delta,
    c.alice_output_vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(a.alice_output) AS output_delta
  FROM clear_v3_events c
  JOIN params p
    ON c.chain_id = p.chain_id
   AND c.raindex_address = p.raindex_address
  JOIN order_add_events oe
    ON oe.chain_id = c.chain_id
   AND oe.raindex_address = c.raindex_address
   AND oe.order_hash = c.alice_order_hash
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_add_events newer
     WHERE newer.chain_id = oe.chain_id
      AND newer.raindex_address = oe.raindex_address
      AND newer.order_hash = oe.order_hash
       AND (
            newer.block_number < c.block_number
         OR (newer.block_number = c.block_number AND newer.log_index <= c.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
  JOIN after_clear_v2_events a
    ON a.chain_id = c.chain_id
   AND a.raindex_address = c.raindex_address
   AND a.transaction_hash = c.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = c.chain_id
         AND ac.raindex_address = c.raindex_address
         AND ac.transaction_hash = c.transaction_hash
         AND ac.log_index > c.log_index
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND io_in.raindex_address = oe.raindex_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = c.alice_input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.raindex_address = oe.raindex_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = c.alice_output_io_index
   AND io_out.io_type = 'output'
  WHERE c.alice_order_hash = p.order_hash
),
clear_bob AS (
  SELECT DISTINCT
    c.block_timestamp,
    c.bob_input_vault_id AS input_vault_id,
    io_in.token AS input_token,
    a.bob_input AS input_delta,
    c.bob_output_vault_id AS output_vault_id,
    io_out.token AS output_token,
    FLOAT_NEGATE(a.bob_output) AS output_delta
  FROM clear_v3_events c
  JOIN params p
    ON c.chain_id = p.chain_id
   AND c.raindex_address = p.raindex_address
  JOIN order_add_events oe
    ON oe.chain_id = c.chain_id
   AND oe.raindex_address = c.raindex_address
   AND oe.order_hash = c.bob_order_hash
   AND (
        oe.block_number < c.block_number
     OR (oe.block_number = c.block_number AND oe.log_index <= c.log_index)
   )
   AND NOT EXISTS (
     SELECT 1
     FROM order_add_events newer
     WHERE newer.chain_id = oe.chain_id
      AND newer.raindex_address = oe.raindex_address
      AND newer.order_hash = oe.order_hash
       AND (
            newer.block_number < c.block_number
         OR (newer.block_number = c.block_number AND newer.log_index <= c.log_index)
       )
       AND (
            newer.block_number > oe.block_number
         OR (newer.block_number = oe.block_number AND newer.log_index > oe.log_index)
       )
   )
  JOIN after_clear_v2_events a
    ON a.chain_id = c.chain_id
   AND a.raindex_address = c.raindex_address
   AND a.transaction_hash = c.transaction_hash
   AND a.log_index = (
       SELECT MIN(ac.log_index)
       FROM after_clear_v2_events ac
       WHERE ac.chain_id = c.chain_id
         AND ac.raindex_address = c.raindex_address
         AND ac.transaction_hash = c.transaction_hash
         AND ac.log_index > c.log_index
   )
  JOIN order_ios io_in
    ON io_in.chain_id = oe.chain_id
   AND io_in.raindex_address = oe.raindex_address
   AND io_in.transaction_hash = oe.transaction_hash
   AND io_in.log_index = oe.log_index
   AND io_in.io_index = c.bob_input_io_index
   AND io_in.io_type = 'input'
  JOIN order_ios io_out
    ON io_out.chain_id = oe.chain_id
   AND io_out.raindex_address = oe.raindex_address
   AND io_out.transaction_hash = oe.transaction_hash
   AND io_out.log_index = oe.log_index
   AND io_out.io_index = c.bob_output_io_index
   AND io_out.io_type = 'output'
  WHERE c.bob_order_hash = p.order_hash
),
clear_trades AS (
  SELECT * FROM clear_alice
  UNION ALL
  SELECT * FROM clear_bob
),
all_trades AS (
  SELECT * FROM take_trades
  UNION ALL
  SELECT * FROM clear_trades
),
filtered_trades AS (
  SELECT * FROM all_trades
  WHERE 1 = 1
  /*START_TS_CLAUSE*/
  /*END_TS_CLAUSE*/
),
input_volumes AS (
  SELECT
    input_vault_id AS vault_id,
    input_token AS token,
    COALESCE(FLOAT_SUM(input_delta), FLOAT_ZERO_HEX()) AS total_in
  FROM filtered_trades
  GROUP BY input_vault_id, input_token
),
output_volumes AS (
  SELECT
    output_vault_id AS vault_id,
    output_token AS token,
    COALESCE(FLOAT_SUM(FLOAT_NEGATE(output_delta)), FLOAT_ZERO_HEX()) AS total_out
  FROM filtered_trades
  GROUP BY output_vault_id, output_token
),
all_vaults AS (
  SELECT vault_id, token FROM input_volumes
  UNION
  SELECT vault_id, token FROM output_volumes
)
SELECT
  av.vault_id,
  av.token,
  tok.name AS token_name,
  tok.symbol AS token_symbol,
  tok.decimals AS token_decimals,
  COALESCE(iv.total_in, FLOAT_ZERO_HEX()) AS total_in,
  COALESCE(ov.total_out, FLOAT_ZERO_HEX()) AS total_out
FROM all_vaults av
CROSS JOIN params p
LEFT JOIN input_volumes iv ON av.vault_id = iv.vault_id AND av.token = iv.token
LEFT JOIN output_volumes ov ON av.vault_id = ov.vault_id AND av.token = ov.token
LEFT JOIN erc20_tokens tok
  ON tok.chain_id = p.chain_id
 AND tok.raindex_address = p.raindex_address
 AND tok.token_address = av.token;
