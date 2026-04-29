pub use crate::erc20::ExtendedTokenInfo;
use crate::{
    dotrain::{types::patterns::FRONTMATTER_SEPARATOR, RainDocument},
    dotrain_order::{DotrainOrder, DotrainOrderError},
    erc20::ERC20,
    utils::amount_formatter::AmountFormatterError,
};
use alloy::primitives::Address;
use alloy_ethers_typecast::ReadableClientError;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rain_math_float::FloatError;
use rain_metaboard_subgraph::metaboard_client::MetaboardSubgraphClientError;
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg,
    order::OrderCfg,
    order_builder::{
        NameAndDescriptionCfg, OrderBuilderCfg, OrderBuilderDeploymentCfg,
        OrderBuilderFieldDefinitionCfg, OrderBuilderPresetCfg, ParseOrderBuilderConfigSourceError,
    },
    yaml::{
        context::ContextProfile,
        dotrain::{DotrainYaml, DotrainYamlValidation},
        emitter, YamlError, YamlParsable,
    },
};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;

pub mod deposits;
pub mod field_values;
pub mod order_operations;
pub mod select_tokens;
pub mod state_management;
pub mod validation;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RaindexOrderBuilder {
    dotrain_order: DotrainOrder,
    selected_deployment: String,
    field_values: BTreeMap<String, field_values::PairValue>,
    deposits: BTreeMap<String, field_values::PairValue>,
    dotrain_hash: String,
}
impl Default for RaindexOrderBuilder {
    fn default() -> Self {
        Self {
            dotrain_order: DotrainOrder::dummy(),
            selected_deployment: "".to_string(),
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            dotrain_hash: "".to_string(),
        }
    }
}

impl RaindexOrderBuilder {
    pub async fn get_deployment_keys(
        dotrain: String,
        settings: Option<Vec<String>>,
    ) -> Result<Vec<String>, RaindexOrderBuilderError> {
        let documents = RaindexOrderBuilder::get_yaml_documents(&dotrain, settings)?;
        Ok(OrderBuilderCfg::parse_deployment_keys(documents)?)
    }

    pub async fn new_with_deployment(
        dotrain: String,
        settings: Option<Vec<String>>,
        selected_deployment: String,
    ) -> Result<RaindexOrderBuilder, RaindexOrderBuilderError> {
        let documents = RaindexOrderBuilder::get_yaml_documents(&dotrain, settings.clone())?;

        let keys = OrderBuilderCfg::parse_deployment_keys(documents.clone())?;
        if !keys.contains(&selected_deployment) {
            return Err(RaindexOrderBuilderError::DeploymentNotFound(
                selected_deployment.clone(),
            ));
        }

        let dotrain_order = DotrainOrder::create_with_profile(
            dotrain.clone(),
            settings,
            ContextProfile::builder(selected_deployment.clone()),
        )
        .await?;

        let dotrain_hash = RaindexOrderBuilder::compute_state_hash(&dotrain_order)?;

        Ok(RaindexOrderBuilder {
            dotrain_order,
            selected_deployment,
            field_values: BTreeMap::new(),
            deposits: BTreeMap::new(),
            dotrain_hash,
        })
    }

    pub fn get_builder_config(&self) -> Result<OrderBuilderCfg, RaindexOrderBuilderError> {
        if !OrderBuilderCfg::check_builder_key_exists(
            self.dotrain_order.dotrain_yaml().documents.clone(),
        )? {
            return Err(RaindexOrderBuilderError::BuilderConfigNotFound);
        }
        let config = self
            .dotrain_order
            .dotrain_yaml()
            .get_order_builder(&self.selected_deployment)?
            .ok_or(RaindexOrderBuilderError::BuilderConfigNotFound)?;
        Ok(config)
    }

    pub fn get_current_deployment(
        &self,
    ) -> Result<OrderBuilderDeploymentCfg, RaindexOrderBuilderError> {
        let config = self.get_builder_config()?;
        let (_, deployment) = config
            .deployments
            .into_iter()
            .find(|(name, _)| name == &self.selected_deployment)
            .ok_or(RaindexOrderBuilderError::DeploymentNotFound(
                self.selected_deployment.clone(),
            ))?;
        Ok(deployment.clone())
    }

