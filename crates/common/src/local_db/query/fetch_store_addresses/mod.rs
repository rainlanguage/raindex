use crate::local_db::{
    query::{SqlStatement, SqlValue},
    RaindexIdentifier,
};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

pub const FETCH_STORE_ADDRESSES_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreAddressRow {
    pub store_address: Address,
}

pub fn fetch_store_addresses_stmt(raindex_id: &RaindexIdentifier) -> SqlStatement {
    SqlStatement::new_with_params(
        FETCH_STORE_ADDRESSES_SQL,
        [
            SqlValue::from(raindex_id.chain_id),
            SqlValue::from(raindex_id.raindex_address),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stmt_binds_chain_and_raindex() {
        let stmt = fetch_store_addresses_stmt(&RaindexIdentifier::new(1, Address::ZERO));
        assert_eq!(stmt.sql, FETCH_STORE_ADDRESSES_SQL);
        assert_eq!(stmt.params.len(), 2);
        assert!(stmt
            .sql
            .to_lowercase()
            .contains("select distinct store_address"));
    }
}
