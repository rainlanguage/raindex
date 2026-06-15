use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use crate::raindex_client::TimeFilter;
use alloy::primitives::Address;
use std::convert::TryFrom;

pub(crate) const TAKE_ORDERS_CHAIN_IDS_CLAUSE: &str = "/*TAKE_ORDERS_CHAIN_IDS_CLAUSE*/";
pub(crate) const TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY: &str = "AND tws.chain_id IN ({list})";
pub(crate) const TAKE_ORDERS_RAINDEXS_CLAUSE: &str = "/*TAKE_ORDERS_RAINDEXS_CLAUSE*/";
pub(crate) const TAKE_ORDERS_RAINDEXS_CLAUSE_BODY: &str = "AND tws.raindex_address IN ({list})";

pub(crate) const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
pub(crate) const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";

pub(crate) fn bind_common_owner_trade_filters(
    stmt: &mut SqlStatement,
    owner: Address,
    chain_ids: &[u32],
    raindex_addresses: &[Address],
    time_filter: &TimeFilter,
    start_ts_body: &str,
    end_ts_body: &str,
) -> Result<(), SqlBuildError> {
    stmt.push(SqlValue::from(owner));

    let mut chain_ids = chain_ids.to_vec();
    chain_ids.sort_unstable();
    chain_ids.dedup();

    let mut raindexs = raindex_addresses.to_vec();
    raindexs.sort();
    raindexs.dedup();

    let chain_ids_iter = || chain_ids.iter().cloned().map(SqlValue::from);
    let raindexs_iter = || raindexs.iter().cloned().map(SqlValue::from);

    stmt.bind_list_clause(
        TAKE_ORDERS_CHAIN_IDS_CLAUSE,
        TAKE_ORDERS_CHAIN_IDS_CLAUSE_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(
        TAKE_ORDERS_RAINDEXS_CLAUSE,
        TAKE_ORDERS_RAINDEXS_CLAUSE_BODY,
        raindexs_iter(),
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    const START_TS_BODY: &str = " AND ts >= {param}";
    const END_TS_BODY: &str = " AND ts <= {param}";

    // Template carrying every marker the binder replaces, so the binder runs in
    // isolation (independent of any caller's query.sql).
    fn template() -> String {
        format!(
            "SELECT 1 WHERE owner = ?1 {} {} {} {}",
            TAKE_ORDERS_CHAIN_IDS_CLAUSE,
            TAKE_ORDERS_RAINDEXS_CLAUSE,
            START_TS_CLAUSE,
            END_TS_CLAUSE,
        )
    }

    fn bind(
        chain_ids: &[u32],
        raindexes: &[Address],
        time_filter: &TimeFilter,
    ) -> Result<SqlStatement, SqlBuildError> {
        let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let mut stmt = SqlStatement::new(template());
        bind_common_owner_trade_filters(
            &mut stmt,
            owner,
            chain_ids,
            raindexes,
            time_filter,
            START_TS_BODY,
            END_TS_BODY,
        )?;
        Ok(stmt)
    }

    fn u64_params(stmt: &SqlStatement) -> Vec<u64> {
        stmt.params()
            .iter()
            .filter_map(|p| match p {
                SqlValue::U64(v) => Some(*v),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn owner_is_first_param_as_lowercase_hex() {
        let owner = address!("0xAbCdEf0000000000000000000000000000000000");
        let mut stmt = SqlStatement::new(template());
        bind_common_owner_trade_filters(
            &mut stmt,
            owner,
            &[],
            &[],
            &TimeFilter::default(),
            START_TS_BODY,
            END_TS_BODY,
        )
        .unwrap();
        assert_eq!(stmt.params().len(), 1);
        assert_eq!(
            stmt.params()[0],
            SqlValue::Text("0xabcdef0000000000000000000000000000000000".to_string())
        );
    }

    // Kills removing chain_ids `.dedup()`: duplicated input must collapse to the
    // unique set, so exactly two U64 params (1, 137) are bound — not four.
    #[test]
    fn chain_ids_are_deduplicated_before_binding() {
        let stmt = bind(&[137, 1, 137, 1], &[], &TimeFilter::default()).unwrap();
        assert_eq!(u64_params(&stmt), vec![1, 137]);
        assert!(stmt.sql().contains("AND tws.chain_id IN (?2, ?3)"));
    }

    // Kills removing chain_ids `.sort_unstable()`: the bound list must be in
    // ascending order regardless of input order.
    #[test]
    fn chain_ids_are_sorted_ascending() {
        let stmt = bind(&[137, 1, 42], &[], &TimeFilter::default()).unwrap();
        assert_eq!(u64_params(&stmt), vec![1, 42, 137]);
    }

    // Kills removing raindex `.dedup()`: duplicate addresses collapse to one param.
    #[test]
    fn raindexes_are_deduplicated_before_binding() {
        let dup = address!("0x2f209e5b67a33b8fe96e28f24628df6da301c8eb");
        let stmt = bind(&[], &[dup, dup], &TimeFilter::default()).unwrap();
        let text_params: Vec<&str> = stmt
            .params()
            .iter()
            .filter_map(|p| match p {
                SqlValue::Text(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        // owner + one (deduplicated) raindex address.
        assert_eq!(
            text_params,
            vec![
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0x2f209e5b67a33b8fe96e28f24628df6da301c8eb",
            ]
        );
    }

    // Kills `start > end` -> `start >= end`: an equal-bound window [t, t] is a
    // valid (single-instant) filter and must NOT be rejected.
    #[test]
    fn equal_start_and_end_timestamps_are_accepted() {
        let stmt = bind(
            &[],
            &[],
            &TimeFilter {
                start: Some(1_000),
                end: Some(1_000),
            },
        )
        .unwrap();
        assert!(stmt.sql().contains("AND ts >="));
        assert!(stmt.sql().contains("AND ts <="));
        // Both timestamps bound as i64 params alongside the owner.
        assert_eq!(stmt.params().len(), 3);
        assert_eq!(stmt.params()[1], SqlValue::I64(1_000));
        assert_eq!(stmt.params()[2], SqlValue::I64(1_000));
    }

    // Kills weakening/removing the `start > end` guard: a strictly inverted
    // window must error.
    #[test]
    fn inverted_window_is_rejected() {
        let err = bind(
            &[],
            &[],
            &TimeFilter {
                start: Some(2_000),
                end: Some(1_000),
            },
        )
        .unwrap_err();
        assert_eq!(err, SqlBuildError::new("start_timestamp > end_timestamp"));
    }

    // Kills removing the start i64::try_from overflow guard: a start beyond
    // i64::MAX must produce the range error, not silently bind a wrong value.
    #[test]
    fn start_timestamp_overflowing_i64_errors() {
        let err = bind(
            &[],
            &[],
            &TimeFilter {
                start: Some(i64::MAX as u64 + 1),
                end: None,
            },
        )
        .unwrap_err();
        match err {
            SqlBuildError::Generic { message } => {
                assert!(
                    message.starts_with("start_timestamp out of range for i64"),
                    "unexpected message: {message}"
                );
            }
            other => panic!("expected Generic range error, got {other:?}"),
        }
    }

    // Kills removing the end i64::try_from overflow guard.
    #[test]
    fn end_timestamp_overflowing_i64_errors() {
        let err = bind(
            &[],
            &[],
            &TimeFilter {
                start: None,
                end: Some(u64::MAX),
            },
        )
        .unwrap_err();
        match err {
            SqlBuildError::Generic { message } => {
                assert!(
                    message.starts_with("end_timestamp out of range for i64"),
                    "unexpected message: {message}"
                );
            }
            other => panic!("expected Generic range error, got {other:?}"),
        }
    }

    // Kills swapping the start/end clause bodies and markers: each body must be
    // spliced at its own marker, and only the start bound present when end=None.
    #[test]
    fn only_start_bound_when_end_is_none() {
        let stmt = bind(
            &[],
            &[],
            &TimeFilter {
                start: Some(500),
                end: None,
            },
        )
        .unwrap();
        assert!(stmt.sql().contains("AND ts >= ?2"));
        assert!(!stmt.sql().contains("AND ts <="));
        assert!(!stmt.sql().contains(START_TS_CLAUSE));
        assert!(!stmt.sql().contains(END_TS_CLAUSE));
        assert_eq!(stmt.params().len(), 2);
        assert_eq!(stmt.params()[1], SqlValue::I64(500));
    }

    #[test]
    fn only_end_bound_when_start_is_none() {
        let stmt = bind(
            &[],
            &[],
            &TimeFilter {
                start: None,
                end: Some(900),
            },
        )
        .unwrap();
        assert!(stmt.sql().contains("AND ts <= ?2"));
        assert!(!stmt.sql().contains("AND ts >="));
        assert_eq!(stmt.params().len(), 2);
        assert_eq!(stmt.params()[1], SqlValue::I64(900));
    }

    // Kills removing either timestamp clause removal in the None/None case: with
    // no time filter, both markers are stripped and no timestamp params bound.
    #[test]
    fn no_timestamp_clauses_when_filter_empty() {
        let stmt = bind(&[], &[], &TimeFilter::default()).unwrap();
        assert!(!stmt.sql().contains("AND ts >="));
        assert!(!stmt.sql().contains("AND ts <="));
        assert!(!stmt.sql().contains(START_TS_CLAUSE));
        assert!(!stmt.sql().contains(END_TS_CLAUSE));
        assert_eq!(stmt.params().len(), 1);
    }

    // Kills removing the chain-id list-clause marker replacement: an empty
    // chain_ids list must strip the marker entirely (no dangling placeholder).
    #[test]
    fn empty_chain_ids_strips_clause() {
        let stmt = bind(&[], &[], &TimeFilter::default()).unwrap();
        assert!(!stmt.sql().contains(TAKE_ORDERS_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql().contains("tws.chain_id IN"));
    }
}
