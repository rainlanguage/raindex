use crate::yaml::context::Context;
use crate::yaml::{
    default_document, optional_string, require_hash, require_string, require_vec, FieldErrorKind,
    YamlError, YamlParsableHash,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    num::ParseIntError,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

const ALLOWED_NETWORK_KEYS: [&str; 6] = [
    "rpcs",
    "chain-id",
    "label",
    "network-id",
    "currency",
    "block-explorer",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct NetworkCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string[]"))]
    pub rpcs: Vec<Url>,
    pub chain_id: u32,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub network_id: Option<u32>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub currency: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional, type = "string"))]
    pub block_explorer: Option<Url>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(NetworkCfg);

impl NetworkCfg {
    pub fn dummy() -> Self {
        NetworkCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            rpcs: vec![Url::parse("http://rpc.com").unwrap()],
            chain_id: 1,
            label: None,
            network_id: None,
            currency: None,
            block_explorer: None,
        }
    }

    pub fn validate_rpc(value: &str) -> Result<Url, ParseNetworkConfigSourceError> {
        Url::parse(value).map_err(ParseNetworkConfigSourceError::RpcParseError)
    }
    pub fn validate_chain_id(value: &str) -> Result<u64, ParseNetworkConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseNetworkConfigSourceError::ChainIdParseError)
    }
    pub fn validate_network_id(value: &str) -> Result<u32, ParseNetworkConfigSourceError> {
        value
            .parse::<u32>()
            .map_err(ParseNetworkConfigSourceError::NetworkIdParseError)
    }
    pub fn validate_block_explorer(value: &str) -> Result<Url, ParseNetworkConfigSourceError> {
        let url =
            Url::parse(value).map_err(ParseNetworkConfigSourceError::BlockExplorerParseError)?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ParseNetworkConfigSourceError::BlockExplorerSchemeError(
                url.scheme().to_string(),
            ));
        }
        Ok(url)
    }

    pub fn update_rpcs(&mut self, rpcs: Vec<String>) -> Result<Self, YamlError> {
        let mut rpc_vec = Vec::new();
        for rpc in rpcs {
            rpc_vec.push(NetworkCfg::validate_rpc(&rpc)?);
        }

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut networks)) =
                document_hash.get_mut(&StrictYaml::String("networks".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut network)) =
                    networks.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    let rpcs = rpc_vec
                        .iter()
                        .map(|rpc| StrictYaml::String(rpc.to_string()))
                        .collect();
                    network[&StrictYaml::String("rpcs".to_string())] = StrictYaml::Array(rpcs);
                    self.rpcs = rpc_vec;
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "networks".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("networks".to_string()),
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

    pub fn parse_rpcs(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        network_key: &str,
    ) -> Result<Vec<Url>, YamlError> {
        let mut res = Vec::new();

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(networks_hash) =
                require_hash(&document_read, Some("networks"), Some("root".to_string()))
            {
                if let Some(network_yaml) =
                    networks_hash.get(&StrictYaml::String(network_key.to_string()))
                {
                    let location = format!("network '{}'", network_key);
                    let rpcs = require_vec(network_yaml, "rpcs", Some(location.clone()))?;

                    for rpc_value in rpcs {
                        let url_str = require_string(rpc_value, None, None)?;
                        let url = NetworkCfg::validate_rpc(&url_str)?;
                        res.push(url);
                    }

                    if res.is_empty() {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "rpcs".to_string(),
                                reason: "must be a non-empty array".to_string(),
                            },
                            location,
                        });
                    }

                    return Ok(res);
                }
            }
        }

        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("rpcs for network '{}'", network_key)),
            location: "root".to_string(),
        })
    }
}

