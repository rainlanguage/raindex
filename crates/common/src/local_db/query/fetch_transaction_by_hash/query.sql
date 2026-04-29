WITH params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS raindex_address,
    ?3 AS transaction_hash
),
combined AS (
  SELECT
    d.transaction_hash,
    d.block_number,
    d.block_timestamp,
    d.sender,
    d.log_index
  FROM deposits d
  JOIN params p
    ON p.chain_id = d.chain_id
   AND p.raindex_address = d.raindex_address
   AND p.transaction_hash = d.transaction_hash
  UNION ALL
  SELECT
    w.transaction_hash,
    w.block_number,
    w.block_timestamp,
    w.sender,
    w.log_index
  FROM withdrawals w
  JOIN params p
    ON p.chain_id = w.chain_id
   AND p.raindex_address = w.raindex_address
   AND p.transaction_hash = w.transaction_hash
)
SELECT
  transaction_hash AS transactionHash,
  block_number AS blockNumber,
  block_timestamp AS blockTimestamp,
  sender
FROM combined
ORDER BY log_index ASC
LIMIT 1;

