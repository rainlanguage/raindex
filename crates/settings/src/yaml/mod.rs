pub mod cache;
pub mod context;
pub mod dotrain;
pub mod emitter;
pub mod raindex;

use crate::{
    remote_networks::ParseRemoteNetworksError, remote_tokens::ParseRemoteTokensError, NetworkCfg,
    ParseDeploymentConfigSourceError, ParseNetworkConfigSourceError, ParseOrderConfigSourceError,
    ParseRaindexConfigSourceError, ParseRainlangConfigSourceError, ParseScenarioConfigSourceError,
    ParseTokenConfigSourceError, TokenCfg,
};
use alloy::primitives::ruint::ParseError as RuintParseError;
use context::{Context, ContextError, ContextProfile};
use dotrain::DotrainYaml;
use raindex::RaindexYaml;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYamlEmitter;
use strict_yaml_rust::{
    strict_yaml::{Array, Hash},
    EmitError, ScanError, StrictYaml, StrictYamlLoader,
};
use thiserror::Error;
use url::ParseError as UrlParseError;

pub trait ValidationConfig {
    fn should_validate_networks(&self) -> bool;
    fn should_validate_remote_networks(&self) -> bool;
    fn should_validate_tokens(&self) -> bool;
    fn should_validate_remote_tokens(&self) -> bool;
    fn should_validate_subgraphs(&self) -> bool;
    fn should_validate_local_db_remotes(&self) -> bool;
    fn should_validate_local_db_sync(&self) -> bool;
    fn should_validate_raindexes(&self) -> bool;
    fn should_validate_metaboards(&self) -> bool;
    fn should_validate_rainlangs(&self) -> bool;
    fn should_validate_orders(&self) -> bool;
    fn should_validate_scenarios(&self) -> bool;
    fn should_validate_deployments(&self) -> bool;
}

pub trait YamlParsable: Sized {
    type ValidationConfig: ValidationConfig;
    fn new(sources: Vec<String>, validate: Self::ValidationConfig) -> Result<Self, YamlError>;

