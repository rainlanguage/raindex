use crate::local_db::query::create_tables::REQUIRED_TABLES;
use crate::local_db::query::{SqlStatement, SqlStatementBatch, SqlValue};
use crate::local_db::RaindexIdentifier;

pub const CLEAR_RAINDEX_DATA_SQL: &str = include_str!("query.sql");

pub fn clear_raindex_data_batch(raindex_id: &RaindexIdentifier) -> SqlStatementBatch {
    let statements: Vec<SqlStatement> = REQUIRED_TABLES
        .iter()
        .copied()
        .filter(|table| *table != "db_metadata")
        .map(|table| {
            SqlStatement::new_with_params(
                format!("DELETE FROM {table}\nWHERE chain_id = ?1 AND raindex_address = ?2"),
                [
                    SqlValue::from(raindex_id.chain_id),
                    SqlValue::from(raindex_id.raindex_address),
                ],
            )
        })
        .collect();

    SqlStatementBatch::from(statements).ensure_transaction()
}

#[cfg(test)]
mod tests {
    use alloy::primitives::Address;

    use super::*;
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use std::collections::HashSet;

    const TABLES_EXCLUDED_FROM_CLEAR: &[&str] = &["db_metadata"];

    fn normalize_ident(s: &str) -> String {
        let s = s.trim();
        let s = s.trim_matches('`').trim_matches('"');
        let s = if s.starts_with('[') && s.ends_with(']') && s.len() >= 2 {
            &s[1..s.len() - 1]
        } else {
            s
        };
        s.to_lowercase()
    }

    #[test]
    fn batch_wraps_transaction_and_matches_tables() {
        let chain_id = 999u32;
        let addr = Address::from([0xAA; 20]);
        let batch = clear_raindex_data_batch(&RaindexIdentifier::new(chain_id, addr));

        let expected_tables: Vec<&str> = REQUIRED_TABLES
            .iter()
            .copied()
            .filter(|table| *table != "db_metadata")
            .collect();

        assert!(batch.is_transaction());
        assert_eq!(batch.len(), expected_tables.len() + 2); // BEGIN + deletes + COMMIT

        let statements = batch.statements();
        assert_eq!(statements.first().unwrap().sql(), "BEGIN TRANSACTION");
        assert_eq!(statements.last().unwrap().sql(), "COMMIT");

        for (idx, table) in expected_tables.iter().enumerate() {
            let stmt = &statements[idx + 1];
            let expected_sql =
                format!("DELETE FROM {table}\nWHERE chain_id = ?1 AND raindex_address = ?2");
            assert_eq!(stmt.sql(), expected_sql);
            assert_eq!(
                stmt.params(),
                &[
                    SqlValue::U64(chain_id as u64),
                    SqlValue::Text("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string())
                ]
            );
        }
    }

    #[test]
    fn sql_covers_expected_tables() {
        let sql = CLEAR_RAINDEX_DATA_SQL.to_ascii_lowercase();
        let expected_tables: HashSet<String> = REQUIRED_TABLES
            .iter()
            .copied()
            .filter(|table| !TABLES_EXCLUDED_FROM_CLEAR.contains(table))
            .map(normalize_ident)
            .collect();

        let mut deleted_tables = HashSet::new();
        let mut tables_missing_scope = Vec::new();
        let needle = "delete from ";
        let mut offset = 0usize;

        while let Some(idx) = sql[offset..].find(needle) {
            let start = offset + idx + needle.len();
            let rest = &sql[start..];
            let end = rest.find('\n').unwrap_or(rest.len());
            let table_name = normalize_ident(rest[..end].trim());
            if !table_name.is_empty() {
                let stmt_start = offset + idx;
                let stmt_rest = &sql[stmt_start..];
                let stmt_end = stmt_rest.find(';').unwrap_or_else(|| {
                    panic!("DELETE statement for table {table_name} missing ';'")
                });
                let stmt = &stmt_rest[..stmt_end];
                let has_chain_guard = stmt.contains("where chain_id = ?1");
                let has_address_guard = stmt.contains("raindex_address = ?2");
                if !(has_chain_guard && has_address_guard) {
                    tables_missing_scope.push(table_name.clone());
                }

                deleted_tables.insert(table_name);
            }
            offset = start + end;
        }

        for table in &expected_tables {
            assert!(
                deleted_tables.contains(table),
                "missing delete for table {table}"
            );
        }

        let unexpected_tables: HashSet<_> = deleted_tables
            .difference(&expected_tables)
            .cloned()
            .collect();
        assert!(
            unexpected_tables.is_empty(),
            "unexpected delete for tables {:?}",
            unexpected_tables
        );

        assert!(
            tables_missing_scope.is_empty(),
            "delete statements missing scope guards for tables {:?}",
            tables_missing_scope
        );

        assert!(
            sql.contains("begin transaction;"),
            "transaction not started"
        );
        assert!(sql.contains("commit;"), "transaction not committed");
    }
}
