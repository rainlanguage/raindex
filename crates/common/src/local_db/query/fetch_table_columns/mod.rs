use crate::local_db::query::SqlStatement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableColumnResponse {
    pub name: String,
}

pub fn fetch_table_columns_stmt(table: &str) -> SqlStatement {
    SqlStatement::new(format!("PRAGMA table_info({});", quote_identifier(table)))
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_table_columns_stmt_quotes_identifier() {
        let stmt = fetch_table_columns_stmt("target_watermarks");
        assert_eq!(stmt.sql(), "PRAGMA table_info(\"target_watermarks\");");
        assert!(stmt.params().is_empty());
    }

    #[test]
    fn fetch_table_columns_stmt_escapes_quotes() {
        let stmt = fetch_table_columns_stmt("bad\"name");
        assert_eq!(stmt.sql(), "PRAGMA table_info(\"bad\"\"name\");");
    }
}
