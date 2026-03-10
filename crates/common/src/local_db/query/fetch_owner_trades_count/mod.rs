use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::Address;
use std::convert::TryFrom;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub use super::fetch_order_trades_count::LocalDbTradeCountRow;

const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND t.chain_id IN ({list})";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE: &str = "/*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY: &str = "AND t.orderbook_address IN ({list})";

const CLEAR_ALICE_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_ALICE_CHAIN_IDS_CLAUSE*/";
const CLEAR_ALICE_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_ALICE_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_ALICE_ORDERBOOKS_CLAUSE*/";
const CLEAR_ALICE_ORDERBOOKS_CLAUSE_BODY: &str = "AND c.orderbook_address IN ({list})";

const CLEAR_BOB_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_BOB_CHAIN_IDS_CLAUSE*/";
const CLEAR_BOB_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_BOB_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_BOB_ORDERBOOKS_CLAUSE*/";
const CLEAR_BOB_ORDERBOOKS_CLAUSE_BODY: &str = "AND c.orderbook_address IN ({list})";

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND block_timestamp >= {param}\n";
const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND block_timestamp <= {param}\n";

#[derive(Debug, Clone)]
pub struct FetchOwnerTradesCountArgs {
    pub owner: Address,
    pub chain_ids: Vec<u32>,
    pub orderbook_addresses: Vec<Address>,
    pub start_timestamp: Option<u64>,
    pub end_timestamp: Option<u64>,
}

pub fn build_fetch_owner_trades_count_stmt(
    args: &FetchOwnerTradesCountArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    stmt.push(SqlValue::from(args.owner));

    let mut chain_ids = args.chain_ids.clone();
    chain_ids.sort_unstable();
    chain_ids.dedup();

    let mut orderbooks = args.orderbook_addresses.clone();
    orderbooks.sort();
    orderbooks.dedup();

    let chain_ids_iter = || chain_ids.iter().cloned().map(SqlValue::from);
    let orderbooks_iter = || orderbooks.iter().cloned().map(SqlValue::from);

    stmt.bind_list_clause(
        TAKE_ORDERS_CHAIN_IDS_CLAUSE,
        TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        TAKE_ORDERS_ORDERBOOKS_CLAUSE,
        TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_ALICE_CHAIN_IDS_CLAUSE,
        CLEAR_ALICE_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_ALICE_ORDERBOOKS_CLAUSE,
        CLEAR_ALICE_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_BOB_CHAIN_IDS_CLAUSE,
        CLEAR_BOB_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_BOB_ORDERBOOKS_CLAUSE,
        CLEAR_BOB_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;

    if let (Some(start), Some(end)) = (args.start_timestamp, args.end_timestamp) {
        if start > end {
            return Err(SqlBuildError::new("start_timestamp > end_timestamp"));
        }
    }

    let start_param = if let Some(v) = args.start_timestamp {
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

    let end_param = if let Some(v) = args.end_timestamp {
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
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn builds_with_chain_ids() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_count_stmt(&FetchOwnerTradesCountArgs {
            owner,
            chain_ids: vec![137, 1],
            orderbook_addresses: vec![],
            start_timestamp: None,
            end_timestamp: None,
        })
        .unwrap();
        assert!(stmt.sql.contains("t.chain_id IN"));
        assert!(stmt.sql.contains("c.chain_id IN"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_CHAIN_IDS_CLAUSE));
    }

    #[test]
    fn builds_with_time_filters() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_count_stmt(&FetchOwnerTradesCountArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            start_timestamp: Some(1000),
            end_timestamp: Some(2000),
        })
        .unwrap();
        assert!(stmt.sql.contains("block_timestamp >="));
        assert!(stmt.sql.contains("block_timestamp <="));
    }

    #[test]
    fn builds_without_filters() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_count_stmt(&FetchOwnerTradesCountArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            start_timestamp: None,
            end_timestamp: None,
        })
        .unwrap();
        assert!(!stmt.sql.contains("block_timestamp >="));
        assert!(!stmt.sql.contains("block_timestamp <="));
        assert_eq!(stmt.params.len(), 1);
    }

    #[test]
    fn extract_trade_count_works() {
        let rows = vec![LocalDbTradeCountRow { trade_count: 42 }];
        assert_eq!(extract_trade_count(&rows), 42);
        let empty: Vec<LocalDbTradeCountRow> = vec![];
        assert_eq!(extract_trade_count(&empty), 0);
    }
}
