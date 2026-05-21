BEGIN TRANSACTION;

DROP VIEW IF EXISTS vault_deltas;

DROP TABLE IF EXISTS target_watermarks;
DROP TABLE IF EXISTS db_metadata;
DROP TABLE IF EXISTS context_values;
DROP TABLE IF EXISTS take_order_contexts;
DROP TABLE IF EXISTS take_orders;
DROP TABLE IF EXISTS order_ios;
DROP TABLE IF EXISTS order_events;
DROP TABLE IF EXISTS withdrawals;
DROP TABLE IF EXISTS deposits;
DROP TABLE IF EXISTS raw_events;
DROP TABLE IF EXISTS clear_v3_events;
DROP TABLE IF EXISTS after_clear_v2_events;
DROP TABLE IF EXISTS meta_events;
DROP TABLE IF EXISTS erc20_tokens;
DROP TABLE IF EXISTS interpreter_store_sets;
DROP TABLE IF EXISTS sync_status;
DROP TABLE IF EXISTS vault_balance_changes;
DROP TABLE IF EXISTS running_vault_balances;

COMMIT;

VACUUM;
