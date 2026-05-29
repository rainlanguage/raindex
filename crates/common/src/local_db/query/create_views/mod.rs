use crate::local_db::query::{SqlStatement, SqlStatementBatch};

const VAULT_DELTAS_VIEW_SQL: &str = include_str!("./vault_deltas.sql");

pub fn create_views_batch() -> SqlStatementBatch {
    SqlStatementBatch::with_statements(vec![vault_deltas_view_stmt()]).ensure_transaction()
}

pub fn vault_deltas_view_stmt() -> SqlStatement {
    SqlStatement::new(VAULT_DELTAS_VIEW_SQL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_wraps_transaction() {
        let batch = create_views_batch();
        assert!(batch.is_transaction());
        assert_eq!(batch.len(), 3); // begin + drop/create view + commit

        let statements = batch.statements();
        assert_eq!(statements.first().unwrap().sql(), "BEGIN TRANSACTION");
        assert_eq!(statements.last().unwrap().sql(), "COMMIT");
        assert_eq!(statements[1].sql(), VAULT_DELTAS_VIEW_SQL);
    }

    #[test]
    fn single_stmt_matches_constant() {
        let stmt = vault_deltas_view_stmt();
        assert_eq!(stmt.sql(), VAULT_DELTAS_VIEW_SQL);
        assert!(stmt.sql().starts_with("DROP VIEW IF EXISTS vault_deltas;"));
        assert!(stmt.sql().contains("CREATE VIEW vault_deltas AS"));
        assert!(stmt.params().is_empty());
    }
}
