use super::*;
use rain_metadata::types::dotrain::{
    gui_state_v1::{DotrainGuiStateV1, ShortenedTokenCfg, ValueCfg},
    source_v1::DotrainSourceV1,
};
use raindex_app_settings::{
    gui::GuiDepositCfg,
    order::{OrderIOCfg, VaultType},
    token::TokenCfg,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct AllGuiConfig {
    pub field_definitions_without_defaults: Vec<GuiFieldDefinitionCfg>,
    pub field_definitions_with_defaults: Vec<GuiFieldDefinitionCfg>,
    pub deposits: Vec<GuiDepositCfg>,
    pub order_inputs: Vec<OrderIOCfg>,
    pub order_outputs: Vec<OrderIOCfg>,
}
impl_wasm_traits!(AllGuiConfig);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedGuiState {
    field_values: BTreeMap<String, GuiPresetCfg>,
    deposits: BTreeMap<String, GuiPresetCfg>,
    select_tokens: BTreeMap<String, TokenCfg>,
    vault_ids: BTreeMap<(VaultType, String), Option<String>>,
    dotrain_hash: String,
    selected_deployment: String,
}

#[wasm_export]
impl DotrainOrderGui {
    fn create_preset(value: &field_values::PairValue, default_value: String) -> GuiPresetCfg {
        if value.is_preset {
            GuiPresetCfg {
                id: value.value.clone(),
                name: None,
                value: default_value,
            }
        } else {
            GuiPresetCfg {
                id: "".to_string(),
                name: None,
                value: value.value.clone(),
            }
        }
    }

    fn preset_to_pair_value(preset: GuiPresetCfg) -> field_values::PairValue {
        if !preset.id.is_empty() {
            field_values::PairValue {
                is_preset: true,
                value: preset.id,
            }
        } else {
            field_values::PairValue {
                is_preset: false,
                value: preset.value,
            }
        }
    }

    fn parse_vault_ids_for_order(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        order_key: &str,
        is_input: bool,
    ) -> Result<BTreeMap<(VaultType, String), Option<String>>, GuiError> {
        let mut vault_ids = BTreeMap::new();
        let r#type = if is_input {
            VaultType::Input
        } else {
            VaultType::Output
        };
        for (token, vault_id) in OrderCfg::parse_vault_ids(documents, order_key, r#type)? {
            vault_ids.insert((r#type, token), vault_id.as_ref().map(|v| v.to_string()));
        }
        Ok(vault_ids)
    }

    #[wasm_export(skip)]
    pub fn generate_dotrain_gui_state_instance_v1(&self) -> Result<DotrainGuiStateV1, GuiError> {
        let trimmed_dotrain = self
            .dotrain_order
            .generate_dotrain_for_deployment(&self.selected_deployment)?;
        let dotrain_hash = DotrainSourceV1(trimmed_dotrain.clone()).hash();

        // Use normalized deposit amounts (resolve presets to actual values)
        let deposits = self
            .get_deposits()?
            .into_iter()
            .map(|d| {
                (
                    d.token.clone(),
                    ValueCfg {
                        id: d.token,
                        name: None,
                        value: d.amount,
                    },
                )
            })
            .collect();

        // Prefer the resolved tokens from the current deployment (captures user selections)
        let select_tokens = {
            let mut result = BTreeMap::new();
            let deployment = self.get_current_deployment()?;
            let network_key = deployment.deployment.order.network.key.clone();

            // Build a key->address map from inputs/outputs that reflects current state
            let mut resolved = HashMap::new();
            for io in deployment
                .deployment
                .order
                .inputs
                .iter()
                .chain(deployment.deployment.order.outputs.iter())
            {
                if let Some(tok) = &io.token {
                    resolved.insert(tok.key.clone(), tok.address);
                }
            }

            // Emit only the tokens configured for selection in this deployment
            if let Some(st) = GuiCfg::parse_select_tokens(
                self.dotrain_order.dotrain_yaml().documents,
                &self.selected_deployment,
            )? {
                for s in st {
                    if let Some(addr) = resolved.get(&s.key) {
                        result.insert(
                            s.key,
                            ShortenedTokenCfg {
                                network: network_key.clone(),
                                address: *addr,
                            },
                        );
                    }
                }
            }
            result
        };

        // Convert vault_ids to "{io_type}_{index}" keys where index matches IO position
        // in the order's inputs/outputs arrays for deterministic reconstruction.
        let deployment = self.get_current_deployment()?;
        let mut vault_ids = BTreeMap::new();
        for (i, input) in deployment.deployment.order.inputs.iter().enumerate() {
            let key = format!("input_{}", i);
            let value = input.vault_id.map(|v| format!("0x{:x}", v));
            vault_ids.insert(key, value);
        }
        for (i, output) in deployment.deployment.order.outputs.iter().enumerate() {
            let key = format!("output_{}", i);
            let value = output.vault_id.map(|v| format!("0x{:x}", v));
            vault_ids.insert(key, value);
        }

        // Convert field values to ValueCfg with normalized value and optional preset ID
        let field_values = self
            .field_values
            .iter()
            .map(|(k, v)| {
                let normalized = self.get_field_value(k.clone())?;
                Ok((
                    k.clone(),
                    ValueCfg {
                        // Preserve preset linkage if applicable; otherwise use the binding key
                        id: if v.is_preset {
                            v.value.clone()
                        } else {
                            k.clone()
                        },
                        name: None,
                        value: normalized.value,
                    },
                ))
            })
            .collect::<Result<_, GuiError>>()?;

        Ok(DotrainGuiStateV1 {
            dotrain_hash,
            field_values,
            deposits,
            select_tokens,
            vault_ids,
            selected_deployment: self.selected_deployment.clone(),
        })
    }

    /// Exports the complete GUI state as a compressed, encoded string.
    ///
    /// Serializes all current configuration including field values, deposits,
    /// selected tokens, and vault IDs into a compact format for persistence
    /// or sharing. The output is gzipped and base64-encoded.
    ///
    /// ## State Contents
    ///
    /// - Field values with preset information
    /// - Deposit amounts with preset references
    /// - Selected token configurations
    /// - Vault ID assignments
    /// - Configuration hash for validation
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.serializeState();
    /// if (result.error) {
    ///   console.error("Serialization error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const state = result.value;
    /// // Do something with the state
    /// ```
    #[wasm_export(
        js_name = "serializeState",
        unchecked_return_type = "string",
        return_description = "Compressed, base64-encoded state data"
    )]
    pub fn serialize_state(&self) -> Result<String, GuiError> {
        let mut field_values = BTreeMap::new();
        for (k, v) in self.field_values.iter() {
            let preset = if v.is_preset {
                let presets = GuiCfg::parse_field_presets(
                    self.dotrain_order.dotrain_yaml().documents.clone(),
                    &self.selected_deployment,
                    k,
                )?
                .ok_or(GuiError::BindingHasNoPresets(k.clone()))?;
                presets
                    .iter()
                    .find(|preset| preset.id == v.value)
                    .ok_or(GuiError::InvalidPreset)?
                    .clone()
            } else {
                Self::create_preset(v, String::default())
            };
            field_values.insert(k.clone(), preset);
        }

        let mut deposits = BTreeMap::new();
        for (k, v) in self.deposits.iter() {
            let preset = Self::create_preset(v, String::default());
            deposits.insert(k.clone(), preset);
        }

        let mut select_tokens: BTreeMap<String, TokenCfg> = BTreeMap::new();
        if let Some(st) = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &self.selected_deployment,
        )? {
            for select_token in st {
                if let Ok(token) = self
                    .dotrain_order
                    .raindex_yaml()
                    .get_token(&select_token.key)
                {
                    select_tokens.insert(select_token.key, token);
                }
            }
        }

        let order_key = DeploymentCfg::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &self.selected_deployment,
        )?;
        let mut vault_ids = BTreeMap::new();
        vault_ids.extend(Self::parse_vault_ids_for_order(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &order_key,
            true,
        )?);
        vault_ids.extend(Self::parse_vault_ids_for_order(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &order_key,
            false,
        )?);

        let state = SerializedGuiState {
            field_values: field_values.clone(),
            deposits: deposits.clone(),
            select_tokens: select_tokens.clone(),
            vault_ids: vault_ids.clone(),
            dotrain_hash: self.dotrain_hash.clone(),
            selected_deployment: self.selected_deployment.clone(),
        };
        let bytes = bincode::serialize(&state)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let compressed = encoder.finish()?;

        Ok(URL_SAFE.encode(compressed))
    }

    /// Restores a GUI instance from previously serialized state.
    ///
    /// Creates a new GUI instance with all configuration restored from a saved state.
    /// The provided dotrain should be the full template that was used when
    /// serializing the state. Hash validation is performed against its
    /// trimmed-for-deployment form to keep the template user-agnostic.
    ///
    /// ## Security
    ///
    /// The function validates that the dotrain content hasn't changed by comparing
    /// hashes. This prevents state injection attacks and ensures consistency.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await DotrainOrderGui.newFromState(dotrainYaml, savedState);
    /// if (result.error) {
    ///   console.error("Restore failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const gui = result.value;
    /// // Do something with the gui
    /// ```
    #[wasm_export(
        js_name = "newFromState",
        preserve_js_class,
        return_description = "Fully restored GUI instance"
    )]
    pub async fn new_from_state(
        #[wasm_export(param_description = "Must match the original dotrain content exactly")]
        dotrain: String,
        #[wasm_export(
            param_description = "Optional additional YAML configuration strings to merge with the frontmatter"
        )]
        settings: Option<Vec<String>>,
        #[wasm_export(param_description = "Previously serialized state string")] serialized: String,
        #[wasm_export(param_description = "Optional callback for future state changes")]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<DotrainOrderGui, GuiError> {
        let compressed = URL_SAFE.decode(serialized)?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;
        let state: SerializedGuiState = bincode::deserialize(&bytes)?;

        let dotrain_order = DotrainOrder::create_with_profile(
            dotrain.clone(),
            settings,
            ContextProfile::gui(state.selected_deployment.clone()),
        )
        .await?;

        let original_dotrain_hash = DotrainOrderGui::compute_state_hash(&dotrain_order)?;
        if original_dotrain_hash != state.dotrain_hash {
            return Err(GuiError::DotrainMismatch);
        }

        let field_values = state
            .field_values
            .into_iter()
            .map(|(k, v)| (k, Self::preset_to_pair_value(v)))
            .collect::<BTreeMap<_, _>>();

        let deposits = state
            .deposits
            .into_iter()
            .map(|(k, v)| (k, Self::preset_to_pair_value(v)))
            .collect::<BTreeMap<_, _>>();

        let dotrain_order_gui = DotrainOrderGui {
            dotrain_order,
            field_values,
            deposits,
            selected_deployment: state.selected_deployment.clone(),
            dotrain_hash: original_dotrain_hash,
            state_update_callback,
        };

        let deployment_select_tokens = GuiCfg::parse_select_tokens(
            dotrain_order_gui.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for (key, token) in state.select_tokens {
            let select_tokens = deployment_select_tokens
                .as_ref()
                .ok_or(GuiError::SelectTokensNotSet)?;
            if !select_tokens.iter().any(|token| token.key == key) {
                return Err(GuiError::TokenNotInSelectTokens(key));
            }
            if dotrain_order_gui.is_select_token_set(key.clone())? {
                TokenCfg::remove_record_from_yaml(
                    dotrain_order_gui.dotrain_order.raindex_yaml().documents,
                    &key,
                )?;
            }
            TokenCfg::add_record_to_yaml(
                dotrain_order_gui.dotrain_order.raindex_yaml().documents,
                &key,
                &token.network.key,
                &token.address.to_string(),
                token.decimals.map(|d| d.to_string()).as_deref(),
                token.label.map(|l| l.to_string()).as_deref(),
                token.symbol.map(|s| s.to_string()).as_deref(),
            )?;
        }

        let order_key = DeploymentCfg::parse_order_key(
            dotrain_order_gui.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for ((is_input, index), vault_id) in state.vault_ids {
            dotrain_order_gui
                .dotrain_order
                .dotrain_yaml()
                .get_order_for_gui_deployment(&order_key, &state.selected_deployment)
                .and_then(|mut order| order.update_vault_id(is_input, index, vault_id))?;
        }

        Ok(dotrain_order_gui)
    }

    /// Manually triggers the state update callback.
    ///
    /// Calls the registered state update callback with the current serialized state.
    /// This is typically called automatically after state-changing operations,
    /// but can be triggered manually if needed.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Manual state update trigger
    /// const result = gui.executeStateUpdateCallback();
    /// if (result.error) {
    ///   console.error("Callback error:", result.error.readableMsg);
    /// }
    /// ```
    #[wasm_export(
        js_name = "executeStateUpdateCallback",
        unchecked_return_type = "void",
        return_description = "Callback executed successfully or no callback registered"
    )]
    pub fn execute_state_update_callback(&self) -> Result<(), GuiError> {
        if let Some(callback) = &self.state_update_callback {
            let state = to_js_value(&self.serialize_state()?)?;
            callback.call1(&JsValue::UNDEFINED, &state).map_err(|e| {
                GuiError::JsError(format!("Failed to execute state update callback: {:?}", e))
            })?;
        }
        Ok(())
    }

    /// Gets comprehensive configuration for complete UI initialization.
    ///
    /// Provides all configuration data needed to build a complete order interface,
    /// organized by requirement type for progressive UI construction.
    ///
    /// ## Configuration Structure
    ///
    /// - `fieldDefinitionsWithoutDefaults` - Required fields needing user input
    /// - `fieldDefinitionsWithDefaults` - Optional fields with fallback values
    /// - `deposits` - Deposit configurations with tokens and presets
    /// - `orderInputs` - Input token configurations
    /// - `orderOutputs` - Output token configurations
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getAllGuiConfig();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const config = result.value;
    ///
    /// // Build required fields section
    /// config.fieldDefinitionsWithoutDefaults.forEach(field => {
    ///   // Do something with the fields
    /// });
    ///
    /// // Build optional fields section
    /// if (config.fieldDefinitionsWithDefaults.length > 0) {
    ///   // Do something with the fields
    /// }
    ///
    /// // Build deposits section
    /// config.deposits.forEach(deposit => {
    ///   // Do something with the deposits
    /// });
    ///
    /// // Show order preview
    /// config.orderInputs.forEach(input => {
    ///   // Do something with the order inputs
    /// });
    /// config.orderOutputs.forEach(output => {
    ///   // Do something with the order outputs
    /// });
    /// ```
    #[wasm_export(
        js_name = "getAllGuiConfig",
        unchecked_return_type = "AllGuiConfig",
        return_description = "Complete configuration package"
    )]
    pub fn get_all_gui_config(&self) -> Result<AllGuiConfig, GuiError> {
        let deployment = self.get_current_deployment()?;

        let field_definitions_without_defaults = self.get_all_field_definitions(Some(false))?;
        let field_definitions_with_defaults = self.get_all_field_definitions(Some(true))?;
        let deposits = deployment.deposits;
        let order_inputs = deployment.deployment.order.inputs.clone();
        let order_outputs = deployment.deployment.order.outputs.clone();

        Ok(AllGuiConfig {
            field_definitions_without_defaults,
            field_definitions_with_defaults,
            deposits,
            order_inputs,
            order_outputs,
        })
    }
}
impl DotrainOrderGui {
    pub fn compute_state_hash(dotrain_order: &DotrainOrder) -> Result<String, GuiError> {
        let yaml = emitter::emit_documents(&dotrain_order.dotrain_yaml().documents)?;

        let rain_document = RainDocument::create(dotrain_order.dotrain()?, None, None, None);
        let rainlang_body = rain_document.body().to_string();

        let tuple = (yaml, rainlang_body);
        let dotrain_bytes = bincode::serialize(&tuple)?;

        let hash = Sha256::digest(dotrain_bytes);
        Ok(URL_SAFE.encode(hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        field_values::FieldValue,
        tests::{get_yaml, initialize_gui_with_select_tokens},
    };
    use alloy::primitives::{Address, U256};
    use js_sys::{eval, Reflect};
    use raindex_app_settings::{network::NetworkCfg, order::VaultType, yaml::YamlParsableHash};
    use raindex_common::dotrain::RainDocument;
    use raindex_common::dotrain_order::DotrainOrder;
    use std::str::FromStr;
    use wasm_bindgen_test::wasm_bindgen_test;

    const SERIALIZED_STATE: &str = "H4sIAAAAAAAA_21Qy27CMBCMadWqUk-oUk-V-gG1EoeiJki9tEK8BFwgAi4oBEOiGDsE84j4CT4ZAusgIvawM-sZ765d0C7xAjgJ-DTgc0w0FQ-AxDDyJhPBgaFlTJEnQClCykv3ut133lavUK3EgmJO5VbEobr3AehLGVV0nQnPZb5YyYplWGU9jjy8jtk-daA0IzW62qu_AS1-O7tDLqEiega5l-7wWUKPqm51Ti8paNe42ZZkI4hto7xqZqpp219AcU3MfjbjRpgktbbc-oMlHzl_5dANOoTM3Xbc7f9bTlTFzeHvu_oLyqgn8bkpntKIiWRBuTwC_jr8_soBAAA=";

    fn encode_state(state: &SerializedGuiState) -> String {
        let bytes = bincode::serialize(state).unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes).unwrap();
        let compressed = encoder.finish().unwrap();
        URL_SAFE.encode(compressed)
    }

    async fn configured_gui() -> DotrainOrderGui {
        let mut gui = initialize_gui_with_select_tokens().await;

        gui.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "TKN3".to_string(),
        );
        gui.set_deposit("token3".to_string(), "100".to_string())
            .await
            .unwrap();
        gui.set_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();
        gui.set_vault_id(
            VaultType::Input,
            "token1".to_string(),
            Some("199".to_string()),
        )
        .unwrap();
        gui.set_vault_id(
            VaultType::Output,
            "token2".to_string(),
            Some("299".to_string()),
        )
        .unwrap();

        gui
    }

