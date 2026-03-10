use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::Address;
use std::convert::TryFrom;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND t.chain_id IN ({list})";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE: &str = "/*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY: &str = "AND t.orderbook_address IN ({list})";

const CLEAR_EVENTS_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/";
const CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_EVENTS_ORDERBOOKS_CLAUSE*/";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE_BODY: &str = "AND c.orderbook_address IN ({list})";

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND tws.block_timestamp >= {param}\n";
const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND tws.block_timestamp <= {param}\n";
const PAGINATION_CLAUSE: &str = "/*PAGINATION_CLAUSE*/";

#[derive(Debug, Clone)]
pub struct FetchOwnerTradesArgs {
    pub owner: Address,
    pub chain_ids: Vec<u32>,
    pub orderbook_addresses: Vec<Address>,
    pub start_timestamp: Option<u64>,
    pub end_timestamp: Option<u64>,
    pub page: Option<u16>,
}

pub const PAGE_SIZE: u16 = 100;

pub fn build_fetch_owner_trades_stmt(
    args: &FetchOwnerTradesArgs,
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
        CLEAR_EVENTS_CHAIN_IDS_CLAUSE,
        CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        TAKE_ORDERS_ORDERBOOKS_CLAUSE,
        TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_EVENTS_ORDERBOOKS_CLAUSE,
        CLEAR_EVENTS_ORDERBOOKS_CLAUSE_BODY,
        orderbooks_iter(),
    )?;

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

    if let Some(page) = args.page {
        let page_size = PAGE_SIZE;
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
    use alloy::{hex, primitives::address};

    #[test]
    fn builds_with_chain_ids_and_owner() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![137, 1, 137],
            orderbook_addresses: vec![],
            start_timestamp: None,
            end_timestamp: None,
            page: None,
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
            start_timestamp: None,
            end_timestamp: None,
            page: None,
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
            start_timestamp: Some(1000),
            end_timestamp: Some(2000),
            page: None,
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
            start_timestamp: None,
            end_timestamp: None,
            page: None,
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
            start_timestamp: None,
            end_timestamp: None,
            page: Some(2),
        })
        .unwrap();
        assert!(stmt.sql.contains("LIMIT"));
        assert!(stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
        let last_two = &stmt.params[stmt.params.len() - 2..];
        assert_eq!(last_two[0], SqlValue::U64(PAGE_SIZE as u64));
        assert_eq!(last_two[1], SqlValue::U64(PAGE_SIZE as u64));
    }

    #[test]
    fn builds_without_pagination_when_none() {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let stmt = build_fetch_owner_trades_stmt(&FetchOwnerTradesArgs {
            owner,
            chain_ids: vec![],
            orderbook_addresses: vec![],
            start_timestamp: None,
            end_timestamp: None,
            page: None,
        })
        .unwrap();
        assert!(!stmt.sql.contains("LIMIT"));
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }
}
