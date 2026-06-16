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