impl YamlParsableHash for NetworkCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut networks = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(networks_hash) =
                require_hash(&document_read, Some("networks"), Some("root".to_string()))
            {
                for (key_yaml, network_yaml) in networks_hash {
                    let network_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("network '{}'", network_key);

                    let mut rpcs = Vec::new();
                    let rpc_vec = require_vec(network_yaml, "rpcs", Some(location.clone()))?;

                    for rpc_value in rpc_vec {
                        let rpc_str = require_string(rpc_value, None, None)?;
                        let rpc_url = NetworkCfg::validate_rpc(&rpc_str)?;
                        rpcs.push(rpc_url);
                    }
                    if rpcs.is_empty() {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "rpcs".to_string(),
                                reason: "must be a non-empty array".to_string(),
                            },
                            location: location.clone(),
                        });
                    }

                    let chain_id_str =
                        require_string(network_yaml, Some("chain-id"), Some(location.clone()))?;

                    let chain_id = chain_id_str.parse::<u32>().map_err(|e| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "chain-id".to_string(),
                            reason: e.to_string(),
                        },
                        location: location.clone(),
                    })?;

                    let label = optional_string(network_yaml, "label");
                    let network_id = optional_string(network_yaml, "network-id")
                        .map(|id| NetworkCfg::validate_network_id(&id))
                        .transpose()
                        .map_err(|e| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "network-id".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        })?;

                    let currency = optional_string(network_yaml, "currency");
                    let block_explorer = if network_yaml["block-explorer"].is_badvalue() {
                        None
                    } else {
                        let value = require_string(
                            network_yaml,
                            Some("block-explorer"),
                            Some(location.clone()),
                        )?;
                        Some(NetworkCfg::validate_block_explorer(&value)?)
                    };

                    let network = NetworkCfg {
                        document: document.clone(),
                        key: network_key.clone(),
                        rpcs,
                        chain_id,
                        label,
                        network_id,
                        currency,
                        block_explorer,
                    };

                    if networks.contains_key(&network_key) {
                        return Err(YamlError::KeyShadowing(network_key, "networks".to_string()));
                    }
                    networks.insert(network_key, network);
                }
            }
        }

        if let Some(context) = context {
            if let Some(yaml_cache) = &context.yaml_cache {
                for (key, network) in &yaml_cache.remote_networks {
                    if networks.contains_key(key) {
                        return Err(YamlError::ParseNetworkConfigSourceError(
                            ParseNetworkConfigSourceError::RemoteNetworkKeyShadowing(key.clone()),
                        ));
                    }
                    networks.insert(key.clone(), network.clone());
                }
            }
        }

        if networks.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(networks)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let networks_key = StrictYaml::String("networks".to_string());
            let Some(networks_value) = root_hash.get(&networks_key) else {
                continue;
            };
            let StrictYaml::Hash(ref networks_hash) = networks_value.clone() else {
                continue;
            };

            let mut sanitized_networks: Vec<(String, StrictYaml)> = Vec::new();

            for (key, value) in networks_hash {
                let Some(key_str) = key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref network_hash) = *value else {
                    continue;
                };

                let mut sanitized = Hash::new();
                for allowed_key in ALLOWED_NETWORK_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = network_hash.get(&key_yaml) {
                        sanitized.insert(key_yaml, v.clone());
                    }
                }
                sanitized_networks.push((key_str.to_string(), StrictYaml::Hash(sanitized)));
            }

            sanitized_networks.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_networks_hash = Hash::new();
            for (key, value) in sanitized_networks {
                new_networks_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(networks_key, StrictYaml::Hash(new_networks_hash));
        }

        Ok(())
    }
}

impl Default for NetworkCfg {
    fn default() -> Self {
        NetworkCfg::dummy()
    }
}
impl PartialEq for NetworkCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.rpcs == other.rpcs
            && self.chain_id == other.chain_id
            && self.label == other.label
            && self.network_id == other.network_id
            && self.currency == other.currency
            && self.block_explorer == other.block_explorer
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseNetworkConfigSourceError {
    #[error("Failed to parse rpcs: {0}")]
    RpcParseError(ParseError),
    #[error("Failed to parse chain_id: {0}")]
    ChainIdParseError(ParseIntError),
    #[error("Failed to parse network_id: {0}")]
    NetworkIdParseError(ParseIntError),
    #[error("Failed to parse block explorer: {0}")]
    BlockExplorerParseError(ParseError),
    #[error("Block explorer URL must use http or https, found: {0}")]
    BlockExplorerSchemeError(String),
    #[error("Remote network key shadowing: {0}")]
    RemoteNetworkKeyShadowing(String),
}

