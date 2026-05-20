use crate::local_db::query::SqlStatement;

pub const CREATE_TABLES_SQL: &str = include_str!("query.sql");

pub const REQUIRED_TABLES: &[&str] = &[
    "db_metadata",
    "target_watermarks",
    "sync_status",
    "raw_events",
    "deposits",
    "withdrawals",
    "order_events",
    "order_ios",
    "take_orders",
    "take_order_contexts",
    "context_values",
    "clear_v3_events",
    "after_clear_v2_events",
    "meta_events",
    "erc20_tokens",
    "interpreter_store_sets",
    "vault_balance_changes",
    "running_vault_balances",
];

pub fn create_tables_sql() -> &'static str {
    CREATE_TABLES_SQL
}

pub fn create_tables_stmt() -> SqlStatement {
    SqlStatement::new(CREATE_TABLES_SQL)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredTableSchema {
    pub name: String,
    pub columns: Vec<String>,
}

pub fn required_table_schema() -> Vec<RequiredTableSchema> {
    parse_required_table_schema(CREATE_TABLES_SQL)
}

fn parse_required_table_schema(sql: &str) -> Vec<RequiredTableSchema> {
    let sql_lower = sql.to_lowercase();
    let mut parsed = Vec::new();
    let mut offset = 0usize;
    let needle = "create table";

    while let Some(rel_idx) = sql_lower[offset..].find(needle) {
        let start = offset + rel_idx;
        let Some(open_paren_rel) = sql_lower[start..].find('(') else {
            break;
        };
        let open_paren = start + open_paren_rel;
        let Some(close_paren) = find_matching_paren(sql, open_paren) else {
            break;
        };

        if let Some(name) = parse_create_table_name(&sql[start + needle.len()..open_paren]) {
            parsed.push(RequiredTableSchema {
                name,
                columns: parse_column_names(&sql[open_paren + 1..close_paren]),
            });
        }

        offset = close_paren + 1;
    }

    parsed
}

fn find_matching_paren(sql: &str, open_paren: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in sql[open_paren..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open_paren + idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_create_table_name(raw: &str) -> Option<String> {
    raw.split_whitespace().last().map(normalize_ident)
}

fn parse_column_names(raw_columns: &str) -> Vec<String> {
    split_top_level_columns(raw_columns)
        .into_iter()
        .filter_map(|definition| {
            let first = definition.split_whitespace().next()?;
            let column = normalize_ident(first);
            match column.as_str() {
                "primary" | "foreign" | "unique" | "check" | "constraint" => None,
                _ => Some(column),
            }
        })
        .collect()
}

fn split_top_level_columns(raw_columns: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for ch in raw_columns.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                let item = current.trim();
                if !item.is_empty() {
                    items.push(item.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let item = current.trim();
    if !item.is_empty() {
        items.push(item.to_string());
    }

    items
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn required_tables_match_sql_create_statements() {
        let parsed_tables: HashSet<String> = required_table_schema()
            .into_iter()
            .map(|table| table.name)
            .collect();

        let required_tables: HashSet<String> =
            REQUIRED_TABLES.iter().map(|t| normalize_ident(t)).collect();

        // Compute diffs
        let missing: HashSet<_> = required_tables
            .difference(&parsed_tables)
            .cloned()
            .collect();
        let extra: HashSet<_> = parsed_tables
            .difference(&required_tables)
            .cloned()
            .collect();

        assert!(
            missing.is_empty() && extra.is_empty(),
            "Table set mismatch. Missing in SQL: {:?}; Extra in SQL: {:?}",
            missing,
            extra
        );
    }

    #[test]
    fn required_table_schema_parses_tables_and_columns() {
        let schema = required_table_schema();
        let by_table: HashMap<_, _> = schema
            .iter()
            .map(|table| (table.name.as_str(), table.columns.as_slice()))
            .collect();

        assert_eq!(schema.len(), REQUIRED_TABLES.len());
        assert!(by_table["db_metadata"].contains(&"db_schema_version".to_string()));
        assert!(by_table["target_watermarks"].contains(&"raindex_address".to_string()));
        assert!(by_table["order_events"].contains(&"order_hash".to_string()));
        assert!(by_table["running_vault_balances"].contains(&"balance".to_string()));
        assert!(!by_table["order_ios"].contains(&"foreign".to_string()));
    }
}
