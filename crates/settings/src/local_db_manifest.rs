use crate::utils::{parse_positive_u32, parse_positive_u64, parse_url};
use crate::yaml::{
    optional_hash, require_hash, require_string, require_vec, FieldErrorKind, YamlError,
};
use alloy::primitives::{Address, Bytes};
use std::collections::HashMap;
use std::str::FromStr;
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml, StrictYamlEmitter};
use url::Url;

pub const MANIFEST_VERSION: u32 = 1;
// Bumped from 2 to 3 for orderbook_address -> raindex_address column rename
pub const DB_SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDbManifest {
    pub manifest_version: u32,
    pub db_schema_version: u32,
    pub networks: HashMap<String, ManifestNetwork>,
}

impl Default for LocalDbManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestNetwork {
    pub chain_id: u32,
    pub raindexes: Vec<ManifestRaindex>,
}

impl ManifestNetwork {
    pub fn new(chain_id: u32) -> Self {
        Self {
            chain_id,
            raindexes: Vec::new(),
        }
    }

    pub fn push_raindex(&mut self, raindex: ManifestRaindex) {
        self.raindexes.push(raindex);
    }

    fn to_yaml_hash(&self) -> Hash {
        let mut hash = Hash::new();
        hash.insert(
            StrictYaml::String("chain-id".to_string()),
            StrictYaml::String(self.chain_id.to_string()),
        );

        let raindexes_yaml: Vec<StrictYaml> = self
            .raindexes
            .iter()
            .map(ManifestRaindex::to_yaml_hash)
            .collect();

        hash.insert(
            StrictYaml::String("raindexes".to_string()),
            StrictYaml::Array(raindexes_yaml),
        );

        hash
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestRaindex {
    pub address: Address,
    pub dump_url: Url,
    pub end_block: u64,
    pub end_block_hash: Bytes,
    pub end_block_time_ms: u64,
}

impl ManifestRaindex {
    fn to_yaml_hash(&self) -> StrictYaml {
        let mut hash = Hash::new();
        hash.insert(
            StrictYaml::String("address".to_string()),
            StrictYaml::String(self.address.to_string()),
        );
        hash.insert(
            StrictYaml::String("dump-url".to_string()),
            StrictYaml::String(self.dump_url.to_string()),
        );
        hash.insert(
            StrictYaml::String("end-block".to_string()),
            StrictYaml::String(self.end_block.to_string()),
        );
        hash.insert(
            StrictYaml::String("end-block-hash".to_string()),
            StrictYaml::String(self.end_block_hash.to_string()),
        );
        hash.insert(
            StrictYaml::String("end-block-time-ms".to_string()),
            StrictYaml::String(self.end_block_time_ms.to_string()),
        );
        StrictYaml::Hash(hash)
    }
}

impl LocalDbManifest {
    pub fn find(&self, chain_id: u32, address: Address) -> Option<&ManifestRaindex> {
        self.networks
            .values()
            .find(|n| n.chain_id == chain_id)
            .and_then(|n| {
                n.raindexes
                    .iter()
                    .find(|raindex_entry| raindex_entry.address == address)
            })
    }

    pub fn new() -> Self {
        Self {
            manifest_version: MANIFEST_VERSION,
            db_schema_version: DB_SCHEMA_VERSION,
            networks: HashMap::new(),
        }
    }

    pub fn add_network<K: Into<String>>(&mut self, key: K, chain_id: u32) -> Result<(), YamlError> {
        let key = key.into();

        if key.trim().is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "key".to_string(),
                    reason: "network name must not be empty".to_string(),
                },
                location: "manifest.networks".to_string(),
            });
        }

        if chain_id == 0 {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "chain-id".to_string(),
                    reason: "must be greater than zero".to_string(),
                },
                location: format!("manifest.networks.{}", key),
            });
        }

        if self.networks.contains_key(&key) {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "networks".to_string(),
                    reason: format!("network '{}' already exists", key),
                },
                location: "manifest".to_string(),
            });
        }

        self.networks.insert(key, ManifestNetwork::new(chain_id));
        Ok(())
    }

    pub fn push_raindex(
        &mut self,
        network_key: &str,
        raindex: ManifestRaindex,
    ) -> Result<(), YamlError> {
        match self.networks.get_mut(network_key) {
            Some(network) => {
                network.push_raindex(raindex);
                Ok(())
            }
            None => Err(YamlError::Field {
                kind: FieldErrorKind::Missing(network_key.to_string()),
                location: "manifest.networks".to_string(),
            }),
        }
    }

    pub fn to_yaml_node(&self) -> StrictYaml {
        let mut root = Hash::new();
        root.insert(
            StrictYaml::String("manifest-version".to_string()),
            StrictYaml::String(self.manifest_version.to_string()),
        );
        root.insert(
            StrictYaml::String("db-schema-version".to_string()),
            StrictYaml::String(self.db_schema_version.to_string()),
        );

        let mut networks_hash = Hash::new();
        let mut network_keys: Vec<_> = self.networks.keys().cloned().collect();
        network_keys.sort();

        for key in network_keys {
            if let Some(network) = self.networks.get(&key) {
                networks_hash.insert(
                    StrictYaml::String(key),
                    StrictYaml::Hash(network.to_yaml_hash()),
                );
            }
        }

        root.insert(
            StrictYaml::String("networks".to_string()),
            StrictYaml::Hash(networks_hash),
        );

        StrictYaml::Hash(root)
    }

    pub fn to_yaml_string(&self) -> Result<String, YamlError> {
        let document = self.to_yaml_node();
        let mut out_str = String::new();
        let mut emitter = StrictYamlEmitter::new(&mut out_str);
        emitter.dump(&document)?;

        let out_str = if out_str.starts_with("---") {
            out_str.trim_start_matches("---").trim_start().to_string()
        } else {
            out_str
        };

        Ok(out_str)
    }
}

