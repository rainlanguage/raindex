use crate::local_db::{
    query::{SqlStatement, SqlStatementBatch, SqlValue},
    RaindexIdentifier,
};

const INSERT_DERIVED_TRADES_SQL: &str = include_str!("query.sql");

pub fn upsert_derived_trades_batch(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatementBatch {
    SqlStatementBatch::from(vec![
        delete_derived_trades_stmt(raindex_id, start_block, end_block),
        insert_derived_trades_stmt(raindex_id, start_block, end_block),
    ])
}

fn delete_derived_trades_stmt(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatement {
    SqlStatement::new_with_params(
        "DELETE FROM derived_trades
WHERE chain_id = ?1
  AND raindex_address = ?2
  AND block_number BETWEEN ?3 AND ?4",
        bind_params(raindex_id, start_block, end_block),
    )
}

fn insert_derived_trades_stmt(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatement {
    SqlStatement::new_with_params(
        INSERT_DERIVED_TRADES_SQL,
        bind_params(raindex_id, start_block, end_block),
    )
}

fn bind_params(raindex_id: &RaindexIdentifier, start_block: u64, end_block: u64) -> [SqlValue; 4] {
    [
        SqlValue::from(raindex_id.chain_id),
        SqlValue::from(raindex_id.raindex_address),
        SqlValue::from(start_block),
        SqlValue::from(end_block),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn batch_binds_delete_and_insert_params() {
        let raindex_id = RaindexIdentifier::new(111, Address::from([0x11u8; 20]));
        let batch = upsert_derived_trades_batch(&raindex_id, 100, 200);

        assert_eq!(batch.len(), 2);
        for stmt in batch.statements() {
            assert_eq!(stmt.params().len(), 4);
            assert_eq!(stmt.params()[0], SqlValue::U64(111));
            assert_eq!(
                stmt.params()[1],
                SqlValue::Text(raindex_id.raindex_address.to_string())
            );
            assert_eq!(stmt.params()[2], SqlValue::U64(100));
            assert_eq!(stmt.params()[3], SqlValue::U64(200));
        }
    }

    #[test]
    fn batch_deletes_window_before_rebuilding() {
        let batch = upsert_derived_trades_batch(&RaindexIdentifier::new(1, Address::ZERO), 0, 10);
        let statements = batch.statements();

        assert!(statements[0].sql().contains("DELETE FROM derived_trades"));
        assert!(statements[0]
            .sql()
            .contains("block_number BETWEEN ?3 AND ?4"));
        assert!(statements[1]
            .sql()
            .contains("INSERT OR REPLACE INTO derived_trades"));
    }

    #[test]
    fn insert_is_window_bounded_but_order_lookup_can_read_history() {
        let batch = upsert_derived_trades_batch(&RaindexIdentifier::new(1, Address::ZERO), 0, 10);
        let sql = batch.statements()[1].sql();

        assert!(sql.contains("t.block_number BETWEEN p.start_block AND p.end_block"));
        assert!(sql.contains("c.block_number BETWEEN p.start_block AND p.end_block"));
        assert!(sql.contains("oe.block_number < mt.block_number"));
        assert!(sql.contains("oe.block_number < mc.block_number"));
    }

    #[test]
    fn insert_preserves_current_trade_ids_and_clear_sides() {
        let batch = upsert_derived_trades_batch(&RaindexIdentifier::new(1, Address::ZERO), 0, 10);
        let sql = batch.statements()[1].sql();

        assert!(sql.contains("'take' AS trade_side"));
        assert!(sql.contains("'alice' AS trade_side"));
        assert!(sql.contains("'bob' AS trade_side"));
        assert!(sql.contains("printf('%016x', tr.log_index)"));
        assert!(sql.contains("WHEN 'alice' THEN '01'"));
        assert!(sql.contains("WHEN 'bob' THEN '02'"));
    }

    #[cfg(not(target_family = "wasm"))]
    mod sqlite_integration {
        use super::*;
        use crate::local_db::query::create_tables::CREATE_TABLES_SQL;
        use crate::local_db::query::{SqlStatement, SqlStatementBatch, SqlValue};
        use rain_math_float::Float;
        use rusqlite::{params, Connection};

        const CHAIN_ID: u32 = 1;
        const RAINDEX: &str = "0x0000000000000000000000000000000000000aaa";
        const OWNER: &str = "0x0000000000000000000000000000000000000bbb";
        const TAKER: &str = "0x0000000000000000000000000000000000000ccc";
        const ALICE: &str = "0x0000000000000000000000000000000000000a11";
        const BOB: &str = "0x0000000000000000000000000000000000000b0b";
        const TOKEN_IN: &str = "0x0000000000000000000000000000000000000011";
        const TOKEN_OUT: &str = "0x0000000000000000000000000000000000000022";
        const VAULT_IN: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";
        const VAULT_OUT: &str =
            "0x0000000000000000000000000000000000000000000000000000000000000002";
        const ORDER_HASH: &str =
            "0x1111111111111111111111111111111111111111111111111111111111111111";
        const ALICE_HASH: &str =
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        const BOB_HASH: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

        fn setup_conn() -> Connection {
            let conn = Connection::open_in_memory().expect("open sqlite");
            crate::local_db::functions::register_all(&conn).expect("register local db functions");
            conn.execute_batch(CREATE_TABLES_SQL)
                .expect("create tables");
            conn
        }

        fn raindex_id() -> RaindexIdentifier {
            RaindexIdentifier::new(CHAIN_ID, RAINDEX.parse().expect("valid raindex"))
        }

        fn float(value: &str) -> String {
            Float::parse(value.to_string())
                .expect("valid float")
                .as_hex()
        }

        fn tx(byte: u8) -> String {
            format!("0x{:064x}", byte)
        }

        fn sqlvalue_to_rusqlite(v: SqlValue) -> rusqlite::types::Value {
            match v {
                SqlValue::Text(t) => rusqlite::types::Value::Text(t),
                SqlValue::I64(i) => rusqlite::types::Value::Integer(i),
                SqlValue::U64(u) => rusqlite::types::Value::Integer(u as i64),
                SqlValue::Null => rusqlite::types::Value::Null,
            }
        }

        fn execute_stmt(conn: &Connection, stmt: &SqlStatement) {
            if stmt.params().is_empty() {
                conn.execute_batch(stmt.sql()).expect("execute SQL batch");
                return;
            }

            let params = stmt
                .params()
                .iter()
                .cloned()
                .map(sqlvalue_to_rusqlite)
                .collect::<Vec<_>>();
            conn.execute(stmt.sql(), rusqlite::params_from_iter(params))
                .expect("execute SQL statement");
        }

        fn execute_batch(conn: &Connection, batch: SqlStatementBatch) {
            for stmt in batch.statements() {
                execute_stmt(conn, stmt);
            }
        }

        fn seed_order(
            conn: &Connection,
            owner: &str,
            nonce: &str,
            order_hash: &str,
            block_number: u64,
            log_index: u64,
        ) {
            let order_tx = tx(log_index as u8);
            conn.execute(
                "INSERT INTO order_events (
                    chain_id, raindex_address, transaction_hash, log_index, block_number,
                    block_timestamp, sender, interpreter_address, store_address, order_hash,
                    event_type, order_owner, order_nonce, order_bytes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'interp', 'store', ?8, 'AddOrderV3', ?9, ?10, '0x')",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    order_tx,
                    log_index,
                    block_number,
                    block_number + 1000,
                    owner,
                    order_hash,
                    owner,
                    nonce
                ],
            )
            .expect("insert order event");

            conn.execute(
                "INSERT INTO order_ios (
                    chain_id, raindex_address, transaction_hash, log_index, io_index, io_type, token, vault_id
                ) VALUES (?1, ?2, ?3, ?4, 0, 'input', ?5, ?6)",
                params![CHAIN_ID, RAINDEX, order_tx, log_index, TOKEN_IN, VAULT_IN],
            )
            .expect("insert input io");
            conn.execute(
                "INSERT INTO order_ios (
                    chain_id, raindex_address, transaction_hash, log_index, io_index, io_type, token, vault_id
                ) VALUES (?1, ?2, ?3, ?4, 0, 'output', ?5, ?6)",
                params![CHAIN_ID, RAINDEX, order_tx, log_index, TOKEN_OUT, VAULT_OUT],
            )
            .expect("insert output io");
        }

        fn seed_take(
            conn: &Connection,
            block_number: u64,
            log_index: u64,
            input: &str,
            output: &str,
        ) {
            conn.execute(
                "INSERT INTO take_orders (
                    chain_id, raindex_address, transaction_hash, log_index, block_number,
                    block_timestamp, sender, order_owner, order_nonce, input_io_index,
                    output_io_index, taker_input, taker_output
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'nonce-1', 0, 0, ?9, ?10)",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    tx(log_index as u8),
                    log_index,
                    block_number,
                    block_number + 1000,
                    TAKER,
                    OWNER,
                    input,
                    output
                ],
            )
            .expect("insert take");
        }

        fn seed_balance_change(
            conn: &Connection,
            tx_hash: &str,
            log_index: u64,
            block_number: u64,
            token: &str,
            vault_id: &str,
            running_balance: &str,
        ) {
            conn.execute(
                "INSERT INTO vault_balance_changes (
                    chain_id, raindex_address, transaction_hash, owner, token, vault_id,
                    block_number, block_timestamp, log_index, change_type, delta, running_balance
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'TEST', ?10, ?11)",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    tx_hash,
                    OWNER,
                    token,
                    vault_id,
                    block_number,
                    block_number + 1000,
                    log_index,
                    float("1"),
                    running_balance
                ],
            )
            .expect("insert balance change");
        }

        #[test]
        fn rebuilds_take_rows_in_window_and_reads_older_order() {
            let conn = setup_conn();
            seed_order(&conn, OWNER, "nonce-1", ORDER_HASH, 10, 1);
            seed_take(&conn, 20, 5, &float("1"), &float("2"));
            seed_balance_change(&conn, &tx(5), 5, 20, TOKEN_IN, VAULT_IN, "input-running");
            seed_balance_change(&conn, &tx(5), 5, 20, TOKEN_OUT, VAULT_OUT, "output-running");

            execute_batch(&conn, upsert_derived_trades_batch(&raindex_id(), 20, 20));

            let row = conn
                .query_row(
                    "SELECT trade_kind, trade_side, order_hash, input_delta, output_delta,
                            input_running_balance, output_running_balance, trade_id
                     FROM derived_trades",
                    [],
                    |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, Option<String>>(6)?,
                            row.get::<_, String>(7)?,
                        ))
                    },
                )
                .expect("derived trade row");

            assert_eq!(row.0, "take");
            assert_eq!(row.1, "take");
            assert_eq!(row.2, ORDER_HASH);
            assert_eq!(row.3, float("2"));
            assert_eq!(row.4, float("-1"));
            assert_eq!(row.5.as_deref(), Some("input-running"));
            assert_eq!(row.6.as_deref(), Some("output-running"));
            assert_eq!(row.7, format!("{}{:016x}", tx(5), 5).replace("0x", "0x"));
        }

        #[test]
        fn incremental_windows_replace_only_target_range() {
            let conn = setup_conn();
            seed_order(&conn, OWNER, "nonce-1", ORDER_HASH, 10, 1);
            seed_take(&conn, 20, 5, &float("1"), &float("2"));
            seed_take(&conn, 30, 6, &float("1"), &float("3"));

            execute_batch(&conn, upsert_derived_trades_batch(&raindex_id(), 20, 20));
            execute_batch(&conn, upsert_derived_trades_batch(&raindex_id(), 30, 30));

            conn.execute(
                "UPDATE take_orders SET taker_output = ?1 WHERE block_number = 20",
                params![float("9")],
            )
            .expect("update take");
            execute_batch(&conn, upsert_derived_trades_batch(&raindex_id(), 20, 20));

            let count: u64 = conn
                .query_row("SELECT COUNT(*) FROM derived_trades", [], |row| row.get(0))
                .expect("count rows");
            let updated_delta: String = conn
                .query_row(
                    "SELECT input_delta FROM derived_trades WHERE block_number = 20",
                    [],
                    |row| row.get(0),
                )
                .expect("updated row");
            let untouched_delta: String = conn
                .query_row(
                    "SELECT input_delta FROM derived_trades WHERE block_number = 30",
                    [],
                    |row| row.get(0),
                )
                .expect("untouched row");

            assert_eq!(count, 2);
            assert_eq!(updated_delta, float("9"));
            assert_eq!(untouched_delta, float("3"));
        }

        #[test]
        fn rebuilds_clear_rows_with_alice_and_bob_trade_ids() {
            let conn = setup_conn();
            seed_order(&conn, ALICE, "alice-nonce", ALICE_HASH, 10, 1);
            seed_order(&conn, BOB, "bob-nonce", BOB_HASH, 11, 2);

            conn.execute(
                "INSERT INTO clear_v3_events (
                    chain_id, raindex_address, transaction_hash, log_index, block_number,
                    block_timestamp, sender, alice_order_hash, alice_order_owner,
                    alice_input_io_index, alice_output_io_index, alice_bounty_vault_id,
                    alice_input_vault_id, alice_output_vault_id, bob_order_hash,
                    bob_order_owner, bob_input_io_index, bob_output_io_index,
                    bob_bounty_vault_id, bob_input_vault_id, bob_output_vault_id
                ) VALUES (
                    ?1, ?2, ?3, 7, 25, 1025, ?4, ?5, ?6, 0, 0, 'alice-bounty',
                    ?7, ?8, ?9, ?10, 0, 0, 'bob-bounty', ?7, ?8
                )",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    tx(7),
                    TAKER,
                    ALICE_HASH,
                    ALICE,
                    VAULT_IN,
                    VAULT_OUT,
                    BOB_HASH,
                    BOB
                ],
            )
            .expect("insert clear");
            conn.execute(
                "INSERT INTO after_clear_v2_events (
                    chain_id, raindex_address, transaction_hash, log_index, block_number,
                    block_timestamp, sender, alice_output, bob_output, alice_input, bob_input
                ) VALUES (?1, ?2, ?3, 8, 25, 1025, ?4, ?5, ?6, ?7, ?8)",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    tx(7),
                    TAKER,
                    float("4"),
                    float("5"),
                    float("6"),
                    float("7")
                ],
            )
            .expect("insert after clear");

            execute_batch(&conn, upsert_derived_trades_batch(&raindex_id(), 25, 25));

            let rows = {
                let mut stmt = conn
                    .prepare("SELECT trade_side, order_owner, trade_id FROM derived_trades ORDER BY trade_side")
                    .expect("prepare select");
                stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .expect("query rows")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect rows")
            };

            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].0, "alice");
            assert_eq!(rows[0].1, ALICE);
            assert!(rows[0].2.ends_with("000000000000000701"));
            assert_eq!(rows[1].0, "bob");
            assert_eq!(rows[1].1, BOB);
            assert!(rows[1].2.ends_with("000000000000000702"));
        }
    }
}
