use crate::yaml::{
    default_document, optional_string, require_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};
use crate::*;
use alloy::primitives::{hex::FromHexError, Address};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use strict_yaml_rust::strict_yaml::Hash;
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::Url;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::context::Context;

const ALLOWED_TOKEN_KEYS: [&str; 5] = ["address", "decimals", "label", "network", "symbol"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TokenCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub network: Arc<NetworkCfg>,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub decimals: Option<u8>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub symbol: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional, type = "string"))]
    pub logo_uri: Option<Url>,
    #[cfg_attr(target_family = "wasm", tsify(optional, type = "Map<string, any>"))]
    pub extensions: Option<HashMap<String, Value>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TokenCfg);

impl TokenCfg {
    pub fn validate_address(value: &str) -> Result<Address, ParseTokenConfigSourceError> {
        Address::from_str(value).map_err(ParseTokenConfigSourceError::AddressParseError)
    }
    pub fn validate_decimals(value: &str) -> Result<u8, ParseTokenConfigSourceError> {
        value
            .parse::<u8>()
            .map_err(ParseTokenConfigSourceError::DecimalsParseError)
    }

    pub fn update_address(&mut self, address: &str) -> Result<Self, YamlError> {
        let address = TokenCfg::validate_address(address)?;

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut tokens)) =
                document_hash.get_mut(&StrictYaml::String("tokens".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut token)) =
                    tokens.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    token[&StrictYaml::String("address".to_string())] =
                        StrictYaml::String(address.to_string());
                    self.address = address;
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "tokens".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("tokens".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(self.clone())
    }

    pub fn add_record_to_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        key: &str,
        network_key: &str,
        address: &str,
        decimals: Option<&str>,
        label: Option<&str>,
        symbol: Option<&str>,
    ) -> Result<(), YamlError> {
        if TokenCfg::parse_from_yaml(documents.clone(), key, None).is_ok() {
            return Err(YamlError::KeyShadowing(
                key.to_string(),
                "tokens".to_string(),
            ));
        }

        let address = TokenCfg::validate_address(address)?;
        let decimals = decimals.map(TokenCfg::validate_decimals).transpose()?;
        NetworkCfg::parse_from_yaml(documents.clone(), network_key, None)?;

        let mut document = documents[0]
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if !document_hash.contains_key(&StrictYaml::String("tokens".to_string()))
                || document_hash
                    .get_mut(&StrictYaml::String("tokens".to_string()))
                    .is_none()
            {
                document_hash.insert(
                    StrictYaml::String("tokens".to_string()),
                    StrictYaml::Hash(Hash::new()),
                );
            }

            if let Some(StrictYaml::Hash(ref mut tokens)) =
                document_hash.get_mut(&StrictYaml::String("tokens".to_string()))
            {
                if tokens.contains_key(&StrictYaml::String(key.to_string())) {
                    return Err(YamlError::KeyShadowing(
                        key.to_string(),
                        "tokens".to_string(),
                    ));
                }

                let mut token_hash = Hash::new();
                token_hash.insert(
                    StrictYaml::String("network".to_string()),
                    StrictYaml::String(network_key.to_string()),
                );
                token_hash.insert(
                    StrictYaml::String("address".to_string()),
                    StrictYaml::String(address.to_string()),
                );
                if let Some(decimals) = decimals {
                    token_hash.insert(
                        StrictYaml::String("decimals".to_string()),
                        StrictYaml::String(decimals.to_string()),
                    );
                }
                if let Some(label) = label {
                    token_hash.insert(
                        StrictYaml::String("label".to_string()),
                        StrictYaml::String(label.to_string()),
                    );
                }
                if let Some(symbol) = symbol {
                    token_hash.insert(
                        StrictYaml::String("symbol".to_string()),
                        StrictYaml::String(symbol.to_string()),
                    );
                }

                tokens.insert(
                    StrictYaml::String(key.to_string()),
                    StrictYaml::Hash(token_hash),
                );
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("tokens".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(())
    }

    pub fn remove_record_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        key: &str,
    ) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;

            if let StrictYaml::Hash(ref mut document_hash) = *document_write {
                if let Some(StrictYaml::Hash(ref mut tokens)) =
                    document_hash.get_mut(&StrictYaml::String("tokens".to_string()))
                {
                    if tokens.contains_key(&StrictYaml::String(key.to_string())) {
                        tokens.remove(&StrictYaml::String(key.to_string()));
                        return Ok(());
                    }
                }
            }
        }

        Err(YamlError::KeyNotFound(key.to_string()))
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        token_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(tokens_hash) = require_hash(&document_read, Some("tokens"), None) {
                if let Some(token_yaml) =
                    tokens_hash.get(&StrictYaml::String(token_key.to_string()))
                {
                    let location = format!("token '{}'", token_key);
                    return require_string(token_yaml, Some("network"), Some(location));
                }
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("network for token '{}'", token_key)),
            location: "root".to_string(),
        })
    }

    fn sanitize_tokens_hash(tokens_hash: &Hash) -> Hash {
        let mut sanitized_tokens: Vec<(String, StrictYaml)> = Vec::new();

        for (token_key, token_value) in tokens_hash {
            let Some(token_key_str) = token_key.as_str() else {
                continue;
            };

            let StrictYaml::Hash(ref token_hash) = *token_value else {
                continue;
            };

            let mut sanitized_token = Hash::new();
            for allowed_key in ALLOWED_TOKEN_KEYS.iter() {
                let key_yaml = StrictYaml::String(allowed_key.to_string());
                if let Some(v) = token_hash.get(&key_yaml) {
                    sanitized_token.insert(key_yaml, v.clone());
                }
            }

            sanitized_tokens.push((token_key_str.to_string(), StrictYaml::Hash(sanitized_token)));
        }

        sanitized_tokens.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut new_tokens_hash = Hash::new();
        for (key, value) in sanitized_tokens {
            new_tokens_hash.insert(StrictYaml::String(key), value);
        }

        new_tokens_hash
    }
}

