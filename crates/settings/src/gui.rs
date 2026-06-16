use crate::{
    yaml::{
        context::{Context, GuiContextTrait},
        default_document, get_hash_value, get_hash_value_as_option, optional_hash, optional_string,
        optional_vec, require_string, require_vec, FieldErrorKind, YamlError, YamlParsableHash,
        YamlParseableValue,
    },
    DeploymentCfg, TokenCfg,
};
use alloy::primitives::{ruint::ParseError, utils::UnitsError};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};

const ALLOWED_GUI_KEYS: [&str; 4] = ["name", "description", "short-description", "deployments"];
const ALLOWED_GUI_DEPLOYMENT_KEYS: [&str; 6] = [
    "name",
    "description",
    "short-description",
    "deposits",
    "fields",
    "select-tokens",
];
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, serialize_hashmap_as_object};

// Validation types
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum FieldValueValidationCfg {
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclusive_minimum: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        maximum: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclusive_maximum: Option<String>,
    },
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<u32>,
    },
    Boolean,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(FieldValueValidationCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct DepositValidationCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DepositValidationCfg);

// Config source for Gui
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiPresetSourceCfg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiPresetSourceCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiDepositSourceCfg {
    pub token: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<String>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub validation: Option<DepositValidationCfg>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiFieldDefinitionSourceCfg {
    pub binding: String,
    pub name: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub description: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<GuiPresetSourceCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub default: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub show_custom_field: Option<bool>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub validation: Option<FieldValueValidationCfg>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiDeploymentSourceCfg {
    pub name: String,
    pub description: String,
    pub short_description: Option<String>,
    pub deposits: Vec<GuiDepositSourceCfg>,
    pub fields: Vec<GuiFieldDefinitionSourceCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_tokens: Option<Vec<GuiSelectTokensCfg>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct GuiConfigSourceCfg {
    pub name: String,
    pub description: String,
    pub short_description: Option<String>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, GuiDeploymentSourceCfg>")
    )]
    pub deployments: HashMap<String, GuiDeploymentSourceCfg>,
}
impl GuiConfigSourceCfg {
    pub fn try_into_gui(
        self,
        deployments: &HashMap<String, Arc<DeploymentCfg>>,
        tokens: &HashMap<String, Arc<TokenCfg>>,
    ) -> Result<GuiCfg, ParseGuiConfigSourceError> {
        let gui_deployments = self
            .deployments
            .iter()
            .map(|(deployment_name, deployment_source)| {
                let deployment = deployments
                    .get(deployment_name)
                    .ok_or(ParseGuiConfigSourceError::DeploymentNotFoundError(
                        deployment_name.clone(),
                    ))
                    .map(Arc::clone)?;

                let deposits = deployment_source
                    .deposits
                    .iter()
                    .map(|deposit_source| {
                        let token = tokens
                            .get(&deposit_source.token)
                            .ok_or(ParseGuiConfigSourceError::TokenNotFoundError(
                                deposit_source.token.clone(),
                            ))
                            .map(Arc::clone)?;

                        Ok(GuiDepositCfg {
                            token_key: deposit_source.token.clone(),
                            token: Some(token.clone()),
                            presets: deposit_source.presets.clone(),
                            validation: deposit_source.validation.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                let fields = deployment_source
                    .fields
                    .iter()
                    .map(|field_source| {
                        Ok(GuiFieldDefinitionCfg {
                            binding: field_source.binding.clone(),
                            name: field_source.name.clone(),
                            description: field_source.description.clone(),
                            presets: field_source
                                .presets
                                .as_ref()
                                .map(|presets| {
                                    presets
                                        .iter()
                                        .enumerate()
                                        .map(|(i, preset)| {
                                            Ok(GuiPresetCfg {
                                                id: i.to_string(),
                                                name: preset.name.clone(),
                                                value: preset.value.clone(),
                                            })
                                        })
                                        .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()
                                })
                                .transpose()?,
                            default: field_source.default.clone(),
                            show_custom_field: field_source.show_custom_field,
                            validation: field_source.validation.clone(),
                        })
                    })
                    .collect::<Result<Vec<_>, ParseGuiConfigSourceError>>()?;

                Ok((
                    deployment_name.clone(),
                    GuiDeploymentCfg {
                        document: default_document(),
                        key: deployment_name.to_string(),
                        deployment,
                        name: deployment_source.name.clone(),
                        description: deployment_source.description.clone(),
                        short_description: deployment_source.short_description.clone(),
                        deposits,
                        fields,
                        select_tokens: deployment_source.select_tokens.clone(),
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, ParseGuiConfigSourceError>>()?;

        Ok(GuiCfg {
            name: self.name,
            description: self.description,
            short_description: self.short_description,
            deployments: gui_deployments,
        })
    }
}

#[derive(Error, Debug)]
pub enum ParseGuiConfigSourceError {
    #[error("Deployment not found: {0}")]
    DeploymentNotFoundError(String),
    #[error("Token not found: {0}")]
    TokenNotFoundError(String),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    UnitsError(#[from] UnitsError),
}

// Config for Gui

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiPresetCfg {
    pub id: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub name: Option<String>,
    pub value: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiPresetCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiDepositCfg {
    pub token_key: String,
    pub token: Option<Arc<TokenCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<String>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub validation: Option<DepositValidationCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiDepositCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiSelectTokensCfg {
    pub key: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub name: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiDeploymentCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub deployment: Arc<DeploymentCfg>,
    pub name: String,
    pub description: String,
    pub short_description: Option<String>,
    pub deposits: Vec<GuiDepositCfg>,
    pub fields: Vec<GuiFieldDefinitionCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub select_tokens: Option<Vec<GuiSelectTokensCfg>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiDeploymentCfg);

impl PartialEq for GuiDeploymentCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.deployment == other.deployment
            && self.name == other.name
            && self.description == other.description
            && self.deposits == other.deposits
            && self.fields == other.fields
            && self.select_tokens == other.select_tokens
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct GuiFieldDefinitionCfg {
    pub binding: String,
    pub name: String,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub description: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub presets: Option<Vec<GuiPresetCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub default: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub show_custom_field: Option<bool>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub validation: Option<FieldValueValidationCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiFieldDefinitionCfg);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct GuiCfg {
    pub name: String,
    pub description: String,
    pub short_description: Option<String>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, GuiDeploymentCfg>")
    )]
    pub deployments: HashMap<String, GuiDeploymentCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(GuiCfg);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct NameAndDescriptionCfg {
    pub name: String,
    pub description: String,
    pub short_description: Option<String>,
}

#[cfg(target_family = "wasm")]
impl_wasm_traits!(NameAndDescriptionCfg);

impl GuiCfg {
    pub fn check_gui_key_exists(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<bool, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
            if optional_hash(&document_read, "gui").is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn parse_deployment_keys(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<Vec<String>, YamlError> {
        let mut deployment_keys = Vec::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let deployments = gui
                    .get(&StrictYaml::String("deployments".to_string()))
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    })?;

                if let StrictYaml::Hash(deployments_hash) = deployments {
                    for (key, _) in deployments_hash {
                        if let StrictYaml::String(key) = key {
                            deployment_keys.push(key.clone());
                        }
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    });
                }
            }
        }

        Ok(deployment_keys)
    }

    pub fn parse_select_tokens(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployment_key: &str,
    ) -> Result<Option<Vec<GuiSelectTokensCfg>>, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                if let Some(StrictYaml::Hash(deployments_hash)) =
                    gui.get(&StrictYaml::String("deployments".to_string()))
                {
                    if let Some(StrictYaml::Hash(deployment_hash)) =
                        deployments_hash.get(&StrictYaml::String(deployment_key.to_string()))
                    {
                        if let Some(StrictYaml::Array(tokens)) =
                            deployment_hash.get(&StrictYaml::String("select-tokens".to_string()))
                        {
                            let mut result = Vec::new();
                            for (index, token) in tokens.iter().enumerate() {
                                if let StrictYaml::Hash(token_hash) = token {
                                    let key = get_hash_value(token_hash, "key", Some(format!("key string missing for select-token index: {index} in gui deployment: {deployment_key}")))?.as_str().ok_or(YamlError::Field {
                                        kind: FieldErrorKind::Missing("key".to_string()),
                                        location: format!("select-token index: {index} in gui deployment: {deployment_key}"),
                                    })?;
                                    let name = get_hash_value_as_option(token_hash, "name")
                                        .map(|s| s.as_str())
                                        .unwrap_or_default();
                                    let description =
                                        get_hash_value_as_option(token_hash, "description")
                                            .map(|s| s.as_str())
                                            .unwrap_or_default();
                                    result.push(GuiSelectTokensCfg {
                                        key: key.to_string(),
                                        name: name.map(|s| s.to_string()),
                                        description: description.map(|s| s.to_string()),
                                    });
                                }
                            }
                            return Ok(Some(result));
                        }
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    });
                }
            }
        }
        Ok(None)
    }

    pub fn parse_order_details(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<NameAndDescriptionCfg, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let name = require_string(
                    get_hash_value(gui, "name", Some("gui".to_string()))?,
                    None,
                    Some("gui".to_string()),
                )?;

                let description = require_string(
                    get_hash_value(gui, "description", Some("gui".to_string()))?,
                    None,
                    Some("gui".to_string()),
                )?;

                let short_description = require_string(
                    get_hash_value(gui, "short-description", Some("gui".to_string()))?,
                    None,
                    Some("gui".to_string()),
                )?;

                return Ok(NameAndDescriptionCfg {
                    name,
                    description,
                    short_description: Some(short_description),
                });
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing("name/description".to_string()),
            location: "gui".to_string(),
        })
    }

    pub fn parse_deployment_details(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, YamlError> {
        let mut deployment_details = BTreeMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let deployments = gui
                    .get(&StrictYaml::String("deployments".to_string()))
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    })?
                    .as_hash()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                for (key_yaml, deployment_yaml) in deployments {
                    let deployment_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("gui deployment '{deployment_key}'");

                    let name =
                        require_string(deployment_yaml, Some("name"), Some(location.clone()))?;

                    let description = require_string(
                        deployment_yaml,
                        Some("description"),
                        Some(location.clone()),
                    )?;

                    let short_description = optional_string(deployment_yaml, "short-description");

                    deployment_details.insert(
                        deployment_key,
                        NameAndDescriptionCfg {
                            name,
                            description,
                            short_description,
                        },
                    );
                }
            }
        }

        Ok(deployment_details)
    }

    pub fn parse_field_presets(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployment_key: &str,
        field_binding: &str,
    ) -> Result<Option<Vec<GuiPresetCfg>>, YamlError> {
        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                if let Some(StrictYaml::Hash(deployments_hash)) =
                    gui.get(&StrictYaml::String("deployments".to_string()))
                {
                    if let Some(StrictYaml::Hash(deployment_hash)) =
                        deployments_hash.get(&StrictYaml::String(deployment_key.to_string()))
                    {
                        if let Some(StrictYaml::Array(fields)) =
                            deployment_hash.get(&StrictYaml::String("fields".to_string()))
                        {
                            for (field_index, field) in fields.iter().enumerate() {
                                if let StrictYaml::Hash(field_hash) = field {
                                    if let Some(StrictYaml::String(binding)) =
                                        field_hash.get(&StrictYaml::String("binding".to_string()))
                                    {
                                        if binding == field_binding {
                                            return match optional_vec(field, "presets") {
                                                Some(presets) => {
                                                    let preset_vec = presets.iter().enumerate()
                                                        .map(|(preset_index, preset_yaml)| {
                                                            let name = optional_string(preset_yaml, "name");
                                                            let value = require_string(
                                                                preset_yaml,
                                                                Some("value"),
                                                                Some(format!(
                                                                    "preset index '{preset_index}' for field index '{field_index}' in gui deployment '{deployment_key}'",
                                                                ))
                                                            )?;

                                                            Ok(GuiPresetCfg {
                                                                id: preset_index.to_string(),
                                                                name,
                                                                value,
                                                            })
                                                        })
                                                        .collect::<Result<Vec<_>, YamlError>>()?;
                                                    Ok(Some(preset_vec))
                                                }
                                                None => Ok(None),
                                            };
                                        }
                                    } else {
                                        return Err(YamlError::Field {
                                            kind: FieldErrorKind::Missing("binding".to_string()),
                                            location: format!("field index: {field_index} in gui deployment '{deployment_key}'"),
                                        });
                                    }
                                }
                            }
                        } else {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::Missing("fields".to_string()),
                                location: format!("gui deployment '{deployment_key}'"),
                            });
                        }
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    });
                }
            }
        }
        Ok(None)
    }

    pub fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let gui_key = StrictYaml::String("gui".to_string());
            let Some(gui_value) = root_hash.get(&gui_key) else {
                continue;
            };
            let StrictYaml::Hash(ref gui_hash) = gui_value.clone() else {
                continue;
            };

            let mut sanitized_gui = Hash::new();
            for allowed_key in ALLOWED_GUI_KEYS.iter() {
                let key_yaml = StrictYaml::String(allowed_key.to_string());
                if let Some(v) = gui_hash.get(&key_yaml) {
                    if *allowed_key == "deployments" {
                        if let StrictYaml::Hash(ref deployments_hash) = *v {
                            let mut sanitized_deployments: Vec<(String, StrictYaml)> = Vec::new();

                            for (dep_key, dep_value) in deployments_hash {
                                let Some(dep_key_str) = dep_key.as_str() else {
                                    continue;
                                };

                                let StrictYaml::Hash(ref deployment_hash) = *dep_value else {
                                    continue;
                                };

                                let mut sanitized_deployment = Hash::new();
                                for allowed_dep_key in ALLOWED_GUI_DEPLOYMENT_KEYS.iter() {
                                    let dep_key_yaml =
                                        StrictYaml::String(allowed_dep_key.to_string());
                                    if let Some(dep_v) = deployment_hash.get(&dep_key_yaml) {
                                        sanitized_deployment.insert(dep_key_yaml, dep_v.clone());
                                    }
                                }
                                sanitized_deployments.push((
                                    dep_key_str.to_string(),
                                    StrictYaml::Hash(sanitized_deployment),
                                ));
                            }

                            sanitized_deployments.sort_by(|(a, _), (b, _)| a.cmp(b));

                            let mut new_deployments_hash = Hash::new();
                            for (key, value) in sanitized_deployments {
                                new_deployments_hash.insert(StrictYaml::String(key), value);
                            }

                            sanitized_gui.insert(key_yaml, StrictYaml::Hash(new_deployments_hash));
                        } else {
                            sanitized_gui.insert(key_yaml, v.clone());
                        }
                    } else {
                        sanitized_gui.insert(key_yaml, v.clone());
                    }
                }
            }

            root_hash.insert(gui_key, StrictYaml::Hash(sanitized_gui));
        }

        Ok(())
    }
}

