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
    SqlStatementBatch::from(vec![change_stmt, running_stmt])
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
    fn batch_targets_change_log_and_running_tables() {
        let batch = upsert_vault_balances_batch(&RaindexIdentifier::new(1, Address::ZERO), 0, 10);
        let sql: Vec<_> = batch
            .statements()
            .iter()
            .map(|s| s.sql().to_lowercase())
            .collect();
        assert!(sql[0].contains("insert or ignore into vault_balance_changes"));
        assert!(sql[1].contains("insert or replace into running_vault_balances"));
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
        let sql = batch.statements()[1].sql().to_lowercase();
        assert!(
            !sql.contains("having not float_is_zero"),
            "should not filter out zero balance batches"
        );
    }

    #[test]
    fn running_stmt_uses_float_sum_for_updates() {
        let batch = upsert_vault_balances_batch(&RaindexIdentifier::new(5, Address::ZERO), 0, 1);
        let sql = batch.statements()[1].sql().to_lowercase();
        assert!(
            sql.contains("insert or replace into running_vault_balances"),
            "missing INSERT OR REPLACE clause"
        );
        assert!(
            sql.contains("coalesce(float_sum"),
            "missing FLOAT_SUM aggregation in query"
        );
    }
}
