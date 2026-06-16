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
/// Match-NONE predicate emitted for an empty `These` filter. SQLite rejects the
/// degenerate `IN ()` form, so we splice a constant-false predicate that the
/// query optimizer prunes to zero rows. This makes "filter to exactly these
/// (none) hashes" return no rows — the deliberate opposite of `All`.
const ORDER_HASH_MATCH_NONE_BODY: &str = "AND 1=0";

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\n  AND tws.block_timestamp >= {param}\n";

const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\n  AND tws.block_timestamp <= {param}\n";

/// Explicit selection of which orders' trades a fetch covers, so the builder
/// never has to read "all" out of an empty slice.
///
/// - [`OrderHashFilter::All`] emits no order-hash predicate at all, so every
///   order's trades for the chain/raindex (within the optional time window) are
///   returned.
/// - [`OrderHashFilter::These`] filters to exactly the given hashes via
///   `WHERE order_hash IN (...)`. An empty slice means *none*: it emits a
///   match-NONE predicate and returns zero rows. It is the deliberate opposite
///   of `All`, not a synonym for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderHashFilter<'a> {
    /// No order-hash predicate: trades for every order are returned.
    All,
    /// Trades for exactly these order hashes. Empty = none (zero rows), never
    /// all.
    These(&'a [B256]),
}

/// Builds the SQL statement for retrieving order trades within the specified
/// window. The `filter` explicitly selects which orders are covered, so trades
/// for one or many orders are fetched in a single query (eliminating the N+1
/// query pattern and per-query connection overhead). The single-order path
/// passes `These(&[hash])`.
///
/// The order-hash predicate rendered depends on `filter`:
/// - [`OrderHashFilter::All`] => no order-hash predicate (every order).
/// - [`OrderHashFilter::These`] non-empty => `AND order_hash IN (...)`.
/// - [`OrderHashFilter::These`] empty => `AND 1=0` (match nothing): an empty
///   `These` is *none*, never all. SQLite rejects `IN ()`, so the constant-false
///   predicate stands in for it.
pub fn build_fetch_order_trades_batch_stmt(
    raindex_id: &RaindexIdentifier,
    filter: OrderHashFilter<'_>,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(raindex_id.chain_id));
    stmt.push(SqlValue::from(raindex_id.raindex_address));
    match filter {
        OrderHashFilter::All => {
            // No order-hash predicate: drop the marker, keep all orders.
            stmt.replace(ORDER_HASH_CLAUSE, "")?;
        }
        OrderHashFilter::These([]) => {
            // Empty `These` means none. `IN ()` is invalid in SQLite, so splice
            // a constant-false predicate that yields zero rows.
            stmt.replace(ORDER_HASH_CLAUSE, ORDER_HASH_MATCH_NONE_BODY)?;
        }
        OrderHashFilter::These(hashes) => {
            stmt.bind_list_clause(
                ORDER_HASH_CLAUSE,
                ORDER_HASH_LIST_BODY,
                hashes.iter().copied().map(SqlValue::from),
            )?;
        }
    }

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
            OrderHashFilter::These(&[hash_a, hash_b]),
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
            OrderHashFilter::These(&[hash]),
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
            OrderHashFilter::These(&[hash]),
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
    fn all_emits_no_order_hash_predicate() {
        // `All` is the only variant that drops the order-hash predicate: no IN
        // list and no match-NONE constant. Every order is returned.
        let stmt = build_fetch_order_trades_batch_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            OrderHashFilter::All,
            None,
            None,
        )
        .unwrap();

        // The marker is consumed and *no* order-hash predicate is rendered (the
        // SELECT list still projects tws.order_hash, so only the predicate forms
        // are asserted absent).
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
        assert!(!stmt.sql.contains("tws.order_hash IN ("));
        assert!(!stmt.sql.contains("tws.order_hash = "));
        // Crucially, `All` does NOT emit the match-NONE predicate that empty
        // `These` does — the two are opposites.
        assert!(!stmt.sql.contains("1=0"));
        // Only the two fixed params (chain id, raindex) remain.
        assert_eq!(stmt.params.len(), 2);
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
    }

    #[test]
    fn these_empty_emits_match_none_predicate_not_dropped_clause() {
        // An empty `These` means *none*: it emits the constant-false predicate
        // `AND 1=0` so zero rows match. This is the deliberate opposite of `All`
        // (which would drop the clause and return every order). It must NOT
        // degenerate to "all".
        let stmt = build_fetch_order_trades_batch_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            OrderHashFilter::These(&[]),
            None,
            None,
        )
        .unwrap();

        // The match-NONE predicate is present...
        assert!(stmt.sql.contains("1=0"));
        // ...and the marker is consumed (not left unsubstituted).
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
        // No IN list / equality predicate and no bound hashes: empty These binds
        // zero placeholders.
        assert!(!stmt.sql.contains("tws.order_hash IN ("));
        assert!(!stmt.sql.contains("tws.order_hash = "));
        // Only the two fixed params (chain id, raindex) — the constant-false
        // predicate binds nothing.
        assert_eq!(stmt.params.len(), 2);
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
    }
}
