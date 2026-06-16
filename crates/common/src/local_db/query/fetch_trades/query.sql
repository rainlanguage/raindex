WITH filtered_trades AS (
  SELECT
    tws.*
  FROM derived_trades tws
  WHERE 1 = 1
  /*CHAIN_IDS_CLAUSE*/
  /*RAINDEXES_CLAUSE*/
  /*TAKERS_CLAUSE*/
  /*OWNERS_CLAUSE*/
  /*ORDER_HASH_CLAUSE*/
  /*ORDER_HASHES_CLAUSE*/
  /*START_TS_CLAUSE*/
  /*END_TS_CLAUSE*/
  /*INPUT_TOKENS_CLAUSE*/
  /*OUTPUT_TOKENS_CLAUSE*/
  ORDER BY tws.block_timestamp DESC, tws.block_number DESC, tws.log_index DESC, tws.trade_kind, tws.trade_side
  /*PAGINATION_CLAUSE*/
)
SELECT
  tws.chain_id,
  tws.trade_kind,
  tws.raindex_address AS raindex,
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
  tws.input_running_balance,
  tws.output_vault_id,
  tws.output_token,
  tok_out.name AS output_token_name,
  tok_out.symbol AS output_token_symbol,
  tok_out.decimals AS output_token_decimals,
  tws.output_delta,
  tws.output_running_balance,
  tws.trade_id
FROM filtered_trades tws
LEFT JOIN erc20_tokens tok_in
  ON tok_in.chain_id = tws.chain_id
 AND tok_in.raindex_address = tws.raindex_address
 AND tok_in.token_address = tws.input_token
LEFT JOIN erc20_tokens tok_out
  ON tok_out.chain_id = tws.chain_id
 AND tok_out.raindex_address = tws.raindex_address
 AND tok_out.token_address = tws.output_token
ORDER BY tws.block_timestamp DESC, tws.block_number DESC, tws.log_index DESC, tws.trade_kind, tws.trade_side;
