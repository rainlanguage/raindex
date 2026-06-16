use crate::remote::tokens::{RemoteTokensError, Token, Tokens};
use crate::yaml::context::Context;
use crate::yaml::{
    default_document, optional_vec, require_string, FieldErrorKind, YamlError, YamlParseableValue,
};
use crate::{NetworkCfg, TokenCfg};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RemoteTokensCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub urls: Vec<Url>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RemoteTokensCfg);

impl RemoteTokensCfg {
    pub fn validate_url(value: &str) -> Result<Url, ParseRemoteTokensError> {
        Url::parse(value).map_err(ParseRemoteTokensError::UrlParseError)
    }

    pub async fn fetch_tokens(
        networks: &HashMap<String, NetworkCfg>,
        remote_tokens: RemoteTokensCfg,
    ) -> Result<HashMap<String, TokenCfg>, ParseRemoteTokensError> {
        let mut tokens: HashMap<String, TokenCfg> = HashMap::new();

        for url in remote_tokens.urls {
            let tokens_res = reqwest::get(url.to_string())
                .await?
                .json::<Tokens>()
                .await?;

            let mut unique_tokens: HashMap<(u32, String), Token> = HashMap::new();
            for token in &tokens_res.tokens {
                let token_id = (token.chain_id, token.address.to_lowercase());
                unique_tokens.insert(token_id, token.clone());
            }

            for token in unique_tokens.values() {
                // skip tokens that error on parse as the list might include invalid records
                if let Ok(Some(token_cfg)) = token
                    .clone()
                    .try_into_token_cfg(networks, remote_tokens.document.clone())
                {
                    if tokens.contains_key(&token_cfg.key) {
                        return Err(ParseRemoteTokensError::ConflictingTokens(
                            token_cfg.key.clone(),
                        ));
                    }
                    tokens.insert(token_cfg.key.clone(), token_cfg);
                }
            }
        }

        Ok(tokens)
    }
}

impl YamlParseableValue for RemoteTokensCfg {
    fn parse_from_yaml(
        _: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<RemoteTokensCfg, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }

    fn parse_from_yaml_optional(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<Option<RemoteTokensCfg>, YamlError> {
        let mut validated_urls: Vec<Url> = Vec::new();
        let mut document_index: usize = 0;

        for (index, document) in documents.iter().enumerate() {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(urls) = optional_vec(&document_read, "using-tokens-from") {
                for url in urls {
                    let url = require_string(url, None, Some("using-tokens-from".to_string()))?;

                    let validated_url =
                        RemoteTokensCfg::validate_url(&url).map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "url".to_string(),
                                reason: e.to_string(),
                            },
                            location: "using-tokens-from".to_string(),
                        })?;

                    if validated_urls.contains(&validated_url) {
                        return Err(YamlError::KeyShadowing(
                            validated_url.to_string(),
                            "using-tokens-from".to_string(),
                        ));
                    }
                    validated_urls.push(validated_url);
                }

                document_index = index;
            }
        }

        if validated_urls.is_empty() {
            return Ok(None);
        }

        Ok(Some(RemoteTokensCfg {
            urls: validated_urls,
            document: documents[document_index].clone(),
        }))
    }
}

impl Default for RemoteTokensCfg {
    fn default() -> Self {
        RemoteTokensCfg {
            document: default_document(),
            urls: vec![Url::parse("http://example.com").unwrap()],
        }
    }
}
impl PartialEq for RemoteTokensCfg {
    fn eq(&self, other: &Self) -> bool {
        self.urls == other.urls
    }
}

