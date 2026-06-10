use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

pub const FETCH_ALL_TOKENS_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbToken {
    pub chain_id: u32,
    pub raindex_address: Address,
    pub token_address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Default)]
pub struct FetchAllTokensArgs {
    pub chain_ids: Vec<u32>,
    pub raindex_addresses: Vec<Address>,
}

const CHAIN_IDS_CLAUSE: &str = "/*CHAIN_IDS_CLAUSE*/";
const CHAIN_IDS_CLAUSE_BODY: &str = "AND chain_id IN ({list})";

const RAINDEXES_CLAUSE: &str = "/*RAINDEXES_CLAUSE*/";
const RAINDEXES_CLAUSE_BODY: &str = "AND raindex_address IN ({list})";

/// Builds the SQL statement used to load all unique ERC20 tokens from the local database.
pub fn build_fetch_all_tokens_stmt(
    args: &FetchAllTokensArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(FETCH_ALL_TOKENS_SQL);

    stmt.bind_list_clause(
        CHAIN_IDS_CLAUSE,
        CHAIN_IDS_CLAUSE_BODY,
        args.chain_ids.iter().cloned().map(SqlValue::from),
    )?;
    stmt.bind_list_clause(
        RAINDEXES_CLAUSE,
        RAINDEXES_CLAUSE_BODY,
        args.raindex_addresses.iter().cloned().map(SqlValue::from),
    )?;

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_stmt_with_no_filters() {
        let args = FetchAllTokensArgs::default();
        let stmt = build_fetch_all_tokens_stmt(&args).expect("should build");
        assert!(!stmt.sql.contains(CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(RAINDEXES_CLAUSE));
        assert!(stmt.params.is_empty());
    }

    #[test]
    fn builds_stmt_with_chain_ids() {
        let args = FetchAllTokensArgs {
            chain_ids: vec![1, 137],
            raindex_addresses: vec![],
        };
        let stmt = build_fetch_all_tokens_stmt(&args).expect("should build");
        assert!(stmt.sql.contains("chain_id IN"));
        assert_eq!(stmt.params.len(), 2);
    }

    #[test]
    fn builds_stmt_with_raindex_addresses() {
        let args = FetchAllTokensArgs {
            chain_ids: vec![],
            raindex_addresses: vec![Address::from([0xab; 20])],
        };
        let stmt = build_fetch_all_tokens_stmt(&args).expect("should build");
        assert!(stmt.sql.contains("raindex_address IN"));
        assert_eq!(stmt.params.len(), 1);
    }

    #[test]
    fn selects_columns_aliased_to_camel_case() {
        let stmt =
            build_fetch_all_tokens_stmt(&FetchAllTokensArgs::default()).expect("should build");
        assert!(stmt.sql.contains("AS chainId"));
        assert!(stmt.sql.contains("AS raindexAddress"));
        assert!(stmt.sql.contains("AS tokenAddress"));
    }

    #[test]
    fn deserializes_camel_case_db_row() {
        let json = r#"{
            "chainId": 42161,
            "raindexAddress": "0x0000000000000000000000000000000000000001",
            "tokenAddress": "0x0000000000000000000000000000000000000002",
            "name": "Token",
            "symbol": "TKN",
            "decimals": 18
        }"#;
        let token: LocalDbToken = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(token.chain_id, 42161);
        assert_eq!(
            token.raindex_address,
            "0x0000000000000000000000000000000000000001"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            token.token_address,
            "0x0000000000000000000000000000000000000002"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(token.name, "Token");
        assert_eq!(token.symbol, "TKN");
        assert_eq!(token.decimals, 18);
    }

    #[test]
    fn rejects_snake_case_db_row() {
        let json = r#"{
            "chain_id": 42161,
            "raindex_address": "0x0000000000000000000000000000000000000001",
            "token_address": "0x0000000000000000000000000000000000000002",
            "name": "Token",
            "symbol": "TKN",
            "decimals": 18
        }"#;
        let err = serde_json::from_str::<LocalDbToken>(json)
            .expect_err("snake_case keys must not deserialize");
        assert!(
            err.to_string().contains("missing field `chainId`"),
            "unexpected error: {err}"
        );
    }
}