impl YamlParseableValue for GuiCfg {
    fn parse_from_yaml(
        _: Vec<Arc<RwLock<StrictYaml>>>,
        _: Option<&Context>,
    ) -> Result<Self, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }

    fn parse_from_yaml_optional(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<Option<Self>, YamlError> {
        let mut gui_res: Option<GuiCfg> = None;
        let mut gui_deployments_res: HashMap<String, GuiDeploymentCfg> = HashMap::new();

        let tokens = TokenCfg::parse_all_from_yaml(documents.clone(), context);

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Some(gui) = optional_hash(&document_read, "gui") {
                let name = get_hash_value(gui, "name", Some("gui".to_string()))?
                    .as_str()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "name".to_string(),
                            expected: "a string".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                let description = get_hash_value(gui, "description", Some("gui".to_string()))?
                    .as_str()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "description".to_string(),
                            expected: "a string".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                let short_description = get_hash_value_as_option(gui, "short-description")
                    .map(|v| {
                        v.as_str().ok_or(YamlError::Field {
                            kind: FieldErrorKind::InvalidType {
                                field: "short-description".to_string(),
                                expected: "a string".to_string(),
                            },
                            location: "gui".to_string(),
                        })
                    })
                    .transpose()?;

                if gui_res.is_none() {
                    gui_res = Some(GuiCfg {
                        name: name.to_string(),
                        description: description.to_string(),
                        short_description: short_description.map(|d| d.to_string()),
                        deployments: gui_deployments_res.clone(),
                    });
                }

                let deployments = gui
                    .get(&StrictYaml::String("deployments".to_string()))
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::Missing("deployments".to_string()),
                        location: "gui".to_string(),
                    })?
                    .as_hash()
                    .ok_or(YamlError::Field {
                        kind: FieldErrorKind::InvalidType {
                            field: "deployments".to_string(),
                            expected: "a map".to_string(),
                        },
                        location: "gui".to_string(),
                    })?;

                for (deployment_name, deployment_yaml) in deployments {
                    let deployment_name = deployment_name.as_str().unwrap_or_default().to_string();
                    let location = format!("gui deployment '{deployment_name}'");

                    if let Some(context) = context {
                        if let Some(current_deployment) = context.get_current_deployment() {
                            if current_deployment != &deployment_name {
                                continue;
                            }
                        }
                    }

                    let mut context = Context::from_context(context);

                    let select_tokens = match optional_vec(deployment_yaml, "select-tokens") {
                            Some(tokens) => Some(
                                tokens
                                    .iter()
                                    .enumerate()
                                    .map(|(select_token_index, select_token_value)| {
                                        let location = format!("select-token index '{select_token_index}' in gui deployment '{deployment_name}'");

                                        select_token_value.as_hash().ok_or(YamlError::Field{
                                            kind: FieldErrorKind::InvalidType {
                                                field: "select-token".to_string(),
                                                expected: "a map".to_string(),
                                            },
                                            location: location.clone(),
                                        })?;

                                        Ok(GuiSelectTokensCfg {
                                            key: require_string(select_token_value, Some("key"), Some(location.clone()))?,
                                            name: optional_string(select_token_value, "name"),
                                            description: optional_string(select_token_value, "description"),
                                    })
                                })
                                .collect::<Result<Vec<_>, YamlError>>()?,
                        ),
                        None => None,
                    };
                    if let Some(ref select_tokens) = select_tokens {
                        context.add_select_tokens(
                            select_tokens
                                .iter()
                                .map(|select_token| select_token.key.clone())
                                .collect::<Vec<_>>(),
                        );
                    }

                    let deployment = DeploymentCfg::parse_from_yaml(
                        documents.clone(),
                        &deployment_name,
                        Some(&context),
                    )?;
                    context.add_order(deployment.order.clone());

                    let name =
                        require_string(deployment_yaml, Some("name"), Some(location.clone()))?;

                    let description = require_string(
                        deployment_yaml,
                        Some("description"),
                        Some(location.clone()),
                    )?;

                    let short_description = optional_string(deployment_yaml, "short-description");

                    let deposits = require_vec(
                        deployment_yaml,
                        "deposits",
                        Some(location.clone()),
                    )?.iter().enumerate().map(|(deposit_index, deposit_value)| {
                        let mut deposit_token = None;

                        let token_key = require_string(
                                                    deposit_value,
                                                    Some("token"),
                                                    Some(format!(
                                                        "deposit index '{deposit_index}' in {location}",
                                                    )),
                                                )?;

                        if let Ok(tokens) = &tokens {
                            let token = tokens.get(&token_key);
                            deposit_token = token.map(|token| Arc::new(token.clone()));
                        }

                        let presets = match optional_vec(deposit_value, "presets") {
                            Some(presets) => Some(presets.iter()
                            .enumerate()
                            .map(|(preset_index, preset_yaml)| {
                                Ok(preset_yaml.as_str().ok_or(YamlError::Field{
                                    kind: FieldErrorKind::InvalidType {
                                        field: "preset value".to_string(),
                                        expected: "a string".to_string(),
                                    },
                                    location: format!(
                                        "presets list index '{preset_index}' for deposit index '{deposit_index}' in {location}",
                                    ),
                                })?.to_string())
                            })
                            .collect::<Result<Vec<_>, YamlError>>()?),
                            None => None,
                        };

                        let validation = optional_hash(deposit_value, "validation").map(|validation_yaml| {
                            parse_deposit_validation(validation_yaml)
                        }).transpose()?;

                        let gui_deposit = GuiDepositCfg {
                            token_key,
                            token: deposit_token,
                            presets,
                            validation,
                        };
                        Ok(gui_deposit)
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let fields = require_vec(
                        deployment_yaml,
                        "fields",
                        Some(location.clone()),
                    )?.iter().enumerate().map(|(field_index, field_yaml)| {
                        let binding = require_string(
                            field_yaml,
                            Some("binding"),
                            Some(format!(
                                "fields list index '{field_index}' in {location}",
                            )),
                        )?;

                        let name = require_string(
                            field_yaml,
                            Some("name"),
                            Some(format!(
                                "fields list index '{field_index}' in {location}",
                            )),
                        )?;
                        let interpolated_name = context.interpolate_with_select_tokens(&name)?;

                        let description = optional_string(field_yaml, "description");
                        let interpolated_description = description.map(|description| context.interpolate_with_select_tokens(&description)).transpose()?;

                        let presets = match optional_vec(field_yaml, "presets") {
                            Some(p) => Some(p.iter().enumerate().map(|(preset_index, preset_yaml)| {
                                let name = optional_string(preset_yaml, "name");
                                let value = require_string(
                                    preset_yaml,
                                    Some("value"),
                                    Some(format!(
                                        "preset index '{preset_index}' for field index '{field_index}' in {location}",
                                    ))
                                )?;

                                let gui_preset = GuiPresetCfg {
                                    id: preset_index.to_string(),
                                    name,
                                    value,
                                };
                                Ok(gui_preset)
                            })
                            .collect::<Result<Vec<_>, YamlError>>()?),
                            None => None,
                        };

                        let default = optional_string(field_yaml, "default");
                        let show_custom_field = optional_string(field_yaml, "show-custom-field").map(|v| v.eq("true"));

                        let validation = optional_hash(field_yaml, "validation").map(|validation_yaml| {
                            parse_field_validation(validation_yaml, &format!(
                                "validation for field index '{field_index}' in {location}"
                            ))
                        }).transpose()?;

                        let gui_field_definition = GuiFieldDefinitionCfg {
                            binding,
                            name: interpolated_name,
                            description: interpolated_description,
                            presets,
                            default,
                            show_custom_field,
                            validation,
                        };
                        Ok(gui_field_definition)
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let gui_deployment = GuiDeploymentCfg {
                        document: document.clone(),
                        key: deployment_name.clone(),
                        deployment: Arc::new(deployment),
                        name,
                        description,
                        short_description,
                        deposits,
                        fields,
                        select_tokens,
                    };

                    if gui_deployments_res.contains_key(&deployment_name) {
                        return Err(YamlError::KeyShadowing(
                            deployment_name.clone(),
                            "gui deployment".to_string(),
                        ));
                    }
                    gui_deployments_res.insert(deployment_name, gui_deployment);
                }
                if let Some(gui) = &mut gui_res {
                    gui.deployments.clone_from(&gui_deployments_res);
                }
            }
        }

        Ok(gui_res)
    }
}

fn parse_deposit_validation(yaml: &Hash) -> Result<DepositValidationCfg, YamlError> {
    Ok(DepositValidationCfg {
        minimum: yaml
            .get(&StrictYaml::String("minimum".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        exclusive_minimum: yaml
            .get(&StrictYaml::String("exclusive-minimum".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        maximum: yaml
            .get(&StrictYaml::String("maximum".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        exclusive_maximum: yaml
            .get(&StrictYaml::String("exclusive-maximum".to_string()))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    })
}

fn parse_field_validation(
    yaml: &Hash,
    location: &str,
) -> Result<FieldValueValidationCfg, YamlError> {
    let validation_type = yaml
        .get(&StrictYaml::String("type".to_string()))
        .and_then(|v| v.as_str())
        .ok_or(YamlError::Field {
            kind: FieldErrorKind::InvalidType {
                field: "type".to_string(),
                expected: "a string".to_string(),
            },
            location: location.to_string(),
        })?;

    match validation_type {
        "number" => Ok(FieldValueValidationCfg::Number {
            minimum: get_hash_value_as_option(yaml, "minimum")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            exclusive_minimum: get_hash_value_as_option(yaml, "exclusive-minimum")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            maximum: get_hash_value_as_option(yaml, "maximum")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            exclusive_maximum: get_hash_value_as_option(yaml, "exclusive-maximum")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }),
        "string" => Ok(FieldValueValidationCfg::String {
            min_length: get_hash_value_as_option(yaml, "min-length")
                .and_then(|v| v.as_str())
                .map(|s| s.parse::<u32>())
                .transpose()
                .map_err(|_| YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "min-length".to_string(),
                        expected: "a valid number".to_string(),
                    },
                    location: location.to_string(),
                })?,
            max_length: get_hash_value_as_option(yaml, "max-length")
                .and_then(|v| v.as_str())
                .map(|s| s.parse::<u32>())
                .transpose()
                .map_err(|_| YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "max-length".to_string(),
                        expected: "a valid number".to_string(),
                    },
                    location: location.to_string(),
                })?,
        }),
        "boolean" => Ok(FieldValueValidationCfg::Boolean),
        _ => Err(YamlError::Field {
            kind: FieldErrorKind::InvalidType {
                field: "type".to_string(),
                expected: "one of: number, string, boolean".to_string(),
            },
            location: location.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{mock_network, mock_rainlang, mock_token},
        yaml::{default_documents, tests::get_document},
        OrderCfg, ScenarioCfg,
    };
    use alloy::primitives::Address;
    use std::sync::RwLock;
    use strict_yaml_rust::StrictYaml;

    #[test]
    fn test_gui_creation_success() {
        let gui_config_source = GuiConfigSourceCfg {
            name: "test-gui".to_string(),
            description: "test-gui-description".to_string(),
            short_description: None,
            deployments: HashMap::from([(
                "test-deployment".to_string(),
                GuiDeploymentSourceCfg {
                    name: "test-deployment".to_string(),
                    description: "test-deployment-description".to_string(),

                    short_description: None,
                    deposits: vec![GuiDepositSourceCfg {
                        token: "test-token".to_string(),
                        presets: Some(vec!["1.3".to_string(), "2.7".to_string()]),
                        validation: None,
                    }],
                    fields: vec![
                        GuiFieldDefinitionSourceCfg {
                            binding: "test-binding".to_string(),
                            name: "test-name".to_string(),
                            description: Some("test-description".to_string()),
                            presets: Some(vec![
                                GuiPresetSourceCfg {
                                    name: Some("test-preset".to_string()),
                                    value: "0.015".to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: Some("test-preset-2".to_string()),
                                    value: "0.3".to_string(),
                                },
                            ]),
                            default: None,
                            show_custom_field: None,
                            validation: None,
                        },
                        GuiFieldDefinitionSourceCfg {
                            binding: "test-binding-2".to_string(),
                            name: "test-name-2".to_string(),
                            description: Some("test-description-2".to_string()),
                            presets: Some(vec![
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "3.2".to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "4.8".to_string(),
                                },
                            ]),
                            default: Some("0.015".to_string()),
                            show_custom_field: Some(true),
                            validation: None,
                        },
                        GuiFieldDefinitionSourceCfg {
                            binding: "test-binding-3".to_string(),
                            name: "test-name-3".to_string(),
                            description: Some("test-description-3".to_string()),
                            presets: Some(vec![
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: Address::default().to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "some-value".to_string(),
                                },
                                GuiPresetSourceCfg {
                                    name: None,
                                    value: "true".to_string(),
                                },
                            ]),
                            default: Some("0.25".to_string()),
                            show_custom_field: Some(false),
                            validation: None,
                        },
                    ],
                    select_tokens: Some(vec![GuiSelectTokensCfg {
                        key: "test-token".to_string(),
                        name: Some("Test name".to_string()),
                        description: Some("Test description".to_string()),
                    }]),
                },
            )]),
        };
        let scenario = ScenarioCfg {
            document: default_document(),
            documents: default_documents(),
            key: "scenario1".into(),
            bindings: HashMap::new(),
            rainlang: mock_rainlang(),
            runs: None,
            blocks: None,
        };
        let order = OrderCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            inputs: vec![],
            outputs: vec![],
            network: mock_network(),
            rainlang: None,
            orderbook: None,
            oracle_url: None,
        };
        let deployment = DeploymentCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "test-deployment".to_string(),
            scenario: Arc::new(scenario),
            order: Arc::new(order),
        };
        let deployments = HashMap::from([("test-deployment".to_string(), Arc::new(deployment))]);
        let tokens = HashMap::from([("test-token".to_string(), mock_token("test-token"))]);

        let gui = gui_config_source
            .try_into_gui(&deployments, &tokens)
            .unwrap();

        assert_eq!(gui.name, "test-gui");
        assert_eq!(gui.description, "test-gui-description");
        assert_eq!(gui.deployments.len(), 1);
        let deployment = &gui.deployments.get("test-deployment").unwrap();
        assert_eq!(deployment.name, "test-deployment");
        assert_eq!(deployment.description, "test-deployment-description");
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = &deployment.deposits[0];
        assert_eq!(
            deposit.token.as_ref().unwrap().label,
            Some("test-token".to_string())
        );
        let presets = deposit.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0], "1.3".to_string());
        assert_eq!(presets[1], "2.7".to_string());
        assert_eq!(deployment.fields.len(), 3);
        let field1 = &deployment.fields[0];
        assert_eq!(field1.binding, "test-binding");
        assert_eq!(field1.name, "test-name");
        assert_eq!(field1.description, Some("test-description".to_string()));
        assert_eq!(field1.default, None);
        assert_eq!(field1.show_custom_field, None);
        let presets = field1.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, Some("test-preset".to_string()));
        assert_eq!(presets[0].value, "0.015".to_string());
        assert_eq!(presets[1].name, Some("test-preset-2".to_string()));
        assert_eq!(presets[1].value, "0.3".to_string());
        let field2 = &deployment.fields[1];
        assert_eq!(field2.binding, "test-binding-2");
        assert_eq!(field2.name, "test-name-2");
        assert_eq!(field2.description, Some("test-description-2".to_string()));
        assert_eq!(field2.default, Some("0.015".to_string()));
        assert_eq!(field2.show_custom_field, Some(true));
        let presets = field2.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, None);
        assert_eq!(presets[1].name, None);
        assert_eq!(presets[1].value, "4.8".to_string());
        let field3 = &deployment.fields[2];
        assert_eq!(field3.binding, "test-binding-3");
        assert_eq!(field3.name, "test-name-3");
        assert_eq!(field3.description, Some("test-description-3".to_string()));
        assert_eq!(field3.default, Some("0.25".to_string()));
        assert_eq!(field3.show_custom_field, Some(false));
        let presets = field3.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].value, Address::default().to_string());
        assert_eq!(presets[1].value, "some-value".to_string());
        assert_eq!(presets[2].value, "true".to_string());
        assert_eq!(
            deployment.select_tokens,
            Some(vec![GuiSelectTokensCfg {
                key: "test-token".to_string(),
                name: Some("Test name".to_string()),
                description: Some("Test description".to_string()),
            }])
        );
    }

    #[test]
    fn test_parse_gui_from_yaml() {
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name:
      - test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "name".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name:
      - test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "name".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description:
      - test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "description".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description:
      - test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "description".to_string(),
                    expected: "a string".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
        - test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        rainlang: registry1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "root".to_string(),
            }
        );

        let yaml_prefix = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: deployment1
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: some name
            description: some description
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deposits".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("token".to_string()),
                location: "deposit index '0' in gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "preset value".to_string(),
                    expected: "a string".to_string()
                },
                location:
                    "presets list index '0' for deposit index '0' in gui deployment 'deployment1'"
                        .to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("fields".to_string()),
                location: "gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("binding".to_string()),
                location: "fields list index '0' in gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "fields list index '0' in gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value:
                        wrong: map
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "value".to_string(),
                    expected: "a string".to_string()
                },
                location: "preset index '0' for field index '0' in gui deployment 'deployment1'"
                    .to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
            select-tokens:
                - test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "select-token".to_string(),
                    expected: "a map".to_string()
                },
                location: "select-token index '0' in gui deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
            select-tokens:
                - test: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("key".to_string()),
                location: "select-token index '0' in gui deployment 'deployment1'".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_gui_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
    deployment2:
        scenario: scenario1
        order: order1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let yaml_two = r#"