    #[wasm_bindgen_test]
    async fn test_compute_state_hash_changes_on_content_change() {
        let dotrain = get_yaml();
        let order1 = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let original_hash = DotrainOrderGui::compute_state_hash(&order1).unwrap();

        let modified = dotrain.replace("Select token deployment", "Select token deployment v2");
        let order2 = DotrainOrder::create(modified, None).await.unwrap();
        let modified_hash = DotrainOrderGui::compute_state_hash(&order2).unwrap();
        assert_ne!(original_hash, modified_hash);

        let order3 = DotrainOrder::create(get_yaml(), None).await.unwrap();
        let repeated_hash = DotrainOrderGui::compute_state_hash(&order3).unwrap();
        assert_eq!(original_hash, repeated_hash);
    }

    #[wasm_bindgen_test]
    async fn test_generate_dotrain_gui_state_instance_v1_contents() {
        let gui = configured_gui().await;
        let state = gui.generate_dotrain_gui_state_instance_v1().unwrap();

        let trimmed = gui
            .dotrain_order
            .generate_dotrain_for_deployment(&gui.selected_deployment)
            .unwrap();
        let expected_hash = DotrainSourceV1(trimmed).hash();
        assert_eq!(state.dotrain_hash, expected_hash);
        assert_eq!(state.selected_deployment, "select-token-deployment");

        let binding_1 = state.field_values.get("binding-1").unwrap();
        assert_eq!(binding_1.id, "binding-1");
        assert_eq!(binding_1.value, "100");

        let binding_2 = state.field_values.get("binding-2").unwrap();
        assert_eq!(binding_2.id, "0");
        assert_eq!(binding_2.value, "0");

        let deposit = state.deposits.get("token3").unwrap();
        assert_eq!(deposit.id, "token3");
        assert_eq!(deposit.value, "100");

        assert!(state.select_tokens.is_empty());
        assert_eq!(
            state.vault_ids.get("input_0"),
            Some(&Some("0xc7".to_string()))
        );
        assert_eq!(
            state.vault_ids.get("output_0"),
            Some(&Some("0x12b".to_string()))
        );
    }