pub fn current_manifest_version() -> u32 {
    MANIFEST_VERSION
}

pub fn is_manifest_version_current(version: u32) -> bool {
    version == MANIFEST_VERSION
}

pub fn parse_manifest_doc(doc: &StrictYaml) -> Result<LocalDbManifest, YamlError> {
    let location_root = "manifest";

    let (manifest_version, db_schema_version) = parse_manifest_header(doc, location_root)?;
    let networks = parse_networks(doc, location_root)?;

    Ok(LocalDbManifest {
        manifest_version,
        db_schema_version,
        networks,
    })
}

fn parse_manifest_header(doc: &StrictYaml, location_root: &str) -> Result<(u32, u32), YamlError> {
    let manifest_version = parse_positive_u32(
        &require_string(
            doc,
            Some("manifest-version"),
            Some(location_root.to_string()),
        )?,
        "manifest-version",
        location_root.to_string(),
    )?;

    // Fail fast if the manifest version is incompatible with the supported version.
    if manifest_version != MANIFEST_VERSION {
        return Err(YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: "manifest-version".to_string(),
                reason: format!(
                    "unsupported manifest version {}, expected {}",
                    manifest_version, MANIFEST_VERSION
                ),
            },
            location: location_root.to_string(),
        });
    }

    let db_schema_version = parse_positive_u32(
        &require_string(
            doc,
            Some("db-schema-version"),
            Some(location_root.to_string()),
        )?,
        "db-schema-version",
        location_root.to_string(),
    )?;

    if db_schema_version != DB_SCHEMA_VERSION {
        return Err(YamlError::Field {
            kind: FieldErrorKind::InvalidValue {
                field: "db-schema-version".to_string(),
                reason: format!(
                    "unsupported database schema version {}, expected {}",
                    db_schema_version, DB_SCHEMA_VERSION
                ),
            },
            location: location_root.to_string(),
        });
    }

    Ok((manifest_version, db_schema_version))
}

fn parse_networks(
    doc: &StrictYaml,
    location_root: &str,
) -> Result<HashMap<String, ManifestNetwork>, YamlError> {
    let mut networks: HashMap<String, ManifestNetwork> = HashMap::new();

    if let Some(networks_hash) = optional_hash(doc, "networks") {
        for (key_yaml, network_yaml) in networks_hash.iter() {
            let network_key = key_yaml
                .as_str()
                .ok_or(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "key".to_string(),
                        expected: "a string".to_string(),
                    },
                    location: "manifest.networks".to_string(),
                })?
                .to_string();
            if network_key.trim().is_empty() {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidValue {
                        field: "key".to_string(),
                        reason: "network name must not be empty".to_string(),
                    },
                    location: "manifest.networks".to_string(),
                });
            }

            let network = parse_single_network(&network_key, network_yaml, location_root)?;
            networks.insert(network_key.clone(), network);
        }
    }

    Ok(networks)
}

