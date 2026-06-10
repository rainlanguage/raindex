use crate::local_db::query::SqlStatement;

pub const CREATE_TABLES_SQL: &str = include_str!("query.sql");

/// Stable `PRAGMA application_id` stamped into the SQLite file header of every
/// raindex local-db. The value is the big-endian ASCII of "RIDX"
/// (0x52 'R', 0x49 'I', 0x44 'D', 0x58 'X'), giving a fixed, human-recognisable
/// magic that identifies the file as a raindex local-db rather than some other
/// SQLite file. It is a 32-bit header label and never changes across schema
/// versions. SQLite exposes `application_id`/`user_version` as signed 32-bit
/// integers, so the constant is typed `i32`.
pub const RAINDEX_APPLICATION_ID: i32 = 0x5249_4458;

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
    "derived_vault_deltas",
    "derived_trades",
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
    use raindex_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn pragmas_match_schema_version() {
        // The application_id PRAGMA literal must equal the documented magic.
        let expected_app_id = format!("PRAGMA application_id = 0x{:08X};", RAINDEX_APPLICATION_ID);
        assert!(
            CREATE_TABLES_SQL.contains(&expected_app_id),
            "create_tables SQL must stamp application_id with the raindex magic ({expected_app_id})"
        );

        // The user_version PRAGMA literal must stay in lockstep with the
        // Rust-side DB_SCHEMA_VERSION constant. A schema bump that forgets to
        // update the SQL header trips this assertion.
        let expected_user_version = format!("PRAGMA user_version = {DB_SCHEMA_VERSION};");
        assert!(
            CREATE_TABLES_SQL.contains(&expected_user_version),
            "create_tables SQL must stamp user_version = {DB_SCHEMA_VERSION} to match DB_SCHEMA_VERSION"
        );
    }

    #[test]
    fn raindex_application_id_is_ridx_ascii() {
        assert_eq!(
            RAINDEX_APPLICATION_ID.to_be_bytes(),
            *b"RIDX",
            "application id must be the big-endian ASCII of \"RIDX\""
        );
    }

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
        assert!(by_table["derived_vault_deltas"].contains(&"delta".to_string()));
        assert!(by_table["derived_trades"].contains(&"trade_id".to_string()));
        assert!(!by_table["order_ios"].contains(&"foreign".to_string()));
    }
}
