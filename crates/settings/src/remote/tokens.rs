use crate::{NetworkCfg, TokenCfg};
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::Url;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub chain_id: u32,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u64,
    #[serde(default, rename = "logoURI")]
    pub logo_uri: Option<Url>,
    #[serde(default)]
    pub extensions: Option<HashMap<String, Value>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub name: String,
    pub timestamp: String,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,
    pub version: Version,
    pub tokens: Vec<Token>,
    #[serde(default, rename = "logoURI")]
    pub logo_uri: Option<String>,
}

impl Token {
    pub fn try_into_token_cfg(
        self,
        networks: &HashMap<String, NetworkCfg>,
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<Option<TokenCfg>, RemoteTokensError> {
        match networks
            .values()
            .find(|network| network.chain_id == self.chain_id)
        {
            Some(network) => {
                let token_cfg = TokenCfg {
                    document: document.clone(),
                    key: format!(
                        "{}-{}-{}",
                        network.key,
                        self.name.replace(' ', "-").clone(),
                        self.address.to_lowercase()
                    ),
                    network: Arc::new(network.clone()),
                    address: Address::from_str(&self.address)
                        .map_err(|e| RemoteTokensError::ParseTokenAddressError(e.to_string()))?,
                    decimals: Some(self.decimals as u8),
                    label: Some(self.name.clone()),
                    symbol: Some(self.symbol),
                    logo_uri: self.logo_uri,
                    extensions: self.extensions,
                };
                Ok(Some(token_cfg))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Error)]
pub enum RemoteTokensError {
    #[error("Failed to parse token address: {0}")]
    ParseTokenAddressError(String),
    #[error("Failed to parse network: {0}")]
    ParseNetworkError(String),
    #[error("Network not found for chain_id: {0}")]
    NetworkNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::default_document;
    use serde_json::json;

    fn test_networks() -> HashMap<String, NetworkCfg> {
        HashMap::from([(
            "test-net".to_string(),
            NetworkCfg {
                document: default_document(),
                key: "test-net".to_string(),
                rpcs: vec![Url::parse("http://localhost:8545").unwrap()],
                chain_id: 1,
                label: None,
                network_id: None,
                currency: None,
            },
        )])
    }

    #[test]
    fn test_try_into_token_cfg_preserves_extensions() {
        let mut ext = HashMap::new();
        ext.insert("isStablecoin".to_string(), json!(true));
        ext.insert("coingeckoId".to_string(), json!("usd-coin"));

        let token = Token {
            chain_id: 1,
            address: "0x0000000000000000000000000000000000000001".to_string(),
            name: "USD Coin".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
            logo_uri: None,
            extensions: Some(ext.clone()),
        };

        let networks = test_networks();
        let doc = default_document();
        let result = token.try_into_token_cfg(&networks, doc).unwrap().unwrap();

        let result_ext = result.extensions.unwrap();
        assert_eq!(result_ext.get("isStablecoin"), Some(&json!(true)));
        assert_eq!(result_ext.get("coingeckoId"), Some(&json!("usd-coin")));
    }

    #[test]
    fn test_try_into_token_cfg_none_extensions() {
        let token = Token {
            chain_id: 1,
            address: "0x0000000000000000000000000000000000000001".to_string(),
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            decimals: 18,
            logo_uri: None,
            extensions: None,
        };

        let networks = test_networks();
        let doc = default_document();
        let result = token.try_into_token_cfg(&networks, doc).unwrap().unwrap();

        assert_eq!(result.extensions, None);
    }

    #[test]
    fn test_try_into_token_cfg_no_matching_network() {
        let token = Token {
            chain_id: 999,
            address: "0x0000000000000000000000000000000000000001".to_string(),
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            decimals: 18,
            logo_uri: None,
            extensions: Some(HashMap::from([("key".to_string(), json!("value"))])),
        };

        let networks = test_networks();
        let doc = default_document();
        let result = token.try_into_token_cfg(&networks, doc).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_deserialize_token_with_extensions() {
        let json_str = r#"{
            "chainId": 1,
            "address": "0x0000000000000000000000000000000000000001",
            "name": "USD Coin",
            "symbol": "USDC",
            "decimals": 6,
            "extensions": {
                "isStablecoin": true,
                "bridgeInfo": {"10": {"tokenAddress": "0xabc"}}
            }
        }"#;

        let token: Token = serde_json::from_str(json_str).unwrap();
        let ext = token.extensions.unwrap();
        assert_eq!(ext.get("isStablecoin"), Some(&json!(true)));
        assert!(ext.get("bridgeInfo").unwrap().is_object());
    }

    #[test]
    fn test_deserialize_token_without_extensions() {
        let json_str = r#"{
            "chainId": 1,
            "address": "0x0000000000000000000000000000000000000001",
            "name": "Token",
            "symbol": "TKN",
            "decimals": 18
        }"#;

        let token: Token = serde_json::from_str(json_str).unwrap();
        assert_eq!(token.extensions, None);
    }
}