impl ParseNetworkConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseNetworkConfigSourceError::RpcParseError(err) => format!(
                "The RPC URL in your network configuration is invalid: {}",
                err
            ),
            ParseNetworkConfigSourceError::ChainIdParseError(err) => format!(
                "The chain ID in your network configuration must be a valid number: {}",
                err
            ),
            ParseNetworkConfigSourceError::NetworkIdParseError(err) => format!(
                "The network ID in your network configuration must be a valid number: {}",
                err
            ),
            ParseNetworkConfigSourceError::BlockExplorerParseError(err) => format!(
                "The block explorer URL in your network configuration is invalid: {}",
                err
            ),
            ParseNetworkConfigSourceError::BlockExplorerSchemeError(scheme) => format!(
                "The block explorer URL in your network configuration must use HTTP or HTTPS, not '{}'",
                scheme
            ),
            ParseNetworkConfigSourceError::RemoteNetworkKeyShadowing(key) => format!(
                "The remote network key '{}' is already defined in network configuration",
                key
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;
    use strict_yaml_rust::StrictYaml;
    use url::Url;

    #[test]
    fn test_parse_networks_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
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

        let yaml = r#"
networks:
    mainnet:
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("rpcs".to_string()),
                location: "network 'mainnet'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'rpcs' in network 'mainnet'"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "rpcs".to_string(),
                    expected: "a vector".to_string(),
                },
                location: "network 'mainnet'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'rpcs' in network 'mainnet' must be a vector"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            -
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseNetworkConfigSourceError(ParseNetworkConfigSourceError::RpcParseError(
                ParseError::RelativeUrlWithoutBase
            ))
        );
        assert_eq!(
            error.to_readable_msg(),
            "Network configuration error in your YAML: Failed to parse rpcs: relative URL without a base"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("chain-id".to_string()),
                location: "network 'mainnet'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'chain-id' in network 'mainnet'"
        );
    }

    #[test]
    fn test_parse_networks_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
            - https://mainnet.infura.io/v3/1234567890
        chain-id: 1
        block-explorer: https://etherscan.io
    testnet:
        rpcs:
            - https://testnet.infura.io
        chain-id: 2
"#;
        let yaml_two = r#"
networks:
    network-one:
        rpcs:
            - https://network-one.infura.io
        chain-id: 3
    network-two:
        rpcs:
            - https://network-two.infura.io
        chain-id: 4
