use crate::local_db::query::fetch_owner_trades_common::{
    bind_common_owner_trade_filters, TAKE_ORDERS_CHAIN_IDS_CLAUSE,
};
use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::raindex_client::TimeFilter;
use alloy::primitives::Address;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub use super::fetch_order_trades_count::LocalDbTradeCountRow;

const START_TS_BODY: &str = "\nAND block_timestamp >= {param}\n";
const END_TS_BODY: &str = "\nAND block_timestamp <= {param}\n";

const CLEAR_ALICE_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_ALICE_CHAIN_IDS_CLAUSE*/";
const CLEAR_ALICE_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_ALICE_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_ALICE_ORDERBOOKS_CLAUSE*/";
const CLEAR_ALICE_ORDERBOOKS_CLAUSE_BODY: &str = "AND c.orderbook_address IN ({list})";

const CLEAR_BOB_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_BOB_CHAIN_IDS_CLAUSE*/";
const CLEAR_BOB_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_BOB_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_BOB_ORDERBOOKS_CLAUSE*/";
const CLEAR_BOB_ORDERBOOKS_CLAUSE_BODY: &str = "AND c.orderbook_address IN ({list})";

#[derive(Debug, Clone)]
pub struct FetchOwnerTradesCountArgs {
    pub owner: Address,
    pub chain_ids: Vec<u32>,
    pub orderbook_addresses: Vec<Address>,
    pub time_filter: TimeFilter,
}

pub fn build_fetch_owner_trades_count_stmt(
    args: &FetchOwnerTradesCountArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    let filters = bind_common_owner_trade_filters(
        &mut stmt,
        args.owner,
        &args.chain_ids,
        &args.orderbook_addresses,
        &args.time_filter,
        START_TS_BODY,
        END_TS_BODY,
    )?;

    let chain_ids_iter = || filters.chain_ids.iter().cloned().map(SqlValue::from);
    let orderbooks_iter = || filters.orderbooks.iter().cloned().map(SqlValue::from);

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

    Ok(stmt)
}

pub fn extract_trade_count(rows: &[LocalDbTradeCountRow]) -> u64 {
    rows.first().map(|row| row.trade_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::fetch_owner_trades_common::{END_TS_CLAUSE, START_TS_CLAUSE};
    use alloy::primitives::address;

    #[test]
    fn builds_with_chain_ids() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_count_stmt(&FetchOwnerTradesCountArgs {
            owner,
            chain_ids: vec![137, 1],
            orderbook_addresses: vec![],
            time_filter: TimeFilter::default(),
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
            time_filter: TimeFilter {
                start: Some(1000),
                end: Some(2000),
            },
        })
        .unwrap();
        assert!(stmt.sql.contains("block_timestamp >="));
        assert!(stmt.sql.contains("block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
    }

    #[test]
    fn builds_without_filters() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_count_stmt(&FetchOwnerTradesCountArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            time_filter: TimeFilter::default(),
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
