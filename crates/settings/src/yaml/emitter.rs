use super::{
    context::Context, FieldErrorKind, YamlError, YamlParsableHash, YamlParsableString,
    YamlParseableValue,
};
use crate::{
    accounts::AccountCfg, local_db_remotes::LocalDbRemoteCfg, local_db_sync::LocalDbSyncCfg,
    metaboard::MetaboardCfg, remote_networks::RemoteNetworksCfg, remote_tokens::RemoteTokensCfg,
    sentry::Sentry, spec_version::SpecVersion, subgraph::SubgraphCfg, ChartCfg, DeploymentCfg,
    NetworkCfg, OrderBuilderCfg, OrderCfg, RaindexCfg, RainlangCfg, ScenarioCfg, TokenCfg,
};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml, StrictYamlEmitter};

const CANONICAL_ROOT_KEYS: &[&str] = &[
    "version",
    "sentry",
    "networks",
    "subgraphs",
    "metaboards",
    "tokens",
    "rainlangs",
    "raindexes",
    "orders",
    "scenarios",
    "deployments",
    "charts",
    "builder",
    "accounts",
    "remote-networks",
    "remote-tokens",
    "local-db-remotes",
    "local-db-syncs",
];

pub fn validate_and_emit_documents(
    documents: &[Arc<RwLock<StrictYaml>>],
    context: Option<&Context>,
) -> Result<String, YamlError> {
    validate_hash_section::<OrderCfg>(documents, context)?;
    validate_hash_section::<ScenarioCfg>(documents, context)?;
    validate_hash_section::<DeploymentCfg>(documents, context)?;
    validate_hash_section::<NetworkCfg>(documents, context)?;
    validate_hash_section::<SubgraphCfg>(documents, context)?;
    validate_hash_section::<MetaboardCfg>(documents, context)?;
    validate_hash_section::<TokenCfg>(documents, context)?;
    validate_hash_section::<RaindexCfg>(documents, context)?;
    validate_hash_section::<RainlangCfg>(documents, context)?;

    ChartCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    RemoteNetworksCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    AccountCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    LocalDbRemoteCfg::parse_all_from_yaml(documents.to_vec(), context)?;
    LocalDbSyncCfg::parse_all_from_yaml(documents.to_vec(), context)?;

    OrderBuilderCfg::parse_from_yaml_optional(documents.to_vec(), context)?;
    RemoteTokensCfg::parse_from_yaml_optional(documents.to_vec(), context)?;

    validate_string_field::<SpecVersion>(documents)?;
    validate_optional_string_field::<Sentry>(documents)?;

    emit_documents(documents)
}

fn validate_hash_section<T: YamlParsableHash>(
    documents: &[Arc<RwLock<StrictYaml>>],
    context: Option<&Context>,
) -> Result<(), YamlError> {
    match T::parse_all_from_yaml(documents.to_vec(), context) {
        Ok(_) => Ok(()),
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(_),
            ..
        }) => Ok(()),
        Err(e) => Err(e),
    }
}

fn validate_string_field<T: YamlParsableString>(
    documents: &[Arc<RwLock<StrictYaml>>],
) -> Result<(), YamlError> {
    if documents.is_empty() {
        return Ok(());
    }
    match T::parse_from_yaml(documents.to_vec()) {
        Ok(_) => Ok(()),
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing(_),
            ..
        }) => Ok(()),
        Err(e) => Err(e),
    }
}

fn validate_optional_string_field<T: YamlParsableString>(
    documents: &[Arc<RwLock<StrictYaml>>],
) -> Result<(), YamlError> {
    for document in documents {
        T::parse_from_yaml_optional(document.clone())?;
    }
    Ok(())
}

