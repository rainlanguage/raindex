use crate::yaml::FieldErrorKind;
use crate::*;
use alloy::primitives::hex::FromHexError;
use alloy::primitives::Address;
use local_db_remotes::LocalDbRemoteCfg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::strict_yaml::Hash;
use strict_yaml_rust::StrictYaml;
use subgraph::SubgraphCfg;
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::context::Context;
use yaml::{
    default_document, optional_string, require_hash, require_string, YamlError, YamlParsableHash,
};

const ALLOWED_RAINDEX_KEYS: [&str; 6] = [
    "address",
    "deployment-block",
    "label",
    "local-db-remote",
    "network",
    "subgraph",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct RaindexCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub address: Address,
    pub network: Arc<NetworkCfg>,
    pub subgraph: Arc<SubgraphCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub local_db_remote: Option<Arc<LocalDbRemoteCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub label: Option<String>,
    pub deployment_block: u64,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(RaindexCfg);

impl RaindexCfg {
    pub fn validate_address(address: &str) -> Result<Address, ParseRaindexConfigSourceError> {
        Address::from_str(address).map_err(ParseRaindexConfigSourceError::AddressParseError)
    }

    pub fn validate_deployment_block(value: &str) -> Result<u64, ParseRaindexConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseRaindexConfigSourceError::DeploymentBlockParseError)
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        raindex_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(raindex_hash) = require_hash(&document_read, Some("raindexes"), None) {
                if let Some(raindex_yaml) =
                    raindex_hash.get(&StrictYaml::String(raindex_key.to_string()))
                {
                    return require_string(raindex_yaml, Some("network"), None)
                        .or_else(|_| Ok(raindex_key.to_string()));
                }
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(format!("network for raindex '{}'", raindex_key)),
            location: "root".to_string(),
        })
    }
}

impl YamlParsableHash for RaindexCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut raindexes = HashMap::new();

        let networks = NetworkCfg::parse_all_from_yaml(documents.clone(), context)?;
        let subgraphs = SubgraphCfg::parse_all_from_yaml(documents.clone(), context)?;
        let local_db_remotes = LocalDbRemoteCfg::parse_all_from_yaml(documents.clone(), context)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(raindex_hash) = require_hash(&document_read, Some("raindexes"), None) {
                for (key_yaml, raindex_yaml) in raindex_hash {
                    let raindex_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("raindex '{}'", raindex_key);

                    let address_str =
                        require_string(raindex_yaml, Some("address"), Some(location.clone()))?;
                    let address = RaindexCfg::validate_address(&address_str).map_err(|e| {
                        YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "address".to_string(),
                                reason: e.to_string(),
                            },
                            location: location.clone(),
                        }
                    })?;

                    let network_name = match optional_string(raindex_yaml, "network") {
                        Some(network_name) => network_name,
                        None => raindex_key.clone(),
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

                    let subgraph_name = match optional_string(raindex_yaml, "subgraph") {
                        Some(subgraph_name) => subgraph_name,
                        None => raindex_key.clone(),
                    };
                    let subgraph =
                        subgraphs
                            .get(&subgraph_name)
                            .ok_or_else(|| YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "subgraph".to_string(),
                                    reason: format!("Subgraph '{}' not found", subgraph_name),
                                },
                                location: location.clone(),
                            })?;

                    let label = optional_string(raindex_yaml, "label");

                    let local_db_remote_name = optional_string(raindex_yaml, "local-db-remote");
                    let local_db_remote = if let Some(name) = local_db_remote_name {
                        local_db_remotes.get(&name).cloned()
                    } else {
                        local_db_remotes.get(&raindex_key).cloned()
                    }
                    .map(Arc::new);

                    let deployment_block_str = require_string(
                        raindex_yaml,
                        Some("deployment-block"),
                        Some(location.clone()),
                    )?;
                    let deployment_block = RaindexCfg::validate_deployment_block(
                        &deployment_block_str,
                    )
                    .map_err(|e| YamlError::Field {
                        kind: FieldErrorKind::InvalidValue {
                            field: "deployment-block".to_string(),
                            reason: e.to_string(),
                        },
                        location: location.clone(),
                    })?;

                    let raindex_entry = RaindexCfg {
                        document: document.clone(),
                        key: raindex_key.clone(),
                        address,
                        network: Arc::new(network.clone()),
                        subgraph: Arc::new(subgraph.clone()),
                        local_db_remote,
                        label,
                        deployment_block,
                    };

                    if raindexes.contains_key(&raindex_key) {
                        return Err(YamlError::KeyShadowing(
                            raindex_key,
                            "raindexes".to_string(),
                        ));
                    }
                    raindexes.insert(raindex_key, raindex_entry);
                }
            }
        }

        if raindexes.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("raindexes".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(raindexes)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let raindex_section_key = StrictYaml::String("raindexes".to_string());
            let Some(raindex_value) = root_hash.get(&raindex_section_key) else {
                continue;
            };
            let StrictYaml::Hash(ref raindex_hash) = *raindex_value else {
                continue;
            };

            let mut sanitized_raindex_entries: Vec<(String, StrictYaml)> = Vec::new();

            for (raindex_key, raindex_value) in raindex_hash {
                let Some(raindex_key_str) = raindex_key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref raindex_entry_hash) = *raindex_value else {
                    continue;
                };

                let mut sanitized_raindex = Hash::new();

                for allowed_key in ALLOWED_RAINDEX_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = raindex_entry_hash.get(&key_yaml) {
                        sanitized_raindex.insert(key_yaml, v.clone());
                    }
                }

                sanitized_raindex_entries.push((
                    raindex_key_str.to_string(),
                    StrictYaml::Hash(sanitized_raindex),
                ));
            }

            sanitized_raindex_entries.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_raindex_hash = Hash::new();
            for (key, value) in sanitized_raindex_entries {
                new_raindex_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(raindex_section_key, StrictYaml::Hash(new_raindex_hash));
        }

        Ok(())
    }
}

