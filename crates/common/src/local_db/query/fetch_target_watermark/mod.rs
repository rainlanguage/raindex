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

pub fn fetch_target_watermark_stmt(raindex_id: &RaindexIdentifier) -> SqlStatement {
    SqlStatement::new_with_params(
        FETCH_TARGET_WATERMARK_SQL,
        [
            SqlValue::from(raindex_id.chain_id),
            SqlValue::from(raindex_id.raindex_address),
        ],
    )
}

/// Fetches every target watermark in one database call so browser bootstrap
/// does not add a WASM boundary crossing for each configured contract.
pub fn fetch_all_target_watermarks_stmt() -> SqlStatement {
    SqlStatement::new(
        "SELECT chain_id, raindex_address, last_block, last_hash, updated_at \
         FROM target_watermarks;",
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
    fn fetch_all_stmt_is_unfiltered_and_param_free() {
        let stmt = fetch_all_target_watermarks_stmt();
        assert!(stmt.sql().contains("FROM target_watermarks"));
        assert!(!stmt.sql().contains("WHERE"));
        assert!(stmt.params().is_empty());
    }
}
