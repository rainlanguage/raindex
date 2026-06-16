use crate::local_db::{
    query::{SqlStatement, SqlValue},
    RaindexIdentifier,
};
use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

/// Transaction info returned from local DB query.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbTransaction {
    pub transaction_hash: B256,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub sender: Address,
}

/// Builds a SQL statement to fetch transaction info by transaction hash
/// from the deposits and withdrawals tables.
pub fn build_fetch_transaction_by_hash_stmt(
    raindex_id: &RaindexIdentifier,
    tx_hash: B256,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        vec![
            SqlValue::from(raindex_id.chain_id),
            SqlValue::from(raindex_id.raindex_address),
            SqlValue::from(tx_hash),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, b256};

    #[test]
    fn builds_correct_sql_with_params() {
        let raindex = address!("0x1234567890123456789012345678901234567890");
        let tx_hash = b256!("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
        let raindex_id = RaindexIdentifier::new(1, raindex);

        let stmt = build_fetch_transaction_by_hash_stmt(&raindex_id, tx_hash);

        assert!(stmt.sql.contains("SELECT"));
        assert!(stmt.sql.contains("FROM deposits"));
        assert!(stmt.sql.contains("FROM withdrawals"));
        assert!(stmt.sql.contains("transaction_hash"));
        assert_eq!(stmt.params.len(), 3);
    }
}
