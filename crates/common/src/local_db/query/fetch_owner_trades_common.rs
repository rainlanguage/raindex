use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::raindex_client::TimeFilter;
use alloy::primitives::Address;
use std::convert::TryFrom;

pub(crate) const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
pub(crate) const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND t.chain_id IN ({list})";
pub(crate) const TAKE_ORDERS_ORDERBOOKS_CLAUSE: &str = "/*TAKE_ORDERS_ORDERBOOKS_CLAUSE*/";
pub(crate) const TAKE_ORDERS_ORDERBOOKS_CLAUSE_BODY: &str = "AND t.orderbook_address IN ({list})";

pub(crate) const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
pub(crate) const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";

pub(crate) struct PreparedOwnerTradeFilters {
    pub chain_ids: Vec<u32>,
    pub orderbooks: Vec<Address>,
}

pub(crate) fn bind_common_owner_trade_filters(
    stmt: &mut SqlStatement,
    owner: Address,
    chain_ids: &[u32],
    orderbook_addresses: &[Address],
    time_filter: &TimeFilter,
    start_ts_body: &str,
    end_ts_body: &str,
) -> Result<PreparedOwnerTradeFilters, SqlBuildError> {
    stmt.push(SqlValue::from(owner));

    let mut chain_ids = chain_ids.to_vec();
    chain_ids.sort_unstable();
    chain_ids.dedup();

    let mut orderbooks = orderbook_addresses.to_vec();
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

    if let (Some(start), Some(end)) = (time_filter.start, time_filter.end) {
        if start > end {
            return Err(SqlBuildError::new("start_timestamp > end_timestamp"));
        }
    }

    let start_param = if let Some(v) = time_filter.start {
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
    stmt.bind_param_clause(START_TS_CLAUSE, start_ts_body, start_param)?;

    let end_param = if let Some(v) = time_filter.end {
        let i = i64::try_from(v).map_err(|e| {
            SqlBuildError::new(format!("end_timestamp out of range for i64: {} ({})", v, e))
        })?;
        Some(SqlValue::I64(i))
    } else {
        None
    };
    stmt.bind_param_clause(END_TS_CLAUSE, end_ts_body, end_param)?;

    Ok(PreparedOwnerTradeFilters {
        chain_ids,
        orderbooks,
    })
}