fn deep_merge_hash(base: &mut Hash, incoming: &Hash) {
    for (key, value) in incoming {
        match (base.get(key), value) {
            (Some(StrictYaml::Hash(existing_hash)), StrictYaml::Hash(incoming_hash)) => {
                let mut merged = existing_hash.clone();
                deep_merge_hash(&mut merged, incoming_hash);
                base.insert(key.clone(), StrictYaml::Hash(merged));
            }
            _ => {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

pub fn emit_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<String, YamlError> {
    let mut merged_hash = Hash::new();

    for document in documents {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        if let StrictYaml::Hash(ref hash) = *document_read {
            deep_merge_hash(&mut merged_hash, hash);
        }
    }

    let mut ordered_hash = Hash::new();
    for key_str in CANONICAL_ROOT_KEYS {
        let key = StrictYaml::String((*key_str).to_string());
        if let Some(value) = merged_hash.remove(&key) {
            ordered_hash.insert(key, value);
        }
    }

    let merged_doc = StrictYaml::Hash(ordered_hash);
    let mut out_str = String::new();
    let mut emitter = StrictYamlEmitter::new(&mut out_str);
    emitter.dump(&merged_doc)?;

    let out_str = if out_str.starts_with("---") {
        out_str.trim_start_matches("---").trim_start().to_string()
    } else {
        out_str
    };

    Ok(out_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::tests::get_document;

    #[test]
    fn test_emit_single_document() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("networks:"));
        assert!(output.contains("mainnet:"));
    }

    #[test]
    fn test_emit_multiple_documents_merges() {
        let yaml1 = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#;
        let yaml2 = r#"
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
"#;
        let result = validate_and_emit_documents(&[get_document(yaml1), get_document(yaml2)], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("networks:"));
        assert!(output.contains("tokens:"));
    }

    #[test]
    fn test_emit_duplicate_key_causes_error() {
        let yaml1 = r#"
networks:
    mainnet:
        rpcs:
            - https://old-rpc.com
        chain-id: 1
"#;
        let yaml2 = r#"
networks:
    mainnet:
        rpcs:
            - https://new-rpc.com
        chain-id: 1
"#;
        let error = validate_and_emit_documents(&[get_document(yaml1), get_document(yaml2)], None)
            .unwrap_err();
        assert_eq!(
            error,
            YamlError::KeyShadowing("mainnet".to_string(), "networks".to_string())
        );
    }

    #[test]
    fn test_emit_strips_yaml_prefix() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.starts_with("---"));
    }

    #[test]
    fn test_emit_empty_documents() {
        let result = validate_and_emit_documents(&[], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.trim().is_empty() || output.trim() == "{}");
    }

    #[test]
    fn test_emit_non_hash_document_skipped() {
        let yaml = r#"
- item1
- item2
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_complete_yaml() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
rainlangs:
    registry1:
        network: mainnet
        address: 0x0000000000000000000000000000000000000001
subgraphs:
    sg1: https://api.thegraph.com/subgraphs
raindexes:
    ob1:
        network: mainnet
        address: 0x0000000000000000000000000000000000000002
        subgraph: sg1
        deployment-block: 1
orders:
    order1:
        rainlang: registry1
        raindex: ob1
        inputs:
            - token: weth
        outputs:
            - token: weth
scenarios:
    scenario1:
        rainlang: registry1
        bindings:
            key: value
deployments:
    deploy1:
        order: order1
        scenario: scenario1
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("networks:"));
        assert!(output.contains("tokens:"));
        assert!(output.contains("rainlangs:"));
        assert!(output.contains("orders:"));
        assert!(output.contains("scenarios:"));
        assert!(output.contains("deployments:"));
    }

    #[test]
    fn test_validate_minimal_yaml() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_yaml() {
        let result = validate_and_emit_documents(&[], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.trim().is_empty() || output.trim() == "{}");
    }

    #[test]
    fn test_validate_invalid_network_chain_id() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: not-a-number
"#;
        let error = validate_and_emit_documents(&[get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "chain-id".to_string(),
                    reason: "invalid digit found in string".to_string()
                },
                location: "network 'mainnet'".to_string()
            }
        );
    }

    #[test]
    fn test_validate_invalid_network_rpc_url() {
        use crate::network::ParseNetworkConfigSourceError;

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - not-a-valid-url
        chain-id: 1
"#;
        let error = validate_and_emit_documents(&[get_document(yaml)], None).unwrap_err();
        assert!(matches!(
            error,
            YamlError::ParseNetworkConfigSourceError(ParseNetworkConfigSourceError::RpcParseError(
                _
            ))
        ));
    }

    #[test]
    fn test_validate_invalid_token_address() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    weth:
        network: mainnet
        address: invalid-address
        decimals: 18
"#;
        let error = validate_and_emit_documents(&[get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "address".to_string(),
                    reason: "Failed to parse address".to_string()
                },
                location: "token 'weth'".to_string()
            }
        );
    }

    #[test]
    fn test_validate_invalid_rainlang_address() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        network: mainnet
        address: invalid-address
"#;
        let error = validate_and_emit_documents(&[get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "address".to_string(),
                    reason: "Failed to parse address".to_string()
                },
                location: "rainlang 'registry1'".to_string()
            }
        );
    }

    #[test]
    fn test_validate_invalid_order_missing_rainlang() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
orders:
    order1:
        rainlang: nonexistent
        inputs:
            - token: weth
        outputs:
            - token: weth
