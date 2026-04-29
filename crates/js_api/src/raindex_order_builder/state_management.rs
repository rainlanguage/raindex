use super::*;
use rain_orderbook_common::raindex_order_builder::state_management as inner_sm;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AllBuilderConfig {
    pub field_definitions_without_defaults: Vec<OrderBuilderFieldDefinitionCfg>,
    pub field_definitions_with_defaults: Vec<OrderBuilderFieldDefinitionCfg>,
    pub deposits: Vec<rain_orderbook_app_settings::order_builder::OrderBuilderDepositCfg>,
    pub order_inputs: Vec<rain_orderbook_app_settings::order::OrderIOCfg>,
    pub order_outputs: Vec<rain_orderbook_app_settings::order::OrderIOCfg>,
}
impl_wasm_traits!(AllBuilderConfig);

#[wasm_export]
impl RaindexOrderBuilder {
    #[wasm_export(
        js_name = "executeStateUpdateCallback",
        unchecked_return_type = "void",
        return_description = "Callback executed successfully or no callback registered"
    )]
    pub fn execute_state_update_callback(&self) -> Result<(), RaindexOrderBuilderWasmError> {
        if let Some(callback) = &self.state_update_callback {
            let serialized = self.inner.serialize_state()?;
            let js_value = JsValue::from_str(&serialized);
            callback.call1(&JsValue::NULL, &js_value).map_err(|e| {
                RaindexOrderBuilderWasmError::JsError(
                    e.as_string().unwrap_or("callback error".to_string()),
                )
            })?;
        }
        Ok(())
    }
}

#[wasm_export]
impl RaindexOrderBuilder {
    #[wasm_export(
        js_name = "serializeState",
        unchecked_return_type = "string",
        return_description = "Base64-encoded compressed serialized state"
    )]
    pub fn serialize_state(&self) -> Result<String, RaindexOrderBuilderWasmError> {
        Ok(self.inner.serialize_state()?)
    }

    #[wasm_export(
        js_name = "newFromState",
        preserve_js_class,
        return_description = "Restored builder instance"
    )]
    pub async fn new_from_state(
        #[wasm_export(param_description = "Complete dotrain YAML content")] dotrain: String,
        #[wasm_export(param_description = "Optional additional YAML settings")] settings: Option<
            Vec<String>,
        >,
        #[wasm_export(param_description = "Serialized state string from serializeState")]
        serialized: String,
        #[wasm_export(param_description = "Optional state update callback function")]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<RaindexOrderBuilder, RaindexOrderBuilderWasmError> {
        let inner = RaindexOrderBuilderInner::new_from_state(dotrain, settings, serialized).await?;
        Ok(RaindexOrderBuilder {
            inner,
            state_update_callback,
        })
    }

    #[wasm_export(
        js_name = "getAllBuilderConfig",
        unchecked_return_type = "AllBuilderConfig",
        return_description = "Complete builder configuration for all fields, deposits, and I/O"
    )]
    pub fn get_all_builder_config(
        &self,
    ) -> Result<inner_sm::AllBuilderConfig, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_all_builder_config()?)
    }

    #[wasm_export(
        js_name = "computeStateHash",
        unchecked_return_type = "string",
        return_description = "Base64-encoded SHA256 hash of the dotrain content"
    )]
    pub async fn compute_state_hash(
        #[wasm_export(param_description = "Complete dotrain YAML content")] dotrain: String,
        #[wasm_export(param_description = "Optional additional YAML settings")] settings: Option<
            Vec<String>,
        >,
    ) -> Result<String, RaindexOrderBuilderWasmError> {
        let dotrain_order =
            rain_orderbook_common::dotrain_order::DotrainOrder::create(dotrain, settings)
                .await
                .map_err(RaindexOrderBuilderError::from)?;
        Ok(RaindexOrderBuilderInner::compute_state_hash(
            &dotrain_order,
        )?)
    }

    #[wasm_export(
        js_name = "generateDotrainBuilderStateInstanceV1",
        unchecked_return_type = "OrderBuilderStateV1",
        return_description = "Builder state instance for metadata embedding"
    )]
    pub fn generate_dotrain_builder_state_instance_v1(
        &self,
    ) -> Result<
        rain_metadata::types::dotrain::order_builder_state_v1::OrderBuilderStateV1,
        RaindexOrderBuilderWasmError,
    > {
        Ok(self.inner.generate_dotrain_builder_state_instance_v1()?)
    }
}