impl YamlParsableHash for TokenCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut tokens = HashMap::new();

        let networks = NetworkCfg::parse_all_from_yaml(documents.clone(), context)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(tokens_hash) = require_hash(&document_read, Some("tokens"), None) {
                for (key_yaml, token_yaml) in tokens_hash {
                    let token_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("token '{}'", token_key);

                    let network_key =
                        require_string(token_yaml, Some("network"), Some(location.clone()))?;
                    let network = networks.get(&network_key).ok_or_else(|| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "network".to_string(),
                            reason: format!("Network '{}' not found", network_key),
                        },
                        location: location.clone(),
                    })?;

                    let address_str =
                        require_string(token_yaml, Some("address"), Some(location.clone()))?;
                    let address =
                        TokenCfg::validate_address(&address_str).map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let decimals = optional_string(token_yaml, "decimals")
                        .map(|d| TokenCfg::validate_decimals(&d))
                        .transpose()
                        .map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "decimals".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let label = optional_string(token_yaml, "label");
                    let symbol = optional_string(token_yaml, "symbol");
                    let logo_uri = optional_string(token_yaml, "logo-uri")
                        .map(|s| Url::parse(&s))
                        .transpose()
                        .map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "logo-uri".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let token = TokenCfg {
                        document: document.clone(),
                        key: token_key.clone(),
                        network: Arc::new(network.clone()),
                        address,
                        decimals,
                        label,
                        symbol,
                        logo_uri,
                        extensions: None,
                    };

                    if tokens.contains_key(&token_key) {
                        return Err(YamlError::KeyShadowing(token_key, "tokens".to_string()));
                    }
                    tokens.insert(token_key, token);
                }
            }
        }

        if let Some(context) = context {
            if let Some(yaml_cache) = &context.yaml_cache {
                for (key, token) in &yaml_cache.remote_tokens {
                    if tokens.contains_key(key) {
                        return Err(YamlError::ParseTokenConfigSourceError(
                            ParseTokenConfigSourceError::RemoteTokenKeyShadowing(key.clone()),
                        ));
                    }
                    tokens.insert(key.clone(), token.clone());
                }
            }
        }

        if tokens.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("tokens".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(tokens)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let tokens_key = StrictYaml::String("tokens".to_string());
            let Some(tokens_value) = root_hash.get(&tokens_key) else {
                continue;
            };
            let StrictYaml::Hash(ref tokens_hash) = *tokens_value else {
                continue;
            };

            let sanitized = Self::sanitize_tokens_hash(tokens_hash);
            root_hash.insert(tokens_key, StrictYaml::Hash(sanitized));
        }

        Ok(())
    }
}

