use crate::*;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::{
    context::Context, default_document, optional_string, require_hash, require_string,
    FieldErrorKind, YamlError, YamlParsableHash,
};

const ALLOWED_RAINLANG_KEYS: [&str; 2] = ["address", "network"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct RainlangCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub network: Arc<NetworkCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RainlangCfg);

impl RainlangCfg {
    pub fn dummy() -> Self {
        RainlangCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::default(),
            network: Arc::new(NetworkCfg::dummy()),
        }
    }

    pub fn validate_address(value: &str) -> Result<Address, ParseRainlangConfigSourceError> {
        Address::from_str(value).map_err(ParseRainlangConfigSourceError::AddressParseError)
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        rainlang_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(rainlangs_hash) = require_hash(&document_read, Some("rainlangs"), None) {
                if let Some(rainlang_yaml) =
                    rainlangs_hash.get(&StrictYaml::String(rainlang_key.to_string()))
                {
                    return require_string(rainlang_yaml, Some("network"), None)
                        .or_else(|_| Ok(rainlang_key.to_string()));
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "rainlangs".to_string(),
                        expected: "a map".to_string(),
                    },
                    location: "root".to_string(),
                });
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("network for rainlang '{}'", rainlang_key)),
            location: "root".to_string(),
        })
    }
}

impl Default for RainlangCfg {
    fn default() -> Self {
        RainlangCfg::dummy()
    }
}

impl PartialEq for RainlangCfg {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.network == other.network
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseRainlangConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(alloy::primitives::hex::FromHexError),
    #[error("Network not found for Rainlang: {0}")]
    NetworkNotFoundError(String),
}

impl ParseRainlangConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseRainlangConfigSourceError::AddressParseError(err) =>
                format!("The rainlang address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseRainlangConfigSourceError::NetworkNotFoundError(network) =>
                format!("The network '{}' specified for this rainlang was not found in your YAML configuration. Please define this network or use an existing one.", network),
        }
    }
}

impl YamlParsableHash for RainlangCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut rainlangs = HashMap::new();

        let networks = NetworkCfg::parse_all_from_yaml(documents.clone(), context)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(rainlangs_hash) = require_hash(&document_read, Some("rainlangs"), None) {
                for (key_yaml, rainlang_yaml) in rainlangs_hash {
                    let rainlang_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("rainlang '{}'", rainlang_key);

                    let address_str =
                        require_string(rainlang_yaml, Some("address"), Some(location.clone()))?;
                    let address = RainlangCfg::validate_address(&address_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let network_name = match optional_string(rainlang_yaml, "network") {
                        Some(network_name) => network_name,
                        None => rainlang_key.clone(),
                    };
                    let network = networks
                        .get(&network_name)
                        .ok_or_else(|| YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "network".to_string(),
                                reason: format!("Network '{}' not found", network_name),
                            },
                            location: location.clone(),
                        })?;

                    let rainlang_cfg = RainlangCfg {
                        document: document.clone(),
                        key: rainlang_key.clone(),
                        address,
                        network: Arc::new(network.clone()),
                    };

                    if rainlangs.contains_key(&rainlang_key) {
                        return Err(YamlError::KeyShadowing(
                            rainlang_key,
                            "rainlangs".to_string(),
                        ));
                    }
                    rainlangs.insert(rainlang_key, rainlang_cfg);
                }
            }
        }

        if rainlangs.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("rainlangs".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(rainlangs)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let rainlangs_key = StrictYaml::String("rainlangs".to_string());
            let Some(rainlangs_value) = root_hash.get(&rainlangs_key) else {
                continue;
            };
            let StrictYaml::Hash(ref rainlangs_hash) = rainlangs_value.clone() else {
                continue;
            };

            let mut sanitized_rainlangs: Vec<(String, StrictYaml)> = Vec::new();

            for (key, value) in rainlangs_hash {
                let Some(key_str) = key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref rainlang_hash) = *value else {
                    continue;
                };

                let mut sanitized = Hash::new();
                for allowed_key in ALLOWED_RAINLANG_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = rainlang_hash.get(&key_yaml) {
                        sanitized.insert(key_yaml, v.clone());
                    }
                }
                sanitized_rainlangs.push((key_str.to_string(), StrictYaml::Hash(sanitized)));
            }

            sanitized_rainlangs.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_rainlangs_hash = Hash::new();
            for (key, value) in sanitized_rainlangs {
                new_rainlangs_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(rainlangs_key, StrictYaml::Hash(new_rainlangs_hash));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_rainlangs_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
    RainlangOne:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let yaml_two = r#"
rainlangs:
    RainlangTwo:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let rainlangs = RainlangCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(rainlangs.len(), 2);
        assert!(rainlangs.contains_key("RainlangOne"));
        assert!(rainlangs.contains_key("RainlangTwo"));

        assert_eq!(
            rainlangs.get("RainlangOne").unwrap().address.to_string(),
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(
            rainlangs.get("RainlangTwo").unwrap().address.to_string(),
            "0x0987654321098765432109876543210987654321"
        );
    }

    #[test]
    fn test_parse_rainlangs_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
    DuplicateRainlang:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let yaml_two = r#"
rainlangs:
    DuplicateRainlang:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = RainlangCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateRainlang".to_string(), "rainlangs".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'DuplicateRainlang' is defined multiple times in your YAML configuration at rainlangs"
        );
    }

    #[test]
    fn test_parse_rainlang_from_yaml_network_key() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
