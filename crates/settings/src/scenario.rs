use crate::{
    yaml::{default_documents, get_hash_value},
    *,
};
use blocks::BlocksCfg;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    num::ParseIntError,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, serialize_hashmap_as_object};
use yaml::{
    context::Context, default_document, optional_hash, optional_string, require_hash,
    require_string, FieldErrorKind, YamlError, YamlParsableHash,
};

const ALLOWED_SCENARIO_KEYS: [&str; 5] = ["bindings", "blocks", "rainlang", "runs", "scenarios"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct ScenarioCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    #[serde(skip, default = "default_documents")]
    pub documents: Arc<Vec<Arc<RwLock<StrictYaml>>>>,
    pub key: String,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, string>")
    )]
    pub bindings: HashMap<String, String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub runs: Option<u64>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub blocks: Option<BlocksCfg>,
    pub rainlang: Arc<RainlangCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ScenarioCfg);

impl ScenarioCfg {
    pub fn validate_runs(value: &str) -> Result<u64, ParseScenarioConfigSourceError> {
        value
            .parse::<u64>()
            .map_err(ParseScenarioConfigSourceError::RunsParseError)
    }

    pub fn validate_blocks(value: &str) -> Result<BlocksCfg, ParseScenarioConfigSourceError> {
        match serde_yaml::from_str::<BlocksCfg>(value) {
            Ok(blocks) => Ok(blocks),
            Err(_) => Err(ParseScenarioConfigSourceError::BlocksParseError(
                value.to_string(),
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn validate_scenario(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        current_document: Arc<RwLock<StrictYaml>>,
        rainlangs: &HashMap<String, RainlangCfg>,
        rainlang: &mut Option<Arc<RainlangCfg>>,
        scenarios: &mut HashMap<String, ScenarioCfg>,
        parent_scenario: ScenarioParent,
        scenario_key: String,
        scenario_yaml: &StrictYaml,
        context: Option<&Context>,
    ) -> Result<(), YamlError> {
        let mut current_bindings = HashMap::new();

        if let Some(bindings) = optional_hash(scenario_yaml, "bindings") {
            for (binding_key, binding_value) in bindings {
                let binding_key = binding_key.as_str().unwrap_or_default();
                let location = format!(
                    "binding key '{}' in scenario '{}'",
                    binding_key, scenario_key
                );

                let binding_value = require_string(binding_value, None, Some(location.clone()))?;

                let interpolated_value = match context {
                    Some(context) => context.interpolate_with_select_tokens(&binding_value)?,
                    None => binding_value.to_string(),
                };

                current_bindings.insert(binding_key.to_string(), interpolated_value);
            }
        }

        let mut bindings = parent_scenario
            .bindings
            .as_ref()
            .map_or_else(HashMap::new, |pb| pb.clone());
        for (k, v) in current_bindings {
            if let Some(parent_value) = parent_scenario.bindings.as_ref().and_then(|pb| pb.get(&k))
            {
                if *parent_value != v {
                    return Err(YamlError::ParseScenarioConfigSourceError(
                        ParseScenarioConfigSourceError::ParentBindingShadowedError(k.to_string()),
                    ));
                }
            }

            let binding_value = match context {
                Some(context) => context.interpolate_with_select_tokens(&v)?,
                None => v.to_string(),
            };

            bindings.insert(k.to_string(), binding_value);
        }

        let runs = optional_string(scenario_yaml, "runs")
            .map(|runs| ScenarioCfg::validate_runs(&runs))
            .transpose()?;
        let blocks = if let Some(blocks) = optional_string(scenario_yaml, "blocks") {
            Some(ScenarioCfg::validate_blocks(&blocks)?)
        } else if let Some(blocks) = optional_hash(scenario_yaml, "blocks") {
            let location = format!("blocks in scenario '{scenario_key}'");

            let range = get_hash_value(blocks, "range", Some(location.clone()))?
                .as_str()
                .ok_or(YamlError::Field {
                    kind: FieldErrorKind::Missing("range".to_string()),
                    location: location.clone(),
                })?;
            let interval = get_hash_value(blocks, "interval", Some(location.clone()))?
                .as_str()
                .ok_or(YamlError::Field {
                    kind: FieldErrorKind::Missing("interval".to_string()),
                    location,
                })?;

            Some(ScenarioCfg::validate_blocks(&format!(
                "range: {range}\ninterval: {interval}"
            ))?)
        } else {
            None
        };

        let mut current_rainlang: Option<RainlangCfg> = None;

        if let Ok(reg) = rainlangs
            .get(&scenario_key)
            .ok_or_else(|| YamlError::KeyNotFound(scenario_key.clone()))
        {
            current_rainlang = Some(reg.clone());
        } else if let Some(rainlang_name) = optional_string(scenario_yaml, "rainlang") {
            current_rainlang = Some(
                rainlangs
                    .get(&rainlang_name)
                    .ok_or_else(|| YamlError::KeyNotFound(rainlang_name.to_string()))?
                    .clone(),
            );
        }

        if let Some(current_rainlang) = current_rainlang {
            if let Some(parent_rainlang) = parent_scenario.rainlang.as_ref() {
                if current_rainlang.key != parent_rainlang.key {
                    return Err(YamlError::ParseScenarioConfigSourceError(
                        ParseScenarioConfigSourceError::ParentRainlangShadowedError(
                            current_rainlang.key.clone(),
                        ),
                    ));
                }
            }
            *rainlang = Some(Arc::new(current_rainlang));
        }

        if scenarios.contains_key(&scenario_key) {
            return Err(YamlError::KeyShadowing(
                scenario_key.clone(),
                "scenarios".to_string(),
            ));
        }
        let key = if parent_scenario.key.is_empty() {
            scenario_key.clone()
        } else {
            format!("{}.{}", parent_scenario.key, scenario_key.clone())
        };
        scenarios.insert(
            key.clone(),
            ScenarioCfg {
                document: current_document.clone(),
                documents: Arc::new(documents.clone()),
                key: key.clone(),
                bindings: bindings.clone(),
                runs,
                blocks,
                rainlang: rainlang.clone().ok_or(
                    ParseScenarioConfigSourceError::RainlangNotFound(scenario_key),
                )?,
            },
        );

        if let Some(scenarios_yaml) = optional_hash(scenario_yaml, "scenarios") {
            for (child_key, child_scenario_yaml) in scenarios_yaml {
                let child_key = child_key.as_str().unwrap_or_default().to_string();
                Self::validate_scenario(
                    documents.clone(),
                    current_document.clone(),
                    rainlangs,
                    rainlang,
                    scenarios,
                    ScenarioParent {
                        key: key.clone(),
                        bindings: Some(bindings.clone()),
                        rainlang: rainlang.clone(),
                    },
                    child_key,
                    child_scenario_yaml,
                    context,
                )?;
            }
        }

        Ok(())
    }

    pub fn update_bindings(
        &mut self,
        bindings: HashMap<String, String>,
    ) -> Result<Self, YamlError> {
        let scenario_parts = self.key.split('.').collect::<Vec<_>>();
        let base_scenario = scenario_parts[0];

        let mut new_bindings = Hash::new();
        let mut unhandled_bindings: HashSet<String> = bindings.keys().cloned().collect();
        for (k, v) in bindings {
            new_bindings.insert(StrictYaml::String(k), StrictYaml::String(v));
        }

        {
            let mut document = self
                .document
                .write()
                .map_err(|_| YamlError::WriteLockError)?;

            if let StrictYaml::Hash(ref mut document_hash) = *document {
                if let Some(StrictYaml::Hash(ref mut scenarios)) =
                    document_hash.get_mut(&StrictYaml::String("scenarios".to_string()))
                {
                    if let Some(StrictYaml::Hash(ref mut scenario)) =
                        scenarios.get_mut(&StrictYaml::String(base_scenario.to_string()))
                    {
                        if let Some(StrictYaml::Hash(ref mut base_bindings)) =
                            scenario.get_mut(&StrictYaml::String("bindings".to_string()))
                        {
                            let updates: Vec<_> = base_bindings
                                .keys()
                                .filter_map(|k| {
                                    if let Some(v) = new_bindings.get(k) {
                                        unhandled_bindings.remove(k.as_str().unwrap_or_default());
                                        Some((k.clone(), v.clone()))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            for (k, v) in updates {
                                base_bindings.insert(k, v);
                            }
                        }

                        let scenario_parts_vec: Vec<_> = scenario_parts.iter().skip(1).collect();
                        let mut current = scenario;

                        for &part in scenario_parts_vec {
                            let next_scenario =
                                if let Some(StrictYaml::Hash(ref mut sub_scenarios)) =
                                    current.get_mut(&StrictYaml::String("scenarios".to_string()))
                                {
                                    if let Some(StrictYaml::Hash(ref mut sub_scenario)) =
                                        sub_scenarios.get_mut(&StrictYaml::String(part.to_string()))
                                    {
                                        if let Some(StrictYaml::Hash(ref mut sub_bindings)) =
                                            sub_scenario.get_mut(&StrictYaml::String(
                                                "bindings".to_string(),
                                            ))
                                        {
                                            let updates: Vec<_> = sub_bindings
                                                .keys()
                                                .filter_map(|k| {
                                                    if let Some(v) = new_bindings.get(k) {
                                                        unhandled_bindings
                                                            .remove(k.as_str().unwrap_or_default());
                                                        Some((k.clone(), v.clone()))
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect();
                                            for (k, v) in updates {
                                                sub_bindings.insert(k, v);
                                            }
                                        }
                                        sub_scenario
                                    } else {
                                        return Err(YamlError::Field {
                                            kind: FieldErrorKind::Missing(part.to_string()),
                                            location: format!(
                                                "sub scenarios of '{}'",
                                                base_scenario
                                            ),
                                        });
                                    }
                                } else {
                                    return Err(YamlError::Field {
                                        kind: FieldErrorKind::Missing("scenarios".to_string()),
                                        location: format!("{} of '{}'", part, base_scenario),
                                    });
                                };
                            current = next_scenario;
                        }

                        if let Some(StrictYaml::Hash(ref mut lowest_bindings)) =
                            current.get_mut(&StrictYaml::String("bindings".to_string()))
                        {
                            for key in unhandled_bindings {
                                if let Some(value) =
                                    new_bindings.get(&StrictYaml::String(key.clone()))
                                {
                                    lowest_bindings.insert(StrictYaml::String(key), value.clone());
                                }
                            }
                        }
                    } else {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::Missing(base_scenario.to_string()),
                            location: "scenarios".to_string(),
                        });
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing("scenarios".to_string()),
                        location: "root".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "document".to_string(),
                        expected: "a hash".to_string(),
                    },
                    location: "root".to_string(),
                });
            }
        }

        Self::parse_from_yaml(self.documents.as_ref().clone(), &self.key, None)
    }

    fn sanitize_scenario_hash(scenario_hash: &Hash) -> Hash {
        let mut sanitized = Hash::new();

        for allowed_key in ALLOWED_SCENARIO_KEYS.iter() {
            let key_yaml = StrictYaml::String(allowed_key.to_string());
            if let Some(v) = scenario_hash.get(&key_yaml) {
                if *allowed_key == "scenarios" {
                    if let StrictYaml::Hash(ref nested_scenarios) = *v {
                        let sanitized_nested = Self::sanitize_scenarios_hash(nested_scenarios);
                        sanitized.insert(key_yaml, StrictYaml::Hash(sanitized_nested));
                    } else {
                        sanitized.insert(key_yaml, v.clone());
                    }
                } else {
                    sanitized.insert(key_yaml, v.clone());
                }
            }
        }

        sanitized
    }

    fn sanitize_scenarios_hash(scenarios_hash: &Hash) -> Hash {
        let mut sanitized_scenarios: Vec<(String, StrictYaml)> = Vec::new();

        for (scenario_key, scenario_value) in scenarios_hash {
            let Some(scenario_key_str) = scenario_key.as_str() else {
                continue;
            };

            let StrictYaml::Hash(ref scenario_hash) = *scenario_value else {
                continue;
            };

            let sanitized_scenario = Self::sanitize_scenario_hash(scenario_hash);
            sanitized_scenarios.push((
                scenario_key_str.to_string(),
                StrictYaml::Hash(sanitized_scenario),
            ));
        }

        sanitized_scenarios.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut new_scenarios_hash = Hash::new();
        for (key, value) in sanitized_scenarios {
            new_scenarios_hash.insert(StrictYaml::String(key), value);
        }

        new_scenarios_hash
    }
}

impl YamlParsableHash for ScenarioCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut scenarios = HashMap::new();

        let rainlangs = RainlangCfg::parse_all_from_yaml(documents.clone(), context)?;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(scenarios_hash) = require_hash(&document_read, Some("scenarios"), None) {
                for (key_yaml, scenario_yaml) in scenarios_hash {
                    let scenario_key = key_yaml.as_str().unwrap_or_default().to_string();

                    let mut rainlang: Option<Arc<RainlangCfg>> = None;

                    Self::validate_scenario(
                        documents.clone(),
                        document.clone(),
                        &rainlangs,
                        &mut rainlang,
                        &mut scenarios,
                        ScenarioParent {
                            key: "".to_string(),
                            bindings: None,
                            rainlang: None,
                        },
                        scenario_key.clone(),
                        scenario_yaml,
                        context,
                    )?;

                    if rainlang.is_none() {
                        return Err(YamlError::ParseScenarioConfigSourceError(
                            ParseScenarioConfigSourceError::RainlangNotFound(scenario_key),
                        ));
                    }
                }
            }
        }

        if scenarios.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("scenarios".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(scenarios)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let scenarios_key = StrictYaml::String("scenarios".to_string());
            let Some(scenarios_value) = root_hash.get(&scenarios_key) else {
                continue;
            };
            let StrictYaml::Hash(ref scenarios_hash) = *scenarios_value else {
                continue;
            };

            let sanitized = Self::sanitize_scenarios_hash(scenarios_hash);
            root_hash.insert(scenarios_key, StrictYaml::Hash(sanitized));
        }

        Ok(())
    }
}

impl Default for ScenarioCfg {
    fn default() -> Self {
        Self {
            document: default_document(),
            documents: default_documents(),
            key: String::new(),
            bindings: HashMap::new(),
            runs: None,
            blocks: None,
            rainlang: Arc::new(RainlangCfg::default()),
        }
    }
}

impl PartialEq for ScenarioCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.bindings == other.bindings
            && self.runs == other.runs
            && self.blocks == other.blocks
            && self.rainlang == other.rainlang
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseScenarioConfigSourceError {
    #[error("Failed to parse runs")]
    RunsParseError(ParseIntError),
    #[error("Parent binding shadowed by child: {0}")]
    ParentBindingShadowedError(String),
    #[error("Parent rainlang shadowed by child: {0}")]
    ParentRainlangShadowedError(String),
    #[error("Rainlang not found: {0}")]
    RainlangNotFound(String),
    #[error("Parent raindex shadowed by child: {0}")]
    ParentRaindexShadowedError(String),
    #[error("Failed to parse blocks: {0}")]
    BlocksParseError(String),
}

impl ParseScenarioConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseScenarioConfigSourceError::RunsParseError(err) =>
                format!("The 'runs' value in your scenario YAML configuration must be a valid number: {}", err),
            ParseScenarioConfigSourceError::ParentBindingShadowedError(binding) =>
                format!("Binding conflict in your YAML configuration: The child scenario is trying to override the binding '{}' that was already defined in a parent scenario. Child scenarios cannot change binding values defined by parents.", binding),
            ParseScenarioConfigSourceError::ParentRainlangShadowedError(rainlang) =>
                format!("Rainlang conflict in your YAML configuration: The child scenario is trying to use rainlang '{}' which differs from the rainlang specified in the parent scenario. Child scenarios must use the same rainlang as their parent.", rainlang),
            ParseScenarioConfigSourceError::RainlangNotFound(scenario) =>
                format!("No rainlang was found for scenario '{}' in your YAML configuration. Please specify a rainlang for this scenario or ensure it inherits one from a parent scenario.", scenario),
            ParseScenarioConfigSourceError::ParentRaindexShadowedError(raindex) =>
                format!("Raindex conflict in your YAML configuration: The child scenario is trying to use raindex '{}' which differs from the raindex specified in the parent scenario. Child scenarios must use the same raindex as their parent.", raindex),
            ParseScenarioConfigSourceError::BlocksParseError(blocks) =>
                format!("Failed to parse the 'blocks' configuration in your YAML: {}. Please ensure it follows the correct format.", blocks),
        }
    }
}

#[derive(Default)]
pub struct ScenarioParent {
    key: String,
    bindings: Option<HashMap<String, String>>,
    rainlang: Option<Arc<RainlangCfg>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockCfg, BlockRangeCfg};
    use yaml::tests::get_document;