gui:
    name: test
    description: test
    deployments:
        deployment2:
            name: test another
            description: test another
            deposits:
                - token: token2
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let res = GuiCfg::parse_from_yaml_optional(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        let gui = res.unwrap();
        assert_eq!(gui.deployments.len(), 2);

        let deployment = gui.deployments.get("deployment1").unwrap();
        assert_eq!(deployment.name, "test");
        assert_eq!(deployment.description, "test");
        assert_eq!(deployment.deposits[0].token.as_ref().unwrap().key, "token1");

        let deployment = gui.deployments.get("deployment2").unwrap();
        assert_eq!(deployment.name, "test another");
        assert_eq!(deployment.description, "test another");
        assert_eq!(deployment.deposits[0].token.as_ref().unwrap().key, "token2");
    }

    #[test]
    fn test_parse_gui_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token2
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
    deployment2:
        scenario: scenario1
        order: order1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let yaml_two = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  presets:
                    - "1"
            fields:
                - binding: test
                  name: test
                  presets:
                    - value: test
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("deployment1".to_string(), "gui deployment".to_string())
        );
    }

    #[test]
    fn test_parse_deployment_keys() {
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
"#;

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments: test
"#;

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
      - test
"#;

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
      - test: test
"#;

        let error = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string()
                },
                location: "gui".to_string(),
            }
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000002
        network: network1
