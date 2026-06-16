SELECT
  l.chain_id AS chainId,
  COALESCE(la.order_hash, l.order_hash) AS orderHash,
  l.order_owner AS owner,
  fa.block_timestamp AS blockTimestamp,
  fa.block_number AS blockNumber,
  l.orderbook_address AS orderbookAddress,
  la.order_bytes AS orderBytes,
  json_group_array(
    CASE
      WHEN lower(ios.io_type) = 'input'
      THEN json_object(
        'ioIndex', ios.io_index,
        'vault', json_object(
          'chainId', ios.chain_id,
          'vaultId', ios.vault_id,
          'token', ios.token,
          'owner', l.order_owner,
          'orderbookAddress', l.orderbook_address,
          'tokenName', COALESCE(tok.name, ''),
          'tokenSymbol', COALESCE(tok.symbol, ''),
          'tokenDecimals', COALESCE(tok.decimals, 0),
          'balance', COALESCE(vb.balance_hex, FLOAT_ZERO_HEX()),
          'inputOrders', NULL,
          'outputOrders', NULL
        )
      )
    END
  ) AS inputs,
  json_group_array(
    CASE
      WHEN lower(ios.io_type) = 'output'
      THEN json_object(
        'ioIndex', ios.io_index,
        'vault', json_object(
          'chainId', ios.chain_id,
          'vaultId', ios.vault_id,
          'token', ios.token,
          'owner', l.order_owner,
          'orderbookAddress', l.orderbook_address,
          'tokenName', COALESCE(tok.name, ''),
          'tokenSymbol', COALESCE(tok.symbol, ''),
          'tokenDecimals', COALESCE(tok.decimals, 0),
          'balance', COALESCE(vb.balance_hex, FLOAT_ZERO_HEX()),
          'inputOrders', NULL,
          'outputOrders', NULL
        )
      )
    END
  ) AS outputs,
  COALESCE(tc.trade_count, 0) + COALESCE(cc.trade_count, 0) AS tradeCount,
  (l.event_type = 'AddOrderV3') AS active,
  la.transaction_hash AS transactionHash,
  (
    SELECT m.meta
    FROM meta_events m
    WHERE m.chain_id = l.chain_id
      AND m.orderbook_address = l.orderbook_address
      AND m.subject = COALESCE(la.order_hash, l.order_hash)
    ORDER BY m.block_number DESC, m.log_index DESC
    LIMIT 1
  ) AS meta
FROM (
  SELECT
    latest.chain_id,
    latest.orderbook_address,
    latest.order_owner,
    latest.order_nonce,
    latest.order_hash,
    latest.transaction_hash,
    latest.log_index,
    latest.block_number,
    latest.block_timestamp,
    latest.event_type
  FROM (
    SELECT
      oe.chain_id,
      oe.orderbook_address,
      oe.order_owner,
      oe.order_nonce,
      oe.order_hash,
      oe.transaction_hash,
      oe.log_index,
      oe.block_number,
      oe.block_timestamp,
      oe.event_type,
      ROW_NUMBER() OVER (
        PARTITION BY
          oe.chain_id,
          oe.orderbook_address,
          oe.order_owner,
          oe.order_nonce
        ORDER BY oe.block_number DESC, oe.log_index DESC
      ) AS row_rank_latest
    FROM order_events oe
    WHERE 1 = 1
      /*MAIN_CHAIN_IDS_CLAUSE*/
      /*MAIN_ORDERBOOKS_CLAUSE*/
      /*TX_HASH_CLAUSE*/
  ) latest
  WHERE latest.row_rank_latest = 1
) l
LEFT JOIN (
  SELECT
    ranked.chain_id,
    ranked.orderbook_address,
    ranked.order_owner,
    ranked.order_nonce,
    ranked.transaction_hash,
    ranked.log_index,
    ranked.order_hash,
    ranked.order_bytes
  FROM (
    SELECT
      oe.chain_id,
      oe.orderbook_address,
      oe.order_owner,
      oe.order_nonce,
      oe.transaction_hash,
      oe.log_index,
      oe.order_hash,
      oe.order_bytes,
      ROW_NUMBER() OVER (
        PARTITION BY
          oe.chain_id,
          oe.orderbook_address,
          oe.order_owner,
          oe.order_nonce
        ORDER BY oe.block_number DESC, oe.log_index DESC
      ) AS row_rank_add
    FROM order_events oe
    WHERE oe.event_type = 'AddOrderV3'
      /*LATEST_ADD_CHAIN_IDS_CLAUSE*/
      /*LATEST_ADD_ORDERBOOKS_CLAUSE*/
  ) ranked
  WHERE ranked.row_rank_add = 1
) la
  ON la.chain_id = l.chain_id
 AND la.orderbook_address = l.orderbook_address
 AND la.order_owner = l.order_owner
 AND la.order_nonce = l.order_nonce