"#;
        let error = validate_and_emit_documents(&[get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "rainlangs".to_string(),
                    reason: "Missing required field 'rainlangs' in root".to_string()
                },
                location: "root".to_string(),
            }
        );
    }

    #[test]
    fn test_validate_spec_version_valid() {
        let yaml = format!(
            r#"
version: {}
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#,
            SpecVersion::current()
        );
        let result = validate_and_emit_documents(&[get_document(&yaml)], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_spec_version_missing_ok() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sentry_valid() {
        let yaml = r#"
sentry: https://sentry.example.com/123
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sentry_missing_ok() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_root_key_dropped() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
unknown-key: some-value
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("networks:"));
        assert!(!output.contains("unknown-key"));
    }

    #[test]
    fn test_emit_canonical_order() {
        let yaml = r#"
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        network: mainnet
        address: 0x0000000000000000000000000000000000000001
"#;
        let result = validate_and_emit_documents(&[get_document(yaml)], None);
        assert!(result.is_ok());
        let output = result.unwrap();
        let networks_pos = output.find("networks:").unwrap();
        let tokens_pos = output.find("tokens:").unwrap();
        let rainlangs_pos = output.find("rainlangs:").unwrap();
        assert!(
            networks_pos < tokens_pos,
            "networks should come before tokens"
        );
        assert!(
            tokens_pos < rainlangs_pos,
            "tokens should come before rainlangs"
        );
    }

    #[test]
    fn test_emit_deep_merges_hash_sections() {
        let yaml1 = r#"
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        network: mainnet
        address: 0x0000000000000000000000000000000000000001
"#;
        let yaml2 = r#"
rainlangs:
    registry2:
        network: mainnet
        address: 0x0000000000000000000000000000000000000002
"#;
        let result = emit_documents(&[get_document(yaml1), get_document(yaml2)]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("registry1:"), "registry1 should be present");
        assert!(output.contains("registry2:"), "registry2 should be present");
        assert!(output.contains("0x0000000000000000000000000000000000000001"));
        assert!(output.contains("0x0000000000000000000000000000000000000002"));
    }

    // Re-parse emitted output into a StrictYaml document so structural/value
    // assertions are exact rather than substring presence checks.
    fn reparse(output: &str) -> StrictYaml {
        strict_yaml_rust::StrictYamlLoader::load_from_str(output).unwrap()[0].clone()
    }

    // Collect the ordered root-level keys of an emitted document.
    fn root_keys(output: &str) -> Vec<String> {
        let doc = reparse(output);
        let StrictYaml::Hash(ref hash) = doc else {
            panic!("expected root hash, got: {output}");
        };
        hash.keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect()
    }

    #[test]
    fn test_emit_later_document_scalar_overwrites_earlier() {
        // Two documents collide on networks.mainnet.chain-id with scalar values.
        // The merge is last-writer-wins: the later document's value must win.
        // (A first-writer-wins merge would yield "1" instead of "137".)
        let yaml1 = r#"
networks:
    mainnet:
        chain-id: 1
"#;
        let yaml2 = r#"
networks:
    mainnet:
        chain-id: 137
"#;
        let output = emit_documents(&[get_document(yaml1), get_document(yaml2)]).unwrap();
        let doc = reparse(&output);
        assert_eq!(
            doc["networks"]["mainnet"]["chain-id"].as_str(),
            Some("137"),
            "the later document's scalar must overwrite the earlier one"
        );
    }

    #[test]
    fn test_emit_canonical_order_full_sequence() {
        // Provide root sections deliberately out of canonical order, including
        // `version` and `sentry` (which the relative-position test omits). The
        // emitted root keys must be the exact CANONICAL_ROOT_KEYS subsequence,
        // i.e. version before sentry before networks ... before deployments.
        let yaml = r#"
deployments:
    deploy1:
        order: order1
        scenario: scenario1
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
sentry: https://sentry.example.com/123
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
version: "6"
scenarios:
    scenario1:
        bindings:
            key: value
"#;
        let output = emit_documents(&[get_document(yaml)]).unwrap();
        let keys = root_keys(&output);
        assert_eq!(
            keys,
            vec![
                "version",
                "sentry",
                "networks",
                "tokens",
                "scenarios",
                "deployments",
            ],
            "root sections must be emitted in canonical order"
        );
    }

    #[test]
    fn test_emit_unknown_root_key_fully_dropped_exact() {
        // Keys outside CANONICAL_ROOT_KEYS are dropped entirely; only canonical
        // keys survive, asserted as the exact remaining key set.
        let yaml = r#"
unknown-key: some-value
networks:
    mainnet:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
another-unknown:
    nested: value
"#;
        let output = emit_documents(&[get_document(yaml)]).unwrap();
        let keys = root_keys(&output);
        assert_eq!(
            keys,
            vec!["networks"],
            "only canonical keys survive; unknown roots are dropped"
        );
    }

    #[test]
    fn test_emit_strips_prefix_exact_first_line() {
        // After stripping the leading `---` document marker, no leading
        // whitespace must remain and the first content line must be the first
        // canonical section header verbatim. (A strip that removed only `---`
        // but not the trailing newline would leave a blank first line.)
        let yaml = r#"
networks:
    mainnet:
        chain-id: 1
"#;
        let output = emit_documents(&[get_document(yaml)]).unwrap();
        assert!(
            !output.starts_with("---"),
            "leading --- marker must be stripped"
        );
        assert!(
            !output.starts_with(char::is_whitespace),
            "stripped output must not start with whitespace"
        );
        assert_eq!(
            output.lines().next(),
            Some("networks:"),
            "first line after strip must be the networks section header"
        );
    }
}