gui:
    name: test
    description: test
    deployments:
      test: test
      test2: test2
"#;

        let keys = GuiCfg::parse_deployment_keys(vec![get_document(yaml)]).unwrap();
        assert_eq!(keys, vec!["test".to_string(), "test2".to_string()]);
    }

    #[test]
    fn test_check_gui_key_exists() {
        let yaml = r#"
        gui:
            name: test
            description: test
        "#;
        let res = GuiCfg::check_gui_key_exists(vec![get_document(yaml)]).unwrap();
        assert!(res);

        let yaml = r#"
        test: test
        "#;
        let res = GuiCfg::check_gui_key_exists(vec![get_document(yaml)]).unwrap();
        assert!(!res);
    }

    #[test]
    fn test_parse_deposit_validation() {
        let yaml_prefix = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

        // Test deposit validation with all fields
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  validation:
                    minimum: "100"
                    exclusive-minimum: "50"
                    maximum: "1000"
                    exclusive-maximum: "2000"
            fields:
                - binding: test
                  name: test
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let deployment = gui.deployments.get("deployment1").unwrap();
        let deposit = &deployment.deposits[0];
        let validation = deposit.validation.as_ref().unwrap();

        assert_eq!(validation.minimum, Some("100".to_string()));
        assert_eq!(validation.exclusive_minimum, Some("50".to_string()));
        assert_eq!(validation.maximum, Some("1000".to_string()));
        assert_eq!(validation.exclusive_maximum, Some("2000".to_string()));

        // Test deposit validation with partial fields
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
                  validation:
                    minimum: "100"
                    maximum: "1000"
            fields:
                - binding: test
                  name: test
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let deposit = &gui.deployments.get("deployment1").unwrap().deposits[0];
        let validation = deposit.validation.as_ref().unwrap();

        assert_eq!(validation.minimum, Some("100".to_string()));
        assert_eq!(validation.exclusive_minimum, None);
        assert_eq!(validation.maximum, Some("1000".to_string()));
        assert_eq!(validation.exclusive_maximum, None);

        // Test deposit without validation
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let deposit = &gui.deployments.get("deployment1").unwrap().deposits[0];
        assert!(deposit.validation.is_none());
    }

    #[test]
    fn test_parse_field_validation_number() {
        let yaml_prefix = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

        // Test number validation with all fields
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: number
                    minimum: "0"
                    exclusive-minimum: "-1"
                    maximum: "100"
                    exclusive-maximum: "101"
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let field = &gui.deployments.get("deployment1").unwrap().fields[0];
        if let Some(FieldValueValidationCfg::Number {
            minimum,
            exclusive_minimum,
            maximum,
            exclusive_maximum,
        }) = &field.validation
        {
            assert_eq!(*minimum, Some("0".to_string()));
            assert_eq!(*exclusive_minimum, Some("-1".to_string()));
            assert_eq!(*maximum, Some("100".to_string()));
            assert_eq!(*exclusive_maximum, Some("101".to_string()));
        } else {
            panic!("Expected Number validation type");
        }

        // Test number validation with partial fields
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: number
                    minimum: "0"
                    maximum: "100"
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let field = &gui.deployments.get("deployment1").unwrap().fields[0];
        if let Some(FieldValueValidationCfg::Number {
            minimum,
            exclusive_minimum,
            maximum,
            exclusive_maximum,
        }) = &field.validation
        {
            assert_eq!(*minimum, Some("0".to_string()));
            assert_eq!(*exclusive_minimum, None);
            assert_eq!(*maximum, Some("100".to_string()));
            assert_eq!(*exclusive_maximum, None);
        } else {
            panic!("Expected Number validation type");
        }
    }

    #[test]
    fn test_parse_field_validation_string() {
        let yaml_prefix = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

        // Test string validation with all fields
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: string
                    min-length: 5
                    max-length: 50
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let field = &gui.deployments.get("deployment1").unwrap().fields[0];
        if let Some(FieldValueValidationCfg::String {
            min_length,
            max_length,
        }) = &field.validation
        {
            assert_eq!(*min_length, Some(5));
            assert_eq!(*max_length, Some(50));
        } else {
            panic!("Expected String validation type");
        }

        // Test string validation with partial fields
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: string
                    min-length: 1
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let field = &gui.deployments.get("deployment1").unwrap().fields[0];
        if let Some(FieldValueValidationCfg::String {
            min_length,
            max_length,
        }) = &field.validation
        {
            assert_eq!(*min_length, Some(1));
            assert_eq!(*max_length, None);
        } else {
            panic!("Expected String validation type");
        }

        // Test string validation with no length constraints
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: string
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let field = &gui.deployments.get("deployment1").unwrap().fields[0];
        if let Some(FieldValueValidationCfg::String {
            min_length,
            max_length,
        }) = &field.validation
        {
            assert_eq!(*min_length, None);
            assert_eq!(*max_length, None);
        } else {
            panic!("Expected String validation type");
        }
    }

    #[test]
    fn test_parse_field_validation_boolean() {
        let yaml_prefix = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

        // Test boolean validation
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: boolean
"#;
        let gui = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap()
        .unwrap();

        let field = &gui.deployments.get("deployment1").unwrap().fields[0];
        if let Some(FieldValueValidationCfg::Boolean) = &field.validation {
            // Boolean validation type correctly parsed
        } else {
            panic!("Expected Boolean validation type");
        }
    }

    #[test]
    fn test_parse_field_validation_errors() {
        let yaml_prefix = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000001
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
deployments:
    deployment1:
        scenario: scenario1
        order: order1
"#;

        // Test missing type field
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    minimum: "0"
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "type".to_string(),
                    expected: "a string".to_string()
                },
                location: "validation for field index '0' in gui deployment 'deployment1'"
                    .to_string(),
            }
        );

        // Test invalid type value
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: invalid-type
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "type".to_string(),
                    expected: "one of: number, string, boolean".to_string()
                },
                location: "validation for field index '0' in gui deployment 'deployment1'"
                    .to_string(),
            }
        );

        // Test invalid min-length value for string
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: string
                    min-length: invalid
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "min-length".to_string(),
                    expected: "a valid number".to_string()
                },
                location: "validation for field index '0' in gui deployment 'deployment1'"
                    .to_string(),
            }
        );

        // Test invalid max-length value for string
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token1
            fields:
                - binding: test
                  name: test
                  validation:
                    type: string
                    max-length: invalid