impl Default for RaindexCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            address: Address::ZERO,
            network: Arc::new(NetworkCfg::default()),
            subgraph: Arc::new(SubgraphCfg::default()),
            local_db_remote: None,
            label: None,
            deployment_block: 0,
        }
    }
}
impl PartialEq for RaindexCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.address == other.address
            && self.network == other.network
            && self.subgraph == other.subgraph
            && self.local_db_remote == other.local_db_remote
            && self.label == other.label
            && self.deployment_block == other.deployment_block
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseRaindexConfigSourceError {
    #[error("Failed to parse address")]
    AddressParseError(FromHexError),
    #[error("Network not found for Raindex: {0}")]
    NetworkNotFoundError(String),
    #[error("Subgraph not found: {0}")]
    SubgraphNotFoundError(String),
    #[error("Failed to parse deployment block: {0}")]
    DeploymentBlockParseError(ParseIntError),
}

impl ParseRaindexConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseRaindexConfigSourceError::AddressParseError(err) =>
                format!("The raindex address in your YAML configuration is invalid. Please provide a valid EVM address: {}", err),
            ParseRaindexConfigSourceError::NetworkNotFoundError(network) =>
                format!("The network '{}' specified for this raindex was not found in your YAML configuration. Please define this network or use an existing one.", network),
            ParseRaindexConfigSourceError::SubgraphNotFoundError(subgraph) =>
                format!("The subgraph '{}' specified for this raindex was not found in your YAML configuration. Please define this subgraph or use an existing one.", subgraph),
            ParseRaindexConfigSourceError::DeploymentBlockParseError(err) =>
                format!("The deployment block in your raindex configuration must be a valid number: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_parse_raindexes_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
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
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
test: test
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("subgraphs".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'subgraphs' in root"
        );

        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
test: test
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("raindexes".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'raindexes' in root"
        );

        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("address".to_string()),
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'address' in raindex 'TestRaindex'"
        );

        let yaml = r#"
