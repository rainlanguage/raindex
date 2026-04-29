BEGIN TRANSACTION;

DELETE FROM context_values
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM take_order_contexts
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM take_orders
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM order_ios
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM clear_v3_events
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM after_clear_v2_events
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM meta_events
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM interpreter_store_sets
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM raw_events
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM deposits
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM withdrawals
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM order_events
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM erc20_tokens
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM sync_status
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM target_watermarks
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM vault_balance_changes
WHERE chain_id = ?1 AND raindex_address = ?2;

DELETE FROM running_vault_balances
WHERE chain_id = ?1 AND raindex_address = ?2;

COMMIT;