"#;

        let documents = vec![get_document(yaml)];
        let network_key = RainlangCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
"#;
        let documents = vec![get_document(yaml)];
        let network_key = RainlangCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs: test
"#;
        let error =
            RainlangCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "rainlangs".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'rainlangs' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
rainlangs:
  - test
"#;
        let error =
            RainlangCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "rainlangs".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'rainlangs' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
  - test: test
"#;
        let error =
            RainlangCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "rainlangs".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'rainlangs' in root must be a map"
        );

        let yaml = r#"
networks:
    mainnet:
        rpc: https://rpc.com
        chain-id: 1
rainlangs:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
"#;
        let res = RainlangCfg::parse_network_key(vec![get_document(yaml)], "mainnet").unwrap();
        assert_eq!(res, "mainnet");
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_keys() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let document = get_document(yaml);
        RainlangCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let rainlangs = root
            .get(&StrictYaml::String("rainlangs".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref rainlangs_hash) = *rainlangs else {
            panic!("expected rainlangs hash");
        };
        let mainnet = rainlangs_hash
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref mainnet_hash) = *mainnet else {
            panic!("expected mainnet hash");
        };

        assert!(mainnet_hash.contains_key(&StrictYaml::String("address".to_string())));
        assert!(mainnet_hash.contains_key(&StrictYaml::String("network".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("another-unknown".to_string())));
        assert_eq!(mainnet_hash.len(), 2);
    }

    #[test]
    fn test_sanitize_documents_preserves_allowed_key_order() {
        let yaml = r#"
rainlangs:
    mainnet:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
        extra: dropped
"#;
        let document = get_document(yaml);
        RainlangCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let rainlangs = root
            .get(&StrictYaml::String("rainlangs".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref rainlangs_hash) = *rainlangs else {
            panic!("expected rainlangs hash");
        };
        let mainnet = rainlangs_hash
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref mainnet_hash) = *mainnet else {
            panic!("expected mainnet hash");
        };

        let keys: Vec<String> = mainnet_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["address", "network"]);
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_entries() {
        let yaml = r#"
rainlangs:
    mainnet: not-a-hash
"#;
        let document = get_document(yaml);
        RainlangCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let rainlangs = root
            .get(&StrictYaml::String("rainlangs".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref rainlangs_hash) = *rainlangs else {
            panic!("expected rainlangs hash");
        };

        assert!(!rainlangs_hash.contains_key(&StrictYaml::String("mainnet".to_string())));
        assert!(rainlangs_hash.is_empty());
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
rainlangs:
    zebra:
        address: 0x0000000000000000000000000000000000000003
    alpha:
        address: 0x0000000000000000000000000000000000000001
    beta:
        address: 0x0000000000000000000000000000000000000002
"#;
        let document = get_document(yaml);
        RainlangCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let rainlangs = root
            .get(&StrictYaml::String("rainlangs".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref rainlangs_hash) = *rainlangs else {
            panic!("expected rainlangs hash");
        };

        let keys: Vec<String> = rainlangs_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_handles_missing_rainlangs_section() {
        let yaml = r#"
other: value
"#;
        let document = get_document(yaml);
        RainlangCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        assert!(!root.contains_key(&StrictYaml::String("rainlangs".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = r#"just a string"#;
        let document = get_document(yaml);
        RainlangCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_rainlangs() {
        let yaml = r#"
rainlangs: not-a-hash
"#;
        let document = get_document(yaml);
        RainlangCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let rainlangs = root
            .get(&StrictYaml::String("rainlangs".to_string()))
            .unwrap();
        assert_eq!(rainlangs.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_documents_per_doc_no_cross_merge() {
        let yaml_one = r#"
rainlangs:
    rainlang-one:
        address: 0x0000000000000000000000000000000000000001
        extra-key: dropped
"#;
        let yaml_two = r#"
rainlangs:
    rainlang-two:
        address: 0x0000000000000000000000000000000000000002
        another-extra: also-dropped
"#;
        let doc_one = get_document(yaml_one);
        let doc_two = get_document(yaml_two);
        let documents = vec![doc_one.clone(), doc_two.clone()];
        RainlangCfg::sanitize_documents(&documents).unwrap();

        {
            let doc_read = doc_one.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let rainlangs = root
                .get(&StrictYaml::String("rainlangs".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref rainlangs_hash) = *rainlangs else {
                panic!("expected rainlangs hash");
            };

            let keys: Vec<String> = rainlangs_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["rainlang-one"]);

            let rainlang_entry = rainlangs_hash
                .get(&StrictYaml::String("rainlang-one".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref rainlang_hash) = *rainlang_entry else {
                panic!("expected rainlang hash");
            };
            assert!(!rainlang_hash.contains_key(&StrictYaml::String("extra-key".to_string())));
        }

        {
            let doc_read = doc_two.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let rainlangs = root
                .get(&StrictYaml::String("rainlangs".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref rainlangs_hash) = *rainlangs else {
                panic!("expected rainlangs hash");
            };

            let keys: Vec<String> = rainlangs_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["rainlang-two"]);

            let rainlang_entry = rainlangs_hash
                .get(&StrictYaml::String("rainlang-two".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref rainlang_hash) = *rainlang_entry else {
                panic!("expected rainlang hash");
            };
            assert!(!rainlang_hash.contains_key(&StrictYaml::String("another-extra".to_string())));
        }
    }
}
