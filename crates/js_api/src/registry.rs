use crate::raindex_order_builder::RaindexOrderBuilder;
use crate::yaml::{RaindexYaml, RaindexYamlError};
use raindex_app_settings::order_builder::NameAndDescriptionCfg;
use raindex_common::registry::{
    DotrainRegistry as DotrainRegistryInner, DotrainRegistryError as DotrainRegistryCoreError,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

/// WASM wrapper around [`raindex_common::registry::DotrainRegistry`].
///
/// The registry system manages dotrain order configurations with layered content merging.
/// The platform-agnostic business logic (fetching, parsing, validation, metadata extraction)
/// lives in `raindex_common::registry`; this type adds the WASM bindings, `js_sys::Function`
/// state-update callbacks, and `JsValue` error conversions on top.
///
/// ## Registry File Format
///
/// The registry file follows a specific format:
/// - **First line**: URL to shared settings YAML file (without a key)
/// - **Subsequent lines**: Order entries in format "key url"
///
/// ```text
/// https://example.com/shared-settings.yaml
/// fixed-limit https://example.com/fixed-limit.rain
/// auction-dca https://example.com/auction-dca.rain
/// ```
///
/// ## Examples
///
/// ```javascript
/// // Initialize registry
/// const registry = await DotrainRegistry.new("https://example.com/registry.txt");
///
/// // Get available orders
/// const orders = await registry.getAllOrderDetails();
///
/// // Get deployments for specific order
/// const deployments = await registry.getDeploymentDetails("fixed-limit");
///
/// // Create order builder instance
/// const builder = await registry.getOrderBuilder("fixed-limit", "mainnet", stateCallback);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct DotrainRegistry {
    inner: DotrainRegistryInner,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct OrderDetailsResult {
    pub valid: BTreeMap<String, NameAndDescriptionCfg>,
    pub invalid: BTreeMap<String, WasmEncodedError>,
}
impl_wasm_traits!(OrderDetailsResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct OrderUrls(pub HashMap<String, String>);
impl_wasm_traits!(OrderUrls);

#[derive(Error, Debug)]
pub enum DotrainRegistryError {
    #[error(transparent)]
    RegistryError(Box<DotrainRegistryCoreError>),
    #[error(transparent)]
    RaindexYamlError(#[from] RaindexYamlError),
}

impl From<DotrainRegistryCoreError> for DotrainRegistryError {
    fn from(err: DotrainRegistryCoreError) -> Self {
        Self::RegistryError(Box::new(err))
    }
}

impl DotrainRegistryError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            DotrainRegistryError::RegistryError(err) => err.to_readable_msg(),
            DotrainRegistryError::RaindexYamlError(err) => err.to_readable_msg(),
        }
    }
}

