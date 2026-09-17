use crate::remote::chains::{ChainId, ChainIdError};
use crate::yaml::context::Context;
use crate::yaml::{
    default_document, require_hash, require_string, FieldErrorKind, YamlError, YamlParsableHash,
};
use crate::NetworkCfg;

const ALLOWED_REMOTE_NETWORKS_KEYS: [&str; 2] = ["format", "url"];
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::{strict_yaml::Hash as StrictHash, StrictYaml};
use thiserror::Error;
use url::{ParseError, Url};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum RemoteNetworks {
    ChainId(Vec<ChainId>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RemoteNetworksCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub url: Url,
    pub format: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RemoteNetworksCfg);

impl RemoteNetworksCfg {
    pub fn validate_url(value: &str) -> Result<Url, ParseRemoteNetworksError> {
        Url::parse(value).map_err(ParseRemoteNetworksError::UrlParseError)
    }

    fn sanitize_remote_networks_hash(remote_networks_hash: &StrictHash) -> StrictHash {
        let mut sanitized_entries: Vec<(String, StrictYaml)> = Vec::new();

        for (entry_key, entry_value) in remote_networks_hash {
            let Some(entry_key_str) = entry_key.as_str() else {
                continue;
            };
            let StrictYaml::Hash(ref entry_hash) = *entry_value else {
                continue;
            };

            let mut sanitized_entry = StrictHash::new();
            for allowed_key in ALLOWED_REMOTE_NETWORKS_KEYS.iter() {
                let key_yaml = StrictYaml::String(allowed_key.to_string());
                if let Some(v) = entry_hash.get(&key_yaml) {
                    sanitized_entry.insert(key_yaml, v.clone());
                }
            }
            sanitized_entries.push((entry_key_str.to_string(), StrictYaml::Hash(sanitized_entry)));
        }
        sanitized_entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut new_hash = StrictHash::new();
        for (key, value) in sanitized_entries {
            new_hash.insert(StrictYaml::String(key), value);
        }
        new_hash
    }

    pub async fn fetch_networks(
        remote_networks: HashMap<String, RemoteNetworksCfg>,
    ) -> Result<HashMap<String, NetworkCfg>, ParseRemoteNetworksError> {
        let mut networks = HashMap::new();

        for (_, remote_network) in remote_networks {
            match remote_network.format.as_str() {
                "chainid" => {
                    let chains = reqwest::get(remote_network.url.to_string())
                        .await?
                        .json::<Vec<ChainId>>()
                        .await?;

                    for chain in &chains {
                        let network: NetworkCfg = chain
                            .clone()
                            .try_into_network_cfg(remote_network.document.clone())?;

                        if networks.contains_key(&network.key) {
                            return Err(ParseRemoteNetworksError::ConflictingNetworks(
                                network.key.clone(),
                            ));
                        }
                        networks.insert(network.key.clone(), network);
                    }
                }
                _ => {
                    return Err(ParseRemoteNetworksError::UnknownFormat(
                        remote_network.format.clone(),
                    ))
                }
            };
        }

        Ok(networks)
    }
}

impl YamlParsableHash for RemoteNetworksCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut remote_networks = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(networks_hash) = require_hash(
                &document_read,
                Some("using-networks-from"),
                Some("root".to_string()),
            ) {
                for (key_yaml, network_yaml) in networks_hash {
                    let key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("using-networks-from '{}'", key);

                    let url_str =
                        require_string(network_yaml, Some("url"), Some(location.clone()))?;
                    let url = RemoteNetworksCfg::validate_url(&url_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "url".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let format =
                        require_string(network_yaml, Some("format"), Some(location.clone()))?;

                    let remote_network = RemoteNetworksCfg {
                        document: document.clone(),
                        key: key.clone(),
                        url,
                        format,
                    };

                    if remote_networks.contains_key(&key) {
                        return Err(YamlError::KeyShadowing(
                            key.clone(),
                            "using-networks-from".to_string(),
                        ));
                    }
                    remote_networks.insert(key, remote_network);
                }
            }
        }

        Ok(remote_networks)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let section_key = StrictYaml::String("using-networks-from".to_string());
            let Some(section_value) = root_hash.get(&section_key) else {
                continue;
            };
            let StrictYaml::Hash(ref remote_networks_hash) = *section_value else {
                continue;
            };

            let sanitized = RemoteNetworksCfg::sanitize_remote_networks_hash(remote_networks_hash);
            root_hash.insert(section_key, StrictYaml::Hash(sanitized));
        }
        Ok(())
    }
}

