use crate::local_db::query::SqlStatement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableColumnResponse {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableSchemaColumnResponse {
    pub table_name: String,
    pub name: String,
}

pub fn fetch_table_columns_stmt(table: &str) -> SqlStatement {
    SqlStatement::new(format!("PRAGMA table_info({});", quote_identifier(table)))
}

/// Fetches the column names for every requested table in one database call.
/// Table names are bound as parameters so callers do not need to interpolate
/// identifiers into the query.
pub fn fetch_table_schema_columns_stmt<'a>(
    tables: impl IntoIterator<Item = &'a str>,
) -> SqlStatement {
    let mut statement = SqlStatement::new("");
    let selects = tables
        .into_iter()
        .map(|table| {
            let placeholder = statement.push(table);
            format!(
                "SELECT {placeholder} AS table_name, name FROM pragma_table_info({placeholder})"
            )
        })
        .collect::<Vec<_>>();

    statement.sql = if selects.is_empty() {
        "SELECT CAST(NULL AS TEXT) AS table_name, CAST(NULL AS TEXT) AS name WHERE 0".to_string()
    } else {
        selects.join(" UNION ALL ")
    };
    statement
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

    #[test]
    fn fetch_table_schema_columns_stmt_binds_all_tables() {
        let stmt = fetch_table_schema_columns_stmt(["orders", "trades"]);
        assert_eq!(
            stmt.sql(),
            "SELECT ?1 AS table_name, name FROM pragma_table_info(?1) UNION ALL SELECT ?2 AS table_name, name FROM pragma_table_info(?2)"
        );
        assert_eq!(stmt.params().len(), 2);
    }

    #[test]
    fn fetch_table_schema_columns_stmt_handles_empty_input() {
        let stmt = fetch_table_schema_columns_stmt([]);
        assert!(stmt.sql().contains("WHERE 0"));
        assert!(stmt.params().is_empty());
    }
}