    #[wasm_bindgen_test]
    async fn test_serialize_state() {
        let gui = configured_gui().await;
        let state = gui.serialize_state().unwrap();
        assert!(!state.is_empty());
        assert_eq!(state, SERIALIZED_STATE);
    }

    #[wasm_bindgen_test]
    async fn test_new_from_state() {
        let gui =
            DotrainOrderGui::new_from_state(get_yaml(), None, SERIALIZED_STATE.to_string(), None)
                .await
                .unwrap();

        assert!(gui.is_select_token_set("token3".to_string()).unwrap());
        assert_eq!(gui.get_deposits().unwrap()[0].amount, "100");
        assert_eq!(
            gui.get_field_value("binding-1".to_string()).unwrap(),
            FieldValue {
                field: "binding-1".to_string(),
                value: "100".to_string(),
                is_preset: false,
            }
        );
        assert_eq!(
            gui.get_field_value("binding-2".to_string()).unwrap(),
            FieldValue {
                field: "binding-2".to_string(),
                value: "0".to_string(),
                is_preset: true,
            }
        );
        let vault_ids = gui.get_vault_ids().unwrap().0;
        assert_eq!(
            vault_ids.get("input").unwrap()["token1"],
            Some(U256::from(199))
        );
        assert_eq!(
            vault_ids.get("output").unwrap()["token2"],
            Some(U256::from(299))
        );
    }

