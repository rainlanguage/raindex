WITH filtered_vault_balances AS (
  SELECT rvb.*
  FROM running_vault_balances rvb
  WHERE 1 = 1
    /*INNER_CHAIN_IDS_CLAUSE*/
    /*INNER_ORDERBOOKS_CLAUSE*/
),
order_io_vaults AS (
  SELECT DISTINCT
    io.chain_id,
    io.orderbook_address,
    oe.order_owner AS owner,
    io.token,
    io.vault_id,
    FLOAT_ZERO_HEX() AS balance
  FROM order_ios io
  JOIN order_events oe
    ON oe.chain_id = io.chain_id
   AND oe.orderbook_address = io.orderbook_address
   AND oe.transaction_hash = io.transaction_hash
   AND oe.log_index = io.log_index
  WHERE 1 = 1
    /*OIO_CHAIN_IDS_CLAUSE*/
    /*OIO_ORDERBOOKS_CLAUSE*/
    AND NOT EXISTS (
      SELECT 1 FROM filtered_vault_balances fvb
      WHERE fvb.chain_id = io.chain_id
       AND fvb.orderbook_address = io.orderbook_address
       AND fvb.owner = oe.order_owner
       AND fvb.token = io.token
       AND fvb.vault_id = io.vault_id
    )
),
vault_balances AS (
  SELECT
    rvb.chain_id,
    rvb.orderbook_address,
    rvb.owner,
    rvb.token,
    rvb.vault_id,
    rvb.balance
  FROM filtered_vault_balances rvb
  WHERE 1 = 1
    /*CHAIN_IDS_CLAUSE*/
    /*ORDERBOOKS_CLAUSE*/
  UNION ALL
  SELECT chain_id, orderbook_address, owner, token, vault_id, balance
  FROM order_io_vaults
),
relevant_vaults AS (
  SELECT DISTINCT chain_id, orderbook_address, owner, token, vault_id
  FROM vault_balances
),
latest_owner_events AS (
  SELECT e1.chain_id,
         e1.orderbook_address,
         e1.order_owner,
         e1.order_nonce,
         e1.event_type,
         e1.transaction_hash,
         e1.log_index
  FROM order_events e1
  WHERE EXISTS (
    SELECT 1
    FROM relevant_vaults rv
    WHERE rv.chain_id = e1.chain_id
      AND rv.orderbook_address = e1.orderbook_address
  )
    AND NOT EXISTS (
      SELECT 1
      FROM order_events e2
      WHERE e2.chain_id = e1.chain_id
        AND e2.orderbook_address = e1.orderbook_address
        AND e2.order_owner = e1.order_owner
        AND e2.order_nonce = e1.order_nonce
        AND (
          e2.block_number > e1.block_number
          OR (e2.block_number = e1.block_number AND e2.log_index > e1.log_index)
        )
    )
),
order_io_items AS (
  SELECT
    io.chain_id,
    io.orderbook_address,
    rv.owner,
    io.token,
    io.vault_id,
    io.io_type,
    '0x01' || ':' || oe.order_hash || ':' ||
      CASE WHEN loe.event_type = 'AddOrderV3' THEN '1' ELSE '0' END AS item
  FROM order_ios io
  JOIN order_events oe
    ON oe.chain_id = io.chain_id
   AND oe.orderbook_address = io.orderbook_address
   AND oe.transaction_hash = io.transaction_hash
   AND oe.log_index = io.log_index
  JOIN relevant_vaults rv
    ON rv.chain_id = io.chain_id
   AND rv.orderbook_address = io.orderbook_address
   AND rv.token = io.token
   AND rv.vault_id = io.vault_id
   AND rv.owner = oe.order_owner
  LEFT JOIN latest_owner_events loe
    ON loe.chain_id = oe.chain_id
   AND loe.orderbook_address = oe.orderbook_address
   AND loe.order_owner = oe.order_owner
   AND loe.order_nonce = oe.order_nonce
),
vault_order_lists AS (
  SELECT
    chain_id,
    orderbook_address,
    owner,
    token,
    vault_id,
    MAX(CASE WHEN io_type = 'input' THEN orders END) AS input_orders,
    MAX(CASE WHEN io_type = 'output' THEN orders END) AS output_orders
  FROM (
    SELECT
      chain_id,
      orderbook_address,
      owner,
      token,
      vault_id,
      io_type,
      GROUP_CONCAT(DISTINCT item) AS orders
    FROM order_io_items
    GROUP BY chain_id, orderbook_address, owner, token, vault_id, io_type
  ) AS per_type
  GROUP BY chain_id, orderbook_address, owner, token, vault_id
)
SELECT
  o.chain_id AS chainId,
  o.orderbook_address AS orderbookAddress,
  o.vault_id AS vaultId,
  o.token,
  o.owner,
  et.name   AS tokenName,
  et.symbol AS tokenSymbol,
  et.decimals AS tokenDecimals,
  vol.input_orders AS inputOrders,
  vol.output_orders AS outputOrders,
  o.balance
FROM vault_balances o
LEFT JOIN vault_order_lists vol
  ON vol.chain_id = o.chain_id
 AND vol.orderbook_address = o.orderbook_address
 AND vol.owner = o.owner
 AND vol.token = o.token
 AND vol.vault_id = o.vault_id
JOIN erc20_tokens et
  ON et.chain_id = o.chain_id
 AND et.orderbook_address = o.orderbook_address
 AND et.token_address = o.token
WHERE 1=1
/*OWNERS_CLAUSE*/
/*TOKENS_CLAUSE*/
/*HIDE_ZERO_BALANCE*/
/*ONLY_ACTIVE_ORDERS_CLAUSE*/
ORDER BY o.chain_id, o.orderbook_address, o.owner, o.token, o.vault_id;
