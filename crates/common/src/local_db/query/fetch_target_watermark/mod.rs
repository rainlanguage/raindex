use crate::local_db::{
    query::{SqlStatement, SqlValue},
    RaindexIdentifier,
};
use alloy::primitives::{Address, Bytes};
use serde::{Deserialize, Serialize};

pub const FETCH_TARGET_WATERMARK_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetWatermarkRow {
    pub chain_id: u32,
    pub raindex_address: Address,
    pub last_block: u64,
    pub last_hash: Bytes,
    pub updated_at: u64,
}

/// Raw watermark identity used while validating an externally installed
/// snapshot. Keeping the address as text lets validation enforce the same
/// exact, lowercase representation used by the engine's SQL lookup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreinstalledTargetWatermarkRow {
    pub chain_id: u32,
    pub raindex_address: String,
    pub last_block: u64,
    pub last_hash: Bytes,
    pub updated_at: u64,
}

pub fn fetch_target_watermark_stmt(raindex_id: &RaindexIdentifier) -> SqlStatement {
    SqlStatement::new_with_params(
        FETCH_TARGET_WATERMARK_SQL,
        [
            SqlValue::from(raindex_id.chain_id),
            SqlValue::from(raindex_id.raindex_address),
        ],
    )
}

/// Fetches all configured target watermarks in one database call so browser
/// bootstrap does not add a WASM boundary crossing per contract. Address
/// matching is case-insensitive only for candidate selection; callers still
/// validate the stored text is canonical before accepting a snapshot.
pub fn fetch_configured_target_watermarks_stmt(configured: &[RaindexIdentifier]) -> SqlStatement {
    let mut params = Vec::with_capacity(configured.len() * 2);
    let clauses = configured
        .iter()
        .enumerate()
        .map(|(index, target)| {
            let chain_placeholder = index * 2 + 1;
            let address_placeholder = chain_placeholder + 1;
            params.push(SqlValue::from(target.chain_id));
            params.push(SqlValue::from(target.raindex_address));
            format!(
                "(chain_id = ?{chain_placeholder} AND lower(raindex_address) = ?{address_placeholder})"
            )
        })
        .collect::<Vec<_>>();
    let predicate = if clauses.is_empty() {
        "0".to_string()
    } else {
        clauses.join(" OR ")
    };

    SqlStatement::new_with_params(
        format!(
            "SELECT chain_id, raindex_address, last_block, last_hash, updated_at \
             FROM target_watermarks WHERE {predicate};"
        ),
        params,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_stmt_binds_params() {
        let stmt = fetch_target_watermark_stmt(&RaindexIdentifier::new(10, Address::ZERO));
        assert!(stmt.sql().to_lowercase().contains("from target_watermarks"));
        assert_eq!(stmt.params().len(), 2);
    }

    #[test]
    fn fetch_stmt_sql_matches_template_and_where_clause() {
        let stmt = fetch_target_watermark_stmt(&RaindexIdentifier::new(1, Address::ZERO));
        // Exact template equality (comes from include_str!)
        assert_eq!(stmt.sql(), FETCH_TARGET_WATERMARK_SQL);
        // Defensive: check placeholders and where clause shape
        let lower = stmt.sql().to_lowercase();
        assert!(lower.contains("where chain_id = ?1 and raindex_address = ?2"));
    }

    #[test]
    fn fetch_stmt_param_order_and_values() {
        let chain_id = 111u32;
        let addr = Address::repeat_byte(0xab);
        let stmt = fetch_target_watermark_stmt(&RaindexIdentifier::new(chain_id, addr));

        let params = stmt.params();
        assert_eq!(params.len(), 2);
        assert_eq!(
            params[0],
            SqlValue::U64(chain_id as u64),
            "first param must be chain_id as U64"
        );
        assert_eq!(
            params[1],
            SqlValue::Text("0xabababababababababababababababababababab".to_string()),
            "second param must be raindex_address as hex string"
        );
        // Ensure address string formatting is 0x-prefixed lowercase hex
        let SqlValue::Text(s) = &params[1] else {
            panic!("expected text param")
        };
        assert!(s.starts_with("0x"));
        assert_eq!(s.len(), 42); // 0x + 40 hex chars
        assert_eq!(s, "0xabababababababababababababababababababab");
    }

    #[test]
    fn fetch_configured_stmt_filters_all_targets_in_one_query() {
        let targets = vec![
            RaindexIdentifier::new(1, Address::repeat_byte(0x11)),
            RaindexIdentifier::new(137, Address::repeat_byte(0x22)),
        ];
        let stmt = fetch_configured_target_watermarks_stmt(&targets);
        assert!(stmt.sql().contains("FROM target_watermarks"));
        assert!(stmt.sql().contains("last_hash"));
        assert!(stmt.sql().contains("updated_at"));
        assert!(stmt.sql().contains("lower(raindex_address)"));
        assert_eq!(stmt.sql().matches(" OR ").count(), 1);
        assert_eq!(stmt.params().len(), 4);
    }

    #[test]
    fn fetch_configured_stmt_empty_selection_matches_nothing() {
        let stmt = fetch_configured_target_watermarks_stmt(&[]);
        assert!(stmt.sql().contains("WHERE 0"));
        assert!(stmt.params().is_empty());
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn fetch_configured_stmt_excludes_unrelated_null_rows_before_deserialization() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE target_watermarks (\
                    chain_id INTEGER NOT NULL, \
                    raindex_address TEXT NOT NULL, \
                    last_block INTEGER NOT NULL, \
                    last_hash TEXT, \
                    updated_at INTEGER\
                 ); \
                 INSERT INTO target_watermarks VALUES (\
                    1, '0x1111111111111111111111111111111111111111', 100, '0xbeef', 1\
                 ); \
                 INSERT INTO target_watermarks VALUES (\
                    137, '0x2222222222222222222222222222222222222222', 100, NULL, NULL\
                 );",
            )
            .unwrap();
        let statement = fetch_configured_target_watermarks_stmt(&[RaindexIdentifier::new(
            1,
            Address::repeat_byte(0x11),
        )]);
        let values = statement.params().iter().map(|value| match value {
            SqlValue::Text(value) => rusqlite::types::Value::Text(value.clone()),
            SqlValue::U64(value) => rusqlite::types::Value::Integer(*value as i64),
            SqlValue::I64(value) => rusqlite::types::Value::Integer(*value),
            SqlValue::Null => rusqlite::types::Value::Null,
        });
        let mut prepared = connection.prepare(statement.sql()).unwrap();
        let rows = prepared
            .query_map(rusqlite::params_from_iter(values), |row| {
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, u64>(4)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 1);
        assert_eq!(rows[0].3, "0xbeef");
    }
}
