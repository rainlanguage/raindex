use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    RaindexIdentifier,
};
use alloy::primitives::B256;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbTradeCountRow {
    #[serde(alias = "trade_count")]
    pub trade_count: u64,
}

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND block_timestamp >= {param}\n";
const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND block_timestamp <= {param}\n";

pub fn build_fetch_trade_count_stmt(
    raindex_id: &RaindexIdentifier,
    order_hash: B256,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(raindex_id.chain_id));
    stmt.push(SqlValue::from(raindex_id.raindex_address));
    stmt.push(SqlValue::from(order_hash));

    // Optional time filters
    if let (Some(start), Some(end)) = (start_timestamp, end_timestamp) {
        if start > end {
            return Err(SqlBuildError::new("start_timestamp > end_timestamp"));
        }
    }

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

pub fn extract_trade_count(rows: &[LocalDbTradeCountRow]) -> u64 {
    rows.first().map(|row| row.trade_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{b256, Address};

    use super::*;

    #[test]
    fn builds_with_time_filters() {
        let stmt = build_fetch_trade_count_stmt(
            &RaindexIdentifier::new(137, Address::ZERO),
            b256!("0x0000000000000000000000000000000000000000000000000000000000ABCDEF"),
            Some(1000),
            Some(2000),
        )
        .unwrap();
        // Time filter clauses present
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert!(stmt.sql.contains("block_timestamp >="));
        assert!(stmt.sql.contains("block_timestamp <="));
        // Params include order hash and two timestamps
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(
            stmt.params[2],
            SqlValue::Text(
                "0x0000000000000000000000000000000000000000000000000000000000abcdef".into()
            )
        );
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let stmt = build_fetch_trade_count_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef"),
            None,
            None,
        )
        .unwrap();
        assert!(!stmt.sql.contains("block_timestamp >="));
        assert!(!stmt.sql.contains("block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert_eq!(stmt.params.len(), 3);
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(
            stmt.params[2],
            SqlValue::Text(
                "0x00000000000000000000000000000000000000000000000000000000deadbeef".into()
            )
        );
    }

    #[test]
    fn extract_trade_count_behaviour() {
        let rows = vec![LocalDbTradeCountRow { trade_count: 7 }];
        assert_eq!(extract_trade_count(&rows), 7);
        let none: Vec<LocalDbTradeCountRow> = vec![];
        assert_eq!(extract_trade_count(&none), 0);
    }
}
