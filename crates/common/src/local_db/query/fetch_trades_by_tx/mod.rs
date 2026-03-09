use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::{Address, B256};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND t.chain_id IN ({list})";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE: &str = "/*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/";
const TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY: &str = "AND t.orderbook_address IN ({list})";

const CLEAR_EVENTS_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/";
const CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE: &str = "/*CLEAR_EVENTS_ORDERBOOKS_CLAUSE*/";
const CLEAR_EVENTS_ORDERBOOKS_CLAUSE_BODY: &str = "AND c.orderbook_address IN ({list})";

#[derive(Debug, Clone)]
pub struct FetchTradesByTxArgs {
    pub chain_ids: Vec<u32>,
    pub orderbook_addresses: Vec<Address>,
    pub tx_hash: B256,
}

pub fn build_fetch_trades_by_tx_stmt(
    args: &FetchTradesByTxArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    stmt.push(SqlValue::from(args.tx_hash));

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
    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex,
        primitives::{address, b256},
    };

    #[test]
    fn builds_with_chain_ids_and_tx_hash() {
        let tx_hash =
            b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let stmt = build_fetch_trades_by_tx_stmt(&FetchTradesByTxArgs {
            chain_ids: vec![137, 1, 137],
            orderbook_addresses: vec![],
            tx_hash,
        })
        .unwrap();
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(
            stmt.params[0],
            SqlValue::Text(hex::encode_prefixed(tx_hash))
        );
        assert_eq!(stmt.params[1], SqlValue::U64(1));
        assert_eq!(stmt.params[2], SqlValue::U64(137));
        assert_eq!(stmt.params[3], SqlValue::U64(1));
        assert_eq!(stmt.params[4], SqlValue::U64(137));
        assert!(stmt.sql.contains("t.chain_id IN (?2, ?3)"));
        assert!(stmt.sql.contains("c.chain_id IN (?4, ?5)"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_CHAIN_IDS_CLAUSE));
    }

    #[test]
    fn builds_with_orderbook_address_filters() {
        let tx_hash =
            b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let ob = address!("0x2f209e5b67a33b8fe96e28f24628df6da301c8eb");
        let stmt = build_fetch_trades_by_tx_stmt(&FetchTradesByTxArgs {
            chain_ids: vec![137],
            orderbook_addresses: vec![ob],
            tx_hash,
        })
        .unwrap();
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(
            stmt.params[0],
            SqlValue::Text(hex::encode_prefixed(tx_hash))
        );
        assert_eq!(stmt.params[1], SqlValue::U64(137));
        assert_eq!(stmt.params[2], SqlValue::U64(137));
        assert_eq!(
            stmt.params[3],
            SqlValue::Text(hex::encode_prefixed(ob))
        );
        assert_eq!(
            stmt.params[4],
            SqlValue::Text(hex::encode_prefixed(ob))
        );
        assert!(stmt.sql.contains("t.orderbook_address IN (?4)"));
        assert!(stmt.sql.contains("c.orderbook_address IN (?5)"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_ORDERBOOKS_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_ORDERBOOKS_CLAUSE));
    }
}
