use crate::local_db::query::fetch_order_trades_count::LocalDbTradeCountRow;
use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::raindex_client::{PaginationParams, TimeFilter};
use alloy::primitives::{Address, B256};
use std::convert::TryFrom;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND t.chain_id IN ({list})";
const TAKE_ORDERS_RAINDEXES_CLAUSE: &str = "/*TAKE_ORDERS_RAINDEXES_CLAUSE*/";
const TAKE_ORDERS_RAINDEXES_CLAUSE_BODY: &str = "AND t.raindex_address IN ({list})";
const TAKE_ORDERS_TAKERS_CLAUSE: &str = "/*TAKE_ORDERS_TAKERS_CLAUSE*/";
const TAKE_ORDERS_TAKERS_CLAUSE_BODY: &str = "AND t.sender IN ({list})";

const CLEAR_EVENTS_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/";
const CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY: &str = "AND c.chain_id IN ({list})";
const CLEAR_EVENTS_RAINDEXES_CLAUSE: &str = "/*CLEAR_EVENTS_RAINDEXES_CLAUSE*/";
const CLEAR_EVENTS_RAINDEXES_CLAUSE_BODY: &str = "AND c.raindex_address IN ({list})";
const CLEAR_EVENTS_TAKERS_CLAUSE: &str = "/*CLEAR_EVENTS_TAKERS_CLAUSE*/";
const CLEAR_EVENTS_TAKERS_CLAUSE_BODY: &str = "AND c.sender IN ({list})";
const CLEAR_EVENTS_ORDER_HASHES_CLAUSE: &str = "/*CLEAR_EVENTS_ORDER_HASHES_CLAUSE*/";
const CLEAR_EVENTS_ORDER_HASHES_CLAUSE_BODY: &str =
    "AND (c.alice_order_hash IN ({list}) OR c.bob_order_hash IN ({list}))";
const TAKE_TRADES_ORDER_HASHES_CLAUSE: &str = "/*TAKE_TRADES_ORDER_HASHES_CLAUSE*/";
const TAKE_TRADES_ORDER_HASHES_CLAUSE_BODY: &str = "AND oe.order_hash IN ({list})";
const CLEAR_ALICE_ORDER_HASHES_CLAUSE: &str = "/*CLEAR_ALICE_ORDER_HASHES_CLAUSE*/";
const CLEAR_ALICE_ORDER_HASHES_CLAUSE_BODY: &str = "AND mc.alice_order_hash IN ({list})";
const CLEAR_BOB_ORDER_HASHES_CLAUSE: &str = "/*CLEAR_BOB_ORDER_HASHES_CLAUSE*/";
const CLEAR_BOB_ORDER_HASHES_CLAUSE_BODY: &str = "AND mc.bob_order_hash IN ({list})";
const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
const OWNERS_CLAUSE_BODY: &str = "AND tws.order_owner IN ({list})";
const ORDER_HASH_CLAUSE: &str = "/*ORDER_HASH_CLAUSE*/";
const ORDER_HASH_CLAUSE_BODY: &str = "AND tws.order_hash = {param}";
const ORDER_HASHES_CLAUSE: &str = "/*ORDER_HASHES_CLAUSE*/";
const ORDER_HASHES_CLAUSE_BODY: &str = "AND tws.order_hash IN ({list})";
const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "AND tws.block_timestamp >= {param}";
const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "AND tws.block_timestamp <= {param}";
const PAGINATION_CLAUSE: &str = "/*PAGINATION_CLAUSE*/";
const INPUT_TOKENS_CLAUSE: &str = "/*INPUT_TOKENS_CLAUSE*/";
const INPUT_TOKENS_CLAUSE_BODY: &str = "AND tws.input_token IN ({list})";
const OUTPUT_TOKENS_CLAUSE: &str = "/*OUTPUT_TOKENS_CLAUSE*/";
const OUTPUT_TOKENS_CLAUSE_BODY: &str = "AND tws.output_token IN ({list})";
const COMBINED_TOKENS_CLAUSE_BODY: &str =
    "AND (tws.input_token IN ({input_list}) OR tws.output_token IN ({output_list}))";