networks:
    SomeNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "network".to_string(),
                    reason: "Network 'TestNetwork' not found".to_string(),
                },
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
        assert_eq!(error.to_readable_msg(), "Invalid value for field 'network' in raindex 'TestRaindex': Network 'TestNetwork' not found");

        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    SomeSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "subgraph".to_string(),
                    reason: "Subgraph 'TestSubgraph' not found".to_string(),
                },
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
        assert_eq!(error.to_readable_msg(), "Invalid value for field 'subgraph' in raindex 'TestRaindex': Subgraph 'TestSubgraph' not found");
    }

    #[test]
    fn test_parse_raindexes_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    RaindexOne:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345
"#;
        let yaml_two = r#"
raindexes:
    RaindexTwo:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 67890
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let raindexes = RaindexCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(raindexes.len(), 2);
        assert!(raindexes.contains_key("RaindexOne"));
        assert!(raindexes.contains_key("RaindexTwo"));

        assert_eq!(
            raindexes.get("RaindexOne").unwrap().address.to_string(),
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(
            raindexes.get("RaindexTwo").unwrap().address.to_string(),
            "0x0987654321098765432109876543210987654321"
        );
    }

    #[test]
    fn test_parse_raindexes_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    DuplicateRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345
"#;
        let yaml_two = r#"
raindexes:
    DuplicateRaindex:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 67890
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = RaindexCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateRaindex".to_string(), "raindexes".to_string())
        );
        assert_eq!(error.to_readable_msg(), "The key 'DuplicateRaindex' is defined multiple times in your YAML configuration at raindexes");
    }

    #[test]
    fn test_parse_raindex_from_yaml_network_key() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    mainnet: https://subgraph.com
raindexes:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        local-db-remote: mainnet
        deployment-block: 12345
"#;

        let documents = vec![get_document(yaml)];
        let network_key = RaindexCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    mainnet: https://subgraph.com
raindexes:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        deployment-block: 12345
        local-db-remote: mainnet
"#;
        let documents = vec![get_document(yaml)];
        let network_key = RaindexCfg::parse_network_key(documents, "mainnet").unwrap();
        assert_eq!(network_key, "mainnet");
    }

    #[test]
    fn test_parse_network_key() {
        let yaml = r#"
raindexes: test
"#;
        let error = RaindexCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("network for raindex 'order1'".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'network for raindex 'order1'' in root"
        );

        let yaml = r#"
raindexes:
  - test
"#;
        let error = RaindexCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("network for raindex 'order1'".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'network for raindex 'order1'' in root"
        );

        let yaml = r#"
raindexes:
  - test: test
"#;
        let error = RaindexCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("network for raindex 'order1'".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'network for raindex 'order1'' in root"
        );
    }

    #[test]
    fn test_deployment_block_missing() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployment-block".to_string()),
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'deployment-block' in raindex 'TestRaindex'"
        );
    }

    #[test]
    fn test_deployment_block_valid_values() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex1:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 0
    TestRaindex2:
        address: 0x0987654321098765432109876543210987654321
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 18446744073709551615
    TestRaindex3:
        address: 0x1111111111111111111111111111111111111111
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 12345678
"#;
        let raindexes = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        assert_eq!(raindexes.len(), 3);
        assert_eq!(raindexes.get("TestRaindex1").unwrap().deployment_block, 0);
        assert_eq!(
            raindexes.get("TestRaindex2").unwrap().deployment_block,
            18446744073709551615
        );
        assert_eq!(
            raindexes.get("TestRaindex3").unwrap().deployment_block,
            12345678
        );
    }

    #[test]
    fn test_deployment_block_negative_number() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: -1
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason: "Failed to parse deployment block: invalid digit found in string"
                        .to_string(),
                },
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
    }

    #[test]
    fn test_deployment_block_too_large() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 18446744073709551616
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason:
                        "Failed to parse deployment block: number too large to fit in target type"
                            .to_string(),
                },
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
    }

    #[test]
    fn test_deployment_block_non_numeric() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: abc123
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason: "Failed to parse deployment block: invalid digit found in string"
                        .to_string(),
                },
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
    }

    #[test]
    fn test_deployment_block_decimal() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 123.45
