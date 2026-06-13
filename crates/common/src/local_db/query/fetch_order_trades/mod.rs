use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    RaindexIdentifier,
};
use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbOrderTrade {
    pub chain_id: u32,
    pub trade_kind: String,
    pub raindex: Address,
    pub order_hash: B256,
    pub order_owner: Address,
    pub order_nonce: String,
    pub transaction_hash: B256,
    pub log_index: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub transaction_sender: Address,
    pub input_vault_id: U256,
    pub input_token: Address,
    pub input_token_name: Option<String>,
    pub input_token_symbol: Option<String>,
    pub input_token_decimals: Option<u8>,
    pub input_delta: String,
    pub input_running_balance: Option<String>,
    pub output_vault_id: U256,
    pub output_token: Address,
    pub output_token_name: Option<String>,
    pub output_token_symbol: Option<String>,
    pub output_token_decimals: Option<u8>,
    pub output_delta: String,
    pub output_running_balance: Option<String>,
    pub trade_id: String,
}

const ORDER_HASH_CLAUSE: &str = "/*ORDER_HASH_CLAUSE*/";
const ORDER_HASH_LIST_BODY: &str = "AND tws.order_hash IN ({list})";

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\n  AND tws.block_timestamp >= {param}\n";

const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\n  AND tws.block_timestamp <= {param}\n";

/// Builds the SQL statement for retrieving order trades within the specified
/// window. Accepts a slice of order hashes and emits a single query with a
/// `WHERE order_hash IN (...)` clause, so trades for one or many orders are
/// fetched in a single query (eliminating the N+1 query pattern and per-query
/// connection overhead). The single-order path passes a one-element slice.
///
/// When `order_hashes` is empty the order-hash clause is removed entirely, so
/// the query degenerates to "all trades for this chain/raindex" within the
/// optional time window. Callers that want an empty result for an empty input
/// should short-circuit before invoking this builder.
pub fn build_fetch_order_trades_batch_stmt(
    raindex_id: &RaindexIdentifier,
    order_hashes: &[B256],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(raindex_id.chain_id));
    stmt.push(SqlValue::from(raindex_id.raindex_address));
    stmt.bind_list_clause(
        ORDER_HASH_CLAUSE,
        ORDER_HASH_LIST_BODY,
        order_hashes.iter().copied().map(SqlValue::from),
    )?;

    // Optional time filters
    let start_param = if let Some(v) = start_timestamp {
        let i = i64::try_from(v).map_err(|e| {
            SqlBuildError::new(format!(
                "start_timestamp out of range for i64: {} ({})",
                v, e
            ))
        })?;
        Some(SqlValue::I64(i))
    } else {
        None
    };
    stmt.bind_param_clause(START_TS_CLAUSE, START_TS_BODY, start_param)?;

    let end_param = if let Some(v) = end_timestamp {
        let i = i64::try_from(v).map_err(|e| {
            SqlBuildError::new(format!("end_timestamp out of range for i64: {} ({})", v, e))
        })?;
        Some(SqlValue::I64(i))
    } else {
        None
    };
    stmt.bind_param_clause(END_TS_CLAUSE, END_TS_BODY, end_param)?;

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex,
        primitives::{b256, Address},
    };

    #[test]
    fn batch_builds_in_clause_with_time_filters() {
        let hash_a = b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let hash_b = b256!("0x00000000000000000000000000000000000000000000000000000000deadface");
        let stmt = build_fetch_order_trades_batch_stmt(
            &RaindexIdentifier::new(137, Address::ZERO),
            &[hash_a, hash_b],
            Some(11),
            Some(22),
        )
        .unwrap();

        // Marker replaced and an IN list (not an equality) is rendered.
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
        assert!(stmt.sql.contains("tws.order_hash IN (?3, ?4)"));
        assert!(!stmt.sql.contains("tws.order_hash = "));

        // Time filters still bound after the order-hash list.
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert!(stmt.sql.contains("tws.block_timestamp >= ?5"));
        assert!(stmt.sql.contains("tws.block_timestamp <= ?6"));

        // Params: chain id, raindex, hash_a, hash_b, start, end
        assert_eq!(stmt.params.len(), 6);
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(stmt.params[2], SqlValue::Text(hex::encode_prefixed(hash_a)));
        assert_eq!(stmt.params[3], SqlValue::Text(hex::encode_prefixed(hash_b)));
        assert_eq!(stmt.params[4], SqlValue::I64(11));
        assert_eq!(stmt.params[5], SqlValue::I64(22));
    }

    #[test]
    fn batch_single_hash_renders_single_placeholder_in_clause() {
        let hash = b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let stmt = build_fetch_order_trades_batch_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            &[hash],
            None,
            None,
        )
        .unwrap();

        assert!(stmt.sql.contains("tws.order_hash IN (?3)"));
        // A one-element list never collapses to the old `= ?` equality form.
        assert!(!stmt.sql.contains("tws.order_hash = "));
        assert!(!stmt.sql.contains("tws.block_timestamp >="));
        assert!(!stmt.sql.contains("tws.block_timestamp <="));
        assert_eq!(stmt.params.len(), 3);
        // Fixed params: chain id (?1), raindex (?2), order hash (?3).
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(stmt.params[2], SqlValue::Text(hex::encode_prefixed(hash)));
    }

    #[test]
    fn batch_single_hash_with_time_filters_binds_window() {
        // The single-order path (a one-element slice) carries the same
        // time-window binding semantics the removed single-order builder had:
        // chain id (?1), raindex (?2), order hash (?3), start (?4), end (?5).
        let hash = b256!("0x00000000000000000000000000000000000000000000000000000000deadface");
        let stmt = build_fetch_order_trades_batch_stmt(
            &RaindexIdentifier::new(137, Address::ZERO),
            &[hash],
            Some(11),
            Some(22),
        )
        .unwrap();

        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert!(stmt.sql.contains("tws.order_hash IN (?3)"));
        assert!(!stmt.sql.contains("tws.order_hash = "));
        assert!(stmt.sql.contains("tws.block_timestamp >= ?4"));
        assert!(stmt.sql.contains("tws.block_timestamp <= ?5"));

        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(stmt.params[2], SqlValue::Text(hex::encode_prefixed(hash)));
        assert_eq!(stmt.params[3], SqlValue::I64(11));
        assert_eq!(stmt.params[4], SqlValue::I64(22));
    }

    #[test]
    fn batch_empty_hashes_drops_order_hash_clause() {
        let stmt = build_fetch_order_trades_batch_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            &[],
            None,
            None,
        )
        .unwrap();

        // With no hashes the order-hash WHERE predicate is removed entirely
        // (the SELECT list still projects tws.order_hash, so only the predicate
        // forms are asserted absent); only the two fixed params (chain id,
        // raindex) remain.
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
        assert!(!stmt.sql.contains("tws.order_hash IN ("));
        assert!(!stmt.sql.contains("tws.order_hash = "));
        assert_eq!(stmt.params.len(), 2);
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
    }
}
