use rain_orderbook_app_settings::order_builder::{
    NameAndDescriptionCfg, OrderBuilderCfg, OrderBuilderDeploymentCfg,
    OrderBuilderFieldDefinitionCfg,
};
pub use rain_orderbook_common::erc20::ExtendedTokenInfo;
use rain_orderbook_common::raindex_order_builder::{
    RaindexOrderBuilder as RaindexOrderBuilderInner, RaindexOrderBuilderError,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

mod deposits;
mod field_values;
mod order_operations;
mod select_tokens;
mod state_management;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[wasm_bindgen]
pub struct RaindexOrderBuilder {
    pub(crate) inner: RaindexOrderBuilderInner,
    #[serde(skip)]
    state_update_callback: Option<js_sys::Function>,
}

#[wasm_export]
impl RaindexOrderBuilder {
    /// Lists all available builder deployment keys from a dotrain YAML file.
    ///
    /// This function parses the builder section of the YAML frontmatter to extract deployment keys that can be used
    /// to initialize a builder instance. Use this to build deployment selectors in your UI.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const dotrain = `
    /// builder:
    ///   deployments:
    ///     mainnet-order:
    ///       name: "Mainnet Trading"
    ///     testnet-order:
    ///       name: "Test order"
    /// `;
    ///
    /// const result = await RaindexOrderBuilder.getDeploymentKeys(dotrain);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const deploymentKeys = result.value;
    /// // Do something with the deploymentKeys
    /// ```
    #[wasm_export(
        js_name = "getDeploymentKeys",
        unchecked_return_type = "string[]",
        return_description = "Array of deployment identifiers (keys from the deployments map)"
    )]
    pub async fn get_deployment_keys(
        #[wasm_export(
            param_description = "Complete dotrain YAML content including the `builder.deployments` section"
        )]
        dotrain: String,
        #[wasm_export(
            param_description = "Optional additional YAML configuration strings to merge with the frontmatter"
        )]
        settings: Option<Vec<String>>,
    ) -> Result<Vec<String>, RaindexOrderBuilderWasmError> {
        Ok(RaindexOrderBuilderInner::get_deployment_keys(dotrain, settings).await?)
    }

    /// Creates a new builder instance for managing a specific deployment configuration.
    ///
    /// This is the primary initialization function that sets up the builder context for a chosen
    /// deployment. The instance tracks field values, deposits, token selections, and provides
    /// methods for generating order transactions.
    ///
    /// ## State Management
    ///
    /// The callback function receives a serialized state string on every change, enabling
    /// auto-save functionality or state synchronization across components.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Basic initialization
    /// const result = await RaindexOrderBuilder.newWithDeployment(dotrainYaml, "mainnet-order");
    /// if (result.error) {
    ///   console.error("Init failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const builder = result.value;
    ///
    /// // With state persistence
    /// const result = await RaindexOrderBuilder.newWithDeployment(
    ///   dotrainYaml,
    ///   "mainnet-order",
    ///   (serializedState) => {
    ///     localStorage.setItem('orderState', serializedState);
    ///   }
    /// );
    /// if (!result.error) {
    ///   const builder = result.value;
    ///   // Use builder instance...
    /// }
    /// ```
    #[wasm_export(
        js_name = "newWithDeployment",
        preserve_js_class,
        return_description = "Initialized builder instance for further operations"
    )]
    pub async fn new_with_deployment(
        #[wasm_export(param_description = "Complete dotrain YAML content with all configurations")]
        dotrain: String,
        #[wasm_export(
            param_description = "Optional additional YAML configuration strings to merge with the frontmatter"
        )]
        settings: Option<Vec<String>>,
        #[wasm_export(
            param_description = "Key of the deployment to activate (must exist in YAML)"
        )]
        selected_deployment: String,
        #[wasm_export(param_description = "Optional function called on state changes. \
            After a state change (deposit, field value, vault id, select token, etc.), the callback is called with the new state. \
            This is useful for auto-saving the state of the builder across sessions.")]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<RaindexOrderBuilder, RaindexOrderBuilderWasmError> {
        let inner =
            RaindexOrderBuilderInner::new_with_deployment(dotrain, settings, selected_deployment)
                .await?;
        Ok(RaindexOrderBuilder {
            inner,
            state_update_callback,
        })
    }

    /// Retrieves the complete builder configuration including all deployments.
    ///
    /// This returns the parsed builder section from the YAML, filtered to include only
    /// the current deployment. Use this to access order-level metadata.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = builder.getBuilderConfig();
    /// if (result.error) {
    ///   console.error("Config error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const config = result.value;
    /// // Do something with the config
    /// ```
    #[wasm_export(
        js_name = "getBuilderConfig",
        unchecked_return_type = "OrderBuilderCfg",
        return_description = "Complete builder configuration with name, description, and deployments"
    )]
    pub fn get_builder_config(&self) -> Result<OrderBuilderCfg, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_builder_config()?)
    }

    /// Gets the active deployment's configuration including fields, deposits, and tokens.
    ///
    /// This is the primary method for accessing deployment-specific settings that define
    /// what inputs are needed from the user. The configuration drives UI generation.
    ///
    /// ## Configuration Structure
    ///
    /// - `fields` - Input fields requiring user configuration
    /// - `deposits` - Token deposits with amounts and presets
    /// - `selectTokens` - Tokens that users must choose addresses for
    /// - `deployment` - Underlying order and scenario configuration
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = builder.getCurrentDeployment();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const deployment = result.value;
    /// // Do something with the deployment
    /// ```
    #[wasm_export(
        js_name = "getCurrentDeployment",
        unchecked_return_type = "OrderBuilderDeploymentCfg",
        return_description = "Active deployment with all configuration details"
    )]
    pub fn get_current_deployment(
        &self,
    ) -> Result<OrderBuilderDeploymentCfg, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_current_deployment()?)
    }

    /// Retrieves detailed token information from YAML configuration or blockchain.
    ///
    /// This function first checks the YAML for cached token data (decimals, name, symbol).
    /// If any information is missing, it queries the blockchain to fetch the complete details.
    /// This hybrid approach minimizes RPC calls while ensuring accurate data.
    ///
    /// ## Network Selection
    ///
    /// The RPC endpoint is determined by the deployment's order network configuration.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Get token info (may query blockchain)
    /// const result = await builder.getTokenInfo("weth");
    /// if (result.error) {
    ///   console.error("Token error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const tokenInfo = result.value;
    /// // Do something with the tokenInfo
    /// ```
    #[wasm_export(
        js_name = "getTokenInfo",
        unchecked_return_type = "ExtendedTokenInfo",
        return_description = "Complete token details including address, decimals, name, symbol, and chain_id"
    )]
    pub async fn get_token_info(
        &self,
        #[wasm_export(param_description = "Token identifier from the YAML tokens section")]
        key: String,
    ) -> Result<ExtendedTokenInfo, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_token_info(key).await?)
    }

    /// Gets information for all tokens used in the current deployment's order.
    ///
    /// This function automatically determines which tokens to fetch based on the deployment:
    /// - If `select-tokens` is defined, returns info for those tokens
    /// - Otherwise, returns info for all input/output tokens in the order
    ///
    /// ## Performance Note
    ///
    /// This may trigger multiple blockchain queries if token data isn't cached in YAML.
    /// Consider caching the results in your application.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await builder.getAllTokenInfos();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const tokens = result.value;
    /// // Do something with the tokens
    /// ```
    #[wasm_export(
        js_name = "getAllTokenInfos",
        unchecked_return_type = "ExtendedTokenInfo[]",
        return_description = "Array of complete token information"
    )]
    pub async fn get_all_token_infos(
        &self,
    ) -> Result<Vec<ExtendedTokenInfo>, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_all_token_infos().await?)
    }

    /// Extracts order-level metadata from a dotrain configuration.
    ///
    /// This static method allows checking order details without creating a builder instance,
    /// useful for displaying order information before deployment selection.
    ///
    /// ## Required Fields
    ///
    /// The YAML must contain:
    /// - `builder.name` - Order display name
    /// - `builder.description` - Full order description
    /// - `builder.short-description` - Brief summary (optional but recommended)
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await getOrderDetails(dotrainYaml);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const details = result.value;
    /// // Do something with the details
    /// ```
    #[wasm_export(
        js_name = "getOrderDetails",
        unchecked_return_type = "NameAndDescriptionCfg",
        return_description = "Order name, description, and optional short description"
    )]
    pub fn get_order_details(
        #[wasm_export(param_description = "Complete dotrain YAML content")] dotrain: String,
        #[wasm_export(
            param_description = "Optional additional YAML configuration strings to merge with the frontmatter"
        )]
        settings: Option<Vec<String>>,
    ) -> Result<NameAndDescriptionCfg, RaindexOrderBuilderWasmError> {
        Ok(RaindexOrderBuilderInner::get_order_details(
            dotrain, settings,
        )?)
    }

    /// Gets metadata for all deployments defined in the configuration.
    ///
    /// This static method extracts name and description for each deployment, useful for
    /// building deployment selection interfaces with rich descriptions.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await getDeploymentDetails(dotrainYaml);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // key is the deployment key
    /// // value is the deployment metadata
    /// for (const [key, value] of result.value) {
    ///   const {
    ///     // name is the deployment name
    ///     // description is the deployment description
    ///     name,
    ///     description,
    ///     // short_description is the deployment short description (optional)
    ///     short_description,
    ///   } = value;
    /// }
    /// ```
    #[wasm_export(
        js_name = "getDeploymentDetails",
        unchecked_return_type = "Map<string, NameAndDescriptionCfg>",
        return_description = "Map of deployment key to metadata"
    )]
    pub fn get_deployment_details(
        #[wasm_export(param_description = "Complete dotrain YAML content")] dotrain: String,
        #[wasm_export(
            param_description = "Optional additional YAML configuration strings to merge with the frontmatter"
        )]
        settings: Option<Vec<String>>,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, RaindexOrderBuilderWasmError> {
        Ok(RaindexOrderBuilderInner::get_deployment_details(
            dotrain, settings,
        )?)
    }

    /// Gets metadata for a specific deployment by key.
    ///
    /// Convenience method that extracts details for a single deployment without
    /// parsing all deployments.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await getDeploymentDetail(dotrainYaml, "mainnet-order");
    /// if (result.error) {
    ///   console.error("Not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const detail = result.value;
    /// // Do something with the detail
    /// ```
    #[wasm_export(
        js_name = "getDeploymentDetail",
        unchecked_return_type = "NameAndDescriptionCfg",
        return_description = "Deployment name and description"
    )]
    pub fn get_deployment_detail(
        #[wasm_export(param_description = "Complete dotrain YAML content")] dotrain: String,
        #[wasm_export(
            param_description = "Optional additional YAML configuration strings to merge with the frontmatter"
        )]
        settings: Option<Vec<String>>,
        #[wasm_export(param_description = "Deployment identifier to look up")] key: String,
    ) -> Result<NameAndDescriptionCfg, RaindexOrderBuilderWasmError> {
        Ok(RaindexOrderBuilderInner::get_deployment_detail(
            dotrain, settings, key,
        )?)
    }

    /// Gets metadata for the currently active deployment.
    ///
    /// Instance method that returns name and description for the deployment
    /// selected during initialization.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = builder.getCurrentDeploymentDetails();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const details = result.value;
    /// // Do something with the details
    /// ```
    #[wasm_export(
        js_name = "getCurrentDeploymentDetails",
        unchecked_return_type = "NameAndDescriptionCfg",
        return_description = "Current deployment's metadata"
    )]
    pub fn get_current_deployment_details(
        &self,
    ) -> Result<NameAndDescriptionCfg, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_current_deployment_details()?)
    }

    /// Exports the current configuration as a complete dotrain text file.
    ///
    /// This generates a valid dotrain file with YAML frontmatter and Rainlang code,
    /// preserving all configurations and bindings. Useful for saving or sharing orders.
    ///
    /// ## Format
    ///
    /// The output follows the standard dotrain format:
    /// ```text
    /// builder:
    ///   ...
    /// ---
    /// #binding1 !The binding value
    /// #calculate-io
    /// ...
    /// ```
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = builder.generateDotrainText();
    /// if (result.error) {
    ///   console.error("Export failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const dotrain = result.value;
    /// // Do something with the dotrain
    /// ```
    #[wasm_export(
        js_name = "generateDotrainText",
        unchecked_return_type = "string",
        return_description = "Complete dotrain content with YAML frontmatter separator"
    )]
    pub fn generate_dotrain_text(&self) -> Result<String, RaindexOrderBuilderWasmError> {
        Ok(self.inner.generate_dotrain_text()?)
    }

    /// Composes the final Rainlang code with all bindings and scenarios applied.
    ///
    /// This method updates scenario bindings from current field values and composes
    /// the Rainlang code ready to be displayed on the UI.
    ///
    /// ## Side Effects
    ///
    /// Updates the internal scenario bindings before composition.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await builder.getComposedRainlang();
    /// if (result.error) {
    ///   console.error("Composition error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const rainlang = result.value;
    /// // Do something with the rainlang
    /// ```
    #[wasm_export(
        js_name = "getComposedRainlang",
        unchecked_return_type = "string",
        return_description = "Composed Rainlang code with comments for each entrypoint"
    )]
    pub async fn get_composed_rainlang(&mut self) -> Result<String, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_composed_rainlang().await?)
    }
}