    #[wasm_bindgen_test]
    async fn test_new_from_state_invalid_dotrain() {
        let dotrain = r#"
            version: 6
            networks:
                test:
                    rpcs:
                        - http://localhost:8085/rpc-url
                    chain-id: 123
            subgraphs:
                test: http://localhost:8085/rpc-url
            tokens:
                token1:
                    network: test
                    address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
            rainlangs:
                test:
                    network: test
                    address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
            raindexes:
                test:
                    address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
                    network: test
                    subgraph: test
                    deployment-block: 12345
            scenarios:
                test:
                    rainlang: test
            orders:
                test:
                    inputs:
                        - token: token1
                    outputs:
                        - token: token1
                    rainlang: test
                    raindex: test
            deployments:
                select-token-deployment:
                    order: test
                    scenario: test
            gui:
                name: Test
                description: Fixed limit order
                deployments:
                    select-token-deployment:
                        name: Test deployment
                        description: Test description
                        deposits:
                            - token: token1
                        fields:
                            - binding: binding-1
                              name: Field 1 name
        ---
        #test
        "#;

        let err = DotrainOrderGui::new_from_state(
            dotrain.to_string(),
            None,
            SERIALIZED_STATE.to_string(),
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(err.to_string(), GuiError::DotrainMismatch.to_string());
        assert_eq!(
            err.to_readable_msg(),
            "There was a mismatch in the dotrain configuration. Please check your YAML configuration for consistency."
        );
    }

    #[wasm_bindgen_test]
    async fn test_new_from_state_rejects_unknown_select_token_key() {
        let dotrain = get_yaml();
        let documents = DotrainOrderGui::get_yaml_documents(&dotrain, None).unwrap();
        let token = TokenCfg::parse_from_yaml(documents.clone(), "token1", None).unwrap();

        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let serialized_state = encode_state(&SerializedGuiState {
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            select_tokens: BTreeMap::from([("token1".to_string(), token)]),
            vault_ids: BTreeMap::new(),
            dotrain_hash: DotrainOrderGui::compute_state_hash(&dotrain_order).unwrap(),
            selected_deployment: "select-token-deployment".to_string(),
        });

        let err = DotrainOrderGui::new_from_state(dotrain, None, serialized_state, None)
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenNotInSelectTokens("token1".to_string()).to_string()
        );
    }

