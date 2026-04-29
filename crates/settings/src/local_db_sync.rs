use crate::utils::{parse_positive_u32, parse_positive_u64};
use crate::yaml::{
    context::Context, default_document, optional_hash, require_string, FieldErrorKind, YamlError,
    YamlParsableHash,
};

const ALLOWED_LOCAL_DB_SYNC_KEYS: [&str; 8] = [
    "batch-size",
    "bootstrap-block-threshold",
    "finality-depth",
    "max-concurrent-batches",
    "rate-limit-delay-ms",
    "retry-attempts",
    "retry-delay-ms",
    "sync-interval-ms",
];
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct LocalDbSyncCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub batch_size: u32,
    pub max_concurrent_batches: u32,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub rate_limit_delay_ms: u64,
    pub finality_depth: u32,
    pub bootstrap_block_threshold: u32,
    pub sync_interval_ms: u64,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(LocalDbSyncCfg);

impl LocalDbSyncCfg {
    fn parse_sync_network_settings(
        network_key: &str,
        yaml: &StrictYaml,
    ) -> Result<LocalDbSyncCfg, YamlError> {
        let location = format!("local-db-sync.{}", network_key);
        let batch_size = parse_positive_u32(
            &require_string(yaml, Some("batch-size"), Some(location.clone()))?,
            "batch-size",
            location.clone(),
        )?;
        let max_concurrent_batches = parse_positive_u32(
            &require_string(yaml, Some("max-concurrent-batches"), Some(location.clone()))?,
            "max-concurrent-batches",
            location.clone(),
        )?;
        let retry_attempts = parse_positive_u32(
            &require_string(yaml, Some("retry-attempts"), Some(location.clone()))?,
            "retry-attempts",
            location.clone(),
        )?;
        let retry_delay_ms = parse_positive_u64(
            &require_string(yaml, Some("retry-delay-ms"), Some(location.clone()))?,
            "retry-delay-ms",
            location.clone(),
        )?;
        let rate_limit_delay_ms = parse_positive_u64(
            &require_string(yaml, Some("rate-limit-delay-ms"), Some(location.clone()))?,
            "rate-limit-delay-ms",
            location.clone(),
        )?;
        let finality_depth = parse_positive_u32(
            &require_string(yaml, Some("finality-depth"), Some(location.clone()))?,
            "finality-depth",
            location.clone(),
        )?;
        let bootstrap_block_threshold = parse_positive_u32(
            &require_string(
                yaml,
                Some("bootstrap-block-threshold"),
                Some(location.clone()),
            )?,
            "bootstrap-block-threshold",
            location.clone(),
        )?;
        let sync_interval_ms = parse_positive_u64(
            &require_string(yaml, Some("sync-interval-ms"), Some(location.clone()))?,
            "sync-interval-ms",
            location,
        )?;

        Ok(LocalDbSyncCfg {
            document: default_document(),
            key: network_key.to_string(),
            batch_size,
            max_concurrent_batches,
            retry_attempts,
            retry_delay_ms,
            rate_limit_delay_ms,
            finality_depth,
            bootstrap_block_threshold,
            sync_interval_ms,
        })
    }
}

impl YamlParsableHash for LocalDbSyncCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        _context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut syncs: HashMap<String, LocalDbSyncCfg> = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(sync_hash) = optional_hash(&document_read, "local-db-sync") {
                for (key_yaml, settings_yaml) in sync_hash {
                    let key = match key_yaml.as_str() {
                        Some(s) if !s.is_empty() => s.to_string(),
                        Some(_) => {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "sync key".to_string(),
                                    reason: "network name cannot be empty".to_string(),
                                },
                                location: "local-db-sync".to_string(),
                            })
                        }
                        None => {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidType {
                                    field: "sync key".to_string(),
                                    expected: "a string".to_string(),
                                },
                                location: "local-db-sync".to_string(),
                            })
                        }
                    };
                    if syncs.contains_key(&key) {
                        return Err(YamlError::KeyShadowing(key, "local-db-sync".to_string()));
                    }

                    let mut cfg = LocalDbSyncCfg::parse_sync_network_settings(&key, settings_yaml)?;
                    cfg.document = document.clone();
                    cfg.key = key.clone();
                    syncs.insert(key, cfg);
                }
            }
        }

        Ok(syncs)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let sync_key = StrictYaml::String("local-db-sync".to_string());
            let Some(sync_value) = root_hash.get(&sync_key) else {
                continue;
            };
            let StrictYaml::Hash(ref sync_hash) = *sync_value else {
                continue;
            };

            let mut sanitized_syncs: Vec<(String, StrictYaml)> = Vec::new();

            for (network_key, network_value) in sync_hash {
                let Some(network_key_str) = network_key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref network_hash) = *network_value else {
                    continue;
                };

                let mut sanitized_network = Hash::new();
                for allowed_key in ALLOWED_LOCAL_DB_SYNC_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = network_hash.get(&key_yaml) {
                        sanitized_network.insert(key_yaml, v.clone());
                    }
                }

                sanitized_syncs.push((
                    network_key_str.to_string(),
                    StrictYaml::Hash(sanitized_network),
                ));
            }

            sanitized_syncs.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_sync_hash = Hash::new();
            for (key, value) in sanitized_syncs {
                new_sync_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(sync_key, StrictYaml::Hash(new_sync_hash));
        }

        Ok(())
    }
}

