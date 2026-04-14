use crate::local_db::query::fetch_owner_trades_common::{
    bind_common_owner_trade_filters, TAKE_ORDERS_CHAIN_IDS_CLAUSE, TAKE_ORDERS_ORDERBOOKS_CLAUSE,
};
use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::raindex_client::{PaginationParams, TimeFilter};
use alloy::primitives::Address;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

const START_TS_BODY: &str = "\nAND tws.block_timestamp >= {param}\n";
const END_TS_BODY: &str = "\nAND tws.block_timestamp <= {param}\n";

const CLEAR_EVENTS_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/";
const CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_EVENTS_ORDERBOOKS_CLAUSE*/";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE_BODY: &str = "AND c.orderbook_address IN ({list})";

const PAGINATION_CLAUSE: &str = "/*PAGINATION_CLAUSE*/";

#[derive(Debug, Clone)]
pub struct FetchOwnerTradesArgs {
    pub owner: Address,
    pub chain_ids: Vec<u32>,
    pub orderbook_addresses: Vec<Address>,
    pub time_filter: TimeFilter,
    pub pagination: PaginationParams,
}

pub const DEFAULT_PAGE_SIZE: u16 = 100;

pub fn build_fetch_owner_trades_stmt(
    args: &FetchOwnerTradesArgs,
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
        CLEAR_EVENTS_CHAIN_IDS_CLAUSE,
        CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_EVENTS_ORDERBOOKS_CLAUSE,
        CLEAR_EVENTS_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;

    if let Some(page) = args.pagination.page {
        let page_size = args.pagination.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page.saturating_sub(1) as u64) * (page_size as u64);
        let limit_placeholder = format!("?{}", stmt.params.len() + 1);
        let offset_placeholder = format!("?{}", stmt.params.len() + 2);
        let pagination = format!("LIMIT {} OFFSET {}", limit_placeholder, offset_placeholder);
        stmt.sql = stmt.sql.replace(PAGINATION_CLAUSE, &pagination);
        stmt.push(SqlValue::U64(page_size as u64));
        stmt.push(SqlValue::U64(offset));
    } else {
        stmt.sql = stmt.sql.replace(PAGINATION_CLAUSE, "");
    }

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::fetch_owner_trades_common::{END_TS_CLAUSE, START_TS_CLAUSE};
    use alloy::{hex, primitives::address};

    #[test]
    fn builds_with_chain_ids_and_owner() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![137, 1, 137],
            orderbook_addresses: vec![],
            time_filter: TimeFilter::default(),
            pagination: PaginationParams::default(),
        })
        .unwrap();
        assert_eq!(stmt.params[0], SqlValue::Text(hex::encode_prefixed(owner)));
        assert!(stmt.sql.contains("t.chain_id IN"));
        assert!(stmt.sql.contains("c.chain_id IN"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_CHAIN_IDS_CLAUSE));
    }

    #[test]
    fn builds_with_orderbook_address_filters() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let ob = address!("0x2f209e5b67a33b8fe96e28f24628df6da301c8eb");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![137],
            orderbook_addresses: vec![ob],
            time_filter: TimeFilter::default(),
            pagination: PaginationParams::default(),
        })
        .unwrap();
        assert!(stmt.sql.contains("t.orderbook_address IN"));
        assert!(stmt.sql.contains("c.orderbook_address IN"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_ORDERBOOKS_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_ORDERBOOKS_CLAUSE));
    }

    #[test]
    fn builds_with_time_filters() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![137],
            orderbook_addresses: vec![],
            time_filter: TimeFilter {
                start: Some(1000),
                end: Some(2000),
            },
            pagination: PaginationParams::default(),
        })
        .unwrap();
        assert!(stmt.sql.contains("tws.block_timestamp >="));
        assert!(stmt.sql.contains("tws.block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            time_filter: TimeFilter::default(),
            pagination: PaginationParams::default(),
        })
        .unwrap();
        assert!(!stmt.sql.contains("tws.block_timestamp >="));
        assert!(!stmt.sql.contains("tws.block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert_eq!(stmt.params.len(), 1);
    }

    #[test]
    fn builds_with_pagination() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            time_filter: TimeFilter::default(),
            pagination: PaginationParams {
                page: Some(2),
                page_size: None,
            },
        })
        .unwrap();
        assert!(stmt.sql.contains("LIMIT"));
        assert!(stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
        let last_two = &stmt.params[stmt.params.len() - 2..];
        assert_eq!(last_two[0], SqlValue::U64(DEFAULT_PAGE_SIZE as u64));
        assert_eq!(last_two[1], SqlValue::U64(DEFAULT_PAGE_SIZE as u64));
    }

    #[test]
    fn rejects_inverted_timestamp_window() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let result = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            time_filter: TimeFilter {
                start: Some(2000),
                end: Some(1000),
            },
            pagination: PaginationParams::default(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn builds_without_pagination_when_none() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            time_filter: TimeFilter::default(),
            pagination: PaginationParams::default(),
        })
        .unwrap();
        assert!(!stmt.sql.contains("LIMIT"));
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }
}
