use crate::csv::TryIntoCsv;
use rain_math_float::Float;
use raindex_subgraph_client::types::common::*;
use serde::{Deserialize, Serialize};

use super::FlattenError;

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenVaultFlattened {
    pub id: String,
    pub owner: SgBytes,
    pub vault_id: SgBytes,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<SgBigInt>,
    pub token_address: String,
    pub balance_display: String,
    pub balance: SgBytes,
}

impl TryFrom<SgVault> for TokenVaultFlattened {
    type Error = FlattenError;

    fn try_from(val: SgVault) -> Result<Self, Self::Error> {
        let balance = Float::from_hex(&val.balance.0)?;
        let balance_display = balance.format()?;

        Ok(Self {
            id: val.id.0,
            owner: val.owner,
            vault_id: val.vault_id,
            token_name: val.token.name,
            token_symbol: val.token.symbol,
            token_decimals: val.token.decimals,
            token_address: val.token.address.0,
            balance_display,
            balance: val.balance,
        })
    }
}

impl TryIntoCsv<TokenVaultFlattened> for Vec<TokenVaultFlattened> {}

#[cfg(test)]
mod tests {
    use super::*;
    use raindex_subgraph_client::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrderAsIO, SgRaindex, SgVault, SgVaultBalanceChangeType,
    };
    use raindex_subgraph_client::utils::float::*;

    #[allow(clippy::too_many_arguments)]
    fn create_sg_vault(
        id: &str,
        owner: &str,
        vault_id_str: &str,
        balance_str: &str,
        token_id: &str,
        token_address: &str,
        token_name: Option<&str>,
        token_symbol: Option<&str>,
        token_decimals_str: Option<&str>,
    ) -> SgVault {
        SgVault {
            id: SgBytes(id.into()),
            owner: SgBytes(owner.into()),
            vault_id: SgBytes(vault_id_str.into()),
            balance: SgBytes(balance_str.into()),
            token: SgErc20 {
                id: SgBytes(token_id.into()),
                address: SgBytes(token_address.into()),
                name: token_name.map(String::from),
                symbol: token_symbol.map(String::from),
                decimals: token_decimals_str.map(|s| SgBigInt(s.into())),
            },
            raindex: SgRaindex {
                id: SgBytes("default_raindex_id".into()),
            },
            orders_as_output: Vec::<SgOrderAsIO>::new(),
            orders_as_input: Vec::<SgOrderAsIO>::new(),
            balance_changes: Vec::<SgVaultBalanceChangeType>::new(),
        }
    }

    #[test]
    fn test_normal_case_all_fields_present() {
        let balance = Float::parse("123.45".to_string()).unwrap();
        let balance_str = balance.as_hex();

        let sg_vault = create_sg_vault(
            "vault_test_001",
            "0xOwnerAddress1",
            "1001",
            &balance_str,
            "token_test_TKA",
            "0xTokenAddressA",
            Some("Test Token A"),
            Some("TKA"),
            Some("18"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.id, "vault_test_001");
        assert_eq!(flattened.owner, SgBytes("0xOwnerAddress1".into()));
        assert_eq!(flattened.vault_id, SgBytes("1001".into()));
        assert_eq!(flattened.token_name, Some("Test Token A".into()));
        assert_eq!(flattened.token_symbol, Some("TKA".into()));
        assert_eq!(flattened.token_decimals, Some(SgBigInt("18".into())));
        assert_eq!(flattened.token_address, "0xTokenAddressA");
        assert_eq!(flattened.balance_display, "123.45");
        assert_eq!(flattened.balance, SgBytes(balance_str.to_string()));
    }

    #[test]
    fn test_normal_case_optional_token_fields_none() {
        let balance = Float::parse("7890".to_string()).unwrap();
        let balance_str = balance.as_hex();

        let sg_vault = create_sg_vault(
            "vault_test_002",
            "0xOwnerAddress2",
            "1002",
            &balance_str,
            "token_test_TKB",
            "0xTokenAddressB",
            None,
            None,
            None,
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();

        assert_eq!(flattened.token_name, None);
        assert_eq!(flattened.token_symbol, None);
        assert_eq!(flattened.token_decimals, None);
        assert_eq!(flattened.balance_display, "7890");
        assert_eq!(flattened.balance, SgBytes(balance_str.to_string()));
    }

    #[test]
    fn test_edge_case_zero_balance() {
        let balance_str = F0.as_hex();

        let sg_vault = create_sg_vault(
            "vault_test_003",
            "0xOwnerAddress3",
            "1003",
            &balance_str,
            "token_test_TKC",
            "0xTokenAddressC",
            Some("Test Token C"),
            Some("TKC"),
            Some("18"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_ok());
        let flattened = result.unwrap();
        assert_eq!(flattened.balance_display, "0");
        assert_eq!(flattened.balance, SgBytes(balance_str.to_string()));
    }

    #[test]
    fn test_edge_case_zero_decimals_specified() {
        let balance = Float::parse("98765".to_string()).unwrap();
        let balance_str = balance.as_hex();

        let sg_vault = create_sg_vault(
            "vault_test_004",
            "0xOwnerAddress4",
            "1004",
            &balance_str,
            "token_test_TKD",
            "0xTokenAddressD",
            Some("Test Token D"),
            Some("TKD"),
            Some("0"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        let flattened = result.unwrap();

        assert_eq!(flattened.balance_display, "98765");
        assert_eq!(flattened.balance, SgBytes(balance_str.to_string()));
    }

    #[test]
    fn test_error_case_invalid_balance_non_numeric() {
        let sg_vault = create_sg_vault(
            "vault_test_006",
            "0xOwnerAddress6",
            "1006",
            "not-a-number",
            "token_test_TKF",
            "0xTokenAddressF",
            None,
            None,
            Some("18"),
        );
        let result = TokenVaultFlattened::try_from(sg_vault);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_case_19_decimals() {
        let balance = Float::parse("98765.0000000000000000001".to_string()).unwrap();
        let balance_str = balance.as_hex();

        let sg_vault = create_sg_vault(
            "vault_test_004",
            "0xOwnerAddress4",
            "1004",
            &balance_str,
            "token_test_TKD",
            "0xTokenAddressD",
            Some("Test Token D"),
            Some("TKD"),
            Some("0"),
        );

        let result = TokenVaultFlattened::try_from(sg_vault);
        let flattened = result.unwrap();

        assert_eq!(flattened.balance_display, "98765.0000000000000000001");
        assert_eq!(flattened.balance, SgBytes(balance_str.to_string()));
    }
}