#[derive(Error, Debug)]
pub enum RaindexOrderBuilderWasmError {
    #[error(transparent)]
    BuilderError(#[from] RaindexOrderBuilderError),
    #[error("JavaScript error: {0}")]
    JsError(String),
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
}

impl RaindexOrderBuilderWasmError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::BuilderError(err) => err.to_readable_msg(),
            Self::JsError(msg) => format!("A JavaScript error occurred: {}", msg),
            Self::SerdeWasmBindgenError(err) => format!("Data serialization error: {}", err),
        }
    }
}

impl From<RaindexOrderBuilderWasmError> for JsValue {
    fn from(value: RaindexOrderBuilderWasmError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<RaindexOrderBuilderWasmError> for WasmEncodedError {
    fn from(value: RaindexOrderBuilderWasmError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::order_builder::OrderBuilderPresetCfg;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_app_settings::yaml::{FieldErrorKind, YamlError};
    use wasm_bindgen_test::wasm_bindgen_test;

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
        # Number validation tests
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

        # String validation tests
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

        # Boolean validation test
        - binding: enabled-field
          name: Enabled
          description: Boolean field
          validation:
            type: boolean

        # Fields with presets and validation
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

        # Field without validation
        - binding: no-validation-field
          name: No Validation
          description: Field without any validation

      deposits:
        # Deposit with minimum amount validation
        - token: token1
          validation:
            minimum: 100

        # Deposit with maximum amount validation
        - token: token2
          validation:
            maximum: 10000

        # Deposit with exclusive bounds
        - token: token3
          validation:
            exclusive-minimum: 0
            exclusive-maximum: 50000

        # Deposit with all constraints
        - token: token4
          validation:
            minimum: 10
            maximum: 1000

        # Deposit without validation
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
            None,
        )
        .await
        .unwrap()
    }

    pub async fn initialize_builder_with_select_tokens() -> RaindexOrderBuilder {
        RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "select-token-deployment".to_string(),
            None,
        )
        .await
        .unwrap()
    }