impl From<DotrainRegistryError> for WasmEncodedError {
    fn from(value: DotrainRegistryError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[wasm_bindgen]
impl DotrainRegistry {
    #[wasm_bindgen(getter = registryUrl)]
    pub fn registry_url(&self) -> String {
        self.inner.registry_url()
    }
    #[wasm_bindgen(getter)]
    pub fn registry(&self) -> String {
        self.inner.registry()
    }
    #[wasm_bindgen(getter = settingsUrl)]
    pub fn settings_url(&self) -> String {
        self.inner.settings_url()
    }
    #[wasm_bindgen(getter)]
    pub fn settings(&self) -> String {
        self.inner.settings()
    }
    #[wasm_bindgen(getter = orderUrls)]
    pub fn order_urls(&self) -> OrderUrls {
        OrderUrls(self.inner.order_urls())
    }
    #[wasm_bindgen(getter = orders)]
    pub fn orders(&self) -> OrderUrls {
        OrderUrls(self.inner.orders())
    }
}

#[wasm_export]
impl DotrainRegistry {
    /// Creates a new DotrainRegistry instance by fetching and parsing the registry file.
    ///
    /// The registry file should contain a settings YAML URL on the first line (without a key),
    /// followed by order entries in the format "key url".
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await DotrainRegistry.new("https://example.com/registry.txt");
    /// if (result.error) {
    ///   console.error("Registry creation failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const registry = result.value;
    /// ```
    #[wasm_export(
        js_name = "new",
        preserve_js_class,
        return_description = "DotrainRegistry instance with settings and orders loaded"
    )]
    pub async fn new(
        #[wasm_export(
            js_name = "registryUrl",
            param_description = "URL to the registry file containing settings and order definitions"
        )]
        registry_url: String,
    ) -> Result<DotrainRegistry, DotrainRegistryError> {
        let inner = DotrainRegistryInner::new(registry_url).await?;
        Ok(DotrainRegistry { inner })
    }

    /// Validates a registry file without downloading settings or order content.
    ///
    /// Useful for lightweight format checks (e.g., user-input registry URLs) before
    /// performing a full registry load.
    #[wasm_export(
        js_name = "validate",
        return_description = "Validates the registry URL and format without fetching settings or orders",
        unchecked_return_type = "void"
    )]
    pub async fn validate(
        #[wasm_export(
            js_name = "registryUrl",
            param_description = "URL to the registry file containing settings and order definitions"
        )]
        registry_url: String,
    ) -> Result<(), DotrainRegistryError> {
        DotrainRegistryInner::validate(registry_url).await?;
        Ok(())
    }

    /// Gets details for all orders in the registry.
    ///
    /// This method extracts name and description information for each order,
    /// useful for building the initial order selection UI. Any order that
    /// fails to parse/validate will be placed in the `invalid` map with its
    /// corresponding error.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await registry.getAllOrderDetails();
    /// if (result.error) {
    ///   console.error("Failed to get order details:", result.error.readableMsg);
    ///   return;
    /// }
    /// const orderDetails = result.value;
    /// // Map of order key -> {name, description, short_description}
    /// for (const [orderKey, details] of orderDetails) {
    ///   console.log(`${orderKey}: ${details.name}`);
    /// }
    /// ```
    #[wasm_export(
        js_name = "getAllOrderDetails",
        unchecked_return_type = "{ valid: Map<string, NameAndDescriptionCfg>, invalid: Map<string, WasmEncodedError> }",
        return_description = "Valid and invalid order metadata grouped by order key"
    )]
    pub fn get_all_order_details(&self) -> Result<OrderDetailsResult, DotrainRegistryError> {
        let details = self.inner.get_all_order_details();
        let invalid = details
            .invalid
            .into_iter()
            .map(|(order_key, err)| {
                (
                    order_key,
                    WasmEncodedError {
                        msg: err.to_string(),
                        readable_msg: err.to_readable_msg(),
                    },
                )
            })
            .collect();

        Ok(OrderDetailsResult {
            valid: details.valid,
            invalid,
        })
    }

    /// Returns a list of all order keys available in the registry.
    ///
    /// Use this method to get the available order identifiers.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await registry.getOrderKeys();
    /// if (result.error) {
    ///   console.error("Failed to fetch order keys:", result.error.readableMsg);
    ///   return;
    /// }
    /// const keys = result.value;
    /// console.log("Available orders:", keys);
    /// ```
    #[wasm_export(
        js_name = "getOrderKeys",
        unchecked_return_type = "string[]",
        return_description = "Array of order keys available in the registry"
    )]
    pub fn get_order_keys(&self) -> Result<Vec<String>, DotrainRegistryError> {
        Ok(self.inner.get_order_keys())
    }

    /// Gets deployment details for a specific order.
    ///
    /// This method extracts deployment information for a given order,
    /// useful for building the deployment selection UI.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await registry.getDeploymentDetails("fixed-limit");
    /// if (result.error) {
    ///   console.error("Failed to get deployment details:", result.error.readableMsg);
    ///   return;
    /// }
    /// const deploymentDetails = result.value;
    /// // Map of deployment key -> {name, description, short_description}
    /// for (const [deploymentKey, details] of deploymentDetails) {
    ///   console.log(`${deploymentKey}: ${details.name}`);
    /// }
    /// ```
    #[wasm_export(
        js_name = "getDeploymentDetails",
        unchecked_return_type = "Map<string, NameAndDescriptionCfg>",
        return_description = "Map of deployment key to deployment metadata"
    )]
    pub fn get_deployment_details(
        &self,
        #[wasm_export(
            js_name = "orderKey",
            param_description = "Order key to get deployment details for"
        )]
        order_key: String,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, DotrainRegistryError> {
        Ok(self.inner.get_deployment_details(order_key)?)
    }

    /// Creates a RaindexOrderBuilder instance for a specific order and deployment.
    ///
    /// This is a convenience method that combines getting a DotrainOrder and creating a builder.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Simple usage without state callback
    /// const result = await registry.getOrderBuilder("fixed-limit", "mainnet-deployment");
    /// if (result.error) {
    ///   console.error("Failed to create order builder:", result.error.readableMsg);
    ///   return;
    /// }
    /// const builder = result.value;
    ///
    /// // Usage with state update callback for auto-saving
    /// const stateCallback = (newState) => {
    ///   localStorage.setItem('builder-state', JSON.stringify(newState));
    /// };
    /// const resultWithCallback = await registry.getOrderBuilder(
    ///   "fixed-limit",
    ///   "mainnet-deployment",
    ///   undefined,
    ///   stateCallback
    /// );
    ///
    /// // Usage restoring from serialized state (with optional callback)
    /// const savedState = localStorage.getItem('builder-state');
    /// const resultFromState = await registry.getOrderBuilder(
    ///   "fixed-limit",
    ///   "mainnet-deployment",
    ///   savedState,
    ///   stateCallback
    /// );
    /// ```
    #[wasm_export(
        js_name = "getOrderBuilder",
        preserve_js_class,
        unchecked_return_type = "RaindexOrderBuilder",
        return_description = "RaindexOrderBuilder instance for the specified order and deployment"
    )]
    pub async fn get_order_builder(
        &self,
        #[wasm_export(
            js_name = "orderKey",
            param_description = "Order key to fetch the order builder for"
        )]
        order_key: String,
        #[wasm_export(
            js_name = "deploymentKey",
            param_description = "Deployment key to create the order builder for"
        )]
        deployment_key: String,
        #[wasm_export(
            js_name = "serializedState",
            param_description = "Optional serialized builder state string used to restore form progress before falling back to deployment defaults"
        )]
        serialized_state: Option<String>,
        #[wasm_export(
            js_name = "stateUpdateCallback",
            param_description = "Optional function called on state changes. \
            After a state change (deposit, field value, vault id, select token, etc.), the callback is called with the new state. \
            This is useful for auto-saving the state of the builder across sessions."
        )]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<RaindexOrderBuilder, DotrainRegistryError> {
        let inner = self
            .inner
            .get_order_builder(order_key, deployment_key, serialized_state)
            .await?;
        Ok(RaindexOrderBuilder::from_inner(
            inner,
            state_update_callback,
        ))
    }

    /// Creates an RaindexYaml instance from the registry's shared settings.
    ///
    /// This method provides access to the RaindexYaml SDK, allowing you to query tokens,
    /// networks, raindexes, and other configuration from the shared settings YAML.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const yamlResult = registry.getRaindexYaml();
    /// if (yamlResult.error) {
    ///   console.error("Failed to get RaindexYaml:", yamlResult.error.readableMsg);
    ///   return;
    /// }
    /// const raindexYaml = yamlResult.value;
    /// ```
    #[wasm_export(
        js_name = "getRaindexYaml",
        preserve_js_class,
        unchecked_return_type = "RaindexYaml",
        return_description = "RaindexYaml instance from registry settings"
    )]
    pub fn get_raindex_yaml(&self) -> Result<RaindexYaml, DotrainRegistryError> {
        let yaml = RaindexYaml::new(vec![self.inner.settings()], None)?;
        Ok(yaml)
    }
}

