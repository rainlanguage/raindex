use crate::blocks::BlocksCfg;
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{
    impl_wasm_traits, prelude::*, serialize_hashmap_as_object, serialize_opt_hashmap_as_object,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct UnitTestConfigSource {
    pub test: TestConfigSource,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(UnitTestConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ScenarioConfigSource {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(optional, type = "Record<string, string>")
    )]
    pub bindings: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<BlocksCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rainlang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_opt_hashmap_as_object"),
        tsify(optional, type = "Record<string, ScenarioConfigSource>")
    )]
    pub scenarios: Option<HashMap<String, ScenarioConfigSource>>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ScenarioConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TestConfigSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculate_entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle_entrypoint: Option<String>,
    pub scenario_name: String,
    pub scenario: ScenarioConfigSource,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TestConfigSource);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct TestConfig {
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub calculate_entrypoint: Option<String>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub handle_entrypoint: Option<String>,
    pub scenario_name: String,
    pub scenario: Arc<ScenarioCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(TestConfig);

impl TestConfigSource {
    pub fn into_test_config(self) -> TestConfig {
        let mut bindings = HashMap::new();
        for (k, v) in &self.scenario.bindings {
            bindings.insert(k.to_string(), v.to_string());
        }

        let scenario = Arc::new(ScenarioCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            documents: Arc::new(Vec::new()),
            key: self.scenario_name.clone(),
            bindings: bindings.clone(),
            runs: self.scenario.runs,
            blocks: self.scenario.blocks.clone(),
            rainlang: Arc::new(RainlangCfg::dummy()),
        });

        TestConfig {
            calculate_entrypoint: self.calculate_entrypoint,
            handle_entrypoint: self.handle_entrypoint,
            scenario_name: self.scenario_name.clone(),
            scenario,
        }
    }
}
