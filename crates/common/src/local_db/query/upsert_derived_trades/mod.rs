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
}
