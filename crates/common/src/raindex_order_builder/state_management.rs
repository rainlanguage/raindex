use super::*;
use rain_metadata::types::dotrain::{
    order_builder_state_v1::{OrderBuilderStateV1, ShortenedTokenCfg, ValueCfg},
    source_v1::DotrainSourceV1,
};
use rain_orderbook_app_settings::{
    order::{OrderIOCfg, VaultType},
    order_builder::OrderBuilderDepositCfg,
    token::TokenCfg,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllBuilderConfig {
    pub field_definitions_without_defaults: Vec<OrderBuilderFieldDefinitionCfg>,
    pub field_definitions_with_defaults: Vec<OrderBuilderFieldDefinitionCfg>,
    pub deposits: Vec<OrderBuilderDepositCfg>,
    pub order_inputs: Vec<OrderIOCfg>,
    pub order_outputs: Vec<OrderIOCfg>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SerializedBuilderState {
    field_values: BTreeMap<String, OrderBuilderPresetCfg>,
    deposits: BTreeMap<String, OrderBuilderPresetCfg>,
    select_tokens: BTreeMap<String, TokenCfg>,
    vault_ids: BTreeMap<(VaultType, String), Option<String>>,
    dotrain_hash: String,
    selected_deployment: String,
}

impl RaindexOrderBuilder {
    fn create_preset(
        value: &field_values::PairValue,
        default_value: String,
    ) -> OrderBuilderPresetCfg {
        if value.is_preset {
            OrderBuilderPresetCfg {
                id: value.value.clone(),
                name: None,
                value: default_value,
            }
        } else {
            OrderBuilderPresetCfg {
                id: "".to_string(),
                name: None,
                value: value.value.clone(),
            }
        }
    }

    fn preset_to_pair_value(preset: OrderBuilderPresetCfg) -> field_values::PairValue {
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
    ) -> Result<BTreeMap<(VaultType, String), Option<String>>, RaindexOrderBuilderError> {
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

    pub fn generate_dotrain_builder_state_instance_v1(
        &self,
    ) -> Result<OrderBuilderStateV1, RaindexOrderBuilderError> {
        let trimmed_dotrain = self
            .dotrain_order
            .generate_dotrain_for_deployment(&self.selected_deployment)?;
        let dotrain_hash = DotrainSourceV1(trimmed_dotrain.clone()).hash();

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

        let select_tokens = {
            let mut result = BTreeMap::new();
            let deployment = self.get_current_deployment()?;
            let network_key = deployment.deployment.order.network.key.clone();

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

            if let Some(st) = OrderBuilderCfg::parse_select_tokens(
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

        let field_values = self
            .field_values
            .iter()
            .map(|(k, v)| {
                let normalized = self.get_field_value(k.clone())?;
                Ok((
                    k.clone(),
                    ValueCfg {
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
            .collect::<Result<_, RaindexOrderBuilderError>>()?;

        Ok(OrderBuilderStateV1 {
            dotrain_hash,
            field_values,
            deposits,
            select_tokens,
            vault_ids,
            selected_deployment: self.selected_deployment.clone(),
        })
    }

    pub fn serialize_state(&self) -> Result<String, RaindexOrderBuilderError> {
        let mut field_values = BTreeMap::new();
        for (k, v) in self.field_values.iter() {
            let preset = if v.is_preset {
                let presets = OrderBuilderCfg::parse_field_presets(
                    self.dotrain_order.dotrain_yaml().documents.clone(),
                    &self.selected_deployment,
                    k,
                )?
                .ok_or(RaindexOrderBuilderError::BindingHasNoPresets(k.clone()))?;
                presets
                    .iter()
                    .find(|preset| preset.id == v.value)
                    .ok_or(RaindexOrderBuilderError::InvalidPreset)?
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
        if let Some(st) = OrderBuilderCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents.clone(),
            &self.selected_deployment,
        )? {
            for select_token in st {
                if let Ok(token) = self
                    .dotrain_order
                    .orderbook_yaml()
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

        let state = SerializedBuilderState {
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

    pub async fn new_from_state(
        dotrain: String,
        settings: Option<Vec<String>>,
        serialized: String,
    ) -> Result<RaindexOrderBuilder, RaindexOrderBuilderError> {
        let compressed = URL_SAFE.decode(serialized)?;

        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)?;
        let state: SerializedBuilderState = bincode::deserialize(&bytes)?;

        let dotrain_order = DotrainOrder::create_with_profile(
            dotrain.clone(),
            settings,
            ContextProfile::builder(state.selected_deployment.clone()),
        )
        .await?;

        let original_dotrain_hash = RaindexOrderBuilder::compute_state_hash(&dotrain_order)?;
        if original_dotrain_hash != state.dotrain_hash {
            return Err(RaindexOrderBuilderError::DotrainMismatch);
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

        let builder = RaindexOrderBuilder {
            dotrain_order,
            field_values,
            deposits,
            selected_deployment: state.selected_deployment.clone(),
            dotrain_hash: original_dotrain_hash,
        };

        let deployment_select_tokens = OrderBuilderCfg::parse_select_tokens(
            builder.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for (key, token) in state.select_tokens {
            let select_tokens = deployment_select_tokens
                .as_ref()
                .ok_or(RaindexOrderBuilderError::SelectTokensNotSet)?;
            if !select_tokens.iter().any(|token| token.key == key) {
                return Err(RaindexOrderBuilderError::TokenNotInSelectTokens(key));
            }
            if builder.is_select_token_set(key.clone())? {
                TokenCfg::remove_record_from_yaml(
                    builder.dotrain_order.orderbook_yaml().documents,
                    &key,
                )?;
            }
            TokenCfg::add_record_to_yaml(
                builder.dotrain_order.orderbook_yaml().documents,
                &key,
                &token.network.key,
                &token.address.to_string(),
                token.decimals.map(|d| d.to_string()).as_deref(),
                token.label.map(|l| l.to_string()).as_deref(),
                token.symbol.map(|s| s.to_string()).as_deref(),
            )?;
        }

        let order_key = DeploymentCfg::parse_order_key(
            builder.dotrain_order.dotrain_yaml().documents,
            &state.selected_deployment,
        )?;
        for ((is_input, index), vault_id) in state.vault_ids {
            builder
                .dotrain_order
                .dotrain_yaml()
                .get_order_for_builder_deployment(&order_key, &state.selected_deployment)
                .and_then(|mut order| order.update_vault_id(is_input, index, vault_id))?;
        }

        Ok(builder)
    }

    pub fn get_all_builder_config(&self) -> Result<AllBuilderConfig, RaindexOrderBuilderError> {
        let deployment = self.get_current_deployment()?;

        let field_definitions_without_defaults = self.get_all_field_definitions(Some(false))?;
        let field_definitions_with_defaults = self.get_all_field_definitions(Some(true))?;
        let deposits = deployment.deposits;
        let order_inputs = deployment.deployment.order.inputs.clone();
        let order_outputs = deployment.deployment.order.outputs.clone();

        Ok(AllBuilderConfig {
            field_definitions_without_defaults,
            field_definitions_with_defaults,
            deposits,
            order_inputs,
            order_outputs,
        })
    }

    pub fn compute_state_hash(
        dotrain_order: &DotrainOrder,
    ) -> Result<String, RaindexOrderBuilderError> {
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
    use crate::dotrain::RainDocument;
    use crate::dotrain_order::DotrainOrder;
    use crate::raindex_order_builder::{
        field_values::FieldValue,
        tests::{get_yaml, initialize_builder_with_select_tokens},
    };
    use alloy::primitives::{Address, U256};
    use rain_orderbook_app_settings::{
        network::NetworkCfg, order::VaultType, yaml::YamlParsableHash,
    };
    use std::str::FromStr;

    const SERIALIZED_STATE: &str = "H4sIAAAAAAAA_2NigABOKJ2UmZeSmZeuawjlMzAwQ2lDAwN0RUaMUAEDBjgLxmCD0iX52al5xthMw64SlccD5RXn56bq5qWWlOcXZcP0yULpjJKSAit9_Zz85MScjPziEisLAwtT_aKCZN3SopxqkApGEMkIs9o1xEMEyhQyCauYgEYwCjGyQ6VDQG5QMGZkgfG9_YwZGJjgnkFzuyHcCkNLS5ArUWSN4LJGlpY6UGZKYKlbVV5aVVJeUl5mYUlyQaJZcGVgVmmuhZNBVmSxQYRFortnaLmPsWGyrTgsLFJzUpNLdMGG6qakFuTkV-am5pUAADgqxtfKAQAA";

    fn encode_state(state: &SerializedBuilderState) -> String {
        let bytes = bincode::serialize(state).unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes).unwrap();
        let compressed = encoder.finish().unwrap();
        URL_SAFE.encode(compressed)
    }

    async fn configured_builder() -> RaindexOrderBuilder {
        let mut builder = initialize_builder_with_select_tokens().await;

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "TKN3".to_string(),
        );
        builder
            .set_deposit("token3".to_string(), "100".to_string())
            .await
            .unwrap();
        builder
            .set_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        builder
            .set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();
        builder
            .set_vault_id(
                VaultType::Input,
                "token1".to_string(),
                Some("199".to_string()),
            )
            .unwrap();
        builder
            .set_vault_id(
                VaultType::Output,
                "token2".to_string(),
                Some("299".to_string()),
            )
            .unwrap();

        builder
    }

    #[tokio::test]
    async fn test_compute_state_hash_changes_on_content_change() {
        let dotrain = get_yaml();
        let order1 = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let original_hash = RaindexOrderBuilder::compute_state_hash(&order1).unwrap();

        let modified = dotrain.replace("Select token deployment", "Select token deployment v2");
        let order2 = DotrainOrder::create(modified, None).await.unwrap();
        let modified_hash = RaindexOrderBuilder::compute_state_hash(&order2).unwrap();
        assert_ne!(original_hash, modified_hash);

        let order3 = DotrainOrder::create(get_yaml(), None).await.unwrap();
        let repeated_hash = RaindexOrderBuilder::compute_state_hash(&order3).unwrap();
        assert_eq!(original_hash, repeated_hash);
    }

    #[tokio::test]
    async fn test_generate_dotrain_builder_state_instance_v1_contents() {
        let builder = configured_builder().await;
        let state = builder
            .generate_dotrain_builder_state_instance_v1()
            .unwrap();

        let trimmed = builder
            .dotrain_order
            .generate_dotrain_for_deployment(&builder.selected_deployment)
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

    #[tokio::test]
    async fn test_serialize_state() {
        let builder = configured_builder().await;
        let state = builder.serialize_state().unwrap();
        assert!(!state.is_empty());
        assert_eq!(state, SERIALIZED_STATE);
    }

    #[tokio::test]
    async fn test_new_from_state() {
        let builder =
            RaindexOrderBuilder::new_from_state(get_yaml(), None, SERIALIZED_STATE.to_string())
                .await
                .unwrap();

        assert!(builder.is_select_token_set("token3".to_string()).unwrap());
        assert_eq!(builder.get_deposits().unwrap()[0].amount, "100");
        assert_eq!(
            builder.get_field_value("binding-1".to_string()).unwrap(),
            FieldValue {
                field: "binding-1".to_string(),
                value: "100".to_string(),
                is_preset: false,
            }
        );
        assert_eq!(
            builder.get_field_value("binding-2".to_string()).unwrap(),
            FieldValue {
                field: "binding-2".to_string(),
                value: "0".to_string(),
                is_preset: true,
            }
        );
        let vault_ids = builder.get_vault_ids().unwrap().0;
        assert_eq!(
            vault_ids.get("input").unwrap()["token1"],
            Some(U256::from(199))
        );
        assert_eq!(
            vault_ids.get("output").unwrap()["token2"],
            Some(U256::from(299))
        );
    }

    #[tokio::test]
    async fn test_new_from_state_invalid_dotrain() {
        let dotrain = r#"
            version: 5
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
            orderbooks:
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
                    orderbook: test
            deployments:
                select-token-deployment:
                    order: test
                    scenario: test
            builder:
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

        let err = RaindexOrderBuilder::new_from_state(
            dotrain.to_string(),
            None,
            SERIALIZED_STATE.to_string(),
        )
        .await
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DotrainMismatch.to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "There was a mismatch in the dotrain configuration. Please check your YAML configuration for consistency."
        );
    }

    #[tokio::test]
    async fn test_new_from_state_rejects_unknown_select_token_key() {
        let dotrain = get_yaml();
        let documents = RaindexOrderBuilder::get_yaml_documents(&dotrain, None).unwrap();
        let token = TokenCfg::parse_from_yaml(documents.clone(), "token1", None).unwrap();

        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let serialized_state = encode_state(&SerializedBuilderState {
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            select_tokens: BTreeMap::from([("token1".to_string(), token)]),
            vault_ids: BTreeMap::new(),
            dotrain_hash: RaindexOrderBuilder::compute_state_hash(&dotrain_order).unwrap(),
            selected_deployment: "select-token-deployment".to_string(),
        });

        let err = RaindexOrderBuilder::new_from_state(dotrain, None, serialized_state)
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::TokenNotInSelectTokens("token1".to_string()).to_string()
        );
    }

    #[tokio::test]
    async fn test_new_from_state_replaces_existing_select_token_record() {
        let dotrain = get_yaml();
        let documents = RaindexOrderBuilder::get_yaml_documents(&dotrain, None).unwrap();
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
        let serialized_state = encode_state(&SerializedBuilderState {
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            select_tokens: BTreeMap::from([("token3".to_string(), replacement_token.clone())]),
            vault_ids: BTreeMap::new(),
            dotrain_hash: RaindexOrderBuilder::compute_state_hash(&dotrain_order).unwrap(),
            selected_deployment: "select-token-deployment".to_string(),
        });

        let builder = RaindexOrderBuilder::new_from_state(
            dotrain_with_existing_token,
            None,
            serialized_state,
        )
        .await
        .unwrap();

        let restored_token = builder
            .dotrain_order
            .orderbook_yaml()
            .get_token("token3")
            .unwrap();
        assert_eq!(restored_token.address, replacement_token.address);
        assert_eq!(restored_token.symbol, replacement_token.symbol);
        assert_eq!(restored_token.label, replacement_token.label);
    }

    #[tokio::test]
    async fn test_serialize_state_errors_on_missing_preset() {
        let builder = initialize_builder_with_select_tokens().await;
        let mut builder = builder;

        builder.field_values.insert(
            "binding-2".to_string(),
            field_values::PairValue {
                is_preset: true,
                value: "non-existent".to_string(),
            },
        );

        let err = builder.serialize_state().unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::InvalidPreset.to_string()
        );
    }
}