fn parse_single_network(
    network_key: &str,
    network_yaml: &StrictYaml,
    location_root: &str,
) -> Result<ManifestNetwork, YamlError> {
    let location_network = format!("{}.networks.{}", location_root, network_key);

    let _network_hash = require_hash(network_yaml, None, Some(location_network.clone()))?;

    let chain_id = parse_positive_u32(
        &require_string(
            network_yaml,
            Some("chain-id"),
            Some(location_network.clone()),
        )?,
        "chain-id",
        location_network.clone(),
    )?;

    let raindexes_yaml = require_vec(network_yaml, "raindexes", Some(location_network.clone()))?;

    let mut raindexes: Vec<ManifestRaindex> = Vec::new();
    for (idx, ob_yaml) in raindexes_yaml.iter().enumerate() {
        raindexes.push(parse_single_raindex(idx, ob_yaml, &location_network)?);
    }

    Ok(ManifestNetwork {
        chain_id,
        raindexes,
    })
}

fn parse_single_raindex(
    idx: usize,
    ob_yaml: &StrictYaml,
    location_network: &str,
) -> Result<ManifestRaindex, YamlError> {
    let location_ob = format!("{}.raindexes[{}]", location_network, idx);

    let address_str = require_string(ob_yaml, Some("address"), Some(location_ob.clone()))?;
    let address = Address::from_str(&address_str).map_err(|e| YamlError::Field {
        kind: crate::yaml::FieldErrorKind::InvalidValue {
            field: "address".to_string(),
            reason: e.to_string(),
        },
        location: location_ob.clone(),
    })?;

    let dump_url_str = require_string(ob_yaml, Some("dump-url"), Some(location_ob.clone()))?;
    let dump_url = parse_url(&dump_url_str, "dump-url", location_ob.clone())?;

    let end_block = parse_positive_u64(
        &require_string(ob_yaml, Some("end-block"), Some(location_ob.clone()))?,
        "end-block",
        location_ob.clone(),
    )?;

    let end_block_hash_str =
        require_string(ob_yaml, Some("end-block-hash"), Some(location_ob.clone()))?;
    let end_block_hash = Bytes::from_str(&end_block_hash_str).map_err(|e| YamlError::Field {
        kind: FieldErrorKind::InvalidValue {
            field: "end-block-hash".to_string(),
            reason: e.to_string(),
        },
        location: location_ob.clone(),
    })?;

    let end_block_time_ms = parse_positive_u64(
        &require_string(
            ob_yaml,
            Some("end-block-time-ms"),
            Some(location_ob.clone()),
        )?,
        "end-block-time-ms",
        location_ob.clone(),
    )?;

    Ok(ManifestRaindex {
        address,
        dump_url,
        end_block,
        end_block_hash,
        end_block_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use strict_yaml_rust::StrictYamlLoader;

    fn load(yaml: &str) -> StrictYaml {
        StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone()
    }

    fn sample_raindex() -> ManifestRaindex {
        ManifestRaindex {
            address: Address::from_str("0x00000000000000000000000000000000000000aa").unwrap(),
            dump_url: Url::parse("https://example.com/dump").unwrap(),
            end_block: 42,
            end_block_hash: Bytes::from_str("0x0abc").unwrap(),
            end_block_time_ms: 1234,
        }
    }

    #[test]
    fn test_manifest_builder_roundtrip_and_determinism() {
        let mut manifest = LocalDbManifest::new();
        manifest.add_network("mainnet", 1).unwrap();
        manifest.add_network("polygon", 137).unwrap();

        let ob_main = ManifestRaindex {
            address: Address::from_str("0x0000000000000000000000000000000000000001").unwrap(),
            dump_url: Url::parse("https://example.com/mainnet").unwrap(),
            end_block: 50,
            end_block_hash: Bytes::from_str("0x0def").unwrap(),
            end_block_time_ms: 2000,
        };
        let ob_polygon = ManifestRaindex {
            address: Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
            dump_url: Url::parse("https://example.com/polygon").unwrap(),
            end_block: 60,
            end_block_hash: Bytes::from_str("0x0fed").unwrap(),
            end_block_time_ms: 3000,
        };

        manifest.push_raindex("mainnet", ob_main.clone()).unwrap();
        manifest
            .push_raindex("polygon", ob_polygon.clone())
            .unwrap();

        let yaml_one = manifest.to_yaml_string().unwrap();
        let yaml_two = manifest.to_yaml_string().unwrap();
        assert_eq!(yaml_one, yaml_two);

        let doc = StrictYamlLoader::load_from_str(&yaml_one).unwrap();
        let reparsed = parse_manifest_doc(&doc[0]).unwrap();
        assert_eq!(manifest, reparsed);
    }

    #[test]
    fn test_push_raindex_without_network_errors() {
        let mut manifest = LocalDbManifest::new();
        let raindex = sample_raindex();
        let err = manifest
            .push_raindex("missing", raindex)
            .expect_err("should error when network missing");
        match err {
            YamlError::Field {
                kind: FieldErrorKind::Missing(field),
                location,
            } => {
                assert_eq!(field, "missing");
                assert_eq!(location, "manifest.networks");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_add_network_duplicate_key_errors() {
        let mut manifest = LocalDbManifest::new();
        manifest.add_network("mainnet", 1).unwrap();
        let err = manifest
            .add_network("mainnet", 1)
            .expect_err("duplicate network should error");
        match err {
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { field, reason },
                location,
            } => {
                assert_eq!(field, "networks");
                assert!(reason.contains("already exists"));
                assert_eq!(location, "manifest");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_add_network_rejects_zero_chain_id() {
        let mut manifest = LocalDbManifest::new();
        let err = manifest
            .add_network("goerli", 0)
            .expect_err("zero chain id should error");
        match err {
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { field, reason },
                location,
            } => {
                assert_eq!(field, "chain-id");
                assert_eq!(reason, "must be greater than zero");
                assert_eq!(location, "manifest.networks.goerli");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_header_helper_ok_and_incompatible() {
        // OK header
        let yaml_ok = r#"
manifest-version: 1
db-schema-version: 3
networks: {}
"#;
        let doc = load(yaml_ok);
        let (mv, sv) = parse_manifest_header(&doc, "manifest").expect("header parses");
        assert_eq!(mv, 1);
        assert_eq!(sv, 3);

        // Incompatible manifest version
        let yaml_bad = r#"
manifest-version: 999
db-schema-version: 3
networks: {}
"#;
        let err = parse_manifest_header(&load(yaml_bad), "manifest").unwrap_err();
        match err {
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { field, .. },
                location,
            } => {
                assert_eq!(field, "manifest-version");
                assert_eq!(location, "manifest");
            }
            other => panic!("unexpected error: {other:?}"),
        }

        // Incompatible db schema version
        let yaml_bad_schema = r#"
manifest-version: 1
db-schema-version: 999
networks: {}
"#;
        let err = parse_manifest_header(&load(yaml_bad_schema), "manifest").unwrap_err();
        match err {
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { field, .. },
                location,
            } => {
                assert_eq!(field, "db-schema-version");
                assert_eq!(location, "manifest");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_networks_helper_empty_key_rejected() {
        let yaml = r#"
manifest-version: 1
db-schema-version: 3
networks:
  "":
    chain-id: 1
    raindexes: []
"#;
        let err = parse_networks(&load(yaml), "manifest").unwrap_err();
        match err {
            YamlError::Field { kind, location } => match kind {
                FieldErrorKind::InvalidValue { field, .. } => {
                    assert_eq!(field, "key");
                    assert_eq!(location, "manifest.networks");
                }
                other => panic!("unexpected field error kind: {other:?}"),
            },
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_raindex_helper_ok_and_bad_address() {
        let base_loc = "manifest.networks.mainnet";

        // Valid raindex
        let ob_ok = r#"
address: "0x0000000000000000000000000000000000000001"
dump-url: "http://example.com"
end-block: 1
end-block-hash: "0x0abc"
end-block-time-ms: 1
"#;
        let raindex_doc = load(ob_ok);
        let raindex_entry =
            parse_single_raindex(0, &raindex_doc, base_loc).expect("raindex parses");
        assert_eq!(raindex_entry.end_block, 1);
        assert_eq!(raindex_entry.end_block_time_ms, 1);

        // Bad address
        let ob_bad = r#"
address: "not-an-address"
dump-url: "http://example.com"
end-block: 1
end-block-hash: "0x0abc"
end-block-time-ms: 1
"#;
        let err = parse_single_raindex(0, &load(ob_bad), base_loc).unwrap_err();
        match err {
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { field, .. },
                location,
            } => {
                assert_eq!(field, "address");
                assert_eq!(location, "manifest.networks.mainnet.raindexes[0]");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_missing_manifest_version() {
        let yaml = r#"
db-schema-version: 3
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_missing_db_schema_version() {
        let yaml = r#"
manifest-version: 1
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_missing_networks_defaults_to_empty() {
        let yaml = r#"
manifest-version: 1
db-schema-version: 3
"#;
        let manifest = parse_manifest_doc(&load(yaml)).unwrap();
        assert!(manifest.networks.is_empty());
    }

    #[test]
    fn test_manifest_doc_rejects_mismatched_db_schema_version() {
        let yaml = r#"
manifest-version: 1
db-schema-version: 999
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        match err {
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { field, .. },
                location,
            } => {
                assert_eq!(field, "db-schema-version");
                assert_eq!(location, "manifest");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_zero_or_invalid_manifest_and_schema_versions() {
        let yaml_zero_manifest = r#"
manifest-version: 0
db-schema-version: 3
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml_zero_manifest)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let yaml_zero_schema = r#"
manifest-version: 1
db-schema-version: 0
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml_zero_schema)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_network_missing_chain_id_and_invalid() {
        let yaml_missing = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet: {}
"#;
        let err = parse_manifest_doc(&load(yaml_missing)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let yaml_zero = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 0
    raindexes: []
"#;
        let err = parse_manifest_doc(&load(yaml_zero)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_raindexes_required_and_type() {
        let yaml_missing = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
"#;
        let err = parse_manifest_doc(&load(yaml_missing)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));

        let yaml_non_list = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes: {}
"#;
        let err = parse_manifest_doc(&load(yaml_non_list)).unwrap_err();
        assert!(matches!(err, YamlError::Field { .. }));
    }

    #[test]
    fn test_raindex_missing_fields() {
        // Full, valid baseline
        let good = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block: 1
        end-block-hash: "0x0abc"
        end-block-time-ms: 1
"#;
        assert!(parse_manifest_doc(&load(good)).is_ok());

        // Now omit each required field individually
        let missing_address = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - dump-url: "http://example.com"
        end-block: 1
        end-block-hash: "0x0abc"
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_address)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_dump = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        end-block: 1
        end-block-hash: "0x0abc"
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_dump)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_end_block = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block-hash: "0x0abc"
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_end_block)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_end_hash = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block: 1
        end-block-time-ms: 1
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_end_hash)).unwrap_err(),
            YamlError::Field { .. }
        ));

        let missing_end_time = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com"
        end-block: 1
        end-block-hash: "0x0abc"
"#;
        assert!(matches!(
            parse_manifest_doc(&load(missing_end_time)).unwrap_err(),
            YamlError::Field { .. }
        ));
    }

    #[test]
    fn test_find_across_networks_and_negatives() {
        let yaml = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x1111111111111111111111111111111111111111"
        dump-url: "http://example.com/a"
        end-block: 10
        end-block-hash: "0x0a"
        end-block-time-ms: 100
  other:
    chain-id: 2
    raindexes:
      - address: "0x2222222222222222222222222222222222222222"
        dump-url: "http://example.com/b"
        end-block: 20
        end-block-hash: "0x0b"
        end-block-time-ms: 200
"#;
        let m = parse_manifest_doc(&load(yaml)).unwrap();

        assert!(m
            .find(
                1,
                Address::from_str("0x1111111111111111111111111111111111111111").unwrap()
            )
            .is_some());
        assert!(m
            .find(
                2,
                Address::from_str("0x2222222222222222222222222222222222222222").unwrap()
            )
            .is_some());

        assert!(m
            .find(
                1,
                Address::from_str("0xdeadbeef00000000000000000000000000000000").unwrap()
            )
            .is_none());
        assert!(m
            .find(
                999,
                Address::from_str("0x1111111111111111111111111111111111111111").unwrap()
            )
            .is_none());
    }

    #[test]
    fn test_manifest_version_helpers() {
        assert!(is_manifest_version_current(current_manifest_version()));
        assert!(!is_manifest_version_current(0));
    }

    #[test]
    fn test_incompatible_manifest_version_rejected() {
        let yaml = r#"
manifest-version: 999
db-schema-version: 3
networks: {}
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        match err {
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue { field, reason: _ },
                location,
            } => {
                assert_eq!(field, "manifest-version");
                assert_eq!(location, "manifest");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_empty_network_key_is_rejected() {
        let yaml = r#"
manifest-version: 1
db-schema-version: 3
networks:
  "":
    chain-id: 1
    raindexes: []
"#;
        let err = parse_manifest_doc(&load(yaml)).unwrap_err();
        match err {
            YamlError::Field { kind, location } => match kind {
                FieldErrorKind::InvalidValue { field, reason: _ } => {
                    assert_eq!(field, "key");
                    assert_eq!(location, "manifest.networks");
                }
                other => panic!("unexpected field error kind: {other:?}"),
            },
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