    pub async fn get_token_info(
        &self,
        key: String,
    ) -> Result<ExtendedTokenInfo, RaindexOrderBuilderError> {
        let token = self.dotrain_order.orderbook_yaml().get_token(&key)?;
        Ok(ExtendedTokenInfo::from_token_cfg(&token).await?)
    }

    pub async fn get_all_token_infos(
        &self,
    ) -> Result<Vec<ExtendedTokenInfo>, RaindexOrderBuilderError> {
        let select_tokens = self.get_select_tokens()?;

        let token_keys = match select_tokens.is_empty() {
            true => {
                let order_key = DeploymentCfg::parse_order_key(
                    self.dotrain_order.dotrain_yaml().documents,
                    &self.selected_deployment,
                )?;
                OrderCfg::parse_io_token_keys(
                    self.dotrain_order.dotrain_yaml().documents,
                    &order_key,
                )?
            }
            false => select_tokens
                .iter()
                .map(|token| token.key.clone())
                .collect(),
        };

        let mut result = Vec::new();
        for key in token_keys.iter() {
            result.push(self.get_token_info(key.clone()).await?);
        }
        Ok(result)
    }

    pub fn get_order_details(
        dotrain: String,
        settings: Option<Vec<String>>,
    ) -> Result<NameAndDescriptionCfg, RaindexOrderBuilderError> {
        let documents = RaindexOrderBuilder::get_yaml_documents(&dotrain, settings)?;
        Ok(OrderBuilderCfg::parse_order_details(documents)?)
    }

