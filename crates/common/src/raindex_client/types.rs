use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct ChainIds(#[tsify(type = "number[]")] pub Vec<u32>);
impl_wasm_traits!(ChainIds);

#[derive(Serialize, Deserialize, Debug, Clone, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct TimeFilter {
    pub start: Option<u64>,
    pub end: Option<u64>,
}
impl_wasm_traits!(TimeFilter);

#[derive(Serialize, Deserialize, Debug, Clone, Default, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    pub page: Option<u16>,
    pub page_size: Option<u16>,
}
impl_wasm_traits!(PaginationParams);
