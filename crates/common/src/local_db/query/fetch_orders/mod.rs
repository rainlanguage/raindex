use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::utils::serde::bool_from_int_or_bool;
use alloy::primitives::{Address, Bytes, B256};
use serde::{Deserialize, Serialize};

use super::fetch_orders_common::bind_common_order_filters;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FetchOrdersActiveFilter {
    #[default]
    All,
    Active,
    Inactive,
}

#[derive(Debug, Clone, Default)]
pub struct FetchOrdersTokensFilter {
    pub inputs: Vec<Address>,
    pub outputs: Vec<Address>,
}

#[derive(Debug, Clone, Default)]
pub struct FetchOrdersArgs {
    pub chain_ids: Vec<u32>,
    pub raindex_addresses: Vec<Address>,
    pub filter: FetchOrdersActiveFilter,
    pub owners: Vec<Address>,
    pub order_hash: Option<B256>,
    pub tx_hash: Option<B256>,
    pub tokens: FetchOrdersTokensFilter,
    pub page: Option<u16>,
    pub page_size: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbOrder {
    pub chain_id: u32,
    pub order_hash: B256,
    pub owner: Address,
    pub block_timestamp: u64,
    pub block_number: u64,
    pub raindex_address: Address,
    pub order_bytes: Bytes,
    pub transaction_hash: B256,
    pub inputs: Option<String>,
    pub outputs: Option<String>,
    pub trade_count: u64,
    #[serde(deserialize_with = "bool_from_int_or_bool")]
    pub active: bool,
    pub meta: Option<Bytes>,
}

const FIRST_ADD_CHAIN_IDS_CLAUSE: &str = "/*FIRST_ADD_CHAIN_IDS_CLAUSE*/";
const FIRST_ADD_CHAIN_IDS_CLAUSE_BODY: &str = "AND oe.chain_id IN ({list})";
const FIRST_ADD_RAINDEXES_CLAUSE: &str = "/*FIRST_ADD_RAINDEXES_CLAUSE*/";
const FIRST_ADD_RAINDEXES_CLAUSE_BODY: &str = "AND oe.raindex_address IN ({list})";

const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND t.chain_id IN ({list})";
const TAKE_ORDERS_RAINDEXES_CLAUSE: &str = "/*TAKE_ORDERS_RAINDEXES_CLAUSE*/";
const TAKE_ORDERS_RAINDEXES_CLAUSE_BODY: &str = "AND t.raindex_address IN ({list})";

const CLEAR_EVENTS_CHAIN_IDS_CLAUSE: &str = "/*CLEAR_EVENTS_CHAIN_IDS_CLAUSE*/";
const CLEAR_EVENTS_CHAIN_IDS_CLAUSE_BODY: &str = "AND entries.chain_id IN ({list})";
const CLEAR_EVENTS_RAINDEXES_CLAUSE: &str = "/*CLEAR_EVENTS_RAINDEXES_CLAUSE*/";
const CLEAR_EVENTS_RAINDEXES_CLAUSE_BODY: &str = "AND entries.raindex_address IN ({list})";
const TX_HASH_CLAUSE: &str = "/*TX_HASH_CLAUSE*/";
const TX_HASH_CLAUSE_BODY: &str = "AND oe.transaction_hash = {param}";
const PAGINATION_CLAUSE: &str = "/*PAGINATION_CLAUSE*/";

pub fn build_fetch_orders_stmt(args: &FetchOrdersArgs) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    let prepared = bind_common_order_filters(&mut stmt, args)?;

    let chain_ids_iter = || prepared.chain_ids.iter().cloned().map(SqlValue::from);
    let raindexes_iter = || prepared.raindexes.iter().cloned().map(SqlValue::from);