"#;
        let error = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "deployment-block".to_string(),
                    reason: "Failed to parse deployment block: invalid digit found in string"
                        .to_string(),
                },
                location: "raindex 'TestRaindex'".to_string(),
            }
        );
    }

    #[test]
    fn test_raindex_local_db_remote_absent_defaults_to_raindex_key() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
local-db-remotes:
    TestRaindex: https://example.com/localdb/TestRaindex
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        deployment-block: 123
"#;
        let raindexes = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let raindex_cfg = raindexes.get("TestRaindex").unwrap();
        let remote = raindex_cfg
            .local_db_remote
            .as_ref()
            .expect("expected remote");
        assert_eq!(remote.key, "TestRaindex");
        assert_eq!(
            remote.url.to_string(),
            "https://example.com/localdb/TestRaindex"
        );
    }

    #[test]
    fn test_raindex_local_db_remote_resolves() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
local-db-remotes:
    mainnet: https://example.com/localdb/mainnet
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: mainnet
        deployment-block: 123
"#;
        let raindexes = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let raindex_cfg = raindexes.get("TestRaindex").unwrap();
        let remote = raindex_cfg
            .local_db_remote
            .as_ref()
            .expect("expected remote");
        assert_eq!(remote.key, "mainnet");
        assert_eq!(
            remote.url.to_string(),
            "https://example.com/localdb/mainnet"
        );
    }

    #[test]
    fn test_raindex_local_db_remote_not_found() {
        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: missing
        deployment-block: 123
"#;
        let raindexes = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let raindex_cfg = raindexes.get("TestRaindex").unwrap();
        assert!(raindex_cfg.local_db_remote.is_none());

        let yaml = r#"
networks:
    TestNetwork:
        rpcs:
            - https://rpc.com
        chain-id: 1
subgraphs:
    TestSubgraph: https://subgraph.com
raindexes:
    TestRaindex:
        address: 0x1234567890123456789012345678901234567890
        network: TestNetwork
        subgraph: TestSubgraph
        local-db-remote: missing
        deployment-block: 123
"#;
        let raindexes = RaindexCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        let raindex_cfg = raindexes.get("TestRaindex").unwrap();
        assert!(raindex_cfg.local_db_remote.is_none());
    }

    #[test]
    fn test_sanitize_drops_unknown_keys() {
        let yaml = r#"
raindexes:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        subgraph: mainnet
        deployment-block: 12345
        unknown-key: should-be-removed
        another-unknown: also-removed
"#;
        let doc = get_document(yaml);
        RaindexCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let raindexes = root
            .get(&StrictYaml::String("raindexes".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let mainnet = raindexes
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(mainnet.contains_key(&StrictYaml::String("address".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("network".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("subgraph".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("deployment-block".to_string())));
        assert!(!mainnet.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!mainnet.contains_key(&StrictYaml::String("another-unknown".to_string())));
    }

    #[test]
    fn test_sanitize_preserves_all_allowed_keys() {
        let yaml = r#"
raindexes:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        subgraph: mainnet
        local-db-remote: mainnet
        label: Mainnet Raindex
        deployment-block: 12345
"#;
        let doc = get_document(yaml);
        RaindexCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let raindexes = root
            .get(&StrictYaml::String("raindexes".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let mainnet = raindexes
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(
            mainnet.get(&StrictYaml::String("address".to_string())),
            Some(&StrictYaml::String(
                "0x1234567890123456789012345678901234567890".to_string()
            ))
        );
        assert_eq!(
            mainnet.get(&StrictYaml::String("network".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            mainnet.get(&StrictYaml::String("subgraph".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            mainnet.get(&StrictYaml::String("local-db-remote".to_string())),
            Some(&StrictYaml::String("mainnet".to_string()))
        );
        assert_eq!(
            mainnet.get(&StrictYaml::String("label".to_string())),
            Some(&StrictYaml::String("Mainnet Raindex".to_string()))
        );
        assert_eq!(
            mainnet.get(&StrictYaml::String("deployment-block".to_string())),
            Some(&StrictYaml::String("12345".to_string()))
        );
    }

    #[test]
    fn test_sanitize_drops_non_hash_raindex_entries() {
        let yaml = r#"
raindexes:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        deployment-block: 12345
    invalid-string: just-a-string
"#;
        let doc = get_document(yaml);
        RaindexCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let raindexes = root
            .get(&StrictYaml::String("raindexes".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(raindexes.contains_key(&StrictYaml::String("mainnet".to_string())));
        assert!(!raindexes.contains_key(&StrictYaml::String("invalid-string".to_string())));
    }

    #[test]
    fn test_sanitize_sorts_raindexes_lexicographically() {
        let yaml = r#"
raindexes:
    zebra:
        address: 0x1111111111111111111111111111111111111111
        deployment-block: 1
    alpha:
        address: 0x2222222222222222222222222222222222222222
        deployment-block: 2
    mainnet:
        address: 0x3333333333333333333333333333333333333333
        deployment-block: 3
"#;
        let doc = get_document(yaml);
        RaindexCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let raindexes = root
            .get(&StrictYaml::String("raindexes".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let keys: Vec<_> = raindexes.keys().map(|k| k.as_str().unwrap()).collect();
        assert_eq!(keys, vec!["alpha", "mainnet", "zebra"]);
    }

    #[test]
    fn test_sanitize_handles_missing_raindexes_section() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
"#;
        let doc = get_document(yaml);
        RaindexCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        assert!(!root.contains_key(&StrictYaml::String("raindexes".to_string())));
    }

    #[test]
    fn test_sanitize_handles_non_hash_root() {
        let yaml = r#"just-a-string"#;
        let doc = get_document(yaml);
        RaindexCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        assert!(doc_read.as_str().is_some());
    }

    #[test]
    fn test_sanitize_skips_non_hash_raindexes_section() {
        let yaml = r#"
raindexes: not-a-hash
"#;
        let doc = get_document(yaml);
        RaindexCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let raindexes = root
            .get(&StrictYaml::String("raindexes".to_string()))
            .unwrap();
        assert_eq!(raindexes.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_per_document_isolation() {
        let yaml1 = r#"
raindexes:
    from-doc1:
        address: 0x1111111111111111111111111111111111111111
        deployment-block: 1
        extra-key: removed
"#;
        let yaml2 = r#"
raindexes:
    from-doc2:
        address: 0x2222222222222222222222222222222222222222
        deployment-block: 2
        another-extra: also-removed
"#;
        let doc1 = get_document(yaml1);
        let doc2 = get_document(yaml2);

        RaindexCfg::sanitize_documents(&[doc1.clone(), doc2.clone()]).unwrap();

        let doc1_read = doc1.read().unwrap();
        let root1 = doc1_read.as_hash().unwrap();
        let raindexes1 = root1
            .get(&StrictYaml::String("raindexes".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let from_doc1 = raindexes1
            .get(&StrictYaml::String("from-doc1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert!(!from_doc1.contains_key(&StrictYaml::String("extra-key".to_string())));
        assert!(!raindexes1.contains_key(&StrictYaml::String("from-doc2".to_string())));

        let doc2_read = doc2.read().unwrap();
        let root2 = doc2_read.as_hash().unwrap();
        let raindexes2 = root2
            .get(&StrictYaml::String("raindexes".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let from_doc2 = raindexes2
            .get(&StrictYaml::String("from-doc2".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert!(!from_doc2.contains_key(&StrictYaml::String("another-extra".to_string())));
        assert!(!raindexes2.contains_key(&StrictYaml::String("from-doc1".to_string())));
    }
}