    pub async fn initialize_validation_builder() -> RaindexOrderBuilder {
        RaindexOrderBuilder::new_with_deployment(
            get_yaml_with_validation(),
            None,
            "validation-deployment".to_string(),
            None,
        )
        .await
        .unwrap()
    }

    #[wasm_bindgen_test]
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

    #[wasm_bindgen_test]
    async fn test_new_with_deployment() {
        let res = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await;
        assert!(res.is_ok());

        let err = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "invalid-deployment".to_string(),
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::DeploymentNotFound("invalid-deployment".to_string())
                .to_string()
        );
        assert_eq!(err.to_readable_msg(), "The deployment 'invalid-deployment' could not be found. Please select a valid deployment from your YAML configuration.");
    }

    #[wasm_bindgen_test]
    async fn test_get_builder_config() {
        let builder = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();

        let builder_config = builder.get_builder_config().unwrap();
        assert_eq!(builder_config.name, "Fixed limit".to_string());
        assert_eq!(builder_config.description, "Fixed limit order".to_string());
        assert_eq!(builder_config.deployments.len(), 1);
        let deployment = builder_config.deployments.get("some-deployment").unwrap();
        assert_eq!(deployment.name, "Buy WETH with USDC on Base.".to_string());
        assert_eq!(
            deployment.description,
            "Buy WETH with USDC for fixed price on Base network.".to_string()
        );
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = deployment.deposits[0].clone();
        assert!(deposit.token.is_some());
        assert_eq!(deposit.token.unwrap().key, "token1");
        assert_eq!(
            deposit.presets,
            Some(vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ])
        );
        assert_eq!(deployment.fields.len(), 2);
        let field = deployment.fields[0].clone();
        assert_eq!(field.name, "Field 1 name");
        assert_eq!(field.description, Some("Field 1 description".to_string()));
        assert_eq!(
            field.presets,
            Some(vec![
                OrderBuilderPresetCfg {
                    id: "0".to_string(),
                    name: Some("Preset 1".to_string()),
                    value: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "1".to_string(),
                    name: Some("Preset 2".to_string()),
                    value: "false".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "2".to_string(),
                    name: Some("Preset 3".to_string()),
                    value: "some-string".to_string(),
                },
            ])
        );
        let field = deployment.fields[1].clone();
        assert_eq!(field.name, "Field 2 name");
        assert_eq!(field.description, Some("Field 2 description".to_string()));
        assert_eq!(field.show_custom_field, Some(true));
        assert_eq!(
            field.presets,
            Some(vec![
                OrderBuilderPresetCfg {
                    id: "0".to_string(),
                    name: None,
                    value: "99.2".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "1".to_string(),
                    name: None,
                    value: "582.1".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "2".to_string(),
                    name: None,
                    value: "648.239".to_string(),
                },
            ])
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_current_deployment() {
        let builder = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();

        let deployment = builder.get_current_deployment().unwrap();
        assert_eq!(deployment.name, "Buy WETH with USDC on Base.".to_string());
        assert_eq!(
            deployment.description,
            "Buy WETH with USDC for fixed price on Base network.".to_string()
        );
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = deployment.deposits[0].clone();
        assert!(deposit.token.is_some());
        assert_eq!(deposit.token.unwrap().key, "token1");
        assert_eq!(
            deposit.presets,
            Some(vec![
                "0".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
                "10000".to_string()
            ])
        );
        assert_eq!(deployment.fields.len(), 2);
        let field = deployment.fields[0].clone();
        assert_eq!(field.name, "Field 1 name");
        assert_eq!(field.description, Some("Field 1 description".to_string()));
        assert_eq!(
            field.presets,
            Some(vec![
                OrderBuilderPresetCfg {
                    id: "0".to_string(),
                    name: Some("Preset 1".to_string()),
                    value: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "1".to_string(),
                    name: Some("Preset 2".to_string()),
                    value: "false".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "2".to_string(),
                    name: Some("Preset 3".to_string()),
                    value: "some-string".to_string(),
                },
            ])
        );
        let field = deployment.fields[1].clone();
        assert_eq!(field.name, "Field 2 name");
        assert_eq!(field.description, Some("Field 2 description".to_string()));
        assert_eq!(field.show_custom_field, Some(true));
        assert_eq!(
            field.presets,
            Some(vec![
                OrderBuilderPresetCfg {
                    id: "0".to_string(),
                    name: None,
                    value: "99.2".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "1".to_string(),
                    name: None,
                    value: "582.1".to_string(),
                },
                OrderBuilderPresetCfg {
                    id: "2".to_string(),
                    name: None,
                    value: "648.239".to_string(),
                },
            ])
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_token_info_local() {
        let builder = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();

        let token1_info = builder.get_token_info("token1".to_string()).await.unwrap();
        assert_eq!(
            token1_info.address.to_string(),
            "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"
        );
        assert_eq!(token1_info.decimals, 6);
        assert_eq!(token1_info.name, "Token 1");
        assert_eq!(token1_info.symbol, "T1");

        let token2_info = builder.get_token_info("token2".to_string()).await.unwrap();
        assert_eq!(
            token2_info.address.to_string(),
            "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"
        );
        assert_eq!(token2_info.decimals, 18);
        assert_eq!(token2_info.name, "Token 2");
        assert_eq!(token2_info.symbol, "T2");

        let err = builder
            .get_token_info("invalid-token".to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::KeyNotFound("invalid-token".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Key 'invalid-token' not found"
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_all_token_infos_local() {
        let builder = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();

        let token_infos = builder.get_all_token_infos().await.unwrap();
        assert_eq!(token_infos.len(), 2);
        assert_eq!(
            token_infos[0].address.to_string(),
            "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"
        );
        assert_eq!(token_infos[0].decimals, 6);
        assert_eq!(token_infos[0].name, "Token 1");
        assert_eq!(token_infos[0].symbol, "T1");
        assert_eq!(
            token_infos[1].address.to_string(),
            "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"
        );
        assert_eq!(token_infos[1].decimals, 18);
        assert_eq!(token_infos[1].name, "Token 2");
        assert_eq!(token_infos[1].symbol, "T2");
    }

    #[wasm_bindgen_test]
    fn test_get_order_details() {
        let order_details = RaindexOrderBuilder::get_order_details(get_yaml(), None).unwrap();
        assert_eq!(order_details.name, "Fixed limit");
        assert_eq!(order_details.description, "Fixed limit order");
        assert_eq!(
            order_details.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_order_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "builder".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'name' in builder"
        );

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    name: Test name
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_order_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "builder".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'description' in builder"
        );

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    name: Test name
    description: Test description
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_order_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("short-description".to_string()),
                location: "builder".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'short-description' in builder"
        );
    }

    #[wasm_bindgen_test]
    fn test_get_deployment_details() {
        let deployment_details =
            RaindexOrderBuilder::get_deployment_details(get_yaml(), None).unwrap();
        assert_eq!(deployment_details.len(), 3);
        let deployment_detail = deployment_details.get("some-deployment").unwrap();
        assert_eq!(deployment_detail.name, "Buy WETH with USDC on Base.");
        assert_eq!(
            deployment_detail.description,
            "Buy WETH with USDC for fixed price on Base network."
        );
        assert_eq!(
            deployment_detail.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );
        let deployment_detail = deployment_details.get("other-deployment").unwrap();
        assert_eq!(deployment_detail.name, "Test test");
        assert_eq!(deployment_detail.description, "Test test test");
        assert_eq!(deployment_detail.short_description, None);
        let deployment_detail = deployment_details.get("select-token-deployment").unwrap();
        assert_eq!(deployment_detail.name, "Select token deployment");
        assert_eq!(
            deployment_detail.description,
            "Select token deployment description"
        );
        assert_eq!(deployment_detail.short_description, None);

        let yaml = format!(
            r#"
version: {spec_version}
test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let details = RaindexOrderBuilder::get_deployment_details(yaml.to_string(), None).unwrap();
        assert_eq!(details.len(), 0);

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_deployment_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "builder".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'deployments' in builder"
        );

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    deployments: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_deployment_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string(),
                },
                location: "builder".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Field 'deployments' must be a map in builder"
        );

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    deployments:
        - test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_deployment_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "deployments".to_string(),
                    expected: "a map".to_string(),
                },
                location: "builder".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Field 'deployments' must be a map in builder"
        );

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    deployments:
        test: test
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let details = RaindexOrderBuilder::get_deployment_details(yaml.to_string(), None).unwrap();
        assert_eq!(details.len(), 0);

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    deployments:
        test:
            unknown-field: value
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_deployment_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("name".to_string()),
                location: "builder deployment 'test'".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'name' in builder deployment 'test'"
        );

        let yaml = format!(
            r#"
version: {spec_version}
builder:
    deployments:
        test:
            name: Test name
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );
        let err = RaindexOrderBuilder::get_deployment_details(yaml.to_string(), None).unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("description".to_string()),
                location: "builder deployment 'test'".to_string(),
            }
            .to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "YAML configuration error: Missing required field 'description' in builder deployment 'test'"
        );
    }

    #[wasm_bindgen_test]
    fn test_get_deployment_detail() {
        let deployment_detail = RaindexOrderBuilder::get_deployment_detail(
            get_yaml(),
            None,
            "some-deployment".to_string(),
        )
        .unwrap();
        assert_eq!(deployment_detail.name, "Buy WETH with USDC on Base.");
        assert_eq!(
            deployment_detail.description,
            "Buy WETH with USDC for fixed price on Base network."
        );
        assert_eq!(
            deployment_detail.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );
    }

    #[wasm_bindgen_test]
    async fn test_get_current_deployment_detail() {
        let builder = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();

        let deployment_detail = builder.get_current_deployment_details().unwrap();
        assert_eq!(deployment_detail.name, "Buy WETH with USDC on Base.");
        assert_eq!(
            deployment_detail.description,
            "Buy WETH with USDC for fixed price on Base network."
        );
        assert_eq!(
            deployment_detail.short_description,
            Some("Buy WETH with USDC on Base.".to_string())
        );
    }

    #[wasm_bindgen_test]
    async fn test_generate_dotrain_text() {
        let builder = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();
        let original_current_deployment = builder.get_current_deployment_details().unwrap();

        let dotrain_text = builder.generate_dotrain_text().unwrap();
        let builder = RaindexOrderBuilder::new_with_deployment(
            dotrain_text,
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();
        let new_current_deployment = builder.get_current_deployment_details().unwrap();

        assert_eq!(new_current_deployment, original_current_deployment);
    }

    #[wasm_bindgen_test]
    async fn test_get_composed_rainlang() {
        let mut builder = RaindexOrderBuilder::new_with_deployment(
            get_yaml(),
            None,
            "some-deployment".to_string(),
            None,
        )
        .await
        .unwrap();

        let rainlang = builder.get_composed_rainlang().await.unwrap();
        let expected_rainlang =
            "/* 0. calculate-io */ \n_ _: 0 0;\n\n/* 1. handle-io */ \n:;".to_string();
        assert_eq!(rainlang, expected_rainlang);
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
                None,
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
