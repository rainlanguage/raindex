SELECT COUNT(*) AS trade_count
FROM derived_trades tws
WHERE tws.chain_id = ?1
  AND tws.raindex_address = ?2
  AND tws.order_hash = ?3
/*START_TS_CLAUSE*/
/*END_TS_CLAUSE*/;