impl Default for TokenCfg {
    fn default() -> Self {
        TokenCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            network: Arc::new(NetworkCfg::dummy()),
            address: Address::ZERO,
            decimals: None,
            label: None,
            symbol: None,
            logo_uri: None,
            extensions: None,
        }
    }
}
impl PartialEq for TokenCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.network == other.network
            && self.address == other.address
            && self.decimals == other.decimals
            && self.label == other.label
            && self.symbol == other.symbol
            && self.logo_uri == other.logo_uri
            && self.extensions == other.extensions
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseTokenConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Failed to parse decimals")]
    DecimalsParseError(std::num::ParseIntError),
    #[error("Network not found for token: {0}")]
    NetworkNotFoundError(String),
    #[error("Remote token key shadowing: {0}")]
    RemoteTokenKeyShadowing(String),
}

impl ParseTokenConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseTokenConfigSourceError::AddressParseError(err) =>
                format!("The token address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseTokenConfigSourceError::DecimalsParseError(err) =>
                format!("The token decimals in your YAML configuration must be a valid number between 0 and 255: {}", err),
            ParseTokenConfigSourceError::NetworkNotFoundError(network) =>
                format!("The network '{}' specified for this token was not found in your YAML configuration. Please define this network or use an existing one.", network),
            ParseTokenConfigSourceError::RemoteTokenKeyShadowing(key) =>
                format!("The remote token key '{}' is already defined in token configuration", key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use yaml::tests::get_document;

    #[test]
    fn test_parse_tokens_errors() {
        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
test: test
"#,
            )],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'networks' in root"
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
test: test
"#,
            )],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("tokens".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'tokens' in root"
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        address: "0x1234567890123456789012345678901234567890"
"#,
            )],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("network".to_string()),
                location: "token 'token1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'network' in token 'token1'"
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "nonexistent"
        address: "0x1234567890123456789012345678901234567890"
"#,
            )],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "network".to_string(),
                    reason: "Network 'nonexistent' not found".to_string(),
                },
                location: "token 'token1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Invalid value for field 'network' in token 'token1': Network 'nonexistent' not found"
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "mainnet"
"#,
            )],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("address".to_string()),
                location: "token 'token1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'address' in token 'token1'"
        );

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "mainnet"
        address: "not_a_valid_address"
"#,
            )],
            None,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { .. },
                location: _
            }
        ));
        assert!(error
            .to_readable_msg()
            .contains("Invalid value for field 'address' in token 'token1'"));

        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(
                r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: "mainnet"
        address: "0x1234567890123456789012345678901234567890"
        decimals: "not_a_number"
"#,
            )],
            None,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { .. },
                location: _
            }
        ));
        assert!(error
            .to_readable_msg()
            .contains("Invalid value for field 'decimals' in token 'token1'"));
    }

    #[test]
    fn test_parse_tokens_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    dai:
        network: mainnet
        address: "0x6b175474e89094c44da98b954eedeac495271d0f"
        decimals: "18"
        logo-uri: "https://example.com/dai-logo.png"
    usdc:
        network: mainnet
        address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        decimals: "6"
"#;
        let yaml_two = r#"
