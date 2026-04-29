pub use rain_interpreter_eval::trace::*;
use raindex_app_settings::chart::ChartCfg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, serialize_hashmap_as_object};

#[cfg(not(target_family = "wasm"))]
mod impls;
#[cfg(not(target_family = "wasm"))]
pub use impls::*;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ChartData {
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, FuzzResultFlat>")
    )]
    pub scenarios_data: HashMap<String, FuzzResultFlat>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ChartCfg>")
    )]
    pub charts: HashMap<String, ChartCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ChartData);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FuzzResultFlat {
    pub scenario: String,
    pub data: RainEvalResultsTable,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(FuzzResultFlat);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DeploymentsDebugDataMap {
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, DeploymentDebugData>")
    )]
    pub data_map: HashMap<String, DeploymentDebugData>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentsDebugDataMap);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DeploymentDebugData {
    pub pairs_data: Vec<DeploymentDebugPairData>,
    pub block_number: u64,
    pub chain_id: u32,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentDebugData);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DeploymentDebugPairData {
    pub order: String,
    pub scenario: String,
    pub pair: String,
    pub result: Option<FuzzResultFlat>,
    pub error: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentDebugPairData);
