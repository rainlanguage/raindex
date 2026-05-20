use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    RaindexIdentifier,
};
use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbVaultVolume {
    pub vault_id: U256,
    pub token: Address,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<u8>,
    pub total_in: String,
    pub total_out: String,
}

const START_TS_CLAUSE: &str = "/*START_TS_CLAUSE*/";
const START_TS_BODY: &str = "\nAND block_timestamp >= {param}\n";

const END_TS_CLAUSE: &str = "/*END_TS_CLAUSE*/";
const END_TS_BODY: &str = "\nAND block_timestamp <= {param}\n";

pub fn build_fetch_order_vaults_volume_stmt(
    raindex_id: &RaindexIdentifier,
    order_hash: B256,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);
    stmt.push(SqlValue::from(raindex_id.chain_id));
    stmt.push(SqlValue::from(raindex_id.raindex_address));
    stmt.push(SqlValue::from(order_hash));

    let start_param = if let Some(v) = start_timestamp {
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

    let end_param = if let Some(v) = end_timestamp {
        let i = i64::try_from(v).map_err(|e| {
            SqlBuildError::new(format!("end_timestamp out of range for i64: {} ({})", v, e))
        })?;
        Some(SqlValue::I64(i))
    } else {
        None
    };
    stmt.bind_param_clause(END_TS_CLAUSE, END_TS_BODY, end_param)?;

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex,
        primitives::{b256, Address},
    };

    #[test]
    fn builds_with_chain_id_and_filters() {
        let order_hash =
            b256!("0x00000000000000000000000000000000000000000000000000000000deadface");
        let stmt = build_fetch_order_vaults_volume_stmt(
            &RaindexIdentifier::new(137, Address::ZERO),
            order_hash,
            Some(11),
            Some(22),
        )
        .unwrap();
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert!(stmt.sql.contains("block_timestamp >="));
        assert!(stmt.sql.contains("block_timestamp <="));
        assert_eq!(stmt.params.len(), 5);
        assert_eq!(stmt.params[0], SqlValue::U64(137));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(
            stmt.params[2],
            SqlValue::Text(hex::encode_prefixed(order_hash))
        );
    }

    #[test]
    fn builds_without_time_filters_when_none() {
        let order_hash =
            b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef");
        let stmt = build_fetch_order_vaults_volume_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            order_hash,
            None,
            None,
        )
        .unwrap();
        assert!(!stmt.sql.contains("block_timestamp >="));
        assert!(!stmt.sql.contains("block_timestamp <="));
        assert!(!stmt.sql.contains(START_TS_CLAUSE));
        assert!(!stmt.sql.contains(END_TS_CLAUSE));
        assert_eq!(stmt.params.len(), 3);
        assert_eq!(stmt.params[0], SqlValue::U64(1));
        assert_eq!(stmt.params[1], SqlValue::Text(Address::ZERO.to_string()));
        assert_eq!(
            stmt.params[2],
            SqlValue::Text(hex::encode_prefixed(order_hash))
        );
    }
}