    fn from_documents(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Self;
    fn from_raindex_yaml(raindex_yaml: RaindexYaml) -> Self;
    fn from_dotrain_yaml(dotrain_yaml: DotrainYaml) -> Self;

    fn get_yaml_string(document: Arc<RwLock<StrictYaml>>) -> Result<String, YamlError> {
        let document = document.read().unwrap();
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

pub trait YamlParsableHash: Sized + Clone {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError>;

    fn parse_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        key: &str,
        context: Option<&Context>,
    ) -> Result<Self, YamlError> {
        let all = Self::parse_all_from_yaml(documents, context)?;
        all.get(key)
            .ok_or_else(|| YamlError::KeyNotFound(key.to_string()))
            .cloned()
    }

    fn sanitize_documents(_documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        Ok(())
    }
}

pub trait YamlParsableVector: Sized {
    fn parse_all_from_yaml(document: Arc<RwLock<StrictYaml>>) -> Result<Vec<Self>, YamlError>;
}

pub trait YamlParsableString {
    fn parse_from_yaml(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Result<String, YamlError>;

    fn parse_from_yaml_optional(
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<Option<String>, YamlError>;
}

pub trait YamlParseableValue: Sized {
    fn parse_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<Self, YamlError>;

    fn parse_from_yaml_optional(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<Option<Self>, YamlError>;
}

pub trait ContextProvider {
    fn create_context(&self) -> Context {
        Context::new()
    }

    fn expand_context_with_remote_networks(&self, context: &mut Context) {
        context.set_remote_networks(self.get_remote_networks_from_cache());
    }
    fn get_remote_networks_from_cache(&self) -> HashMap<String, NetworkCfg>;

    fn expand_context_with_remote_tokens(&self, context: &mut Context) {
        context.set_remote_tokens(self.get_remote_tokens_from_cache());
    }
    fn get_remote_tokens_from_cache(&self) -> HashMap<String, crate::TokenCfg>;

    fn expand_context_with_current_deployment(&self, context: &mut Context, deployment: &str) {
        context.add_current_deployment(deployment.to_string());
    }

    fn expand_context_with_current_order(
        &self,
        context: &mut Context,
        current_order: Option<String>,
    ) {
        if let Some(order) = current_order {
            context.add_current_order(order);
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum FieldErrorKind {
    #[error("Missing required field '{0}'")]
    Missing(String),

    #[error("Field '{field}' must be {expected}")]
    InvalidType { field: String, expected: String },

    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },
}

#[derive(Debug, Error)]
pub enum YamlError {
    #[error(transparent)]
    ScanError(#[from] ScanError),
    #[error(transparent)]
    EmitError(#[from] EmitError),
    #[error(transparent)]
    UrlParseError(#[from] UrlParseError),
    #[error(transparent)]
    RuintParseError(#[from] RuintParseError),

    #[error("{kind} in {location}")]
    Field {
        kind: FieldErrorKind,
        location: String,
    },

    #[error("YAML parse error: {0}")]
    ParseError(String),

    #[error("Key '{0}' not found")]
    KeyNotFound(String),
    #[error("Key '{0}' is already defined in {1}")]
    KeyShadowing(String, String),

    #[error("Failed to acquire read lock")]
    ReadLockError,
    #[error("Failed to acquire write lock")]
    WriteLockError,

    #[error("YAML file is empty")]
    EmptyFile,

    #[error("Error while converting to YAML string")]
    ConvertError,

    #[error("Invalid trait function")]
    InvalidTraitFunction,

    #[error("{0} not found")]
    NotFound(String),

    #[error(transparent)]
    ParseNetworkConfigSourceError(#[from] ParseNetworkConfigSourceError),
    #[error(transparent)]
    ParseTokenConfigSourceError(#[from] ParseTokenConfigSourceError),
    #[error(transparent)]
    ParseRaindexConfigSourceError(#[from] ParseRaindexConfigSourceError),
    #[error(transparent)]
    ParseRainlangConfigSourceError(#[from] ParseRainlangConfigSourceError),
    #[error(transparent)]
    ParseOrderConfigSourceError(#[from] ParseOrderConfigSourceError),
    #[error(transparent)]
    ParseScenarioConfigSourceError(#[from] ParseScenarioConfigSourceError),
    #[error(transparent)]
    ParseDeploymentConfigSourceError(#[from] ParseDeploymentConfigSourceError),
    #[error(transparent)]
    ContextError(#[from] ContextError),
    #[error(transparent)]
    ParseRemoteNetworksError(#[from] ParseRemoteNetworksError),
    #[error(transparent)]
    ParseRemoteTokensError(#[from] ParseRemoteTokensError),
}

impl PartialEq for YamlError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Field {
                    kind: k1,
                    location: l1,
                },
                Self::Field {
                    kind: k2,
                    location: l2,
                },
            ) => k1 == k2 && l1 == l2,
            (Self::ParseError(s1), Self::ParseError(s2)) => s1 == s2,
            (Self::KeyNotFound(k1), Self::KeyNotFound(k2)) => k1 == k2,
            (Self::KeyShadowing(k1, l1), Self::KeyShadowing(k2, l2)) => k1 == k2 && l1 == l2,
            (Self::ReadLockError, Self::ReadLockError) => true,
            (Self::WriteLockError, Self::WriteLockError) => true,
            (Self::EmptyFile, Self::EmptyFile) => true,
            (Self::ConvertError, Self::ConvertError) => true,
            (Self::InvalidTraitFunction, Self::InvalidTraitFunction) => true,
            // For external error types, we'll compare their string representations
            (Self::ScanError(e1), Self::ScanError(e2)) => e1.to_string() == e2.to_string(),
            (Self::EmitError(e1), Self::EmitError(e2)) => e1.to_string() == e2.to_string(),
            (Self::UrlParseError(e1), Self::UrlParseError(e2)) => e1.to_string() == e2.to_string(),
            (Self::RuintParseError(e1), Self::RuintParseError(e2)) => {
                e1.to_string() == e2.to_string()
            }
            (Self::ParseNetworkConfigSourceError(e1), Self::ParseNetworkConfigSourceError(e2)) => {
                e1 == e2
            }
            (Self::ParseTokenConfigSourceError(e1), Self::ParseTokenConfigSourceError(e2)) => {
                e1 == e2
            }
            (Self::ParseRaindexConfigSourceError(e1), Self::ParseRaindexConfigSourceError(e2)) => {
                e1 == e2
            }
            (
                Self::ParseRainlangConfigSourceError(e1),
                Self::ParseRainlangConfigSourceError(e2),
            ) => e1 == e2,
            (Self::ParseOrderConfigSourceError(e1), Self::ParseOrderConfigSourceError(e2)) => {
                e1 == e2
            }
            (
                Self::ParseScenarioConfigSourceError(e1),
                Self::ParseScenarioConfigSourceError(e2),
            ) => e1 == e2,
            (
                Self::ParseDeploymentConfigSourceError(e1),
                Self::ParseDeploymentConfigSourceError(e2),
            ) => e1 == e2,
            (Self::ContextError(e1), Self::ContextError(e2)) => e1.to_string() == e2.to_string(),
            (Self::ParseRemoteNetworksError(e1), Self::ParseRemoteNetworksError(e2)) => {
                e1.to_string() == e2.to_string()
            }
            (Self::ParseRemoteTokensError(e1), Self::ParseRemoteTokensError(e2)) => {
                e1.to_string() == e2.to_string()
            }
            (Self::NotFound(s1), Self::NotFound(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl YamlError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            YamlError::ScanError(err) => format!(
                "There is a syntax error in your YAML configuration: {}",
                err
            ),
            YamlError::EmitError(err) => format!("Failed to generate YAML output: {}", err),
            YamlError::UrlParseError(err) => {
                format!("Invalid URL in your YAML configuration: {}", err)
            }
            YamlError::RuintParseError(err) => {
                format!("Invalid number format in your YAML configuration: {}", err)
            }
            YamlError::Field { kind, location } => match kind {
                FieldErrorKind::Missing(field) => {
                    format!("Missing required field '{}' in {}", field, location)
                }
                FieldErrorKind::InvalidType { field, expected } => {
                    format!("Field '{}' in {} must be {}", field, location, expected)
                }
                FieldErrorKind::InvalidValue { field, reason } => format!(
                    "Invalid value for field '{}' in {}: {}",
                    field, location, reason
                ),
            },
            YamlError::NotFound(msg) => {
                format!(
                    "The requested item \"{}\" could not be found in the YAML configuration.",
                    msg
                )
            }
            YamlError::ParseError(msg) => {
                format!("Failed to parse your YAML configuration: {}", msg)
            }
            YamlError::KeyNotFound(key) => {
                format!("The key '{}' was not found in your YAML configuration", key)
            }
            YamlError::KeyShadowing(key, location) => format!(
                "The key '{}' is defined multiple times in your YAML configuration at {}",
                key, location
            ),
            YamlError::ReadLockError => {
                "Failed to read the YAML configuration due to a lock error".to_string()
            }
            YamlError::WriteLockError => {
                "Failed to modify the YAML configuration due to a lock error".to_string()
            }
            YamlError::EmptyFile => "Your YAML configuration file is empty".to_string(),
            YamlError::ConvertError => {
                "Failed to convert your configuration to YAML format".to_string()
            }
            YamlError::InvalidTraitFunction => {
                "There is an internal error in the YAML processing".to_string()
            }
            YamlError::ParseNetworkConfigSourceError(err) => {
                format!("Network configuration error in your YAML: {}", err)
            }
            YamlError::ParseTokenConfigSourceError(err) => format!(
                "Token configuration error in your YAML: {}",
                err.to_readable_msg()
            ),
            YamlError::ParseRaindexConfigSourceError(err) => format!(
                "Raindex configuration error in your YAML: {}",
                err.to_readable_msg()
            ),
            YamlError::ParseRainlangConfigSourceError(err) => format!(
                "Rainlang configuration error in your YAML: {}",
                err.to_readable_msg()
            ),
            YamlError::ParseOrderConfigSourceError(err) => format!(
                "Order configuration error in your YAML: {}",
                err.to_readable_msg()
            ),
            YamlError::ParseScenarioConfigSourceError(err) => format!(
                "Scenario configuration error in your YAML: {}",
                err.to_readable_msg()
            ),
            YamlError::ParseDeploymentConfigSourceError(err) => format!(
                "Deployment configuration error in your YAML: {}",
                err.to_readable_msg()
            ),
            YamlError::ContextError(err) => {
                format!("Context error in your YAML: {}", err.to_readable_msg())
            }
            YamlError::ParseRemoteNetworksError(err) => {
                format!("Remote networks configuration error: {}", err)
            }
            YamlError::ParseRemoteTokensError(err) => {
                format!("Remote tokens configuration error: {}", err)
            }
        }
    }
}

pub fn load_yaml(yaml: &str) -> Result<StrictYaml, YamlError> {
    let docs = StrictYamlLoader::load_from_str(yaml)?;
    if docs.is_empty() {
        return Err(YamlError::EmptyFile);
    }
    Ok(docs[0].clone())
}

pub fn require_string(
    value: &StrictYaml,
    field: Option<&str>,
    location: Option<String>,
) -> Result<String, YamlError> {
    match field {
        Some(field) => {
            if value[field].is_badvalue() {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing(field.to_string()),
                    location: location.unwrap_or_else(|| "document".to_string()),
                });
            }
            value[field]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: field.to_string(),
                        expected: "a string".to_string(),
                    },
                    location: location.unwrap_or_else(|| "document".to_string()),
                })
        }
        None => value
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "value".to_string(),
                    expected: "a string".to_string(),
                },
                location: location.unwrap_or_else(|| "document".to_string()),
            }),
    }
}
pub fn optional_string(value: &StrictYaml, field: &str) -> Option<String> {
    value[field].as_str().map(|s| s.to_string())
}

pub fn require_hash<'a>(
    value: &'a StrictYaml,
    field: Option<&str>,
    location: Option<String>,
) -> Result<&'a Hash, YamlError> {
    match field {
        Some(field) => {
            if value[field].is_badvalue() {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing(field.to_string()),
                    location: location.unwrap_or_else(|| "document".to_string()),
                });
            }
            value[field].as_hash().ok_or_else(|| YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: field.to_string(),
                    expected: "a map".to_string(),
                },
                location: location.unwrap_or_else(|| "document".to_string()),
            })
        }
        None => value.as_hash().ok_or_else(|| YamlError::Field {
            kind: FieldErrorKind::InvalidType {
                field: "value".to_string(),
                expected: "a map".to_string(),
            },
            location: location.unwrap_or_else(|| "document".to_string()),
        }),
    }
}
pub fn optional_hash<'a>(value: &'a StrictYaml, field: &str) -> Option<&'a Hash> {
    value[field].as_hash()
}

pub fn get_hash_value<'a>(
    hash: &'a Hash,
    field: &str,
    location: Option<String>,
) -> Result<&'a StrictYaml, YamlError> {
    hash.get(&StrictYaml::String(field.to_string()))
        .ok_or_else(|| YamlError::Field {
            kind: FieldErrorKind::Missing(field.to_string()),
            location: location.unwrap_or_else(|| "document".to_string()),
        })
}

pub fn get_hash_value_as_option<'a>(hash: &'a Hash, field: &str) -> Option<&'a StrictYaml> {
    hash.get(&StrictYaml::String(field.to_string()))
}

