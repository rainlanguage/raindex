use crate::local_db::{
    query::{SqlBuildError, SqlStatement, SqlValue},
    RaindexIdentifier,
};
use crate::types::VaultBalanceChangeKind;
use alloy::primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");
const CHANGE_TYPES_CLAUSE: &str = "/*CHANGE_TYPES_CLAUSE*/";
const CHANGE_TYPES_CLAUSE_BODY: &str = "AND vbc.change_type IN ({list})";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbVaultBalanceChange {
    pub transaction_hash: B256,
    pub log_index: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub owner: Address,
    pub change_type: String,
    pub token: Address,
    pub vault_id: U256,
    pub delta: String,
    pub running_balance: String,
}

pub fn build_fetch_balance_changes_stmt(
    raindex_id: &RaindexIdentifier,
    vault_id: U256,
    token: Address,
    owner: Address,
    filter_kinds: Option<&[VaultBalanceChangeKind]>,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    stmt.push(SqlValue::from(raindex_id.chain_id));
    stmt.push(SqlValue::from(raindex_id.raindex_address));
    stmt.push(SqlValue::from(vault_id));
    stmt.push(SqlValue::from(token));
    stmt.push(SqlValue::from(owner));

    let change_type_strings: Vec<&'static str> = filter_kinds
        .map(|kinds| {
            kinds
                .iter()
                .flat_map(|k| k.to_local_db_change_types().iter().copied())
                .collect()
        })
        .unwrap_or_default();

    stmt.bind_list_clause(
        CHANGE_TYPES_CLAUSE,
        CHANGE_TYPES_CLAUSE_BODY,
        change_type_strings.into_iter().map(SqlValue::from),
    )?;

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::Address;

    use super::*;

    #[test]
    fn builds_with_params_no_filter() {
        let stmt = build_fetch_balance_changes_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            U256::from(1),
            Address::ZERO,
            Address::ZERO,
            None,
        )
        .unwrap();
        assert!(stmt.sql.contains("params AS"));
        assert!(stmt.sql.contains("?1 AS chain_id"));
        assert_eq!(stmt.params.len(), 5);
        assert!(!stmt.sql.contains("change_type IN"));
    }

    #[test]
    fn builds_with_single_filter_type() {
        let stmt = build_fetch_balance_changes_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            U256::from(1),
            Address::ZERO,
            Address::ZERO,
            Some(&[VaultBalanceChangeKind::Deposit]),
        )
        .unwrap();
        assert!(stmt.sql.contains("params AS"));
        assert!(stmt.sql.contains("?1 AS chain_id"));
        assert!(stmt.sql.contains("change_type IN (?6)"));
        assert_eq!(stmt.params.len(), 6);
    }

    #[test]
    fn builds_with_take_order_filter_expands_to_two_types() {
        let stmt = build_fetch_balance_changes_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            U256::from(1),
            Address::ZERO,
            Address::ZERO,
            Some(&[VaultBalanceChangeKind::TakeOrder]),
        )
        .unwrap();
        assert!(stmt.sql.contains("params AS"));
        assert!(stmt.sql.contains("change_type IN (?6, ?7)"));
        assert_eq!(stmt.params.len(), 7);
    }

    #[test]
    fn builds_with_clear_filter_expands_to_four_types() {
        let stmt = build_fetch_balance_changes_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            U256::from(1),
            Address::ZERO,
            Address::ZERO,
            Some(&[VaultBalanceChangeKind::Clear]),
        )
        .unwrap();
        assert!(stmt.sql.contains("params AS"));
        assert!(stmt.sql.contains("change_type IN (?6, ?7, ?8, ?9)"));
        assert_eq!(stmt.params.len(), 9);
    }

    #[test]
    fn builds_with_multiple_filter_types() {
        let stmt = build_fetch_balance_changes_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            U256::from(1),
            Address::ZERO,
            Address::ZERO,
            Some(&[
                VaultBalanceChangeKind::Deposit,
                VaultBalanceChangeKind::Withdrawal,
            ]),
        )
        .unwrap();
        assert!(stmt.sql.contains("params AS"));
        assert!(stmt.sql.contains("change_type IN (?6, ?7)"));
        assert_eq!(stmt.params.len(), 7);
    }

    #[test]
    fn builds_with_empty_filter_array() {
        let stmt = build_fetch_balance_changes_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            U256::from(1),
            Address::ZERO,
            Address::ZERO,
            Some(&[]),
        )
        .unwrap();
        assert!(stmt.sql.contains("params AS"));
        assert_eq!(stmt.params.len(), 5);
        assert!(!stmt.sql.contains("change_type IN"));
    }
}
