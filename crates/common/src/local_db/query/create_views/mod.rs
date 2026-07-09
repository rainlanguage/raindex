use crate::local_db::query::{SqlScript, SqlStatement, SqlStatementBatch};

const VAULT_DELTAS_VIEW_SQL: &str = include_str!("./vault_deltas.sql");

pub fn create_views_batch() -> SqlStatementBatch {
    SqlScript::new(VAULT_DELTAS_VIEW_SQL)
        .statement_batch()
        .ensure_transaction()
}

pub fn drop_vault_deltas_view_stmt() -> SqlStatement {
    SqlStatement::new(view_statement(0))
}

pub fn create_vault_deltas_view_stmt() -> SqlStatement {
    SqlStatement::new(view_statement(1))
}

fn view_statement(index: usize) -> &'static str {
    SqlScript::new(VAULT_DELTAS_VIEW_SQL)
        .statements()
        .get(index)
        .copied()
        .expect("vault_deltas view SQL contains drop and create statements")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_wraps_transaction() {
        let batch = create_views_batch();
        assert!(batch.is_transaction());
        assert_eq!(batch.len(), 4); // begin + drop view + create view + commit

        let statements = batch.statements();
        assert_eq!(statements.first().unwrap().sql(), "BEGIN TRANSACTION");
        assert_eq!(statements.last().unwrap().sql(), "COMMIT");
        assert_eq!(statements[1].sql(), drop_vault_deltas_view_stmt().sql());
        assert_eq!(statements[2].sql(), create_vault_deltas_view_stmt().sql());
    }

    #[test]
    fn statements_come_from_combined_script() {
        let statements = SqlScript::new(VAULT_DELTAS_VIEW_SQL).statements();
        assert_eq!(statements.len(), 2);
        assert!(statements[0].starts_with("DROP VIEW IF EXISTS vault_deltas;"));
        assert!(statements[1]
            .trim_start()
            .starts_with("CREATE VIEW vault_deltas AS"));
    }
}
