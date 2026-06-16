use super::fetch_orders::FetchOrdersArgs;
use super::fetch_orders_common::bind_common_order_filters;
use crate::local_db::query::{SqlBuildError, SqlStatement};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbOrdersCountRow {
    pub orders_count: u32,
}

pub fn build_fetch_orders_count_stmt(
    args: &FetchOrdersArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    bind_common_order_filters(&mut stmt, args)?;
    Ok(stmt)
}

pub fn extract_orders_count(rows: &[LocalDbOrdersCountRow]) -> u32 {
    rows.first().map(|row| row.orders_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::super::fetch_orders::FetchOrdersActiveFilter;
    use super::super::fetch_orders_common::{
        INPUT_TOKENS_CLAUSE, LATEST_ADD_CHAIN_IDS_CLAUSE, MAIN_CHAIN_IDS_CLAUSE, ORDER_HASH_CLAUSE,
        OUTPUT_TOKENS_CLAUSE, OWNERS_CLAUSE,
    };
    use super::*;
    use alloy::primitives::{address, b256, Address};

    #[test]
    fn builds_count_with_no_filters() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("COUNT(*)"));
        assert!(stmt.sql.contains("orders_count"));
        assert!(stmt.sql.contains("?1 = 'all'"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(INPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(OUTPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
    }

    #[test]
    fn builds_count_with_active_filter() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137],
            filter: FetchOrdersActiveFilter::Active,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'active'"));
    }

    #[test]
    fn builds_count_with_owners_and_order_hash() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            owners: vec![address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB")],
            order_hash: Some(b256!(
                "0x00000000000000000000000000000000000000000000000000000000deadbeef"
            )),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("l.order_owner IN ("));
        assert!(stmt
            .sql
            .contains("COALESCE(la.order_hash, l.order_hash) = "));
    }

    #[test]
    fn builds_count_with_chain_ids() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137, 1, 137],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(!stmt.sql.contains(MAIN_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(LATEST_ADD_CHAIN_IDS_CLAUSE));
        assert!(stmt.sql.contains("AND oe.chain_id IN (?"));
    }

    #[test]
    fn builds_count_with_orderbooks() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            orderbook_addresses: vec![Address::ZERO],
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND oe.orderbook_address IN (?"));
    }

    #[test]
    fn builds_count_with_combined_tokens_when_inputs_equal_outputs() {
        use super::super::fetch_orders::FetchOrdersTokensFilter;

        let token_a = address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB");
        let token_b = address!("0x1111111111111111111111111111111111111111");
        let tokens = vec![token_a, token_b];

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: tokens.clone(),
                outputs: tokens,
            },
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();

        assert!(
            stmt.sql.contains("lower(io2.io_type) = 'input'"),
            "should contain input check in combined clause"
        );
        assert!(
            stmt.sql.contains("lower(io2.io_type) = 'output'"),
            "should contain output check in combined clause"
        );
        assert!(
            stmt.sql.contains(" OR "),
            "should use OR to combine input and output checks"
        );
        assert!(
            !stmt.sql.contains(INPUT_TOKENS_CLAUSE),
            "input tokens placeholder should be replaced"
        );
        assert!(
            !stmt.sql.contains(OUTPUT_TOKENS_CLAUSE),
            "output tokens placeholder should be replaced"
        );
    }

    #[test]
    fn builds_count_with_separate_tokens_when_inputs_differ_from_outputs() {
        use super::super::fetch_orders::FetchOrdersTokensFilter;

        let token_a = address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB");
        let token_b = address!("0x1111111111111111111111111111111111111111");

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![token_a],
                outputs: vec![token_b],
            },
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_count_stmt(&args).unwrap();

        assert!(
            !stmt.sql.contains(INPUT_TOKENS_CLAUSE),
            "input tokens placeholder should be replaced"
        );
        assert!(
            !stmt.sql.contains(OUTPUT_TOKENS_CLAUSE),
            "output tokens placeholder should be replaced"
        );
        let exists_count = stmt.sql.matches("AND EXISTS").count();
        assert_eq!(
            exists_count, 2,
            "should have two separate EXISTS subqueries when tokens differ"
        );
    }

    #[test]
    fn extract_count_returns_value() {
        let rows = vec![LocalDbOrdersCountRow { orders_count: 42 }];
        assert_eq!(extract_orders_count(&rows), 42);
    }

    #[test]
    fn extract_count_returns_zero_for_empty() {
        let rows: Vec<LocalDbOrdersCountRow> = vec![];
        assert_eq!(extract_orders_count(&rows), 0);
    }
}