#[derive(Debug, Clone, Default)]
pub struct FetchTradesTokensFilter {
    pub inputs: Vec<Address>,
    pub outputs: Vec<Address>,
}

#[derive(Debug, Clone, Default)]
pub struct FetchTradesArgs {
    pub chain_ids: Vec<u32>,
    pub raindex_addresses: Vec<Address>,
    pub owners: Vec<Address>,
    pub takers: Vec<Address>,
    pub order_hash: Option<B256>,
    pub order_hashes: Vec<B256>,
    pub tokens: FetchTradesTokensFilter,
    pub time_filter: TimeFilter,
    pub pagination: PaginationParams,
}

pub fn build_fetch_trades_stmt(args: &FetchTradesArgs) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    let mut chain_ids = args.chain_ids.clone();
    chain_ids.sort_unstable();
    chain_ids.dedup();

    let mut raindexes = args.raindex_addresses.clone();
    raindexes.sort();
    raindexes.dedup();

    let chain_ids_iter = || chain_ids.iter().cloned().map(SqlValue::from);
    let raindexes_iter = || raindexes.iter().cloned().map(SqlValue::from);

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
        TAKE_ORDERS_RAINDEXES_CLAUSE,
        TAKE_ORDERS_RAINDEXES_CLAUSE_BODY,
        raindexes_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_EVENTS_RAINDEXES_CLAUSE,
        CLEAR_EVENTS_RAINDEXES_CLAUSE_BODY,
        raindexes_iter(),
    )?;
    let mut takers = args.takers.clone();
    takers.sort();
    takers.dedup();
    let takers_iter = || takers.iter().cloned().map(SqlValue::from);
    stmt.bind_list_clause(
        TAKE_ORDERS_TAKERS_CLAUSE,
        TAKE_ORDERS_TAKERS_CLAUSE_BODY,
        takers_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_EVENTS_TAKERS_CLAUSE,
        CLEAR_EVENTS_TAKERS_CLAUSE_BODY,
        takers_iter(),
    )?;
    let mut owners = args.owners.clone();
    owners.sort();
    owners.dedup();
    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        owners.into_iter().map(SqlValue::from),
    )?;
    stmt.bind_param_clause(
        ORDER_HASH_CLAUSE,
        ORDER_HASH_CLAUSE_BODY,
        args.order_hash.map(SqlValue::from),
    )?;
    let mut order_hashes = args.order_hashes.clone();
    order_hashes.sort();
    order_hashes.dedup();
    let order_hashes_iter = || order_hashes.iter().cloned().map(SqlValue::from);
    stmt.bind_list_clause(
        CLEAR_EVENTS_ORDER_HASHES_CLAUSE,
        CLEAR_EVENTS_ORDER_HASHES_CLAUSE_BODY,
        order_hashes_iter(),
    )?;
    stmt.bind_list_clause(
        TAKE_TRADES_ORDER_HASHES_CLAUSE,
        TAKE_TRADES_ORDER_HASHES_CLAUSE_BODY,
        order_hashes_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_ALICE_ORDER_HASHES_CLAUSE,
        CLEAR_ALICE_ORDER_HASHES_CLAUSE_BODY,
        order_hashes_iter(),
    )?;
    stmt.bind_list_clause(
        CLEAR_BOB_ORDER_HASHES_CLAUSE,
        CLEAR_BOB_ORDER_HASHES_CLAUSE_BODY,
        order_hashes_iter(),
    )?;
    stmt.bind_list_clause(
        ORDER_HASHES_CLAUSE,
        ORDER_HASHES_CLAUSE_BODY,
        order_hashes_iter(),
    )?;

    if let (Some(start), Some(end)) = (args.time_filter.start, args.time_filter.end) {
        if start > end {
            return Err(SqlBuildError::new("start_timestamp > end_timestamp"));
        }
    }
    let start_param = args
        .time_filter
        .start
        .map(|v| {
            i64::try_from(v).map(SqlValue::I64).map_err(|e| {
                SqlBuildError::new(format!(
                    "start_timestamp out of range for i64: {} ({})",
                    v, e
                ))
            })
        })
        .transpose()?;
    stmt.bind_param_clause(START_TS_CLAUSE, START_TS_BODY, start_param)?;

    let end_param = args
        .time_filter
        .end
        .map(|v| {
            i64::try_from(v).map(SqlValue::I64).map_err(|e| {
                SqlBuildError::new(format!("end_timestamp out of range for i64: {} ({})", v, e))
            })
        })
        .transpose()?;
    stmt.bind_param_clause(END_TS_CLAUSE, END_TS_BODY, end_param)?;
    bind_token_filters(&mut stmt, &args.tokens)?;
    if let Some(page) = args.pagination.page {
        let page_size = args.pagination.page_size.unwrap_or(100);
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

pub fn build_fetch_trades_count_stmt(
    args: &FetchTradesArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut args = args.clone();
    args.pagination = PaginationParams::default();
    let stmt = build_fetch_trades_stmt(&args)?;
    let inner_sql = stmt.sql.trim().trim_end_matches(';').trim();
    Ok(SqlStatement {
        sql: format!(
            "SELECT COUNT(*) AS trade_count FROM ({}) AS filtered_trades",
            inner_sql
        ),
        params: stmt.params,
    })
}

pub fn extract_trades_count(rows: &[LocalDbTradeCountRow]) -> u64 {
    rows.first().map(|row| row.trade_count).unwrap_or(0)
}

fn bind_token_filters(
    stmt: &mut SqlStatement,
    tokens: &FetchTradesTokensFilter,
) -> Result<(), SqlBuildError> {
    let mut input_tokens = tokens.inputs.clone();
    input_tokens.sort();
    input_tokens.dedup();

    let mut output_tokens = tokens.outputs.clone();
    output_tokens.sort();
    output_tokens.dedup();

    let has_inputs = !input_tokens.is_empty();
    let has_outputs = !output_tokens.is_empty();

    if has_inputs && has_outputs && input_tokens == output_tokens {
        let input_placeholders: Vec<String> = input_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", stmt.params.len() + i + 1))
            .collect();
        let input_list = input_placeholders.join(", ");
        for token in &input_tokens {
            stmt.push(SqlValue::from(*token));
        }

        let output_placeholders: Vec<String> = output_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", stmt.params.len() + i + 1))
            .collect();
        let output_list = output_placeholders.join(", ");
        for token in &output_tokens {
            stmt.push(SqlValue::from(*token));
        }

        let clause = COMBINED_TOKENS_CLAUSE_BODY
            .replace("{input_list}", &input_list)
            .replace("{output_list}", &output_list);
        stmt.replace(INPUT_TOKENS_CLAUSE, &clause)?;
        stmt.replace(OUTPUT_TOKENS_CLAUSE, "")?;
    } else {
        stmt.bind_list_clause(
            INPUT_TOKENS_CLAUSE,
            INPUT_TOKENS_CLAUSE_BODY,
            input_tokens.into_iter().map(SqlValue::from),
        )?;
        stmt.bind_list_clause(
            OUTPUT_TOKENS_CLAUSE,
            OUTPUT_TOKENS_CLAUSE_BODY,
            output_tokens.into_iter().map(SqlValue::from),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex,
        primitives::{address, b256},
    };

    #[test]
    fn builds_with_chain_ids() {
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs {
            chain_ids: vec![137, 1, 137],
            raindex_addresses: vec![],
            ..Default::default()
        })
        .unwrap();
        assert_eq!(stmt.params.len(), 4);
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::U64(137));
        assert_eq!(stmt.params[2], SqlValue::U64(1));
        assert_eq!(stmt.params[3], SqlValue::U64(137));
        assert!(stmt.sql.contains("t.chain_id IN (?1, ?2)"));
        assert!(stmt.sql.contains("c.chain_id IN (?3, ?4)"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_CHAIN_IDS_CLAUSE));
    }

    #[test]
    fn builds_with_raindex_address_filters() {
        let ob = address!("0x2f209e5b67a33b8fe96e28f24628df6da301c8eb");
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs {
            chain_ids: vec![137],
            raindex_addresses: vec![ob],
            ..Default::default()
        })
        .unwrap();
        assert_eq!(stmt.params.len(), 4);
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::U64(137));
        assert_eq!(stmt.params[2], SqlValue::Text(hex::encode_prefixed(ob)));
        assert_eq!(stmt.params[3], SqlValue::Text(hex::encode_prefixed(ob)));
        assert!(stmt.sql.contains("t.raindex_address IN (?3)"));
        assert!(stmt.sql.contains("c.raindex_address IN (?4)"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_RAINDEXES_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_RAINDEXES_CLAUSE));
    }

    #[test]
    fn builds_with_directional_token_filters() {
        let input = address!("0x1111111111111111111111111111111111111111");
        let output = address!("0x2222222222222222222222222222222222222222");
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs {
            tokens: FetchTradesTokensFilter {
                inputs: vec![input],
                outputs: vec![output],
            },
            ..Default::default()
        })
        .unwrap();

        assert!(stmt.sql.contains("tws.input_token IN (?1)"));
        assert!(stmt.sql.contains("tws.output_token IN (?2)"));
        assert_eq!(stmt.params[0], SqlValue::Text(hex::encode_prefixed(input)));
        assert_eq!(stmt.params[1], SqlValue::Text(hex::encode_prefixed(output)));
    }

    #[test]
    fn builds_with_taker_filters() {
        let taker_a = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let taker_b = address!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs {
            takers: vec![taker_b, taker_a, taker_a],
            ..Default::default()
        })
        .unwrap();

        assert!(stmt.sql.contains("t.sender IN (?1, ?2)"));
        assert!(stmt.sql.contains("c.sender IN (?3, ?4)"));
        assert!(!stmt.sql.contains("tws.transaction_sender IN"));
        assert!(!stmt.sql.contains(TAKE_ORDERS_TAKERS_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_TAKERS_CLAUSE));
        assert_eq!(
            stmt.params[0],
            SqlValue::Text(hex::encode_prefixed(taker_a))
        );
        assert_eq!(
            stmt.params[1],
            SqlValue::Text(hex::encode_prefixed(taker_b))
        );
        assert_eq!(
            stmt.params[2],
            SqlValue::Text(hex::encode_prefixed(taker_a))
        );
        assert_eq!(
            stmt.params[3],
            SqlValue::Text(hex::encode_prefixed(taker_b))
        );
    }

    #[test]
    fn builds_with_batch_order_hash_filters() {
        let hash_a = b256!("0x1111111111111111111111111111111111111111111111111111111111111111");
        let hash_b = b256!("0x2222222222222222222222222222222222222222222222222222222222222222");
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs {
            order_hashes: vec![hash_b, hash_a, hash_a],
            ..Default::default()
        })
        .unwrap();

        assert!(stmt
            .sql
            .contains("c.alice_order_hash IN (?1, ?2) OR c.bob_order_hash IN (?1, ?2)"));
        assert!(stmt.sql.contains("oe.order_hash IN (?3, ?4)"));
        assert!(stmt.sql.contains("mc.alice_order_hash IN (?5, ?6)"));
        assert!(stmt.sql.contains("mc.bob_order_hash IN (?7, ?8)"));
        assert!(stmt.sql.contains("tws.order_hash IN (?9, ?10)"));
        assert!(!stmt.sql.contains(ORDER_HASHES_CLAUSE));
        assert!(!stmt.sql.contains(TAKE_TRADES_ORDER_HASHES_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_ALICE_ORDER_HASHES_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_BOB_ORDER_HASHES_CLAUSE));
        assert!(!stmt.sql.contains(CLEAR_EVENTS_ORDER_HASHES_CLAUSE));
        assert_eq!(stmt.params[0], SqlValue::Text(hex::encode_prefixed(hash_a)));
        assert_eq!(stmt.params[1], SqlValue::Text(hex::encode_prefixed(hash_b)));
        assert_eq!(stmt.params[6], SqlValue::Text(hex::encode_prefixed(hash_a)));
        assert_eq!(stmt.params[7], SqlValue::Text(hex::encode_prefixed(hash_b)));
        assert_eq!(stmt.params[8], SqlValue::Text(hex::encode_prefixed(hash_a)));
        assert_eq!(stmt.params[9], SqlValue::Text(hex::encode_prefixed(hash_b)));
    }

    #[test]
    fn builds_with_same_token_as_either_side_filter() {
        let token = address!("0x1111111111111111111111111111111111111111");
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs {
            tokens: FetchTradesTokensFilter {
                inputs: vec![token],
                outputs: vec![token],
            },
            ..Default::default()
        })
        .unwrap();

        assert!(stmt
            .sql
            .contains("tws.input_token IN (?1) OR tws.output_token IN (?2)"));
        assert!(!stmt.sql.contains(INPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(OUTPUT_TOKENS_CLAUSE));
        assert_eq!(stmt.params[0], SqlValue::Text(hex::encode_prefixed(token)));
        assert_eq!(stmt.params[1], SqlValue::Text(hex::encode_prefixed(token)));
    }

    #[test]
    fn builds_with_pagination() {
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs {
            pagination: PaginationParams {
                page: Some(2),
                page_size: Some(50),
            },
            ..Default::default()
        })
        .unwrap();

        assert!(stmt.sql.contains("LIMIT ?1 OFFSET ?2"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
        assert_eq!(stmt.params, vec![SqlValue::U64(50), SqlValue::U64(50)]);
    }

    #[test]
    fn disambiguates_clear_sides_in_trade_id_and_sort_order() {
        let stmt = build_fetch_trades_stmt(&FetchTradesArgs::default()).unwrap();

        assert!(stmt.sql.contains("'alice' AS trade_side"));
        assert!(stmt.sql.contains("'bob' AS trade_side"));
        assert!(stmt.sql.contains("WHEN 'alice' THEN '01'"));
        assert!(stmt.sql.contains("WHEN 'bob' THEN '02'"));
        assert!(stmt.sql.contains("tws.trade_kind, tws.trade_side\n"));
    }

    #[test]
    fn builds_count_query_without_pagination() {
        let stmt = build_fetch_trades_count_stmt(&FetchTradesArgs {
            pagination: PaginationParams {
                page: Some(2),
                page_size: Some(50),
            },
            ..Default::default()
        })
        .unwrap();

        assert!(stmt
            .sql
            .starts_with("SELECT COUNT(*) AS trade_count FROM ("));
        assert!(!stmt.sql.contains("LIMIT"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn builds_token_filtered_count_query_without_inner_trailing_semicolon() {
        let token = address!("0x1111111111111111111111111111111111111111");
        let stmt = build_fetch_trades_count_stmt(&FetchTradesArgs {
            tokens: FetchTradesTokensFilter {
                inputs: vec![token],
                outputs: vec![token],
            },
            ..Default::default()
        })
        .unwrap();

        assert!(!stmt.sql.contains(";\n) AS filtered_trades"));
        assert!(!stmt.sql.contains(";) AS filtered_trades"));
        assert!(stmt
            .sql
            .contains("tws.input_token IN (?1) OR tws.output_token IN (?2)"));

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::local_db::functions::register_all(&conn).unwrap();
        conn.execute_batch(crate::local_db::query::create_tables::create_tables_sql())
            .unwrap();
        conn.prepare(&stmt.sql).unwrap();
    }

    #[test]
    fn extract_trades_count_works() {
        let rows = vec![LocalDbTradeCountRow { trade_count: 42 }];
        assert_eq!(extract_trades_count(&rows), 42);
        let empty: Vec<LocalDbTradeCountRow> = vec![];
        assert_eq!(extract_trades_count(&empty), 0);
    }
}