impl Default for LocalDbSyncCfg {
    fn default() -> Self {
        LocalDbSyncCfg {
            document: default_document(),
            key: String::new(),
            batch_size: 1,
            max_concurrent_batches: 1,
            retry_attempts: 1,
            retry_delay_ms: 1,
            rate_limit_delay_ms: 1,
            finality_depth: 1,
            bootstrap_block_threshold: 1,
            sync_interval_ms: 1,
        }
    }
}

impl PartialEq for LocalDbSyncCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.batch_size == other.batch_size
            && self.max_concurrent_batches == other.max_concurrent_batches
            && self.retry_attempts == other.retry_attempts
            && self.retry_delay_ms == other.retry_delay_ms
            && self.rate_limit_delay_ms == other.rate_limit_delay_ms
            && self.finality_depth == other.finality_depth
            && self.bootstrap_block_threshold == other.bootstrap_block_threshold
            && self.sync_interval_ms == other.sync_interval_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    use strict_yaml_rust::StrictYaml;

    struct FullSyncArgs<'a> {
        network: &'a str,
        batch_size: u32,
        max_concurrent_batches: u32,
        retry_attempts: u32,
        retry_delay_ms: u64,
        rate_limit_delay_ms: u64,
        finality_depth: u32,
        bootstrap_block_threshold: u32,
        sync_interval_ms: u64,
    }

    fn full_sync_yaml(args: FullSyncArgs<'_>) -> String {
        format!(
            r#"
local-db-sync:
  {network}:
    batch-size: {batch_size}
    max-concurrent-batches: {max_concurrent_batches}
    retry-attempts: {retry_attempts}
    retry-delay-ms: {retry_delay_ms}
    rate-limit-delay-ms: {rate_limit_delay_ms}
    finality-depth: {finality_depth}
    bootstrap-block-threshold: {bootstrap_block_threshold}
    sync-interval-ms: {sync_interval_ms}
"#,
            network = args.network,
            batch_size = args.batch_size,
            max_concurrent_batches = args.max_concurrent_batches,
            retry_attempts = args.retry_attempts,
            retry_delay_ms = args.retry_delay_ms,
            rate_limit_delay_ms = args.rate_limit_delay_ms,
            finality_depth = args.finality_depth,
            bootstrap_block_threshold = args.bootstrap_block_threshold,
            sync_interval_ms = args.sync_interval_ms,
        )
    }

    #[test]
    fn test_parse_sync_missing_section_is_ok() {
        let yaml = r#"
not-sync:
  some: value
"#;
        let syncs = LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();
        assert!(syncs.is_empty());
    }

    #[test]
    fn test_parse_sync_from_yaml_multiple_files() {
        let yaml_one = full_sync_yaml(FullSyncArgs {
            network: "arbitrum",
            batch_size: 100,
            max_concurrent_batches: 5,
            retry_attempts: 3,
            retry_delay_ms: 50,
            rate_limit_delay_ms: 10,
            finality_depth: 100,
            bootstrap_block_threshold: 25,
            sync_interval_ms: 5000,
        });
        let yaml_two = full_sync_yaml(FullSyncArgs {
            network: "mainnet",
            batch_size: 200,
            max_concurrent_batches: 10,
            retry_attempts: 5,
            retry_delay_ms: 100,
            rate_limit_delay_ms: 20,
            finality_depth: 64,
            bootstrap_block_threshold: 30,
            sync_interval_ms: 10000,
        });

        let documents = vec![get_document(&yaml_one), get_document(&yaml_two)];
        let syncs = LocalDbSyncCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(syncs.len(), 2);
        assert!(syncs.contains_key("arbitrum"));
        assert!(syncs.contains_key("mainnet"));

        let expected_arb = LocalDbSyncCfg {
            document: default_document(),
            key: "arbitrum".to_string(),
            batch_size: 100,
            max_concurrent_batches: 5,
            retry_attempts: 3,
            retry_delay_ms: 50,
            rate_limit_delay_ms: 10,
            finality_depth: 100,
            bootstrap_block_threshold: 25,
            sync_interval_ms: 5000,
        };
        let expected_eth = LocalDbSyncCfg {
            document: default_document(),
            key: "mainnet".to_string(),
            batch_size: 200,
            max_concurrent_batches: 10,
            retry_attempts: 5,
            retry_delay_ms: 100,
            rate_limit_delay_ms: 20,
            finality_depth: 64,
            bootstrap_block_threshold: 30,
            sync_interval_ms: 10000,
        };

        assert_eq!(syncs.get("arbitrum").unwrap(), &expected_arb);
        assert_eq!(syncs.get("mainnet").unwrap(), &expected_eth);
    }

    #[test]
    fn test_parse_sync_from_yaml_duplicate_key() {
        let yaml_one = full_sync_yaml(FullSyncArgs {
            network: "mainnet",
            batch_size: 100,
            max_concurrent_batches: 2,
            retry_attempts: 2,
            retry_delay_ms: 10,
            rate_limit_delay_ms: 5,
            finality_depth: 32,
            bootstrap_block_threshold: 40,
            sync_interval_ms: 5000,
        });
        let yaml_two = full_sync_yaml(FullSyncArgs {
            network: "mainnet",
            batch_size: 101,
            max_concurrent_batches: 3,
            retry_attempts: 3,
            retry_delay_ms: 11,
            rate_limit_delay_ms: 6,
            finality_depth: 33,
            bootstrap_block_threshold: 41,
            sync_interval_ms: 5000,
        });

        let documents = vec![get_document(&yaml_one), get_document(&yaml_two)];
        let error = LocalDbSyncCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("mainnet".to_string(), "local-db-sync".to_string())
        );
    }

    #[test]
    fn test_parse_sync_missing_field() {
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    # retry-delay-ms missing
    rate-limit-delay-ms: 1
    finality-depth: 1
    bootstrap-block-threshold: 2
    sync-interval-ms: 5000
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("retry-delay-ms".to_string()),
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_missing_bootstrap_block_threshold() {
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    retry-delay-ms: 1
    rate-limit-delay-ms: 1
    finality-depth: 1
    sync-interval-ms: 5000
    # bootstrap-block-threshold missing
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("bootstrap-block-threshold".to_string()),
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_invalid_value_non_numeric() {
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: "abc"
    retry-delay-ms: 1
    rate-limit-delay-ms: 1
    finality-depth: 1
    bootstrap-block-threshold: 2
    sync-interval-ms: 5000
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "retry-attempts".to_string(),
                    reason: "invalid digit found in string".to_string()
                },
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_invalid_bootstrap_block_threshold_zero_rejected() {
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    retry-delay-ms: 1
    rate-limit-delay-ms: 1
    finality-depth: 1
    bootstrap-block-threshold: 0
    sync-interval-ms: 5000
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "bootstrap-block-threshold".to_string(),
                    reason: "must be a positive integer".to_string()
                },
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_invalid_value_zero_rejected() {
        let yaml = r#"
local-db-sync:
  devnet:
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    retry-delay-ms: 1
    rate-limit-delay-ms: 0
    finality-depth: 1
    bootstrap-block-threshold: 2
    sync-interval-ms: 5000
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "rate-limit-delay-ms".to_string(),
                    reason: "must be a positive integer".to_string()
                },
                location: "local-db-sync.devnet".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sync_empty_key_rejected() {
        let yaml = r#"
local-db-sync:
  "":
    batch-size: 1
    max-concurrent-batches: 1
    retry-attempts: 1
    retry-delay-ms: 1
    rate-limit-delay-ms: 1
    finality-depth: 1
    bootstrap-block-threshold: 2
    sync-interval-ms: 5000
"#;
        let error =
            LocalDbSyncCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "sync key".to_string(),
                    reason: "network name cannot be empty".to_string()
                },
                location: "local-db-sync".to_string(),
            }
        );
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_keys() {
        let yaml = r#"
local-db-sync:
    mainnet:
        batch-size: 100
        max-concurrent-batches: 5
        retry-attempts: 3
        retry-delay-ms: 50
        rate-limit-delay-ms: 10
        finality-depth: 64
        bootstrap-block-threshold: 25
        sync-interval-ms: 5000
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let document = get_document(yaml);
        LocalDbSyncCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let sync_hash = root_hash
            .get(&StrictYaml::String("local-db-sync".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let mainnet_hash = sync_hash
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(mainnet_hash.len(), 8);
        assert!(mainnet_hash.contains_key(&StrictYaml::String("batch-size".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!mainnet_hash.contains_key(&StrictYaml::String("another-unknown".to_string())));
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
local-db-sync:
    zebra:
        batch-size: 1
        max-concurrent-batches: 1
        retry-attempts: 1
        retry-delay-ms: 1
        rate-limit-delay-ms: 1
        finality-depth: 1
        bootstrap-block-threshold: 1
        sync-interval-ms: 5000
    alpha:
        batch-size: 2
        max-concurrent-batches: 2
        retry-attempts: 2
        retry-delay-ms: 2
        rate-limit-delay-ms: 2
        finality-depth: 2
        bootstrap-block-threshold: 2
        sync-interval-ms: 5000
"#;
        let document = get_document(yaml);
        LocalDbSyncCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let sync_hash = root_hash
            .get(&StrictYaml::String("local-db-sync".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let keys: Vec<&str> = sync_hash.iter().filter_map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["alpha", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_entries() {
        let yaml = r#"
local-db-sync:
    mainnet:
        batch-size: 100
        max-concurrent-batches: 5
        retry-attempts: 3
        retry-delay-ms: 50
        rate-limit-delay-ms: 10
        finality-depth: 64
        bootstrap-block-threshold: 25
        sync-interval-ms: 5000
    invalid: not_a_hash
"#;
        let document = get_document(yaml);
        LocalDbSyncCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let sync_hash = root_hash
            .get(&StrictYaml::String("local-db-sync".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(sync_hash.len(), 1);
        assert!(sync_hash.contains_key(&StrictYaml::String("mainnet".to_string())));
        assert!(!sync_hash.contains_key(&StrictYaml::String("invalid".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_missing_section() {
        let yaml = r#"
other-section: value
"#;
        let document = get_document(yaml);
        LocalDbSyncCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = "just a string";
        let document = get_document(yaml);
        LocalDbSyncCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_section() {
        let yaml = r#"
local-db-sync: not_a_hash
"#;
        let document = get_document(yaml);
        LocalDbSyncCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_per_document_isolation() {
        let yaml_one = r#"
local-db-sync:
    mainnet:
        batch-size: 100
        max-concurrent-batches: 5
        retry-attempts: 3
        retry-delay-ms: 50
        rate-limit-delay-ms: 10
        finality-depth: 64
        bootstrap-block-threshold: 25
        sync-interval-ms: 5000
        unknown: dropped
"#;
        let yaml_two = r#"
local-db-sync:
    polygon:
        batch-size: 200
        max-concurrent-batches: 10
        retry-attempts: 5
        retry-delay-ms: 100
        rate-limit-delay-ms: 20
        finality-depth: 128
        bootstrap-block-threshold: 30
        sync-interval-ms: 5000
"#;
        let doc_one = get_document(yaml_one);
        let doc_two = get_document(yaml_two);
        LocalDbSyncCfg::sanitize_documents(&[doc_one.clone(), doc_two.clone()]).unwrap();

        let syncs = LocalDbSyncCfg::parse_all_from_yaml(vec![doc_one, doc_two], None).unwrap();
        assert_eq!(syncs.len(), 2);
        assert!(syncs.contains_key("mainnet"));
        assert!(syncs.contains_key("polygon"));
    }
}