#[cfg(target_family = "wasm")]
#[wasm_export]
impl DotrainRegistry {
    /// Creates a RaindexClient instance from the registry's shared settings.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const clientResult = await registry.getRaindexClient(
    ///   localDb.query.bind(localDb),
    ///   localDb.wipeAndRecreate.bind(localDb),
    ///   updateStatus,
    /// );
    /// if (clientResult.error) {
    ///   console.error("Failed to get RaindexClient:", clientResult.error.readableMsg);
    ///   return;
    /// }
    /// const raindexClient = clientResult.value;
    /// ```
    #[wasm_export(
        js_name = "getRaindexClient",
        preserve_js_class,
        unchecked_return_type = "RaindexClient",
        return_description = "RaindexClient instance from registry settings"
    )]
    pub async fn get_raindex_client(
        &self,
        #[wasm_export(
            js_name = "queryCallback",
            param_description = "Optional JavaScript function to execute local database queries"
        )]
        query_callback: Option<js_sys::Function>,
        #[wasm_export(
            js_name = "wipeCallback",
            param_description = "Optional JavaScript function to wipe and recreate the database"
        )]
        wipe_callback: Option<js_sys::Function>,
        #[wasm_export(
            js_name = "statusCallback",
            param_description = "Optional callback invoked with the current local DB sync status"
        )]
        status_callback: Option<js_sys::Function>,
    ) -> Result<raindex_common::raindex_client::RaindexClient, DotrainRegistryError> {
        let client = raindex_common::raindex_client::RaindexClient::new(
            vec![self.inner.settings()],
            None,
            query_callback,
            wipe_callback,
            status_callback,
        )
        .await
        .map_err(DotrainRegistryCoreError::from)?;
        Ok(client)
    }
}