#[derive(Error, Debug)]
pub enum ParseRemoteTokensError {
    #[error("Conflicting remote token in response, a token with key '{0}' already exists")]
    ConflictingTokens(String),
    #[error(transparent)]
    UrlParseError(ParseError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    RemoteTokensError(#[from] RemoteTokensError),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::primitives::Address;
    use httpmock::MockServer;

    use super::*;
    use crate::yaml::{tests::get_document, FieldErrorKind};

    #[test]
    fn test_parse_remote_tokens_from_yaml() {
        let yaml = r#"
using-tokens-from:
  - test
"#;
        let error =
            RemoteTokensCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "url".to_string(),
                    reason: "relative URL without a base".to_string(),
                },
                location: "using-tokens-from".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_remote_tokens_from_multiple_files() {
        let yaml_one = r#"
using-tokens-from:
  - http://test.com
"#;
        let yaml_two = r#"
using-tokens-from:
  - http://test.com
"#;
        let error = RemoteTokensCfg::parse_from_yaml_optional(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing(
                "http://test.com/".to_string(),
                "using-tokens-from".to_string()
            )
        );
    }

    #[tokio::test]
    async fn test_fetch_remote_tokens() {
        let server = MockServer::start_async().await;
        let yaml = format!(
            r#"
using-tokens-from:
  - {}
"#,
            server.base_url()
        );
        let remote_tokens =
            RemoteTokensCfg::parse_from_yaml_optional(vec![get_document(&yaml)], None)
                .unwrap()
                .unwrap();

        let response = r#"
{
    "name": "Remote",
    "timestamp": "2021-01-01T00:00:00.000Z",
    "keywords": [],
    "version": {
        "major": 1,
        "minor": 0,
        "patch": 0
    },
    "tokens": [
        {
            "chainId": 123,
            "address": "0x0000000000000000000000000000000000000001",
            "name": "Token1",
            "symbol": "T1",
            "decimals": 18,
            "logoURI": "https://example.com/token1-logo.png",
            "extensions": {"isStablecoin": true}
        },
        {
            "chainId": 234,
            "address": "0x0000000000000000000000000000000000000002",
            "name": "Token2",
            "symbol": "T2",
            "decimals": 18
        }
    ],
    "logoURI": "http://localhost.com"
}
        "#;
        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(response);
            })
            .await;

        let networks = HashMap::from([
            (
                "remote-network".to_string(),
                NetworkCfg {
                    document: default_document(),
                    key: "remote-network".to_string(),
                    rpcs: vec![Url::parse("http://localhost:8085/rpc-url").unwrap()],
                    chain_id: 123,
                    label: None,
                    network_id: None,
                    currency: None,
                },
            ),
            (
                "remote2-network".to_string(),
                NetworkCfg {
                    document: default_document(),
                    key: "remote2-network".to_string(),
                    rpcs: vec![Url::parse("http://localhost:8085/rpc-url").unwrap()],
                    chain_id: 234,
                    label: None,
                    network_id: None,
                    currency: None,
                },
            ),
        ]);
        let tokens = RemoteTokensCfg::fetch_tokens(&networks, remote_tokens)
            .await
            .unwrap();

        assert_eq!(tokens.len(), 2_usize);

        let token1_key = "remote-network-Token1-0x0000000000000000000000000000000000000001";
        let token = tokens.get(token1_key).unwrap();
        assert_eq!(token.key, token1_key);
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        assert_eq!(token.network.key, "remote-network");
        assert_eq!(token.network.chain_id, 123);
        assert_eq!(
            token.logo_uri,
            Some(Url::parse("https://example.com/token1-logo.png").unwrap())
        );
        let ext = token.extensions.as_ref().unwrap();
        assert_eq!(
            ext.get("isStablecoin"),
            Some(&serde_json::Value::Bool(true))
        );

        let token2_key = "remote2-network-Token2-0x0000000000000000000000000000000000000002";
        let token = tokens.get(token2_key).unwrap();
        assert_eq!(token.key, token2_key);
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(token.network.key, "remote2-network");
        assert_eq!(token.network.chain_id, 234);
        assert_eq!(token.logo_uri, None);
        assert_eq!(token.extensions, None);
    }