impl Default for RemoteNetworksCfg {
    fn default() -> Self {
        RemoteNetworksCfg {
            document: default_document(),
            key: "".to_string(),
            url: Url::parse("https://example.com").unwrap(),
            format: "".to_string(),
        }
    }
}
impl PartialEq for RemoteNetworksCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.url == other.url && self.format == other.format
    }
}

#[derive(Error, Debug)]
pub enum ParseRemoteNetworksError {
    #[error(transparent)]
    UrlParseError(ParseError),
    #[error("Unknown format: {}", 0)]
    UnknownFormat(String),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Conflicting remote network in response, a network with key '{0}' already exists")]
    ConflictingNetworks(String),
    #[error(transparent)]
    ChainIdError(#[from] ChainIdError),
}

#[cfg(test)]
mod tests {
    use httpmock::MockServer;

    use super::*;
    use crate::yaml::{tests::get_document, FieldErrorKind};

    #[test]
    fn test_parse_remote_networks_from_yaml() {
        let yaml = r#"
using-networks-from:
    test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("url".to_string()),
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url:
        - test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "url".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url:
        - test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "url".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "url".to_string(),
                    reason: "relative URL without a base".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("format".to_string()),
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      format:
        - test: test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "format".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );

        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      format:
        - test
"#;
        let error =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "format".to_string(),
                    expected: "a string".to_string(),
                },
                location: "using-networks-from 'test'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_remote_networks_from_yaml_duplicate_key() {
        let yaml = r#"
using-networks-from:
    test:
      url: https://example.com
      format: chainid
"#;
        let error = RemoteNetworksCfg::parse_all_from_yaml(
            vec![get_document(yaml), get_document(yaml)],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing("test".to_string(), "using-networks-from".to_string())
        );
    }

    #[tokio::test]
    async fn test_fetch_remote_networks() {
        let server = MockServer::start_async().await;
        let yaml = format!(
            r#"
using-networks-from:
    test:
      url: {} # Use the actual mock server URL here
      format: chainid
"#,
            server.base_url()
        );
        let remote_networks =
            RemoteNetworksCfg::parse_all_from_yaml(vec![get_document(&yaml)], None).unwrap();

        let response = r#"
[
    {
        "name": "Remote",
        "chain": "remote-network",
        "chainId": 123,
        "rpc": ["http://localhost:8085/rpc-url"],
        "networkId": 123,
        "nativeCurrency": {
            "name": "Remote",
            "symbol": "RN",
            "decimals": 18
        },
        "infoURL": "http://localhost:8085/info-url",
        "shortName": "remote-network",
        "explorers": [
            {
                "name": "Incompatible Explorer",
                "url": "https://incompatible.example.com",
                "standard": "none"
            },
            {
                "name": "Remote Explorer",
                "url": "https://explorer.example.com",
                "standard": "EIP3091"
            }
        ]
    },
    {
        "name": "Remote2",
        "chain": "remote2-network",
        "chainId": 234,
        "rpc": ["http://localhost:8085/rpc-url"],
        "networkId": 123,
        "nativeCurrency": {
            "name": "Remote2",
            "symbol": "RN",
            "decimals": 18
        },
        "infoURL": "http://localhost:8085/info-url",
        "shortName": "remote2-network"
    }
]
        "#;
        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(response);
            })
            .await;

        let networks = RemoteNetworksCfg::fetch_networks(remote_networks)
            .await
            .unwrap();

        assert_eq!(networks.len(), 2_usize);

        let network = networks.get("remote-network").unwrap();
        assert_eq!(network.key, "remote-network");
        assert_eq!(
            network.rpcs,
            vec![Url::parse("http://localhost:8085/rpc-url").unwrap()]
        );
        assert_eq!(network.chain_id, 123);
        assert_eq!(
            network.block_explorer,
            Some(Url::parse("https://explorer.example.com").unwrap())
        );

        let network = networks.get("remote2-network").unwrap();
        assert_eq!(network.key, "remote2-network");
        assert_eq!(
            network.rpcs,
            vec![Url::parse("http://localhost:8085/rpc-url").unwrap()]
        );
        assert_eq!(network.chain_id, 234);
        assert_eq!(network.block_explorer, None);
    }

    #[test]
    fn test_sanitize_drops_unknown_keys() {
        let yaml = r#"
using-networks-from:
    test:
        url: https://example.com
        format: chainid
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let doc = get_document(yaml);
        RemoteNetworksCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let section = root_hash
            .get(&StrictYaml::String("using-networks-from".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let entry = section
            .get(&StrictYaml::String("test".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(entry.len(), 2);
        assert!(entry.get(&StrictYaml::String("url".to_string())).is_some());
        assert!(entry
            .get(&StrictYaml::String("format".to_string()))
            .is_some());
        assert!(entry
            .get(&StrictYaml::String("unknown-key".to_string()))
            .is_none());
        assert!(entry
            .get(&StrictYaml::String("another-unknown".to_string()))
            .is_none());
    }

    #[test]
    fn test_sanitize_preserves_allowed_keys() {
        let yaml = r#"
using-networks-from:
    test:
        url: https://example.com
        format: chainid
"#;
        let doc = get_document(yaml);
        RemoteNetworksCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let section = root_hash
            .get(&StrictYaml::String("using-networks-from".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let entry = section
            .get(&StrictYaml::String("test".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(
            entry.get(&StrictYaml::String("url".to_string())),
            Some(&StrictYaml::String("https://example.com".to_string()))
        );
        assert_eq!(
            entry.get(&StrictYaml::String("format".to_string())),
            Some(&StrictYaml::String("chainid".to_string()))
        );
    }

    #[test]
    fn test_sanitize_drops_non_hash_entries() {
        let yaml = r#"
using-networks-from:
    valid:
        url: https://example.com
        format: chainid
    invalid-string: just-a-string
"#;
        let doc = get_document(yaml);
        RemoteNetworksCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let section = root_hash
            .get(&StrictYaml::String("using-networks-from".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(section.len(), 1);
        assert!(section
            .get(&StrictYaml::String("valid".to_string()))
            .is_some());
        assert!(section
            .get(&StrictYaml::String("invalid-string".to_string()))
            .is_none());
    }

    #[test]
    fn test_sanitize_lexicographic_ordering() {
        let yaml = r#"
using-networks-from:
    zebra:
        url: https://zebra.com
        format: chainid
    alpha:
        url: https://alpha.com
        format: chainid
    middle:
        url: https://middle.com
        format: chainid
"#;
        let doc = get_document(yaml);
        RemoteNetworksCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let section = root_hash
            .get(&StrictYaml::String("using-networks-from".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let keys: Vec<&str> = section.iter().map(|(k, _)| k.as_str().unwrap()).collect();
        assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn test_sanitize_handles_missing_section() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
"#;
        let doc = get_document(yaml);
        let result = RemoteNetworksCfg::sanitize_documents(std::slice::from_ref(&doc));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_handles_non_hash_root() {
        let yaml = "just-a-string";
        let doc = get_document(yaml);
        let result = RemoteNetworksCfg::sanitize_documents(std::slice::from_ref(&doc));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_skips_non_hash_section_value() {
        let yaml = r#"
using-networks-from: just-a-string
"#;
        let doc = get_document(yaml);
        let result = RemoteNetworksCfg::sanitize_documents(std::slice::from_ref(&doc));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_per_document_isolation() {
        let yaml1 = r#"
using-networks-from:
    doc1-entry:
        url: https://doc1.com
        format: chainid
        extra: should-drop
"#;
        let yaml2 = r#"
using-networks-from:
    doc2-entry:
        url: https://doc2.com
        format: chainid
        extra: should-drop
"#;
        let doc1 = get_document(yaml1);
        let doc2 = get_document(yaml2);
        RemoteNetworksCfg::sanitize_documents(&[doc1.clone(), doc2.clone()]).unwrap();

        let doc1_read = doc1.read().unwrap();
        let section1 = doc1_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("using-networks-from".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let entry1 = section1
            .get(&StrictYaml::String("doc1-entry".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert_eq!(entry1.len(), 2);

        let doc2_read = doc2.read().unwrap();
        let section2 = doc2_read
            .as_hash()
            .unwrap()
            .get(&StrictYaml::String("using-networks-from".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let entry2 = section2
            .get(&StrictYaml::String("doc2-entry".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert_eq!(entry2.len(), 2);
    }
}
