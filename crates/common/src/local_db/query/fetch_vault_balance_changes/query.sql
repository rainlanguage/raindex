WITH params AS (
  SELECT
    ?1 AS chain_id,
    ?2 AS raindex_address,
    ?3 AS vault_id,
    ?4 AS token,
    ?5 AS owner
)
SELECT
  vbc.transaction_hash AS transactionHash,
  vbc.log_index AS logIndex,
  vbc.block_number AS blockNumber,
  vbc.block_timestamp AS blockTimestamp,
  vbc.owner,
  COALESCE(
    (
      SELECT d.sender
      FROM deposits d
      WHERE d.chain_id = vbc.chain_id
        AND d.raindex_address = vbc.raindex_address
        AND d.transaction_hash = vbc.transaction_hash
        AND d.log_index = vbc.log_index
        AND vbc.change_type = 'DEPOSIT'
      LIMIT 1
    ),
    (
      SELECT w.sender
      FROM withdrawals w
      WHERE w.chain_id = vbc.chain_id
        AND w.raindex_address = vbc.raindex_address
        AND w.transaction_hash = vbc.transaction_hash
        AND w.log_index = vbc.log_index
        AND vbc.change_type = 'WITHDRAW'
      LIMIT 1
    ),
    (
      SELECT t.sender
      FROM take_orders t
      WHERE t.chain_id = vbc.chain_id
        AND t.raindex_address = vbc.raindex_address
        AND t.transaction_hash = vbc.transaction_hash
        AND t.log_index = vbc.log_index
        AND vbc.change_type IN ('TAKE_INPUT', 'TAKE_OUTPUT')
      LIMIT 1
    ),
    (
      SELECT c.sender
      FROM clear_v3_events c
      WHERE c.chain_id = vbc.chain_id
        AND c.raindex_address = vbc.raindex_address
        AND c.transaction_hash = vbc.transaction_hash
        AND c.log_index = vbc.log_index
        AND vbc.change_type IN (
          'CLEAR_ALICE_INPUT',
          'CLEAR_ALICE_OUTPUT',
          'CLEAR_BOB_INPUT',
          'CLEAR_BOB_OUTPUT',
          'CLEAR_ALICE_BOUNTY',
          'CLEAR_BOB_BOUNTY'
        )
      LIMIT 1
    ),
    vbc.owner
  ) AS transactionSender,
  vbc.change_type AS changeType,
  vbc.token,
  vbc.vault_id AS vaultId,
  vbc.delta,
  vbc.running_balance AS runningBalance
FROM vault_balance_changes vbc
JOIN params p
  ON p.chain_id = vbc.chain_id
 AND p.raindex_address = vbc.raindex_address
 AND p.vault_id = vbc.vault_id
 AND p.token = vbc.token
 AND p.owner = vbc.owner
/*CHANGE_TYPES_CLAUSE*/
ORDER BY vbc.block_timestamp DESC, vbc.block_number DESC, vbc.log_index DESC;
