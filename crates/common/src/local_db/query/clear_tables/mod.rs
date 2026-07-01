use crate::local_db::query::{SqlScript, SqlStatement, SqlStatementBatch};

pub const CLEAR_TABLES_SQL: &str = include_str!("query.sql");

pub fn clear_tables_batch() -> SqlStatementBatch {
    let mut statements = SqlScript::new(CLEAR_TABLES_SQL).statements();
    statements.retain(|sql| !sql.trim().eq_ignore_ascii_case("VACUUM;"));

    SqlStatementBatch::with_statements(statements.into_iter().map(SqlStatement::new).collect())
}

pub fn vacuum_stmt() -> SqlStatement {
    SqlStatement::new("VACUUM;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use std::collections::HashSet;

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
    fn script_drops_tables_and_vacuums() {
        let lower = CLEAR_TABLES_SQL.to_lowercase();
        assert!(lower.contains("begin transaction"));
        assert!(lower.contains("drop view if exists vault_deltas"));
        assert!(lower.contains("drop table if exists"));
        assert!(lower.contains("vacuum"));
    }

    #[test]
    fn batch_excludes_vacuum_from_transaction() {
        let batch = clear_tables_batch();
        assert!(batch.is_transaction());
        assert!(!batch
            .statements()
            .iter()
            .any(|stmt| stmt.sql().trim().eq_ignore_ascii_case("VACUUM;")));
        assert_eq!(vacuum_stmt().sql().trim(), "VACUUM;");
    }

    #[test]
    fn drops_all_required_tables() {
        let sql = CLEAR_TABLES_SQL.to_lowercase();

        // Parse all table names that appear in DROP TABLE statements
        let mut dropped: HashSet<String> = HashSet::new();
        let mut offset = 0usize;
        let needle = "drop table";

        while let Some(rel_idx) = sql[offset..].find(needle) {
            let start = offset + rel_idx;

            // Find the terminating semicolon for this DROP statement
            let end_rel = sql[start..]
                .find(';')
                .expect("DROP TABLE without terminating ';'");
            let end = start + end_rel;

            // Slice the statement body and split into tokens
            let stmt_body = &sql[start + needle.len()..end];
            // Expect tokens like: [IF, NOT, EXISTS, <table_name>] or [IF, EXISTS, <table_name>] or [<table_name>]
            let tokens: Vec<&str> = stmt_body.split_whitespace().collect();
            if let Some(last) = tokens.last() {
                let table = normalize_ident(last);
                if !table.is_empty() {
                    dropped.insert(table);
                }
            }

            offset = end + 1;
        }

        let required: HashSet<String> =
            REQUIRED_TABLES.iter().map(|t| normalize_ident(t)).collect();

        let missing: HashSet<_> = required.difference(&dropped).cloned().collect();
        let extra: HashSet<_> = dropped.difference(&required).cloned().collect();

        assert!(
            missing.is_empty() && extra.is_empty(),
            "Mismatch in tables to drop. Missing: {:?}; Extra: {:?}",
            missing,
            extra
        );
    }
}