    stmt.bind_list_clause(
        FIRST_ADD_CHAIN_IDS_CLAUSE,
        FIRST_ADD_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
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
        FIRST_ADD_RAINDEXES_CLAUSE,
        FIRST_ADD_RAINDEXES_CLAUSE_BODY,
        raindexes_iter(),
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

    let tx_hash_val = args.tx_hash.as_ref().map(|hash| SqlValue::from(*hash));
    stmt.bind_param_clause(TX_HASH_CLAUSE, TX_HASH_CLAUSE_BODY, tx_hash_val)?;

    if let (Some(page), Some(page_size)) = (args.page, args.page_size) {
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
    use super::super::fetch_orders_common::{
        INPUT_TOKENS_CLAUSE, LATEST_ADD_CHAIN_IDS_CLAUSE, LATEST_ADD_RAINDEXES_CLAUSE,
        MAIN_CHAIN_IDS_CLAUSE, MAIN_RAINDEXES_CLAUSE, ORDER_HASH_CLAUSE, ORDER_HASH_CLAUSE_BODY,
        OUTPUT_TOKENS_CLAUSE, OWNERS_CLAUSE,
    };
    use super::*;
    use alloy::hex;
    use alloy::primitives::{address, b256};
    use std::str::FromStr;

    #[test]
    fn filter_active_all_and_no_extras() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'all'"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(INPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(OUTPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));
    }

    #[test]
    fn owners_tokens_and_order_hash_filters_with_params() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137, 1],
            raindex_addresses: vec![address!("0xabcdef0000000000000000000000000000000000")],
            filter: FetchOrdersActiveFilter::Active,
            owners: vec![
                address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB"),
                address!("0x7D3Dd01feD0C16A6c353ce3BACF26467726EF96e"),
                address!("0x87d08841bdAd4aB82883a322D2c0eF557EC154fE"),
            ],
            order_hash: Some(b256!(
                "0x00000000000000000000000000000000000000000000000000000000deadbeef"
            )),
            tx_hash: None,
            tokens: FetchOrdersTokensFilter {
                inputs: vec![
                    address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB"),
                    address!("0x7D3Dd01feD0C16A6c353ce3BACF26467726EF96e"),
                ],
                outputs: vec![address!("0xF3dEe5b36E3402893e6953A8670E37D329683ABB")],
            },
            page: None,
            page_size: None,
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();

        // Active filter parameterized
        assert!(stmt.sql.contains("?1 = 'active'"));

        // Owners clause present, tokens clause present, order hash clause present
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(INPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(OUTPUT_TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(ORDER_HASH_CLAUSE));

        // Params include active filter followed by chain/raindex filters
        assert!(stmt.params.len() >= 3);
        assert_eq!(stmt.params[0], SqlValue::Text("active".to_string()));
    }