tokens:
    weth:
        network: mainnet
        address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
        decimals: "18"
        logo-uri: "https://example.com/weth-logo.png"
    usdt:
        network: mainnet
        address: "0xdac17f958d2ee523a2206206994597c13d831ec7"
        decimals: "6"
"#;
        let tokens = TokenCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(
            tokens.get("dai").unwrap().address,
            Address::from_str("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap()
        );
        assert_eq!(tokens.get("dai").unwrap().decimals, Some(18));
        assert_eq!(
            tokens.get("dai").unwrap().logo_uri,
            Some(Url::parse("https://example.com/dai-logo.png").unwrap())
        );
        assert_eq!(
            tokens.get("usdc").unwrap().address,
            Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap()
        );
        assert_eq!(tokens.get("usdc").unwrap().decimals, Some(6));
        assert_eq!(tokens.get("usdc").unwrap().logo_uri, None);
        assert_eq!(
            tokens.get("weth").unwrap().address,
            Address::from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap()
        );
        assert_eq!(tokens.get("weth").unwrap().decimals, Some(18));
        assert_eq!(
            tokens.get("weth").unwrap().logo_uri,
            Some(Url::parse("https://example.com/weth-logo.png").unwrap())
        );
        assert_eq!(
            tokens.get("usdt").unwrap().address,
            Address::from_str("0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap()
        );
        assert_eq!(tokens.get("usdt").unwrap().decimals, Some(6));
        assert_eq!(tokens.get("usdt").unwrap().logo_uri, None);
    }

    #[test]
    fn test_parse_tokens_invalid_logo_uri() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    token1:
        network: mainnet
        address: "0x1234567890123456789012345678901234567890"
        logo-uri: "not_a_valid_url"
"#;
        let error = TokenCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert!(matches!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { .. },
                location: _
            }
        ));
        assert!(error
            .to_readable_msg()
            .contains("Invalid value for field 'logo-uri' in token 'token1'"));
    }

    #[test]
    fn test_parse_tokens_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    dai:
        network: mainnet
        address: "0x6b175474e89094c44da98b954eedeac495271d0f"
    usdc:
        network: mainnet
        address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
"#;
        let yaml_two = r#"
tokens:
    dai:
        network: mainnet
        address: "0x6b175474e89094c44da98b954eedeac495271d0f"