"#;
        let error = GuiCfg::parse_from_yaml_optional(
            vec![get_document(&format!("{yaml_prefix}{yaml}"))],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "max-length".to_string(),
                    expected: "a valid number".to_string()
                },
                location: "validation for field index '0' in gui deployment 'deployment1'"
                    .to_string(),
            }
        );
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_gui_keys() {
        let yaml = r#"
gui:
    name: test-gui
    description: test description
    short-description: short desc
    unknown-key: should-be-dropped
    deployments:
        deployment1:
            name: test
            description: test desc
            deposits: []
            fields: []
            unknown-deployment-key: also-dropped
"#;
        let document = get_document(yaml);
        GuiCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let gui = root.get(&StrictYaml::String("gui".to_string())).unwrap();
        let StrictYaml::Hash(ref gui_hash) = *gui else {
            panic!("expected gui hash");
        };

        assert!(gui_hash.contains_key(&StrictYaml::String("name".to_string())));
        assert!(gui_hash.contains_key(&StrictYaml::String("description".to_string())));
        assert!(gui_hash.contains_key(&StrictYaml::String("short-description".to_string())));
        assert!(gui_hash.contains_key(&StrictYaml::String("deployments".to_string())));
        assert!(!gui_hash.contains_key(&StrictYaml::String("unknown-key".to_string())));

        let deployments = gui_hash
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };
        let deployment1 = deployments_hash
            .get(&StrictYaml::String("deployment1".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployment1_hash) = *deployment1 else {
            panic!("expected deployment1 hash");
        };

        assert!(deployment1_hash.contains_key(&StrictYaml::String("name".to_string())));
        assert!(deployment1_hash.contains_key(&StrictYaml::String("description".to_string())));
        assert!(deployment1_hash.contains_key(&StrictYaml::String("deposits".to_string())));
        assert!(deployment1_hash.contains_key(&StrictYaml::String("fields".to_string())));
        assert!(!deployment1_hash
            .contains_key(&StrictYaml::String("unknown-deployment-key".to_string())));
    }

    #[test]
    fn test_sanitize_documents_preserves_allowed_gui_key_order() {
        let yaml = r#"
gui:
    deployments:
        deployment1:
            fields: []
            deposits: []
            description: desc
            name: name
    short-description: short
    description: desc
    name: name
"#;
        let document = get_document(yaml);
        GuiCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let gui = root.get(&StrictYaml::String("gui".to_string())).unwrap();
        let StrictYaml::Hash(ref gui_hash) = *gui else {
            panic!("expected gui hash");
        };

        let keys: Vec<String> = gui_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(
            keys,
            vec!["name", "description", "short-description", "deployments"]
        );

        let deployments = gui_hash
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };
        let deployment1 = deployments_hash
            .get(&StrictYaml::String("deployment1".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployment1_hash) = *deployment1 else {
            panic!("expected deployment1 hash");
        };

        let dep_keys: Vec<String> = deployment1_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(dep_keys, vec!["name", "description", "deposits", "fields"]);
    }

    #[test]
    fn test_sanitize_documents_deployments_lexicographic_order() {
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        zebra:
            name: z
            description: z
            deposits: []
            fields: []
        alpha:
            name: a
            description: a
            deposits: []
            fields: []
        beta:
            name: b
            description: b
            deposits: []
            fields: []
"#;
        let document = get_document(yaml);
        GuiCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let gui = root.get(&StrictYaml::String("gui".to_string())).unwrap();
        let StrictYaml::Hash(ref gui_hash) = *gui else {
            panic!("expected gui hash");
        };
        let deployments = gui_hash
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };

        let keys: Vec<String> = deployments_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_handles_missing_gui_section() {
        let yaml = r#"
other: value
"#;
        let document = get_document(yaml);
        GuiCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        assert!(!root.contains_key(&StrictYaml::String("gui".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = r#"just a string"#;
        let document = get_document(yaml);
        GuiCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_gui() {
        let yaml = r#"
gui: not-a-hash
"#;
        let document = get_document(yaml);
        GuiCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let gui = root.get(&StrictYaml::String("gui".to_string())).unwrap();
        assert_eq!(gui.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_deployments() {
        let yaml = r#"
gui:
    name: test
    description: test
    deployments:
        valid:
            name: valid
            description: valid desc
            deposits: []
            fields: []
        invalid: not-a-hash
"#;
        let document = get_document(yaml);
        GuiCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let gui = root.get(&StrictYaml::String("gui".to_string())).unwrap();
        let StrictYaml::Hash(ref gui_hash) = *gui else {
            panic!("expected gui hash");
        };
        let deployments = gui_hash
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };

        assert!(deployments_hash.contains_key(&StrictYaml::String("valid".to_string())));
        assert!(!deployments_hash.contains_key(&StrictYaml::String("invalid".to_string())));
        assert_eq!(deployments_hash.len(), 1);
    }
}