    #[wasm_bindgen_test]
    async fn test_new_from_state_replaces_existing_select_token_record() {
        let dotrain = get_yaml();
        let documents = DotrainOrderGui::get_yaml_documents(&dotrain, None).unwrap();
        TokenCfg::add_record_to_yaml(
            documents.clone(),
            "token3",
            "some-network",
            "0x0000000000000000000000000000000000000001",
            Some("18"),
            Some("Existing Token 3"),
            Some("OLD3"),
        )
        .unwrap();

        let yaml_frontmatter = DotrainYaml::get_yaml_string(documents[0].clone()).unwrap();
        let rain_document = RainDocument::create(dotrain.clone(), None, None, None);
        let dotrain_with_existing_token = format!(
            "{}\n{}\n{}",
            yaml_frontmatter,
            FRONTMATTER_SEPARATOR,
            rain_document.body()
        );

        let network = NetworkCfg::parse_from_yaml(documents.clone(), "some-network", None).unwrap();
        let replacement_token = TokenCfg {
            document: documents[0].clone(),
            key: "token3".to_string(),
            network: Arc::new(network),
            address: Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
            decimals: Some(6),
            label: Some("Replaced Token 3".to_string()),
            symbol: Some("NEW3".to_string()),
            logo_uri: None,
            extensions: None,
        };

        let dotrain_order = DotrainOrder::create(dotrain_with_existing_token.clone(), None)
            .await
            .unwrap();
        let serialized_state = encode_state(&SerializedGuiState {
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            select_tokens: BTreeMap::from([("token3".to_string(), replacement_token.clone())]),
            vault_ids: BTreeMap::new(),
            dotrain_hash: DotrainOrderGui::compute_state_hash(&dotrain_order).unwrap(),
            selected_deployment: "select-token-deployment".to_string(),
        });

        let gui = DotrainOrderGui::new_from_state(
            dotrain_with_existing_token,
            None,
            serialized_state,
            None,
        )
        .await
        .unwrap();

        let restored_token = gui
            .dotrain_order
            .raindex_yaml()
            .get_token("token3")
            .unwrap();
        assert_eq!(restored_token.address, replacement_token.address);
        assert_eq!(restored_token.symbol, replacement_token.symbol);
        assert_eq!(restored_token.label, replacement_token.label);
    }

