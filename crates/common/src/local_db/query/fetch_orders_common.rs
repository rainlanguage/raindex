use super::fetch_orders::FetchOrdersArgs;
use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::Address;

use super::fetch_orders::FetchOrdersActiveFilter;

pub(crate) const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
pub(crate) const OWNERS_CLAUSE_BODY: &str = "AND l.order_owner IN ({list})";

pub(crate) const ORDER_HASH_CLAUSE: &str = "/*ORDER_HASH_CLAUSE*/";
pub(crate) const ORDER_HASH_CLAUSE_BODY: &str =
    "AND COALESCE(la.order_hash, l.order_hash) = {param}";

pub(crate) const INPUT_TOKENS_CLAUSE: &str = "/*INPUT_TOKENS_CLAUSE*/";
pub(crate) const INPUT_TOKENS_CLAUSE_BODY: &str = "AND EXISTS (
      SELECT 1 FROM order_ios io2
      WHERE io2.chain_id = l.chain_id
        AND io2.raindex_address = l.raindex_address
        AND io2.transaction_hash = la.transaction_hash
        AND io2.log_index = la.log_index
        AND lower(io2.io_type) = 'input'
        AND io2.token IN ({list})
    )";

pub(crate) const OUTPUT_TOKENS_CLAUSE: &str = "/*OUTPUT_TOKENS_CLAUSE*/";
pub(crate) const OUTPUT_TOKENS_CLAUSE_BODY: &str = "AND EXISTS (
      SELECT 1 FROM order_ios io2
      WHERE io2.chain_id = l.chain_id
        AND io2.raindex_address = l.raindex_address
        AND io2.transaction_hash = la.transaction_hash
        AND io2.log_index = la.log_index
        AND lower(io2.io_type) = 'output'
        AND io2.token IN ({list})
    )";

pub(crate) const COMBINED_TOKENS_CLAUSE_BODY: &str = "AND EXISTS (
      SELECT 1 FROM order_ios io2
      WHERE io2.chain_id = l.chain_id
        AND io2.raindex_address = l.raindex_address
        AND io2.transaction_hash = la.transaction_hash
        AND io2.log_index = la.log_index
        AND (
          (lower(io2.io_type) = 'input' AND io2.token IN ({input_list}))
          OR
          (lower(io2.io_type) = 'output' AND io2.token IN ({output_list}))
        )
    )";

pub(crate) const MAIN_CHAIN_IDS_CLAUSE: &str = "/*MAIN_CHAIN_IDS_CLAUSE*/";
pub(crate) const MAIN_CHAIN_IDS_CLAUSE_BODY: &str = "AND oe.chain_id IN ({list})";
pub(crate) const MAIN_RAINDEXES_CLAUSE: &str = "/*MAIN_RAINDEXES_CLAUSE*/";
pub(crate) const MAIN_RAINDEXES_CLAUSE_BODY: &str = "AND oe.raindex_address IN ({list})";

pub(crate) const LATEST_ADD_CHAIN_IDS_CLAUSE: &str = "/*LATEST_ADD_CHAIN_IDS_CLAUSE*/";
pub(crate) const LATEST_ADD_CHAIN_IDS_CLAUSE_BODY: &str = "AND oe.chain_id IN ({list})";
pub(crate) const LATEST_ADD_RAINDEXES_CLAUSE: &str = "/*LATEST_ADD_RAINDEXES_CLAUSE*/";
pub(crate) const LATEST_ADD_RAINDEXES_CLAUSE_BODY: &str = "AND oe.raindex_address IN ({list})";

pub(crate) struct PreparedFilters {
    pub chain_ids: Vec<u32>,
    pub raindexes: Vec<Address>,
}

pub(crate) fn bind_common_order_filters(
    stmt: &mut SqlStatement,
    args: &FetchOrdersArgs,
) -> Result<PreparedFilters, SqlBuildError> {
    let active_str = match args.filter {
        FetchOrdersActiveFilter::All => "all",
        FetchOrdersActiveFilter::Active => "active",
        FetchOrdersActiveFilter::Inactive => "inactive",
    };
    stmt.push(SqlValue::from(active_str));

    let mut chain_ids = args.chain_ids.clone();
    chain_ids.sort_unstable();
    chain_ids.dedup();

    let mut raindexes = args.raindex_addresses.clone();
    raindexes.sort();
    raindexes.dedup();

    let chain_ids_iter = || chain_ids.iter().cloned().map(SqlValue::from);
    let raindexes_iter = || raindexes.iter().cloned().map(SqlValue::from);

    stmt.bind_list_clause(
        MAIN_CHAIN_IDS_CLAUSE,
        MAIN_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        LATEST_ADD_CHAIN_IDS_CLAUSE,
        LATEST_ADD_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;

    stmt.bind_list_clause(
        MAIN_RAINDEXES_CLAUSE,
        MAIN_RAINDEXES_CLAUSE_BODY,
        raindexes_iter(),
    )?;
    stmt.bind_list_clause(
        LATEST_ADD_RAINDEXES_CLAUSE,
        LATEST_ADD_RAINDEXES_CLAUSE_BODY,
        raindexes_iter(),
    )?;

    let mut owners = args.owners.clone();
    owners.sort();
    owners.dedup();
    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        owners.into_iter().map(SqlValue::from),
    )?;

    let order_hash_val = args.order_hash.as_ref().map(|hash| SqlValue::from(*hash));
    stmt.bind_param_clause(ORDER_HASH_CLAUSE, ORDER_HASH_CLAUSE_BODY, order_hash_val)?;

    let mut input_tokens = args.tokens.inputs.clone();
    input_tokens.sort();
    input_tokens.dedup();

    let mut output_tokens = args.tokens.outputs.clone();
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
        let input_list_str = input_placeholders.join(", ");

        for token in &input_tokens {
            stmt.push(SqlValue::from(*token));
        }

        let output_placeholders: Vec<String> = output_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", stmt.params.len() + i + 1))
            .collect();
        let output_list_str = output_placeholders.join(", ");

        for token in &output_tokens {
            stmt.push(SqlValue::from(*token));
        }

        let combined_clause = COMBINED_TOKENS_CLAUSE_BODY
            .replace("{input_list}", &input_list_str)
            .replace("{output_list}", &output_list_str);

        stmt.sql = stmt.sql.replace(INPUT_TOKENS_CLAUSE, &combined_clause);
        stmt.sql = stmt.sql.replace(OUTPUT_TOKENS_CLAUSE, "");
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

    Ok(PreparedFilters {
        chain_ids,
        raindexes,
    })
}