    #[test]
    fn test_parse_scenarios_from_yaml() {
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
test: test
"#;
        let error = ScenarioCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("scenarios".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'scenarios' in root"
        );

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
scenarios:
    scenario1:
        bindings:
            key1:
                - value1
"#;
        let error = ScenarioCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "value".to_string(),
                    expected: "a string".to_string()
                },
                location: "binding key 'key1' in scenario 'scenario1'".to_string()
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'value' in binding key 'key1' in scenario 'scenario1' must be a string"
        );

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
scenarios:
    scenario1:
        bindings:
            key1:
                - value1: value2
"#;
        let error = ScenarioCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "value".to_string(),
                    expected: "a string".to_string()
                },
                location: "binding key 'key1' in scenario 'scenario1'".to_string()
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Field 'value' in binding key 'key1' in scenario 'scenario1' must be a string"
        );

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
scenarios:
    scenario1:
        rainlang: mainnet
        bindings:
            key1: some-value
        scenarios:
            scenario2:
                bindings:
                    key1: value
"#;
        let error = ScenarioCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error.to_string(),
            YamlError::ParseScenarioConfigSourceError(
                ParseScenarioConfigSourceError::ParentBindingShadowedError("key1".to_string())
            )
            .to_string()
        );
        assert_eq!(error.to_readable_msg(), "Scenario configuration error in your YAML: Binding conflict in your YAML configuration: The child scenario is trying to override the binding 'key1' that was already defined in a parent scenario. Child scenarios cannot change binding values defined by parents.");

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
    testnet:
        rpcs:
            - https://rpc.com
        chain-id: 2
