use crate::local_db::{
    query::{SqlStatement, SqlStatementBatch, SqlValue},
    RaindexIdentifier,
};

const INSERT_DERIVED_VAULT_DELTAS_SQL: &str = include_str!("query.sql");

pub fn upsert_derived_vault_deltas_batch(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatementBatch {
    SqlStatementBatch::from(vec![
        delete_derived_vault_deltas_stmt(raindex_id, start_block, end_block),
        insert_derived_vault_deltas_stmt(raindex_id, start_block, end_block),
    ])
}

fn delete_derived_vault_deltas_stmt(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatement {
    SqlStatement::new_with_params(
        "DELETE FROM derived_vault_deltas
WHERE chain_id = ?1
  AND raindex_address = ?2
  AND block_number BETWEEN ?3 AND ?4",
        bind_params(raindex_id, start_block, end_block),
    )
}

fn insert_derived_vault_deltas_stmt(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatement {
    SqlStatement::new_with_params(
        INSERT_DERIVED_VAULT_DELTAS_SQL,
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
        let batch = upsert_derived_vault_deltas_batch(&raindex_id, 100, 200);

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
        let batch =
            upsert_derived_vault_deltas_batch(&RaindexIdentifier::new(1, Address::ZERO), 0, 10);
        let statements = batch.statements();

        assert!(statements[0]
            .sql()
            .contains("DELETE FROM derived_vault_deltas"));
        assert!(statements[0]
            .sql()
            .contains("block_number BETWEEN ?3 AND ?4"));
        assert!(statements[1]
            .sql()
            .contains("INSERT OR REPLACE INTO derived_vault_deltas"));
    }

    #[test]
    fn insert_is_window_bounded_from_vault_deltas() {
        let batch =
            upsert_derived_vault_deltas_batch(&RaindexIdentifier::new(1, Address::ZERO), 0, 10);
        let sql = batch.statements()[1].sql();

        assert!(sql.contains("FROM vault_deltas vd"));
        assert!(sql.contains("vd.block_number BETWEEN ?3 AND ?4"));
    }

    #[cfg(not(target_family = "wasm"))]
    mod sqlite_integration {
        use super::*;
        use crate::local_db::query::create_tables::CREATE_TABLES_SQL;
        use crate::local_db::query::create_views::create_views_batch;
        use rain_math_float::Float;
        use rusqlite::{params, Connection};

        const CHAIN_ID: u32 = 8453;
        const RAINDEX: &str = "0xe522cb4a5fcb2eb31a52ff41a4653d85a4fd7c9d";
        const OWNER: &str = "0xowner";
        const TOKEN: &str = "0xtoken";
        const VAULT: &str = "1";

        fn setup_conn() -> Connection {
            let conn = Connection::open_in_memory().expect("open sqlite");
            crate::local_db::functions::register_all(&conn).expect("register local db functions");
            conn.execute_batch(CREATE_TABLES_SQL)
                .expect("create tables");
            for stmt in create_views_batch().statements() {
                conn.execute_batch(stmt.sql()).expect("create views");
            }
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

        fn seed_deposit(conn: &Connection, block_number: u64, log_index: u64, amount: &str) {
            conn.execute(
                "INSERT INTO deposits (
                    chain_id, raindex_address, transaction_hash, log_index, block_number,
                    block_timestamp, sender, token, vault_id, deposit_amount, deposit_amount_uint256
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    tx(log_index as u8),
                    log_index,
                    block_number,
                    block_number + 1000,
                    OWNER,
                    TOKEN,
                    VAULT,
                    float(amount),
                    amount
                ],
            )
            .expect("insert deposit");
        }

        fn seed_withdrawal(conn: &Connection, block_number: u64, log_index: u64, amount: &str) {
            conn.execute(
                "INSERT INTO withdrawals (
                    chain_id, raindex_address, transaction_hash, log_index, block_number,
                    block_timestamp, sender, token, vault_id, target_amount, withdraw_amount,
                    withdraw_amount_uint256
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    tx(log_index as u8),
                    log_index,
                    block_number,
                    block_number + 1000,
                    OWNER,
                    TOKEN,
                    VAULT,
                    float(amount),
                    float(amount),
                    amount
                ],
            )
            .expect("insert withdrawal");
        }

        #[test]
        fn rebuilds_window_and_matches_vault_deltas_view() {
            let conn = setup_conn();
            seed_deposit(&conn, 20, 1, "10");
            seed_withdrawal(&conn, 30, 2, "4");

            execute_batch(
                &conn,
                upsert_derived_vault_deltas_batch(&raindex_id(), 20, 30),
            );

            let view_rows: Vec<(String, String)> = conn
                .prepare(
                    "SELECT kind, delta FROM vault_deltas
                     WHERE block_number BETWEEN 20 AND 30
                     ORDER BY block_number, log_index",
                )
                .expect("prepare view query")
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("query view")
                .collect::<Result<_, _>>()
                .expect("view rows");
            let derived_rows: Vec<(String, String)> = conn
                .prepare(
                    "SELECT kind, delta FROM derived_vault_deltas
                     ORDER BY block_number, log_index",
                )
                .expect("prepare derived query")
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("query derived")
                .collect::<Result<_, _>>()
                .expect("derived rows");

            assert_eq!(derived_rows, view_rows);
            assert_eq!(derived_rows.len(), 2);
            assert_eq!(derived_rows[0].0, "DEPOSIT");
            assert_eq!(derived_rows[1].0, "WITHDRAW");
        }

        #[test]
        fn incremental_windows_replace_only_target_range() {
            let conn = setup_conn();
            seed_deposit(&conn, 20, 1, "10");
            seed_deposit(&conn, 30, 2, "12");

            execute_batch(
                &conn,
                upsert_derived_vault_deltas_batch(&raindex_id(), 20, 20),
            );
            execute_batch(
                &conn,
                upsert_derived_vault_deltas_batch(&raindex_id(), 30, 30),
            );

            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM derived_vault_deltas", [], |row| {
                    row.get::<_, i64>(0)
                })
                .expect("count"),
                2
            );

            conn.execute(
                "UPDATE deposits SET deposit_amount = ?1 WHERE block_number = 20",
                params![float("99")],
            )
            .expect("update source deposit");
            execute_batch(
                &conn,
                upsert_derived_vault_deltas_batch(&raindex_id(), 20, 20),
            );

            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM derived_vault_deltas", [], |row| {
                    row.get::<_, i64>(0)
                })
                .expect("count"),
                2
            );
            assert_eq!(
                conn.query_row(
                    "SELECT delta FROM derived_vault_deltas WHERE block_number = 20",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .expect("block 20 delta"),
                float("99")
            );
            assert_eq!(
                conn.query_row(
                    "SELECT delta FROM derived_vault_deltas WHERE block_number = 30",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .expect("block 30 delta"),
                float("12")
            );
        }
    }
}