pub fn require_vec<'a>(
    value: &'a StrictYaml,
    field: &str,
    location: Option<String>,
) -> Result<&'a Array, YamlError> {
    if value[field].is_badvalue() {
        return Err(YamlError::Field {
            kind: FieldErrorKind::Missing(field.to_string()),
            location: location.unwrap_or_else(|| "document".to_string()),
        });
    }
    value[field].as_vec().ok_or_else(|| YamlError::Field {
        kind: FieldErrorKind::InvalidType {
            field: field.to_string(),
            expected: "a vector".to_string(),
        },
        location: location.unwrap_or_else(|| "document".to_string()),
    })
}
pub fn optional_vec<'a>(value: &'a StrictYaml, field: &str) -> Option<&'a Array> {
    value[field].as_vec()
}

pub fn default_document() -> Arc<RwLock<StrictYaml>> {
    Arc::new(RwLock::new(StrictYaml::String("".to_string())))
}

pub fn clone_section_entry(
    documents: &[Arc<RwLock<StrictYaml>>],
    section: &str,
    key: &str,
) -> Result<StrictYaml, YamlError> {
    let section_key = StrictYaml::String(section.to_string());
    let entry_key = StrictYaml::String(key.to_string());

    for document in documents {
        let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
        if let StrictYaml::Hash(root_hash) = &*document_read {
            if let Some(StrictYaml::Hash(section_hash)) = root_hash.get(&section_key) {
                if let Some(value) = section_hash.get(&entry_key) {
                    return Ok(value.clone());
                }
            }
        }
    }

    Err(YamlError::NotFound(format!("{} -> {}", section, key)))
}