    #[test]
    fn input_tokens_clause_only_when_inputs_present() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![address!("0x00000000000000000000000000000000000000aa")],
                outputs: vec![],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'input'"));
        assert!(stmt.sql.contains("AND io2.token IN ("));
        assert!(!stmt.sql.contains("AND lower(io2.io_type) = 'output'"));
    }

    #[test]
    fn output_tokens_clause_only_when_outputs_present() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![],
                outputs: vec![address!("0x00000000000000000000000000000000000000bb")],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'output'"));
        assert!(stmt.sql.contains("AND io2.token IN ("));
        assert!(!stmt.sql.contains("AND lower(io2.io_type) = 'input'"));
    }

    #[test]
    fn combined_or_clause_when_inputs_and_outputs_identical() {
        // When inputs == outputs, use OR logic for "any-IO" filtering
        let token_addr = address!("0x00000000000000000000000000000000000000aa");

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![token_addr],
                outputs: vec![token_addr],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        // Should have a single EXISTS with OR logic inside
        assert!(stmt
            .sql
            .contains("(lower(io2.io_type) = 'input' AND io2.token IN ("));
        assert!(stmt
            .sql
            .contains("(lower(io2.io_type) = 'output' AND io2.token IN ("));
        assert!(stmt.sql.contains(" OR "));
        // Should only have one EXISTS clause, not two
        assert_eq!(stmt.sql.matches("AND EXISTS (").count(), 1);
    }

    #[test]
    fn separate_and_clauses_when_inputs_and_outputs_differ() {
        // When inputs != outputs, use AND logic for directional filtering
        let input_addr = address!("0x00000000000000000000000000000000000000aa");
        let output_addr = address!("0x00000000000000000000000000000000000000bb");

        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tokens: FetchOrdersTokensFilter {
                inputs: vec![input_addr],
                outputs: vec![output_addr],
            },
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();
        // Should have two separate EXISTS clauses (AND logic)
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'input'"));
        assert!(stmt.sql.contains("AND lower(io2.io_type) = 'output'"));
        // Should NOT have OR in the token filtering (io_type check)
        assert!(!stmt.sql.contains("(lower(io2.io_type) = 'input'"));
        // Should have two EXISTS clauses
        assert_eq!(stmt.sql.matches("AND EXISTS (").count(), 2);

        // Verify both tokens are in params
        let input_param = SqlValue::from(input_addr);
        let output_param = SqlValue::from(output_addr);
        assert!(stmt.params.contains(&input_param));
        assert!(stmt.params.contains(&output_param));
    }

    #[test]
    fn filter_inactive_string() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            filter: FetchOrdersActiveFilter::Inactive,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("?1 = 'inactive'"));
    }

    #[test]
    fn chain_id_filters_apply_across_all_sections() {
        let args = FetchOrdersArgs {
            chain_ids: vec![137, 1, 137],
            filter: FetchOrdersActiveFilter::All,
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();

        for marker in [
            MAIN_CHAIN_IDS_CLAUSE,
            LATEST_ADD_CHAIN_IDS_CLAUSE,
            FIRST_ADD_CHAIN_IDS_CLAUSE,
            TAKE_ORDERS_CHAIN_IDS_CLAUSE,
            CLEAR_EVENTS_CHAIN_IDS_CLAUSE,
        ] {
            assert!(
                !stmt.sql.contains(marker),
                "expected marker {marker} to be replaced"
            );
        }

        assert!(
            stmt.sql.contains("AND oe.chain_id IN (?"),
            "main chain filter missing"
        );
        assert!(
            stmt.sql.contains("AND t.chain_id IN (?"),
            "take_orders chain filter missing"
        );
        assert!(
            stmt.sql.contains("AND entries.chain_id IN (?"),
            "clear_events chain filter missing"
        );

        // Chain IDs should be deduplicated before binding; only 2 unique values expected.
        let unique_chain_ids: std::collections::HashSet<_> = stmt
            .params
            .iter()
            .filter_map(|value| match value {
                SqlValue::U64(id) => Some(*id),
                _ => None,
            })
            .collect();
        assert!(
            unique_chain_ids.contains(&1_u64) && unique_chain_ids.contains(&137_u64),
            "expected both chain IDs to be present in params"
        );
    }

    #[test]
    fn raindex_filters_lowercase_and_optional() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            raindex_addresses: vec![
                Address::from_str("0xAbCDeF0000000000000000000000000000000000").unwrap(),
                Address::from_str("0xabcdef0000000000000000000000000000000000").unwrap(),
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
            ],
            ..FetchOrdersArgs::default()
        };

        let stmt = build_fetch_orders_stmt(&args).unwrap();

        for marker in [
            MAIN_RAINDEXES_CLAUSE,
            LATEST_ADD_RAINDEXES_CLAUSE,
            FIRST_ADD_RAINDEXES_CLAUSE,
            TAKE_ORDERS_RAINDEXES_CLAUSE,
            CLEAR_EVENTS_RAINDEXES_CLAUSE,
        ] {
            assert!(
                !stmt.sql.contains(marker),
                "expected marker {marker} to be replaced"
            );
        }

        assert!(
            stmt.sql.contains("AND oe.raindex_address IN (?"),
            "main raindex filter missing"
        );
        assert!(
            stmt.sql.contains("AND t.raindex_address IN (?"),
            "take_orders raindex filter missing"
        );

        // Only the trimmed, lowercased address should appear in bound params.
        let lower_addr = "0xabcdef0000000000000000000000000000000000";
        let text_params: Vec<&String> = stmt
            .params
            .iter()
            .filter_map(|value| match value {
                SqlValue::Text(text) => Some(text),
                _ => None,
            })
            .collect();
        assert!(
            text_params.iter().any(|text| text.as_str() == lower_addr),
            "expected lowercase raindex address in params"
        );
        for text in text_params {
            let lowered = text.to_ascii_lowercase();
            assert_eq!(
                text.as_str(),
                lowered.as_str(),
                "raindex param should be lowercase"
            );
        }

        // When raindexes are omitted entirely, no raindex clause should remain.
        let args_no_raindexes = FetchOrdersArgs {
            chain_ids: vec![1],
            ..FetchOrdersArgs::default()
        };
        let stmt_no_raindexes = build_fetch_orders_stmt(&args_no_raindexes).unwrap();
        assert!(
            !stmt_no_raindexes.sql.contains("oe.raindex_address IN ("),
            "raindex clause should not appear when list is empty"
        );
    }

    #[test]
    fn tx_hash_clause_included_when_present() {
        let tx_hash = b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            tx_hash: Some(tx_hash),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(
            !stmt.sql.contains(TX_HASH_CLAUSE),
            "tx hash marker should be replaced"
        );
        assert!(
            stmt.sql.contains("oe.transaction_hash = ?"),
            "tx hash clause should be present"
        );
        let expected = SqlValue::Text(hex::encode_prefixed(tx_hash));
        assert!(
            stmt.params.contains(&expected),
            "tx hash param should be bound"
        );
    }

    #[test]
    fn pagination_clause_page1() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(1),
            page_size: Some(10),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("LIMIT"), "should contain LIMIT clause");
        assert!(stmt.sql.contains("OFFSET"), "should contain OFFSET clause");
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
        let last_two: Vec<&SqlValue> = stmt.params.iter().rev().take(2).collect();
        assert_eq!(last_two[1], &SqlValue::U64(10));
        assert_eq!(last_two[0], &SqlValue::U64(0));
    }

    #[test]
    fn pagination_clause_page3() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(3),
            page_size: Some(25),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("LIMIT"));
        assert!(stmt.sql.contains("OFFSET"));
        let last_two: Vec<&SqlValue> = stmt.params.iter().rev().take(2).collect();
        assert_eq!(last_two[1], &SqlValue::U64(25));
        assert_eq!(last_two[0], &SqlValue::U64(50));
    }

    #[test]
    fn pagination_clause_page0_saturates_to_zero_offset() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(0),
            page_size: Some(10),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(stmt.sql.contains("LIMIT"));
        let last_two: Vec<&SqlValue> = stmt.params.iter().rev().take(2).collect();
        assert_eq!(last_two[1], &SqlValue::U64(10));
        assert_eq!(last_two[0], &SqlValue::U64(0));
    }

    #[test]
    fn pagination_clause_omitted_when_only_page_set() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: Some(2),
            page_size: None,
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }

    #[test]
    fn pagination_clause_omitted_when_only_page_size_set() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            page: None,
            page_size: Some(10),
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }

    #[test]
    fn pagination_clause_omitted_when_neither_set() {
        let args = FetchOrdersArgs {
            chain_ids: vec![1],
            ..FetchOrdersArgs::default()
        };
        let stmt = build_fetch_orders_stmt(&args).unwrap();
        assert!(!stmt.sql.contains("OFFSET"));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
    }

    #[test]
    fn missing_order_hash_marker_yields_error() {
        // Simulate the ORDER_HASH_CLAUSE marker being removed from the template.
        let bad_template = QUERY_TEMPLATE.replace(ORDER_HASH_CLAUSE, "");
        let mut stmt = SqlStatement::new(bad_template);
        // Push the fixed params expected by the template (?1 active filter)
        stmt.push(SqlValue::Text("all".to_string()));
        let err = stmt
            .bind_param_clause(
                ORDER_HASH_CLAUSE,
                ORDER_HASH_CLAUSE_BODY,
                Some(SqlValue::Text("0xhash".into())),
            )
            .unwrap_err();
        assert!(matches!(err, SqlBuildError::MissingMarker { .. }));
    }

    #[test]
    fn missing_param_token_in_body_yields_error() {
        let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
        // Push the fixed params (?1 active filter)
        stmt.push(SqlValue::Text("all".to_string()));
        // Remove {param} token from the body to simulate drift between code and template
        let bad_body = ORDER_HASH_CLAUSE_BODY.replace("{param}", "");
        let err = stmt
            .bind_param_clause(
                ORDER_HASH_CLAUSE,
                &bad_body,
                Some(SqlValue::Text("0xhash".into())),
            )
            .unwrap_err();
        assert!(matches!(err, SqlBuildError::MissingMarker { .. }));
    }
}
