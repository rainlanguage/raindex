use crate::local_db::{
    query::{SqlStatement, SqlStatementBatch, SqlValue},
    RaindexIdentifier,
};

const UPSERT_RUNNING_SQL: &str = include_str!("insert_running_balances.sql");
const INSERT_BALANCE_CHANGES_SQL: &str = include_str!("insert_balance_changes.sql");

pub fn upsert_vault_balances_batch(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatementBatch {
    let change_stmt = build_stmt(
        INSERT_BALANCE_CHANGES_SQL,
        raindex_id,
        start_block,
        end_block,
    );
    let running_stmt = build_stmt(UPSERT_RUNNING_SQL, raindex_id, start_block, end_block);
    SqlStatementBatch::from(vec![
        delete_balance_changes_stmt(raindex_id, start_block, end_block),
        change_stmt,
        running_stmt,
    ])
}

fn delete_balance_changes_stmt(
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatement {
    SqlStatement::new_with_params(
        "DELETE FROM vault_balance_changes
WHERE chain_id = ?1
  AND raindex_address = ?2
  AND block_number BETWEEN ?3 AND ?4",
        [
            SqlValue::from(raindex_id.chain_id),
            SqlValue::from(raindex_id.raindex_address),
            SqlValue::from(start_block),
            SqlValue::from(end_block),
        ],
    )
}

fn build_stmt(
    template: &str,
    raindex_id: &RaindexIdentifier,
    start_block: u64,
    end_block: u64,
) -> SqlStatement {
    SqlStatement::new_with_params(
        template,
        [
            SqlValue::from(raindex_id.chain_id),
            SqlValue::from(raindex_id.raindex_address),
            SqlValue::from(start_block),
            SqlValue::from(end_block),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn batch_binds_all_params() {
        let raindex_id = RaindexIdentifier::new(111, Address::from([0x11u8; 20]));
        let batch = upsert_vault_balances_batch(&raindex_id, 100, 200);
        assert_eq!(batch.len(), 3);
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
    fn batch_targets_change_log_and_running_tables() {
        let batch = upsert_vault_balances_batch(&RaindexIdentifier::new(1, Address::ZERO), 0, 10);
        let sql: Vec<_> = batch
            .statements()
            .iter()
            .map(|s| s.sql().to_lowercase())
            .collect();
        assert!(sql[0].contains("delete from vault_balance_changes"));
        assert!(sql[1].contains("insert or ignore into vault_balance_changes"));
        assert!(sql[2].contains("insert or replace into running_vault_balances"));
    }

    #[test]
    fn batch_filters_block_range() {
        let batch = upsert_vault_balances_batch(
            &RaindexIdentifier::new(3, Address::from([0x55; 20])),
            123,
            456,
        );
        for stmt in batch.statements() {
            let sql = stmt.sql().to_lowercase();
            assert!(
                sql.contains("block_number between")
                    && (sql.contains("?3") || sql.contains("start_block"))
                    && (sql.contains("?4") || sql.contains("end_block")),
                "missing block filter"
            );
        }
    }

    #[test]
    fn running_stmt_includes_zero_balance_batches() {
        let batch = upsert_vault_balances_batch(&RaindexIdentifier::new(4, Address::ZERO), 0, 0);
        let sql = batch.statements()[2].sql().to_lowercase();
        assert!(
            !sql.contains("having not float_is_zero"),
            "should not filter out zero balance batches"
        );
    }

    #[test]
    fn running_stmt_uses_float_sum_for_updates() {
        let batch = upsert_vault_balances_batch(&RaindexIdentifier::new(5, Address::ZERO), 0, 1);
        let sql = batch.statements()[2].sql().to_lowercase();
        assert!(
            sql.contains("insert or replace into running_vault_balances"),
            "missing INSERT OR REPLACE clause"
        );
        assert!(
            sql.contains("float_sum"),
            "missing FLOAT_SUM aggregation in query"
        );
    }

    #[cfg(not(target_family = "wasm"))]
    mod sqlite_integration {
        use super::*;
        use crate::local_db::query::create_tables::CREATE_TABLES_SQL;
        use rain_math_float::Float;
        use rusqlite::{params, Connection};

        const CHAIN_ID: u32 = 8453;
        const RAINDEX: &str = "0xe522cb4a5fcb2eb31a52ff41a4653d85a4fd7c9d";
        const OWNER: &str = "0xa9c16673f65ae808688cb18952afe3d9658c808f";
        const TOKEN: &str = "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913";
        const VAULT: &str = "0x000000000000000000000000000000000000000000000000000000000000fab4";

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

        fn sqlvalue_to_rusqlite(v: SqlValue) -> rusqlite::types::Value {
            match v {
                SqlValue::Text(t) => rusqlite::types::Value::Text(t),
                SqlValue::I64(i) => rusqlite::types::Value::Integer(i),
                SqlValue::U64(u) => rusqlite::types::Value::Integer(u as i64),
                SqlValue::Null => rusqlite::types::Value::Null,
            }
        }

        fn execute_stmt(conn: &Connection, stmt: &SqlStatement) {
            let params = stmt
                .params()
                .iter()
                .cloned()
                .map(sqlvalue_to_rusqlite)
                .collect::<Vec<_>>();
            conn.execute(stmt.sql(), rusqlite::params_from_iter(params))
                .expect("execute SQL statement");
        }

        fn execute_balance_batch(conn: &Connection, start_block: u64, end_block: u64) {
            for stmt in
                upsert_vault_balances_batch(&raindex_id(), start_block, end_block).statements()
            {
                execute_stmt(conn, stmt);
            }
        }

        fn seed_delta(
            conn: &Connection,
            block_number: u64,
            log_index: u64,
            kind: &str,
            amount: &str,
        ) {
            conn.execute(
                "INSERT INTO derived_vault_deltas (
                    chain_id, raindex_address, transaction_hash, log_index, block_number,
                    block_timestamp, owner, kind, token, vault_id, delta
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    CHAIN_ID,
                    RAINDEX,
                    format!("0x{:064x}", log_index),
                    log_index,
                    block_number,
                    block_number + 1000,
                    OWNER,
                    kind,
                    TOKEN,
                    VAULT,
                    float(amount)
                ],
            )
            .expect("insert derived delta");
        }

        fn running_balance(conn: &Connection) -> String {
            conn.query_row(
                "SELECT balance FROM running_vault_balances
                 WHERE chain_id = ?1
                   AND raindex_address = ?2
                   AND owner = ?3
                   AND token = ?4
                   AND vault_id = ?5",
                params![CHAIN_ID, RAINDEX, OWNER, TOKEN, VAULT],
                |row| row.get::<_, String>(0),
            )
            .expect("running balance")
        }

        fn change_running_balance(conn: &Connection, block_number: u64) -> String {
            conn.query_row(
                "SELECT running_balance FROM vault_balance_changes
                 WHERE chain_id = ?1
                   AND raindex_address = ?2
                   AND owner = ?3
                   AND token = ?4
                   AND vault_id = ?5
                   AND block_number = ?6",
                params![CHAIN_ID, RAINDEX, OWNER, TOKEN, VAULT, block_number],
                |row| row.get::<_, String>(0),
            )
            .expect("change running balance")
        }

        #[test]
        fn replaying_same_window_does_not_double_count_running_balance() {
            let conn = setup_conn();
            seed_delta(&conn, 10, 1, "DEPOSIT", "10");

            execute_balance_batch(&conn, 10, 10);
            assert_eq!(running_balance(&conn), float("10"));
            assert_eq!(change_running_balance(&conn, 10), float("10"));

            execute_balance_batch(&conn, 10, 10);
            assert_eq!(running_balance(&conn), float("10"));
            assert_eq!(change_running_balance(&conn, 10), float("10"));
            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM vault_balance_changes", [], |row| {
                    row.get::<_, i64>(0)
                })
                .expect("change count"),
                1
            );
        }

        #[test]
        fn replaying_later_window_recomputes_from_all_derived_deltas() {
            let conn = setup_conn();
            seed_delta(&conn, 10, 1, "DEPOSIT", "10");
            seed_delta(&conn, 20, 2, "WITHDRAW", "-10");

            execute_balance_batch(&conn, 10, 10);
            assert_eq!(running_balance(&conn), float("10"));

            execute_balance_batch(&conn, 20, 20);
            assert_eq!(running_balance(&conn), float("0"));
            assert_eq!(change_running_balance(&conn, 20), float("0"));

            execute_balance_batch(&conn, 20, 20);
            assert_eq!(running_balance(&conn), float("0"));
            assert_eq!(change_running_balance(&conn, 20), float("0"));
        }
    }
}