rainlangs:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
    testnet:
        address: 0x1234567890123456789012345678901234567890
        network: testnet
scenarios:
    scenario1:
        rainlang: mainnet
        bindings:
            key1: some-value
        scenarios:
            scenario2:
                bindings:
                    key2: value
                rainlang: testnet
"#;
        let error = ScenarioCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error.to_string(),
            YamlError::ParseScenarioConfigSourceError(
                ParseScenarioConfigSourceError::ParentRainlangShadowedError("testnet".to_string())
            )
            .to_string()
        );
        assert_eq!(error.to_readable_msg(), "Scenario configuration error in your YAML: Rainlang conflict in your YAML configuration: The child scenario is trying to use rainlang 'testnet' which differs from the rainlang specified in the parent scenario. Child scenarios must use the same rainlang as their parent.");
    }

    #[test]
    fn test_parse_scenarios_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
scenarios:
    scenario1:
        rainlang: mainnet
        bindings:
            key1: binding1
        scenarios:
            scenario2:
                bindings:
                    key2: binding2
"#;
        let yaml_two = r#"
scenarios:
    scenario3:
        rainlang: mainnet
        bindings:
            key3: binding3
        scenarios:
            scenario4:
                bindings:
                    key4: binding4
"#;
        let scenarios = ScenarioCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap();

        assert_eq!(scenarios.len(), 4);
        assert!(scenarios.contains_key("scenario1"));
        assert!(scenarios.contains_key("scenario1.scenario2"));
        assert!(scenarios.contains_key("scenario3"));
        assert!(scenarios.contains_key("scenario3.scenario4"));

        assert_eq!(
            scenarios
                .get("scenario1")
                .unwrap()
                .bindings
                .get("key1")
                .unwrap(),
            "binding1"
        );
        assert_eq!(
            scenarios
                .get("scenario1.scenario2")
                .unwrap()
                .bindings
                .get("key2")
                .unwrap(),
            "binding2"
        );
        assert_eq!(
            scenarios
                .get("scenario3")
                .unwrap()
                .bindings
                .get("key3")
                .unwrap(),
            "binding3"
        );
        assert_eq!(
            scenarios
                .get("scenario3.scenario4")
                .unwrap()
                .bindings
                .get("key4")
                .unwrap(),
            "binding4"
        );
    }

    #[test]
    fn test_parse_scenarios_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
