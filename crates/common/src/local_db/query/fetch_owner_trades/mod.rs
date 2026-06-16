use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    OrderbookIdentifier,
};
use crate::raindex_client::types::{PaginationParams, TimeFilter};
use alloy::primitives::Address;

const QUERY_TEMPLATE: &str = include_str!("query.sql");
const DEFAULT_PAGE_SIZE: u16 = 100;

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND tws.block_timestamp >= {param}\n";
const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND tws.block_timestamp <= {param}\n";

pub fn build_fetch_owner_trades_stmt(
    ob_id: &OrderbookIdentifier,
    owner: Address,
    pagination: &PaginationParams,
    time_filter: &TimeFilter,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(ob_id.chain_id));
    stmt.push(SqlValue::from(ob_id.orderbook_address));
    stmt.push(SqlValue::from(owner));

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
    stmt.bind_param_clause(START_TS_CLAUSE, START_TS_BODY, start_param)?;

    let end_param = if let Some(v) = time_filter.end {
        let i = i64::try_from(v).map_err(|e| {
            SqlBuildError::new(format!("end_timestamp out of range for i64: {} ({})", v, e))
        })?;
        Some(SqlValue::I64(i))
    } else {
        None
    };
    stmt.bind_param_clause(END_TS_CLAUSE, END_TS_BODY, end_param)?;

    let page_num = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.page_size.unwrap_or(DEFAULT_PAGE_SIZE) as i64;
    let offset = (page_num as i64 - 1) * limit;
    stmt.push(SqlValue::I64(limit));
    stmt.push(SqlValue::I64(offset));

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;

    #[test]
    fn builds_with_default_page() {
        let owner = Address::repeat_byte(0xaa);
        let stmt = build_fetch_owner_trades_stmt(
            &OrderbookIdentifier::new(42161, Address::ZERO),
            owner,
            &PaginationParams::default(),
            &TimeFilter::default(),
        )
        .unwrap();
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[0], SqlValue::U64(42161));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert!(
            matches!(&stmt.params[2], SqlValue::Text(s) if s.to_lowercase() == owner.to_string().to_lowercase())
        );
        assert_eq!(stmt.params[3], SqlValue::I64(100));
        assert_eq!(stmt.params[4], SqlValue::I64(0));
        assert!(!stmt.sql.contains("tws.block_timestamp >="));
        assert!(!stmt.sql.contains("tws.block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
    }

    #[test]
    fn builds_with_page_2() {
        let owner = Address::repeat_byte(0xbb);
        let stmt = build_fetch_owner_trades_stmt(
            &OrderbookIdentifier::new(1, Address::ZERO),
            owner,
            &PaginationParams {
                page: Some(2),
                ..Default::default()
            },
            &TimeFilter::default(),
        )
        .unwrap();
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[3], SqlValue::I64(100));
        assert_eq!(stmt.params[4], SqlValue::I64(100));
    }

    #[test]
    fn builds_with_custom_page_size() {
        let owner = Address::repeat_byte(0xcc);
        let stmt = build_fetch_owner_trades_stmt(
            &OrderbookIdentifier::new(42161, Address::ZERO),
            owner,
            &PaginationParams {
                page: Some(1),
                page_size: Some(50),
            },
            &TimeFilter::default(),
        )
        .unwrap();
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[3], SqlValue::I64(50));
        assert_eq!(stmt.params[4], SqlValue::I64(0));
    }

    #[test]
    fn builds_with_time_filters() {
        let owner = Address::repeat_byte(0xdd);
        let stmt = build_fetch_owner_trades_stmt(
            &OrderbookIdentifier::new(137, Address::ZERO),
            owner,
            &PaginationParams {
                page: Some(1),
                ..Default::default()
            },
            &TimeFilter {
                start: Some(1000),
                end: Some(2000),
            },
        )
        .unwrap();
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert!(stmt.sql.contains("tws.block_timestamp >="));
        assert!(stmt.sql.contains("tws.block_timestamp <="));
        assert_eq!(stmt.params.len(), 7);
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert!(
            matches!(&stmt.params[2], SqlValue::Text(s) if s.to_lowercase() == owner.to_string().to_lowercase())
        );
    }
}