    #[wasm_bindgen_test]
    async fn test_serialize_state_errors_on_missing_preset() {
        let gui = initialize_gui_with_select_tokens().await;
        let mut gui = gui;

        gui.field_values.insert(
            "binding-2".to_string(),
            field_values::PairValue {
                is_preset: true,
                value: "non-existent".to_string(),
            },
        );

        let err = gui.serialize_state().unwrap_err();
        assert_eq!(err.to_string(), GuiError::InvalidPreset.to_string());
    }

    #[wasm_bindgen_test]
    async fn test_execute_state_update_callback_noop_without_callback() {
        let gui = initialize_gui_with_select_tokens().await;
        assert!(gui.execute_state_update_callback().is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_execute_state_update_callback() {
        eval(
            r#"
            globalThis.callbackCalled = false;
            globalThis.receivedState = null;
            globalThis.testCallback = function(state) {
                globalThis.callbackCalled = true;
                globalThis.receivedState = state;
            };
        "#,
        )
        .unwrap();

        let global = js_sys::global();
        let callback_js = Reflect::get(&global, &JsValue::from_str("testCallback"))
            .expect("should have testCallback function on globalThis")
            .dyn_into::<js_sys::Function>()
            .expect("testCallback should be a function");

        let mut gui = DotrainOrderGui::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            Some(callback_js.clone()),
        )
        .await
        .unwrap();

        let callback_called = Reflect::get(&global, &JsValue::from_str("callbackCalled"))
            .expect("should have callbackCalled flag on globalThis");
        assert_eq!(callback_called, JsValue::from_bool(false));

        gui.set_deposit("token1".to_string(), "100".to_string())
            .await
            .unwrap();
        gui.set_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "582.1".to_string())
            .unwrap();
        gui.set_vault_id(
            VaultType::Input,
            "token1".to_string(),
            Some("199".to_string()),
        )
        .unwrap();
        gui.set_vault_id(
            VaultType::Output,
            "token2".to_string(),
            Some("299".to_string()),
        )
        .unwrap();

        let callback_called = Reflect::get(&global, &JsValue::from_str("callbackCalled"))
            .expect("should have callbackCalled flag on globalThis");
        assert_eq!(callback_called, JsValue::from_bool(true));

        let received_state_js = Reflect::get(&global, &JsValue::from_str("receivedState"))
            .expect("should have receivedState on globalThis");
        assert!(received_state_js.is_string());
        let received_state_rust: String = received_state_js.as_string().unwrap();
        assert!(!received_state_rust.is_empty());

        let expected_state = gui.serialize_state().unwrap();
        assert_eq!(received_state_rust, expected_state);
    }
}