"#;
        let networks = NetworkCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        assert_eq!(networks.len(), 4);
        assert_eq!(
            networks.get("mainnet").unwrap().block_explorer,
            Some(Url::parse("https://etherscan.io").unwrap())
        );
        assert_eq!(networks.get("testnet").unwrap().block_explorer, None);
        assert_eq!(
            networks.get("mainnet").unwrap().rpcs,
            vec![
                Url::parse("https://mainnet.infura.io").unwrap(),
                Url::parse("https://mainnet.infura.io/v3/1234567890").unwrap(),
            ]
        );
        assert_eq!(
            networks.get("testnet").unwrap().rpcs,
            vec![Url::parse("https://testnet.infura.io").unwrap()]
        );
        assert_eq!(
            networks.get("network-one").unwrap().rpcs,
            vec![Url::parse("https://network-one.infura.io").unwrap()]
        );
        assert_eq!(
            networks.get("network-two").unwrap().rpcs,
            vec![Url::parse("https://network-two.infura.io").unwrap()]
        );
    }

    #[test]
    fn test_parse_network_block_explorer_validation() {
        let invalid_url = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
        block-explorer: not-a-url
"#;
        assert_eq!(
            NetworkCfg::parse_all_from_yaml(vec![get_document(invalid_url)], None).unwrap_err(),
            YamlError::ParseNetworkConfigSourceError(
                ParseNetworkConfigSourceError::BlockExplorerParseError(
                    ParseError::RelativeUrlWithoutBase
                )
            )
        );

        let invalid_scheme = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
        block-explorer: ftp://etherscan.io
"#;
        assert_eq!(
            NetworkCfg::parse_all_from_yaml(vec![get_document(invalid_scheme)], None).unwrap_err(),
            YamlError::ParseNetworkConfigSourceError(
                ParseNetworkConfigSourceError::BlockExplorerSchemeError("ftp".to_string())
            )
        );

        let invalid_type = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
        block-explorer:
            - https://etherscan.io
"#;
        assert_eq!(
            NetworkCfg::parse_all_from_yaml(vec![get_document(invalid_type)], None).unwrap_err(),
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "block-explorer".to_string(),
                    expected: "a string".to_string(),
                },
                location: "network 'mainnet'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_networks_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
    testnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 2
"#;
        let yaml_two = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
"#;
        let error = NetworkCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing("mainnet".to_string(), "networks".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'mainnet' is defined multiple times in your YAML configuration at networks"
        );
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
networks: test
"#;
        let error = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("rpcs for network 'mainnet'".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'rpcs for network 'mainnet'' in root"
        );

        let yaml = r#"
networks:
  - test
"#;
        let error = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("rpcs for network 'mainnet'".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'rpcs for network 'mainnet'' in root"
        );

        let yaml = r#"
networks:
  - test: test
"#;
        let error = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("rpcs for network 'mainnet'".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'rpcs for network 'mainnet'' in root"
        );

        let yaml = r#"
networks:
  mainnet:
    rpcs:
      - https://rpc.com
    chain-id: 1
"#;
        let res = NetworkCfg::parse_rpcs(vec![get_document(yaml)], "mainnet").unwrap();
        assert_eq!(res, vec![Url::parse("https://rpc.com").unwrap()]);
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_keys() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
        label: Mainnet
        network-id: 1
        currency: ETH
        block-explorer: https://etherscan.io
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let networks = root
            .get(&StrictYaml::String("networks".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref networks_hash) = *networks else {
            panic!("expected networks hash");
        };
        let mainnet = networks_hash
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref mainnet_hash) = *mainnet else {
            panic!("expected mainnet hash");
        };

        assert!(mainnet_hash.contains_key(&StrictYaml::String("rpcs".to_string())));
        assert!(mainnet_hash.contains_key(&StrictYaml::String("chain-id".to_string())));
        assert!(mainnet_hash.contains_key(&StrictYaml::String("label".to_string())));
        assert!(mainnet_hash.contains_key(&StrictYaml::String("network-id".to_string())));
        assert!(mainnet_hash.contains_key(&StrictYaml::String("currency".to_string())));
        assert!(mainnet_hash.contains_key(&StrictYaml::String("block-explorer".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("another-unknown".to_string())));
        assert_eq!(mainnet_hash.len(), 6);
    }

    #[test]
    fn test_sanitize_documents_preserves_allowed_key_order() {
        let yaml = r#"
networks:
    mainnet:
        currency: ETH
        network-id: 1
        label: Mainnet
        chain-id: 1
        block-explorer: https://etherscan.io
        rpcs:
            - https://mainnet.infura.io
        extra: dropped
"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let networks = root
            .get(&StrictYaml::String("networks".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref networks_hash) = *networks else {
            panic!("expected networks hash");
        };
        let mainnet = networks_hash
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref mainnet_hash) = *mainnet else {
            panic!("expected mainnet hash");
        };

        let keys: Vec<String> = mainnet_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(
            keys,
            vec![
                "rpcs",
                "chain-id",
                "label",
                "network-id",
                "currency",
                "block-explorer"
            ]
        );
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_entries() {
        let yaml = r#"
networks:
    mainnet: not-a-hash
"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let networks = root
            .get(&StrictYaml::String("networks".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref networks_hash) = *networks else {
            panic!("expected networks hash");
        };

        assert!(!networks_hash.contains_key(&StrictYaml::String("mainnet".to_string())));
        assert!(networks_hash.is_empty());
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_preserves_hash_entries() {
        let yaml = r#"
networks:
    invalid-network: not-a-hash
    valid-network:
        rpcs:
            - https://rpc.example.io
        chain-id: 1
    another-invalid: 12345
"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let networks = root
            .get(&StrictYaml::String("networks".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref networks_hash) = *networks else {
            panic!("expected networks hash");
        };

        assert!(!networks_hash.contains_key(&StrictYaml::String("invalid-network".to_string())));
        assert!(!networks_hash.contains_key(&StrictYaml::String("another-invalid".to_string())));
        assert!(networks_hash.contains_key(&StrictYaml::String("valid-network".to_string())));
        assert_eq!(networks_hash.len(), 1);

        let valid = networks_hash
            .get(&StrictYaml::String("valid-network".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref valid_hash) = *valid else {
            panic!("expected valid-network hash");
        };
        assert!(valid_hash.contains_key(&StrictYaml::String("rpcs".to_string())));
        assert!(valid_hash.contains_key(&StrictYaml::String("chain-id".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_missing_networks_section() {
        let yaml = r#"
other: value
"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        assert!(!root.contains_key(&StrictYaml::String("networks".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = r#"just a string"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_networks() {
        let yaml = r#"
networks: not-a-hash
"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let networks = root
            .get(&StrictYaml::String("networks".to_string()))
            .unwrap();
        assert_eq!(networks.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_documents_per_doc_no_cross_merge() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
        extra-key: dropped
"#;
        let yaml_two = r#"
networks:
    testnet:
        rpcs:
            - https://testnet.infura.io
        chain-id: 2
        another-extra: also-dropped
"#;
        let doc_one = get_document(yaml_one);
        let doc_two = get_document(yaml_two);
        let documents = vec![doc_one.clone(), doc_two.clone()];
        NetworkCfg::sanitize_documents(&documents).unwrap();

        {
            let doc_read = doc_one.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let networks = root
                .get(&StrictYaml::String("networks".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref networks_hash) = *networks else {
                panic!("expected networks hash");
            };

            let keys: Vec<String> = networks_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["mainnet"]);

            let mainnet = networks_hash
                .get(&StrictYaml::String("mainnet".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref mainnet_hash) = *mainnet else {
                panic!("expected mainnet hash");
            };
            assert!(!mainnet_hash.contains_key(&StrictYaml::String("extra-key".to_string())));
            assert!(mainnet_hash.contains_key(&StrictYaml::String("rpcs".to_string())));
            assert!(mainnet_hash.contains_key(&StrictYaml::String("chain-id".to_string())));
        }

        {
            let doc_read = doc_two.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let networks = root
                .get(&StrictYaml::String("networks".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref networks_hash) = *networks else {
                panic!("expected networks hash");
            };

            let keys: Vec<String> = networks_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["testnet"]);

            let testnet = networks_hash
                .get(&StrictYaml::String("testnet".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref testnet_hash) = *testnet else {
                panic!("expected testnet hash");
            };
            assert!(!testnet_hash.contains_key(&StrictYaml::String("another-extra".to_string())));
            assert!(testnet_hash.contains_key(&StrictYaml::String("rpcs".to_string())));
            assert!(testnet_hash.contains_key(&StrictYaml::String("chain-id".to_string())));
        }
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
networks:
    zebra:
        rpcs:
            - https://zebra.io
        chain-id: 3
    alpha:
        rpcs:
            - https://alpha.io
        chain-id: 1
    beta:
        rpcs:
            - https://beta.io
        chain-id: 2
"#;
        let document = get_document(yaml);
        NetworkCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let networks = root
            .get(&StrictYaml::String("networks".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref networks_hash) = *networks else {
            panic!("expected networks hash");
        };

        let keys: Vec<String> = networks_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_parsing_still_fails_for_invalid_shapes_after_sanitization() {
        let yaml = r#"
networks: not-a-hash
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("networks".to_string()),
                location: "root".to_string(),
            }
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs: not-an-array
        chain-id: 1
"#;
        let error = NetworkCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "rpcs".to_string(),
                    expected: "a vector".to_string(),
                },
                location: "network 'mainnet'".to_string(),
            }
        );
    }
}