LEFT JOIN order_ios ios
  ON ios.chain_id = l.chain_id
 AND ios.orderbook_address = l.orderbook_address
 AND ios.transaction_hash = la.transaction_hash
 AND ios.log_index = la.log_index
LEFT JOIN erc20_tokens tok
  ON tok.chain_id = ios.chain_id
 AND tok.orderbook_address = ios.orderbook_address
 AND tok.token_address = ios.token
LEFT JOIN (
  SELECT
    rvb.chain_id,
    rvb.orderbook_address,
    rvb.owner,
    rvb.token,
    rvb.vault_id,
    rvb.balance AS balance_hex
  FROM running_vault_balances rvb
) vb
  ON vb.chain_id = ios.chain_id
 AND vb.orderbook_address = ios.orderbook_address
 AND vb.token = ios.token
 AND vb.vault_id = ios.vault_id
 AND vb.owner = l.order_owner
LEFT JOIN (
  SELECT
    t.chain_id,
    t.orderbook_address,
    t.order_owner,
    t.order_nonce,
    COUNT(*) AS trade_count
  FROM take_orders t
  WHERE 1 = 1
    /*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/
    /*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/
  GROUP BY t.chain_id, t.orderbook_address, t.order_owner, t.order_nonce
) tc
  ON tc.chain_id = l.chain_id
 AND tc.orderbook_address = l.orderbook_address
 AND tc.order_owner = l.order_owner
 AND tc.order_nonce = l.order_nonce
LEFT JOIN (
  SELECT
    entries.chain_id,
    entries.orderbook_address,
    entries.order_hash,
    COUNT(*) AS trade_count
  FROM (
    SELECT
      c.chain_id,
      c.orderbook_address,
      c.alice_order_hash AS order_hash
    FROM clear_v3_events c
    WHERE c.alice_order_hash IS NOT NULL
    UNION ALL
    SELECT
      c.chain_id,
      c.orderbook_address,
      c.bob_order_hash AS order_hash
    FROM clear_v3_events c
    WHERE c.bob_order_hash IS NOT NULL
  ) entries
  WHERE entries.order_hash IS NOT NULL
    /*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/
    /*CLEAR_EVENTS_ORDERBOOKS_CLAUSE*/
  GROUP BY entries.chain_id, entries.orderbook_address, entries.order_hash
) cc
  ON cc.chain_id = l.chain_id
 AND cc.orderbook_address = l.orderbook_address
 AND cc.order_hash = COALESCE(la.order_hash, l.order_hash)
LEFT JOIN (
  SELECT
    ranked.chain_id,
    ranked.orderbook_address,
    ranked.order_owner,
    ranked.order_nonce,
    ranked.block_timestamp,
    ranked.block_number
  FROM (
    SELECT
      oe.chain_id,
      oe.orderbook_address,
      oe.order_owner,
      oe.order_nonce,
      oe.block_timestamp,
      oe.block_number,
      ROW_NUMBER() OVER (
        PARTITION BY
          oe.chain_id,
          oe.orderbook_address,
          oe.order_owner,
          oe.order_nonce
        ORDER BY oe.block_number ASC, oe.log_index ASC
      ) AS row_rank_first_add
    FROM order_events oe
    WHERE oe.event_type = 'AddOrderV3'
      /*FIRST_ADD_CHAIN_IDS_CLAUSE*/
      /*FIRST_ADD_ORDERBOOKS_CLAUSE*/
  ) ranked
  WHERE ranked.row_rank_first_add = 1
) fa
  ON fa.chain_id = l.chain_id
 AND fa.orderbook_address = l.orderbook_address
 AND fa.order_owner = l.order_owner
 AND fa.order_nonce = l.order_nonce
WHERE
  (
    ?1 = 'all'
    OR (?1 = 'active' AND l.event_type = 'AddOrderV3')
    OR (?1 = 'inactive' AND l.event_type = 'RemoveOrderV3')
  )
/*OWNERS_CLAUSE*/
/*ORDER_HASH_CLAUSE*/
/*INPUT_TOKENS_CLAUSE*/
/*OUTPUT_TOKENS_CLAUSE*/
GROUP BY
  l.chain_id,
  COALESCE(la.order_hash, l.order_hash),
  l.order_owner,
  fa.block_timestamp,
  fa.block_number,
  l.orderbook_address,
  l.order_nonce,
  l.event_type,
  la.transaction_hash
ORDER BY fa.block_timestamp DESC
/*PAGINATION_CLAUSE*/;
