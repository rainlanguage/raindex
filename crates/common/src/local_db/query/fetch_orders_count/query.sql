SELECT COUNT(*) AS orders_count FROM (
  SELECT
    l.chain_id
  FROM (
    SELECT
      latest.chain_id,
      latest.orderbook_address,
      latest.order_owner,
      latest.order_nonce,
      latest.order_hash,
      latest.event_type
    FROM (
      SELECT
        oe.chain_id,
        oe.orderbook_address,
        oe.order_owner,
        oe.order_nonce,
        oe.order_hash,
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
    ) latest
    WHERE latest.row_rank_latest = 1
  ) l
  LEFT JOIN (
    SELECT
      ranked.chain_id,
      ranked.orderbook_address,
      ranked.order_owner,
      ranked.order_nonce,
      ranked.order_hash,
      ranked.transaction_hash,
      ranked.log_index
    FROM (
      SELECT
        oe.chain_id,
        oe.orderbook_address,
        oe.order_owner,
        oe.order_nonce,
        oe.order_hash,
        oe.transaction_hash,
        oe.log_index,
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
    l.order_nonce
);