"#;
        let error = TokenCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing("dai".to_string(), "tokens".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'dai' is defined multiple times in your YAML configuration at tokens"
        );
    }

    #[test]
    fn test_sanitize_drops_unknown_keys() {
        let yaml = r#"
tokens:
    dai:
        network: mainnet
        address: 0x6b175474e89094c44da98b954eedeac495271d0f
        decimals: 18
        unknown-key: should-be-removed
        another-unknown: also-removed
"#;
        let doc = get_document(yaml);
        TokenCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let tokens = root
            .get(&StrictYaml::String("tokens".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let dai = tokens
            .get(&StrictYaml::String("dai".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(dai.contains_key(&StrictYaml::String("network".to_string())));
        assert!(dai.contains_key(&StrictYaml::String("address".to_string())));
        assert!(dai.contains_key(&StrictYaml::String("decimals".to_string())));
        assert!(!dai.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!dai.contains_key(&StrictYaml::String("another-unknown".to_string())));
    }

    #[test]
    fn test_sanitize_preserves_all_allowed_keys() {
        let yaml = r#"
tokens:
    dai:
        network: mainnet
        address: 0x6b175474e89094c44da98b954eedeac495271d0f
        decimals: 18
        label: Dai Stablecoin
        symbol: DAI
"#;
        let doc = get_document(yaml);
        TokenCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let tokens = root
            .get(&StrictYaml::String("tokens".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let dai = tokens
            .get(&StrictYaml::String("dai".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(
            dai.get(&StrictYaml::String("network".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            dai.get(&StrictYaml::String("address".to_string())),
            Some(&StrictYaml::String(
                "0x6b175474e89094c44da98b954eedeac495271d0f".to_string()
            ))
        );
        assert_eq!(
            dai.get(&StrictYaml::String("decimals".to_string())),
            Some(&StrictYaml::String("18".to_string()))
        );
        assert_eq!(
            dai.get(&StrictYaml::String("label".to_string())),
            Some(&StrictYaml::String("Dai Stablecoin".to_string()))
        );
        assert_eq!(
            dai.get(&StrictYaml::String("symbol".to_string())),
            Some(&StrictYaml::String("DAI".to_string()))
        );
    }

    #[test]
    fn test_sanitize_drops_non_hash_token_entries() {
        let yaml = r#"
tokens:
    dai:
        network: mainnet
        address: 0x6b175474e89094c44da98b954eedeac495271d0f
    invalid-string: just-a-string
"#;
        let doc = get_document(yaml);
        TokenCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let tokens = root
            .get(&StrictYaml::String("tokens".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(tokens.contains_key(&StrictYaml::String("dai".to_string())));
        assert!(!tokens.contains_key(&StrictYaml::String("invalid-string".to_string())));
    }

    #[test]
    fn test_sanitize_sorts_tokens_lexicographically() {
        let yaml = r#"
tokens:
    zebra:
        network: mainnet
        address: 0x1111111111111111111111111111111111111111
    alpha:
        network: mainnet
        address: 0x2222222222222222222222222222222222222222
    dai:
        network: mainnet
        address: 0x3333333333333333333333333333333333333333
"#;
        let doc = get_document(yaml);
        TokenCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let tokens = root
            .get(&StrictYaml::String("tokens".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let keys: Vec<_> = tokens.keys().map(|k| k.as_str().unwrap()).collect();
        assert_eq!(keys, vec!["alpha", "dai", "zebra"]);
    }

    #[test]
    fn test_sanitize_handles_missing_tokens_section() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
"#;
        let doc = get_document(yaml);
        TokenCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        assert!(!root.contains_key(&StrictYaml::String("tokens".to_string())));
    }

    #[test]
    fn test_sanitize_handles_non_hash_root() {
        let yaml = r#"just-a-string"#;
        let doc = get_document(yaml);
        TokenCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        assert!(doc_read.as_str().is_some());
    }

    #[test]
    fn test_sanitize_skips_non_hash_tokens_section() {
        let yaml = r#"
tokens: not-a-hash
"#;
        let doc = get_document(yaml);
        TokenCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let tokens = root.get(&StrictYaml::String("tokens".to_string())).unwrap();
        assert_eq!(tokens.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_per_document_isolation() {
        let yaml1 = r#"
tokens:
    from-doc1:
        network: mainnet
        address: 0x1111111111111111111111111111111111111111
        extra-key: removed
"#;
        let yaml2 = r#"
tokens:
    from-doc2:
        network: mainnet
        address: 0x2222222222222222222222222222222222222222
        another-extra: also-removed
"#;
        let doc1 = get_document(yaml1);
        let doc2 = get_document(yaml2);

        TokenCfg::sanitize_documents(&[doc1.clone(), doc2.clone()]).unwrap();

        let doc1_read = doc1.read().unwrap();
        let root1 = doc1_read.as_hash().unwrap();
        let tokens1 = root1
            .get(&StrictYaml::String("tokens".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let from_doc1 = tokens1
            .get(&StrictYaml::String("from-doc1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert!(!from_doc1.contains_key(&StrictYaml::String("extra-key".to_string())));
        assert!(!tokens1.contains_key(&StrictYaml::String("from-doc2".to_string())));

        let doc2_read = doc2.read().unwrap();
        let root2 = doc2_read.as_hash().unwrap();
        let tokens2 = root2
            .get(&StrictYaml::String("tokens".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let from_doc2 = tokens2
            .get(&StrictYaml::String("from-doc2".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert!(!from_doc2.contains_key(&StrictYaml::String("another-extra".to_string())));
        assert!(!tokens2.contains_key(&StrictYaml::String("from-doc1".to_string())));
    }
}