rainlangs:
    mainnet:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
scenarios:
    DuplicateScenario:
        rainlang: mainnet
        bindings:
            key1: binding1
"#;
        let yaml_two = r#"
scenarios:
    DuplicateScenario:
        rainlang: mainnet
        bindings:
            key1: binding2
"#;

        let error = ScenarioCfg::parse_all_from_yaml(
            vec![get_document(yaml_one), get_document(yaml_two)],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateScenario".to_string(), "scenarios".to_string())
        );
        assert_eq!(error.to_readable_msg(), "The key 'DuplicateScenario' is defined multiple times in your YAML configuration at scenarios");
    }

    #[test]
    fn test_parse_scenario_blocks() {
        let prefix = r#"
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

        let simple_range = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        blocks: [1..2]
        bindings:
            key1: binding1
"#;
        let scenario = ScenarioCfg::parse_from_yaml(
            vec![get_document(prefix), get_document(simple_range)],
            "mainnet",
            None,
        )
        .unwrap();
        assert_eq!(
            scenario.blocks,
            Some(BlocksCfg::SimpleRange(BlockRangeCfg {
                start: BlockCfg::Number(1),
                end: BlockCfg::Number(2),
            }))
        );

        let simple_range_genesis = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        blocks: [..2]
        bindings:
            key1: binding1
"#;
        let scenario = ScenarioCfg::parse_from_yaml(
            vec![get_document(prefix), get_document(simple_range_genesis)],
            "mainnet",
            None,
        )
        .unwrap();
        assert_eq!(
            scenario.blocks,
            Some(BlocksCfg::SimpleRange(BlockRangeCfg {
                start: BlockCfg::Genesis,
                end: BlockCfg::Number(2),
            }))
        );

        let simple_range_latest = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        blocks: [1..]
        bindings:
            key1: binding1
"#;
        let scenario = ScenarioCfg::parse_from_yaml(
            vec![get_document(prefix), get_document(simple_range_latest)],
            "mainnet",
            None,
        )
        .unwrap();
        assert_eq!(
            scenario.blocks,
            Some(BlocksCfg::SimpleRange(BlockRangeCfg {
                start: BlockCfg::Number(1),
                end: BlockCfg::Latest,
            }))
        );

        let range = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        blocks:
            range: [1..2]
            interval: 10
        bindings:
            key1: binding1
"#;
        let scenario = ScenarioCfg::parse_from_yaml(
            vec![get_document(prefix), get_document(range)],
            "mainnet",
            None,
        )
        .unwrap();
        assert_eq!(
            scenario.blocks,
            Some(BlocksCfg::RangeWithInterval {
                range: BlockRangeCfg {
                    start: BlockCfg::Number(1),
                    end: BlockCfg::Number(2),
                },
                interval: 10,
            })
        );

        let range_genesis = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        blocks:
            range: [..2]
            interval: 10
        bindings:
            key1: binding1
"#;
        let scenario = ScenarioCfg::parse_from_yaml(
            vec![get_document(prefix), get_document(range_genesis)],
            "mainnet",
            None,
        )
        .unwrap();
        assert_eq!(
            scenario.blocks,
            Some(BlocksCfg::RangeWithInterval {
                range: BlockRangeCfg {
                    start: BlockCfg::Genesis,
                    end: BlockCfg::Number(2),
                },
                interval: 10,
            })
        );

        let range_latest = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        blocks:
            range: [1..]
            interval: 10
        bindings:
            key1: binding1
"#;
        let scenario = ScenarioCfg::parse_from_yaml(
            vec![get_document(prefix), get_document(range_latest)],
            "mainnet",
            None,
        )
        .unwrap();
        assert_eq!(
            scenario.blocks,
            Some(BlocksCfg::RangeWithInterval {
                range: BlockRangeCfg {
                    start: BlockCfg::Number(1),
                    end: BlockCfg::Latest,
                },
                interval: 10,
            })
        );
    }

    #[test]
    fn test_sanitize_drops_unknown_keys() {
        let yaml = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        bindings:
            key1: value1
        runs: 10
        unknown-key: should-be-removed
        another-unknown: also-removed
"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let scenarios = root
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let mainnet = scenarios
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(mainnet.contains_key(&StrictYaml::String("rainlang".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("bindings".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("runs".to_string())));
        assert!(!mainnet.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!mainnet.contains_key(&StrictYaml::String("another-unknown".to_string())));
    }

    #[test]
    fn test_sanitize_preserves_all_allowed_keys() {
        let yaml = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        bindings:
            key1: value1
        runs: 10
        blocks: [1..2]
        scenarios:
            nested:
                bindings:
                    key2: value2
"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let scenarios = root
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let mainnet = scenarios
            .get(&StrictYaml::String("mainnet".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(mainnet.contains_key(&StrictYaml::String("rainlang".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("bindings".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("runs".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("blocks".to_string())));
        assert!(mainnet.contains_key(&StrictYaml::String("scenarios".to_string())));
    }

    #[test]
    fn test_sanitize_nested_scenarios_drops_unknown_keys() {
        let yaml = r#"
scenarios:
    parent:
        rainlang: mainnet
        bindings:
            key1: value1
        scenarios:
            child:
                bindings:
                    key2: value2
                unknown-nested: should-be-removed
                scenarios:
                    grandchild:
                        bindings:
                            key3: value3
                        deep-unknown: also-removed
"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let scenarios = root
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let parent = scenarios
            .get(&StrictYaml::String("parent".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let nested = parent
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let child = nested
            .get(&StrictYaml::String("child".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(child.contains_key(&StrictYaml::String("bindings".to_string())));
        assert!(child.contains_key(&StrictYaml::String("scenarios".to_string())));
        assert!(!child.contains_key(&StrictYaml::String("unknown-nested".to_string())));

        let grandchild_nested = child
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let grandchild = grandchild_nested
            .get(&StrictYaml::String("grandchild".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(grandchild.contains_key(&StrictYaml::String("bindings".to_string())));
        assert!(!grandchild.contains_key(&StrictYaml::String("deep-unknown".to_string())));
    }

    #[test]
    fn test_sanitize_drops_non_hash_scenario_entries() {
        let yaml = r#"
scenarios:
    mainnet:
        rainlang: mainnet
        bindings:
            key1: value1
    invalid-string: just-a-string
"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let scenarios = root
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert!(scenarios.contains_key(&StrictYaml::String("mainnet".to_string())));
        assert!(!scenarios.contains_key(&StrictYaml::String("invalid-string".to_string())));
    }

    #[test]
    fn test_sanitize_sorts_scenarios_lexicographically() {
        let yaml = r#"
scenarios:
    zebra:
        rainlang: mainnet
    alpha:
        rainlang: mainnet
    mainnet:
        rainlang: mainnet
"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let scenarios = root
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let keys: Vec<_> = scenarios.keys().map(|k| k.as_str().unwrap()).collect();
        assert_eq!(keys, vec!["alpha", "mainnet", "zebra"]);
    }

    #[test]
    fn test_sanitize_handles_missing_scenarios_section() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - https://rpc.com
        chain-id: 1
"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        assert!(!root.contains_key(&StrictYaml::String("scenarios".to_string())));
    }

    #[test]
    fn test_sanitize_handles_non_hash_root() {
        let yaml = r#"just-a-string"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        assert!(doc_read.as_str().is_some());
    }

    #[test]
    fn test_sanitize_skips_non_hash_scenarios_section() {
        let yaml = r#"
scenarios: not-a-hash
"#;
        let doc = get_document(yaml);
        ScenarioCfg::sanitize_documents(std::slice::from_ref(&doc)).unwrap();

        let doc_read = doc.read().unwrap();
        let root = doc_read.as_hash().unwrap();
        let scenarios = root
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap();
        assert_eq!(scenarios.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_per_document_isolation() {
        let yaml1 = r#"
scenarios:
    from-doc1:
        rainlang: mainnet
        extra-key: removed
"#;
        let yaml2 = r#"
scenarios:
    from-doc2:
        rainlang: testnet
        another-extra: also-removed
"#;
        let doc1 = get_document(yaml1);
        let doc2 = get_document(yaml2);

        ScenarioCfg::sanitize_documents(&[doc1.clone(), doc2.clone()]).unwrap();

        let doc1_read = doc1.read().unwrap();
        let root1 = doc1_read.as_hash().unwrap();
        let scenarios1 = root1
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let from_doc1 = scenarios1
            .get(&StrictYaml::String("from-doc1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert!(!from_doc1.contains_key(&StrictYaml::String("extra-key".to_string())));
        assert!(!scenarios1.contains_key(&StrictYaml::String("from-doc2".to_string())));

        let doc2_read = doc2.read().unwrap();
        let root2 = doc2_read.as_hash().unwrap();
        let scenarios2 = root2
            .get(&StrictYaml::String("scenarios".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let from_doc2 = scenarios2
            .get(&StrictYaml::String("from-doc2".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        assert!(!from_doc2.contains_key(&StrictYaml::String("another-extra".to_string())));
        assert!(!scenarios2.contains_key(&StrictYaml::String("from-doc1".to_string())));
    }
}