    pub fn get_deployment_details(
        dotrain: String,
        settings: Option<Vec<String>>,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, RaindexOrderBuilderError> {
        let documents = RaindexOrderBuilder::get_yaml_documents(&dotrain, settings)?;
        Ok(OrderBuilderCfg::parse_deployment_details(documents)?)
    }

    pub fn get_deployment_detail(
        dotrain: String,
        settings: Option<Vec<String>>,
        key: String,
    ) -> Result<NameAndDescriptionCfg, RaindexOrderBuilderError> {
        let deployment_details = RaindexOrderBuilder::get_deployment_details(dotrain, settings)?;
        let deployment_detail = deployment_details
            .get(&key)
            .ok_or(RaindexOrderBuilderError::DeploymentNotFound(key))?;
        Ok(deployment_detail.clone())
    }

    pub fn get_current_deployment_details(
        &self,
    ) -> Result<NameAndDescriptionCfg, RaindexOrderBuilderError> {
        let deployment_details = OrderBuilderCfg::parse_deployment_details(
            self.dotrain_order.dotrain_yaml().documents.clone(),
        )?;
        Ok(deployment_details
            .get(&self.selected_deployment)
            .ok_or(RaindexOrderBuilderError::DeploymentNotFound(
                self.selected_deployment.clone(),
            ))?
            .clone())
    }

    pub fn generate_dotrain_text(&self) -> Result<String, RaindexOrderBuilderError> {
        let rain_document = RainDocument::create(self.dotrain_order.dotrain()?, None, None, None);
        let dotrain = format!(
            "{}\n{}\n{}",
            emitter::emit_documents(&self.dotrain_order.dotrain_yaml().documents)?,
            FRONTMATTER_SEPARATOR,
            rain_document.body()
        );
        Ok(dotrain)
    }

    pub async fn get_composed_rainlang(&mut self) -> Result<String, RaindexOrderBuilderError> {
        self.update_scenario_bindings()?;
        let dotrain = self.generate_dotrain_text()?;
        let deployment = self.get_current_deployment()?;
        let dotrain_order = DotrainOrder::create_with_profile(
            dotrain.clone(),
            None,
            ContextProfile::builder(deployment.deployment.key.clone()),
        )
        .await?;
        let rainlang = dotrain_order
            .compose_deployment_to_rainlang(self.selected_deployment.clone())
            .await?;
        Ok(rainlang)
    }

    pub fn get_yaml_documents(
        dotrain: &str,
        settings: Option<Vec<String>>,
    ) -> Result<Vec<Arc<RwLock<StrictYaml>>>, RaindexOrderBuilderError> {
        let frontmatter = RainDocument::get_front_matter(dotrain)
            .unwrap_or("")
            .to_string();
        let mut sources = vec![frontmatter];
        if let Some(settings) = settings {
            sources.extend(settings);
        }

        let dotrain_yaml = DotrainYaml::new(sources, DotrainYamlValidation::default())?;
        Ok(dotrain_yaml.documents)
    }

    pub fn dotrain_order(&self) -> &DotrainOrder {
        &self.dotrain_order
    }

    pub fn selected_deployment(&self) -> &str {
        &self.selected_deployment
    }
}

#[derive(Error, Debug)]
pub enum RaindexOrderBuilderError {
    #[error("Builder config not found")]
    BuilderConfigNotFound,
    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),
    #[error("Field binding not found: {0}")]
    FieldBindingNotFound(String),
    #[error("Missing field value: {0}")]
    FieldValueNotSet(String),
    #[error("Deposit token not found in builder config: {0}")]
    DepositTokenNotFound(String),
    #[error("Missing deposit with token: {0}")]
    DepositNotSet(String),
    #[error("Missing deposit token for current deployment: {0}")]
    MissingDepositToken(String),
    #[error("Deposit amount cannot be an empty string")]
    DepositAmountCannotBeEmpty,
    #[error("Orderbook not found")]
    OrderbookNotFound,
    #[error("Order not found: {0}")]
    OrderNotFound(String),
    #[error("Deserialized dotrain mismatch")]
    DotrainMismatch,
    #[error("Vault id not found for output index: {0}")]
    VaultIdNotFound(String),
    #[error("Deployer not found")]
    DeployerNotFound,
    #[error("Token not found {0}")]
    TokenNotFound(String),
    #[error("Invalid preset")]
    InvalidPreset,
    #[error("Presets not set")]
    PresetsNotSet,
    #[error("Select tokens not set")]
    SelectTokensNotSet,
    #[error("Token must be selected: {0}")]
    TokenMustBeSelected(String),
    #[error("Binding has no presets: {0}")]
    BindingHasNoPresets(String),
    #[error("Token not in select tokens: {0}")]
    TokenNotInSelectTokens(String),
    #[error(transparent)]
    DotrainOrderError(#[from] DotrainOrderError),
    #[error(transparent)]
    ParseOrderBuilderConfigSourceError(#[from] ParseOrderBuilderConfigSourceError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
    #[error(transparent)]
    Base64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    FromHexError(#[from] alloy::hex::FromHexError),
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),
    #[error(transparent)]
    DepositError(#[from] crate::deposit::DepositError),
    #[error(transparent)]
    ParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error(transparent)]
    ReadContractParametersBuilderError(
        #[from] alloy_ethers_typecast::ReadContractParametersBuilderError,
    ),
    #[error(transparent)]
    UnitsError(#[from] alloy::primitives::utils::UnitsError),
    #[error(transparent)]
    WritableTransactionExecuteError(#[from] crate::transaction::WritableTransactionExecuteError),
    #[error(transparent)]
    AddOrderArgsError(#[from] crate::add_order::AddOrderArgsError),
    #[error(transparent)]
    ERC20Error(#[from] crate::erc20::Error),
    #[error(transparent)]
    SolTypesError(#[from] alloy::sol_types::Error),
    #[error(transparent)]
    YamlError(#[from] YamlError),
    #[error(transparent)]
    ValidationError(#[from] validation::BuilderValidationError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    AmountFormatterError(#[from] AmountFormatterError),
    #[error(transparent)]
    FloatError(#[from] FloatError),
    #[error(transparent)]
    RainMetadataError(#[from] rain_metadata::Error),
    #[error("No address found in metaboard subgraph")]
    NoAddressInMetaboardSubgraph,
    #[error(transparent)]
    MetaboardSubgraphClientError(#[from] MetaboardSubgraphClientError),
}

impl RaindexOrderBuilderError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::BuilderConfigNotFound =>
                "The builder configuration could not be found. Please check your YAML configuration file.".to_string(),
            Self::DeploymentNotFound(name) =>
                format!("The deployment '{}' could not be found. Please select a valid deployment from your YAML configuration.", name),
            Self::FieldBindingNotFound(field) =>
                format!("The field binding '{}' could not be found in the YAML configuration.", field),
            Self::FieldValueNotSet(field) =>
                format!("The value for field '{}' is required but has not been set.", field),
            Self::DepositTokenNotFound(token) =>
                format!("The deposit token '{}' was not found in the YAML configuration.", token),
            Self::DepositNotSet(token) =>
                format!("A deposit for token '{}' is required but has not been set.", token),
            Self::MissingDepositToken(deployment) =>
                format!("A deposit for token is required but has not been set for deployment '{}'.", deployment),
            Self::DepositAmountCannotBeEmpty =>
                "The deposit amount cannot be an empty string. Please set a valid amount.".to_string(),
            Self::OrderbookNotFound =>
                "The orderbook configuration could not be found. Please check your YAML configuration.".to_string(),
            Self::OrderNotFound(order) =>
                format!("The order '{}' could not be found in the YAML configuration.", order),
            Self::DotrainMismatch =>
                "There was a mismatch in the dotrain configuration. Please check your YAML configuration for consistency.".to_string(),
            Self::VaultIdNotFound(index) =>
                format!("The vault ID for output index '{}' could not be found in the YAML configuration.", index),
            Self::DeployerNotFound =>
                "The deployer configuration could not be found. Please check your YAML configuration.".to_string(),
            Self::TokenNotFound(token) =>
                format!("The token '{}' could not be found in the YAML configuration.", token),
            Self::InvalidPreset =>
                "The selected preset is invalid. Please choose a different preset from your YAML configuration.".to_string(),
            Self::PresetsNotSet =>
                "No presets have been configured. Please check your YAML configuration.".to_string(),
            Self::SelectTokensNotSet =>
                "No tokens have been configured for selection. Please check your YAML configuration.".to_string(),
            Self::TokenMustBeSelected(token) =>
                format!("The token '{}' must be selected to proceed.", token),
            Self::BindingHasNoPresets(binding) =>
                format!("The binding '{}' does not have any presets configured in the YAML configuration.", binding),
            Self::TokenNotInSelectTokens(token) =>
                format!("The token '{}' is not in the list of selectable tokens defined in the YAML configuration.", token),
            Self::DotrainOrderError(err) =>
                format!("Order configuration error in YAML: {}", err),
            Self::ParseOrderBuilderConfigSourceError(err) =>
                format!("Failed to parse YAML builder configuration: {}", err),
            Self::IoError(err) =>
                format!("I/O error: {}", err),
            Self::BincodeError(err) =>
                format!("Data serialization error: {}", err),
            Self::Base64Error(err) =>
                format!("Base64 encoding/decoding error: {}", err),
            Self::FromHexError(err) =>
                format!("Invalid hexadecimal value: {}", err),
            Self::ReadableClientError(err) =>
                format!("Network client error: {}", err),
            Self::DepositError(err) =>
                format!("Deposit error: {}", err),
            Self::ParseError(err) =>
                format!("Number parsing error: {}", err),
            Self::ReadContractParametersBuilderError(err) =>
                format!("Contract parameter error: {}", err),
            Self::UnitsError(err) =>
                format!("Unit conversion error: {}", err),
            Self::WritableTransactionExecuteError(err) =>
                format!("Transaction execution error: {}", err),
            Self::AddOrderArgsError(err) =>
                format!("Invalid order arguments: {}", err),
            Self::ERC20Error(err) =>
                format!("ERC20 token error: {}", err),
            Self::SolTypesError(err) =>
                format!("Solidity type error: {}", err),
            Self::YamlError(err) => format!("YAML configuration error: {}", err),
            Self::ValidationError(err) => format!("Validation error: {}", err),
            Self::UrlParseError(err) => format!("URL parsing error: {err}"),
            Self::AmountFormatterError(err) =>
                format!("There was a problem formatting the amount: {err}"),
            Self::FloatError(err) => {
                format!("There was a problem with the float value: {err}")
            }
            Self::RainMetadataError(err) =>
                format!("There was a problem with the rain metadata: {err}"),
            Self::NoAddressInMetaboardSubgraph =>
                "No address was found in the metaboard subgraph response.".to_string(),
            Self::MetaboardSubgraphClientError(err) =>
                format!("There was a problem with the metaboard subgraph client: {err}"),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_app_settings::yaml::FieldErrorKind;

    pub fn get_yaml() -> String {
        format!(
            r#"
version: {spec_version}
builder:
  name: Fixed limit
  description: Fixed limit order
  short-description: Buy WETH with USDC on Base.
  deployments:
    some-deployment:
      name: Buy WETH with USDC on Base.
      description: Buy WETH with USDC for fixed price on Base network.
      short-description: Buy WETH with USDC on Base.
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
            - "10"
            - "100"
            - "1000"
            - "10000"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0x1234567890abcdef1234567890abcdef12345678"
            - name: Preset 2
              value: "false"
            - name: Preset 3
              value: "some-string"
          default: some-default-value
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          presets:
            - value: "99.2"
            - value: "582.1"
            - value: "648.239"
          show-custom-field: true
    other-deployment:
      name: Test test
      description: Test test test
      deposits:
        - token: token1
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
    select-token-deployment:
      name: Select token deployment
      description: Select token deployment description
      deposits:
        - token: token3
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
      select-tokens:
        - key: token3
          name: Token 3
          description: Token 3 description
        - key: token4
          name: Token 4
          description: Token 4 description
networks:
    some-network:
        rpcs:
            - http://localhost:8085/rpc-url
        chain-id: 123
        network-id: 123
        currency: ETH
    other-network:
        rpcs:
            - http://localhost:8086/rpc-url
        chain-id: 124
        network-id: 124
        currency: ETH2
subgraphs:
    some-sg: https://www.some-sg.com
metaboards:
    some-network: https://metaboard.com
rainlangs:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
        local-db-remote: remote
        deployment-block: 12345
tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: Token 1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: Token 2
        symbol: T2
scenarios:
    some-scenario:
        rainlang: some-deployer
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300
orders:
    some-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token2
          vault-id: 1
      rainlang: some-deployer
      orderbook: some-orderbook
    other-order:
      inputs:
        - token: token1
      outputs:
        - token: token1
      rainlang: some-deployer
      orderbook: some-orderbook
deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    other-deployment:
        scenario: some-scenario.sub-scenario
        order: other-order
    select-token-deployment:
        scenario: some-scenario
        order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        )
    }

    pub fn get_yaml_with_validation() -> String {
        format!(
            r#"
version: {spec_version}
builder:
  name: Validation Test
  description: Test deployment with various validation rules
  deployments:
    validation-deployment:
      name: Validation Test Deployment
      description: Testing all validation scenarios
      fields:
        - binding: price-field
          name: Price Field
          description: Field with number validation
          validation:
            type: number
            decimals: 18
            minimum: 10
            maximum: 1000
        - binding: quantity-field
          name: Quantity Field
          description: Field with exclusive bounds
          validation:
            type: number
            decimals: 18
            exclusive-minimum: 0
            exclusive-maximum: 100000
        - binding: percentage-field
          name: Percentage Field
          description: Field with all number constraints
          validation:
            type: number
            decimals: 18
            minimum: 0
            maximum: 100
            exclusive-maximum: 101
        - binding: simple-number
          name: Simple Number
          description: Number field with no constraints
          validation:
            type: number
            decimals: 18
        - binding: username-field
          name: Username
          description: Field with string length validation
          validation:
            type: string
            min-length: 3
            max-length: 20
        - binding: description-field
          name: Description
          description: Field with only max length
          validation:
            type: string
            max-length: 500
        - binding: code-field
          name: Code
          description: Field with only min length
          validation:
            type: string
            min-length: 5
        - binding: any-string
          name: Any String
          description: String field with no constraints
          validation:
            type: string
        - binding: enabled-field
          name: Enabled
          description: Boolean field
          validation:
            type: boolean
        - binding: preset-number-field
          name: Preset Number
          description: Number field with presets and validation
          presets:
            - name: Low
              value: 50
            - name: Medium
              value: 100
            - name: High
              value: 150
          validation:
            type: number
            decimals: 18
            minimum: 10
            maximum: 200
        - binding: preset-string-field
          name: Preset String
          description: String field with presets and validation
          presets:
            - name: Option A
              value: alpha
            - name: Option B
              value: beta
            - name: Option C
              value: gamma
          validation:
            type: string
            min-length: 4
            max-length: 10
        - binding: no-validation-field
          name: No Validation
          description: Field without any validation
      deposits:
        - token: token1
          validation:
            minimum: 100
        - token: token2
          validation:
            maximum: 10000
        - token: token3
          validation:
            exclusive-minimum: 0
            exclusive-maximum: 50000
        - token: token4
          validation:
            minimum: 10
            maximum: 1000
        - token: token6
networks:
    test-network:
        rpcs:
            - http://localhost:8085
        chain-id: 1
        network-id: 1
        currency: ETH
subgraphs:
    test-sg: https://test.subgraph.com
rainlangs:
    test-deployer:
        network: test-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
    test-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: test-network
        subgraph: test-sg
        local-db-remote: remote
        deployment-block: 12345
tokens:
    token1:
        network: test-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: Token 1
        symbol: T1
    token2:
        network: test-network
        address: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
        decimals: 6
        label: Token 2
        symbol: T2
    token3:
        network: test-network
        address: 0xdAC17F958D2ee523a2206206994597C13D831ec7
        decimals: 6
        label: Token 3
        symbol: T3
    token4:
        network: test-network
        address: 0x6B175474E89094C44Da98b954EedeAC495271d0F
        decimals: 18
        label: Token 4
        symbol: T4
    token5:
        network: test-network
        address: 0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984
        decimals: 18
        label: Token 5
        symbol: T5
    token6:
        network: test-network
        address: 0x2b591e99afE9f32eAA6214f7B7629768c40Eeb39
        decimals: 8
        label: Token 6
        symbol: T6
scenarios:
    test-scenario:
        rainlang: test-deployer
        bindings:
            test: 1
orders:
    test-order:
      inputs:
        - token: token1
          vault-id: 1
      outputs:
        - token: token1
          vault-id: 1
      rainlang: test-deployer
      orderbook: test-orderbook
deployments:
    validation-deployment:
        scenario: test-scenario
        order: test-order
---
#test !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        )
    }

    pub async fn initialize_builder(deployment_name: Option<String>) -> RaindexOrderBuilder {
        RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            deployment_name.unwrap_or("some-deployment".to_string()),
        )
        .await
        .unwrap()
    }

    pub async fn initialize_builder_with_select_tokens() -> RaindexOrderBuilder {
        RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "select-token-deployment".to_string(),
        )
        .await
        .unwrap()
    }

    pub async fn initialize_validation_builder() -> RaindexOrderBuilder {
        RaindexOrderBuilder::new_with_deployment(
            get_yaml_with_validation(),
            None,
            "validation-deployment".to_string(),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_get_deployment_keys() {
        let deployment_keys = RaindexOrderBuilder::get_deployment_keys(get_yaml(), None)
            .await
            .unwrap();
        assert_eq!(
            deployment_keys,
            vec![
                "other-deployment",
                "select-token-deployment",
                "some-deployment"
            ]
        );
    }

    #[tokio::test]
    async fn test_new_with_deployment() {
        let res = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
        )
        .await;
        assert!(res.is_ok());

        let err = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "invalid-deployment".to_string(),
        )
        .await
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DeploymentNotFound("invalid-deployment".to_string())
                .to_string()
        );
    }

    #[tokio::test]
    async fn test_get_builder_config() {
        let builder = initialize_builder(None).await;
        let builder_config = builder.get_builder_config().unwrap();
        assert_eq!(builder_config.name, "Fixed limit".to_string());
        assert_eq!(builder_config.description, "Fixed limit order".to_string());
        assert_eq!(builder_config.deployments.len(), 1);
    }

    #[tokio::test]
    async fn test_get_current_deployment() {
        let builder = initialize_builder(None).await;
        let deployment = builder.get_current_deployment().unwrap();
        assert_eq!(deployment.name, "Buy WETH with USDC on Base.".to_string());
    }

    #[tokio::test]
    async fn test_get_token_info_local() {
        let builder = initialize_builder(None).await;
        let token1_info = builder.get_token_info("token1".to_string()).await.unwrap();
        assert_eq!(
            token1_info.address.to_string(),
            "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"
        );
        assert_eq!(token1_info.decimals, 6);
        assert_eq!(token1_info.name, "Token 1");
        assert_eq!(token1_info.symbol, "T1");
    }

    #[tokio::test]
    async fn test_get_order_details() {
        let order_details = RaindexOrderBuilder::get_order_details(get_yaml(), None).unwrap();
        assert_eq!(order_details.name, "Fixed limit");
        assert_eq!(order_details.description, "Fixed limit order");
        assert_eq!(
            order_details.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_deployment_details() {
        let deployment_details =
            RaindexOrderBuilder::get_deployment_details(get_yaml(), None).unwrap();
        assert_eq!(deployment_details.len(), 3);
        let detail = deployment_details.get("some-deployment").unwrap();
        assert_eq!(detail.name, "Buy WETH with USDC on Base.");
    }

    #[tokio::test]
    async fn test_generate_dotrain_text() {
        let builder = initialize_builder(None).await;
        let original = builder.get_current_deployment_details().unwrap();
        let dotrain_text = builder.generate_dotrain_text().unwrap();
        let builder2 = RaindexOrderBuilder::new_with_deployment(
            dotrain_text,
            None,
            "some-deployment".to_string(),
        )
        .await
        .unwrap();
        let restored = builder2.get_current_deployment_details().unwrap();
        assert_eq!(restored, original);
    }

    #[tokio::test]
    async fn test_get_composed_rainlang() {
        let mut builder = initialize_builder(None).await;
        let rainlang = builder.get_composed_rainlang().await.unwrap();
        let expected = "/* 0. calculate-io */ \n_ _: 0 0;\n\n/* 1. handle-io */ \n:;".to_string();
        assert_eq!(rainlang, expected);
    }

    #[cfg(not(target_family = "wasm"))]
    mod select_token_tests {
        use super::*;
        use httpmock::MockServer;
        use serde_json::json;

        pub const SELECT_TOKEN_YAML: &str = r#"
builder:
    name: Fixed limit
    description: Fixed limit order order
    short-description: Buy WETH with USDC on Base.
    deployments:
        some-deployment:
            name: Select token deployment
            description: Select token deployment description
            deposits:
            - token: token3
              min: 0
              presets:
                - "0"
            fields:
            - binding: binding-1
              name: Field 1 name
              description: Field 1 description
              presets:
                - name: Preset 1
                  value: "0"
            - binding: binding-2
              name: Field 2 name
              description: Field 2 description
              min: 100
              presets:
                - value: "0"
            select-tokens:
            - key: token3
              name: Token 3
              description: Token 3 description
subgraphs:
    some-sg: https://www.some-sg.com
metaboards:
    some-network: https://metaboard.com
rainlangs:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
        deployment-block: 12345
scenarios:
    some-scenario:
        rainlang: some-deployer
        bindings:
            test-binding: 5
        scenarios:
            sub-scenario:
                bindings:
                    another-binding: 300
orders:
    some-order:
        rainlang: some-deployer
        inputs:
            - token: token3
        outputs:
            - token: token3
deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
    normal-deployment:
        scenario: some-scenario
        order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#;

        #[tokio::test]
        async fn test_get_token_info_remote() {
            let server = MockServer::start_async().await;
            let yaml = format!(
                r#"
version: {spec_version}
networks:
    some-network:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
{yaml}
"#,
                spec_version = SpecVersion::current(),
                yaml = SELECT_TOKEN_YAML,
                rpc_url = server.url("/rpc")
            );

            server.mock(|when, then| {
                when.method("POST").path("/rpc").body_contains("252dba42");
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e2031000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000",
                }));
            });

            let mut builder = RaindexOrderBuilder::new_with_deployment(
                yaml.to_string(),
                None,
                "some-deployment".to_string(),
            )
            .await
            .unwrap();

            let err = builder
                .get_token_info("token3".to_string())
                .await
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                YamlError::Field {
                    kind: FieldErrorKind::Missing("tokens".to_string()),
                    location: "root".to_string(),
                }
                .to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "YAML configuration error: Missing required field 'tokens' in root"
            );

            builder
                .set_select_token(
                    "token3".to_string(),
                    "0x0000000000000000000000000000000000000001".to_string(),
                )
                .await
                .unwrap();

            let token_info = builder.get_token_info("token3".to_string()).await.unwrap();
            assert_eq!(
                token_info.address.to_string(),
                "0x0000000000000000000000000000000000000001"
            );
            assert_eq!(token_info.decimals, 6);
            assert_eq!(token_info.name, "Token 1");
            assert_eq!(token_info.symbol, "T1");

            let token_infos = builder.get_all_token_infos().await.unwrap();
            assert_eq!(token_infos.len(), 1);
            assert_eq!(
                token_infos[0].address.to_string(),
                "0x0000000000000000000000000000000000000001"
            );
            assert_eq!(token_infos[0].decimals, 6);
            assert_eq!(token_infos[0].name, "Token 1");
            assert_eq!(token_infos[0].symbol, "T1");
        }
    }
}