pub fn clone_section_entries<'a, I>(
    documents: &[Arc<RwLock<StrictYaml>>],
    section: &str,
    keys: I,
) -> Result<Hash, YamlError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut result = Hash::new();
    for key in keys {
        let value = clone_section_entry(documents, section, key)?;
        result.insert(StrictYaml::String(key.to_string()), value);
    }
    Ok(result)
}

pub fn default_documents() -> Arc<Vec<Arc<RwLock<StrictYaml>>>> {
    Arc::new(Vec::new())
}

pub fn sanitize_all_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
    crate::ChartCfg::sanitize_documents(documents)?;
    crate::RainlangCfg::sanitize_documents(documents)?;
    crate::DeploymentCfg::sanitize_documents(documents)?;
    crate::GuiCfg::sanitize_documents(documents)?;
    crate::LocalDbSyncCfg::sanitize_documents(documents)?;
    crate::NetworkCfg::sanitize_documents(documents)?;
    crate::OrderCfg::sanitize_documents(documents)?;
    crate::RaindexCfg::sanitize_documents(documents)?;
    crate::RemoteNetworksCfg::sanitize_documents(documents)?;
    crate::ScenarioCfg::sanitize_documents(documents)?;
    crate::TokenCfg::sanitize_documents(documents)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn get_document(yaml: &str) -> Arc<RwLock<StrictYaml>> {
        let document = StrictYamlLoader::load_from_str(yaml).unwrap()[0].clone();
        Arc::new(RwLock::new(document))
    }

    #[test]
    fn clone_section_entry_finds_across_documents() {
        let docs = vec![
            get_document(
                r#"
    networks:
        mainnet:
            chain-id: 1
    "#,
            ),
            get_document(
                r#"
    tokens:
        usdc:
            network: mainnet
            address: 0x0000000000000000000000000000000000000001
    "#,
            ),
        ];

        let network = clone_section_entry(&docs, "networks", "mainnet").unwrap();
        assert!(network.as_hash().is_some());

        let token = clone_section_entry(&docs, "tokens", "usdc").unwrap();
        assert!(token.as_hash().is_some());
    }

    #[test]
    fn clone_section_entries_collects_multiple_keys() {
        let docs = vec![
            get_document(
                r#"
    tokens:
        usdc:
            network: mainnet
            address: 0x000
    "#,
            ),
            get_document(
                r#"
    tokens:
        dai:
            network: mainnet
            address: 0x001
    "#,
            ),
        ];

        let entries = clone_section_entries(&docs, "tokens", ["usdc", "dai"]).unwrap();

        assert_eq!(entries.len(), 2);
        assert!(entries.contains_key(&StrictYaml::String("usdc".to_string())));
        assert!(entries.contains_key(&StrictYaml::String("dai".to_string())));
    }

    #[test]
    fn clone_section_entries_missing_key_errors() {
        let docs = vec![get_document(
            r#"
    tokens:
        usdc:
            network: mainnet
            address: 0x000
    "#,
        )];

        let err = clone_section_entries(&docs, "tokens", ["usdc", "dai"]).unwrap_err();
        assert!(matches!(err, YamlError::NotFound(msg) if msg.contains("tokens -> dai")));
    }

    #[test]
    fn clone_section_entry_skips_non_hash_sections() {
        let docs = vec![get_document(
            r#"
    tokens: []
    "#,
        )];

        let err = clone_section_entry(&docs, "tokens", "usdc").unwrap_err();
        assert!(matches!(err, YamlError::NotFound(msg) if msg.contains("tokens -> usdc")));
    }

    #[test]
    fn clone_section_entry_returns_independent_clone() {
        let docs = vec![get_document(
            r#"
    tokens:
        usdc:
            network: mainnet
            address: 0x000
    "#,
        )];

        let mut cloned = clone_section_entry(&docs, "tokens", "usdc").unwrap();
        if let StrictYaml::Hash(ref mut hash) = cloned {
            hash.insert(
                StrictYaml::String("address".to_string()),
                StrictYaml::String("0xDEAD".to_string()),
            );
        } else {
            panic!("Expected hash");
        }

        let doc = docs[0].read().unwrap();
        let original_address = doc["tokens"]["usdc"]["address"]
            .as_str()
            .expect("address should be present");
        assert_eq!(original_address, "0x000");
    }

    #[test]
    fn clone_section_entry_prefers_first_document_on_duplicates() {
        let docs = vec![
            get_document(
                r#"
    tokens:
        usdc:
            network: mainnet
            address: 0x000
    "#,
            ),
            get_document(
                r#"
    tokens:
        usdc:
            network: mainnet
            address: 0x999
    "#,
            ),
        ];

        let cloned = clone_section_entry(&docs, "tokens", "usdc").unwrap();
        let address = cloned["address"].as_str().unwrap();
        assert_eq!(address, "0x000");
    }

    #[test]
    fn clone_section_entry_not_found() {
        let docs = vec![get_document("networks: {}\n")];
        let err = clone_section_entry(&docs, "tokens", "usdc").unwrap_err();
        assert!(matches!(err, YamlError::NotFound(_)));
    }
}