    #[tokio::test]
    async fn test_fetch_remote_tokens_from_multiple_urls() {
        let server = MockServer::start_async().await;

        let yaml = format!(
            r#"
using-tokens-from:
  - {}
  - {}/tokens2
"#,
            server.base_url(),
            server.base_url()
        );

        let remote_tokens =
            RemoteTokensCfg::parse_from_yaml_optional(vec![get_document(&yaml)], None)
                .unwrap()
                .unwrap();

        let response1 = r#"
{
    "name": "Remote1",
    "timestamp": "2021-01-01T00:00:00.000Z",
    "keywords": [],
    "version": {
        "major": 1,
        "minor": 0,
        "patch": 0
    },
    "tokens": [
        {
            "chainId": 123,
            "address": "0x0000000000000000000000000000000000000001",
            "name": "Token1",
            "symbol": "T1",
            "decimals": 18,
            "logoURI": "https://example.com/t1.png"
        },
        {
            "chainId": 234,
            "address": "0x0000000000000000000000000000000000000002",
            "name": "Token2",
            "symbol": "T2",
            "decimals": 18,
            "logoURI": "https://example.com/t2.png"
        }
    ],
    "logoURI": "http://localhost.com"
}
        "#;

        let response2 = r#"
{
    "name": "Remote2",
    "timestamp": "2021-01-01T00:00:00.000Z",
    "keywords": [],
    "version": {
        "major": 1,
        "minor": 0,
        "patch": 0
    },
    "tokens": [
        {
            "chainId": 345,
            "address": "0x0000000000000000000000000000000000000003",
            "name": "Token3",
            "symbol": "T3",
            "decimals": 18,
            "logoURI": "https://example.com/t3.png"
        },
        {
            "chainId": 456,
            "address": "0x0000000000000000000000000000000000000004",
            "name": "Token4",
            "symbol": "T4",
            "decimals": 18
        }
    ],
    "logoURI": "http://localhost.com"
}
        "#;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(response1);
            })
            .await;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/tokens2");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(response2);
            })
            .await;

        let networks = HashMap::from([
            (
                "remote-network".to_string(),
                NetworkCfg {
                    document: default_document(),
                    key: "remote-network".to_string(),
                    rpcs: vec![Url::parse("http://localhost:8085/rpc-url").unwrap()],
                    chain_id: 123,
                    label: None,
                    network_id: None,
                    currency: None,
                },
            ),
            (
                "remote2-network".to_string(),
                NetworkCfg {
                    document: default_document(),
                    key: "remote2-network".to_string(),
                    rpcs: vec![Url::parse("http://localhost:8085/rpc-url").unwrap()],
                    chain_id: 234,
                    label: None,
                    network_id: None,
                    currency: None,
                },
            ),
            (
                "remote3-network".to_string(),
                NetworkCfg {
                    document: default_document(),
                    key: "remote3-network".to_string(),
                    rpcs: vec![Url::parse("http://localhost:8085/rpc-url").unwrap()],
                    chain_id: 345,
                    label: None,
                    network_id: None,
                    currency: None,
                },
            ),
            (
                "remote4-network".to_string(),
                NetworkCfg {
                    document: default_document(),
                    key: "remote4-network".to_string(),
                    rpcs: vec![Url::parse("http://localhost:8085/rpc-url").unwrap()],
                    chain_id: 456,
                    label: None,
                    network_id: None,
                    currency: None,
                },
            ),
        ]);

        let tokens = RemoteTokensCfg::fetch_tokens(&networks, remote_tokens)
            .await
            .unwrap();

        assert_eq!(tokens.len(), 4_usize);

        let token3_key = "remote3-network-Token3-0x0000000000000000000000000000000000000003";
        let token = tokens.get(token3_key).unwrap();
        assert_eq!(token.key, token3_key);
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000003").unwrap()
        );
        assert_eq!(token.network.key, "remote3-network");
        assert_eq!(token.network.chain_id, 345);
        assert_eq!(
            token.logo_uri,
            Some(Url::parse("https://example.com/t3.png").unwrap())
        );

        let token4_key = "remote4-network-Token4-0x0000000000000000000000000000000000000004";
        let token = tokens.get(token4_key).unwrap();
        assert_eq!(token.key, token4_key);
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000004").unwrap()
        );
        assert_eq!(token.network.key, "remote4-network");
        assert_eq!(token.network.chain_id, 456);
        assert_eq!(token.logo_uri, None);
    }
}
