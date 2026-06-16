use crate::gui::{DotrainOrderGui, GuiError};
use crate::yaml::{OrderbookYaml, OrderbookYamlError};
use rain_orderbook_app_settings::gui::NameAndDescriptionCfg;
use rain_orderbook_common::raindex_client::{RaindexClient, RaindexError as RaindexClientError};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use url::Url;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

/// A rainlang system for managing dotrain order configurations with layered content merging.
///
/// The `DotrainRainlang` provides a centralized way to fetch, parse, and manage dotrain order
/// strategies from remote sources. It supports a layered architecture where shared settings
/// are merged with individual order configurations.
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
/// ## Content Merging
///
/// For each order, the final dotrain content is created by merging:
/// ```text
/// <settings.yaml content>
///
/// <.rain file content>
/// ```
///
/// This allows strategies to share common network configurations, tokens, and other settings
/// while maintaining their individual logic.
///
/// ## Usage Flow
///
/// 1. **Registry Creation** → Fetches and parses registry file
/// 2. **List Orders** → Get available order strategies with metadata
/// 3. **List Deployments** → Get deployment options for selected order
/// 4. **Create GUI** → Instantiate DotrainOrderGui with merged content
///
/// ## Examples
///
/// ```javascript
/// // Initialize registry
/// const registry = await DotrainRainlang.new("https://example.com/registry.txt");
///
/// // Get available orders
/// const orders = await registry.getAllOrderDetails();
///
/// // Get deployments for specific order
/// const deployments = await registry.getDeploymentDetails("fixed-limit");
///
/// // Create GUI instance
/// const gui = await registry.getGui("fixed-limit", "mainnet", stateCallback);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct DotrainRainlang {
    /// The URL of the registry file containing the list of orders and settings.
    ///
    /// This is the original URL passed to the constructor and serves as the entry point
    /// for fetching the registry configuration. The registry file should contain a
    /// settings URL on the first line followed by order entries.
    rainlang_url: Url,

    /// The raw content of the registry file as fetched from `rainlang_url`.
    ///
    /// Contains the complete registry file content including the settings URL and
    /// all order entries. This is stored for reference and debugging purposes.
    /// Format: settings URL on first line, then "key url" pairs for each order.
    rainlang: String,

    /// The URL of the shared settings YAML file extracted from the first line of the registry.
    ///
    /// This URL points to a YAML file containing shared configuration such as:
    /// - Network configurations (RPCs, chain IDs)
    /// - Subgraph endpoints
    /// - Orderbook contract addresses
    /// - Token definitions
    /// - Other common settings used across multiple strategies
    settings_url: Url,

    /// The content of the shared settings YAML file fetched from `settings_url`.
    ///
    /// Contains shared configuration that will be merged with individual order content
    /// to create complete dotrain files. This typically includes network settings,
    /// contract addresses, and other common configuration that strategies can inherit.
    settings: String,

    /// A map of order keys to their corresponding .rain file URLs.
    ///
    /// Each entry represents an available order where:
    /// - **Key**: Human-readable order identifier (e.g., "fixed-limit", "auction-dca")
    /// - **Value**: URL pointing to the .rain file containing the order's dotrain configuration
    ///
    /// This mapping is parsed from the registry file lines (excluding the first settings line).
    order_urls: HashMap<String, Url>,

    /// A map of order keys to their corresponding .rain file contents.
    ///
    /// Each entry contains the raw dotrain content for an order:
    /// - **Key**: Order identifier matching the keys in `order_urls`
    /// - **Value**: Raw dotrain content fetched from the corresponding URL
    ///
    /// This content is fetched in parallel during registry initialization and stored
    /// for quick access. It gets merged with `settings` content when creating GUIs.
    orders: HashMap<String, String>,
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
pub enum DotrainRainlangError {
    #[error("Failed to fetch rainlang from URL: {0}")]
    RainlangFetchError(String),
    #[error("Failed to parse rainlang content")]
    RainlangParseError,
    #[error("Failed to fetch settings from URL: {0}")]
    SettingsFetchError(String),
    #[error("Failed to fetch order content from URL: {0}")]
    OrderFetchError(String),
    #[error("Order key not found: {0}")]
    OrderKeyNotFound(String),
    #[error("Invalid rainlang format: {0}")]
    InvalidRainlangFormat(String),
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    GuiError(#[from] GuiError),
    #[error(transparent)]
    OrderbookYamlError(#[from] OrderbookYamlError),
    #[error(transparent)]
    RaindexClientError(#[from] RaindexClientError),
}

impl DotrainRainlangError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            DotrainRainlangError::RainlangFetchError(url) => {
                format!("Unable to fetch the rainlang file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            DotrainRainlangError::RainlangParseError => {
                "The rainlang file format is invalid. Please ensure it follows the expected format with settings URL on the first line and order entries on subsequent lines.".to_string()
            }
            DotrainRainlangError::SettingsFetchError(url) => {
                format!("Unable to fetch the settings file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            DotrainRainlangError::OrderFetchError(url) => {
                format!("Unable to fetch the order file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            DotrainRainlangError::OrderKeyNotFound(key) => {
                format!("The order key '{}' was not found in the rainlang. Please check the available order keys.", key)
            }
            DotrainRainlangError::InvalidRainlangFormat(msg) => {
                format!("Invalid rainlang format: {}", msg)
            }
            DotrainRainlangError::HttpError(msg) => {
                format!("Network error: {}", msg)
            }
            DotrainRainlangError::UrlParseError(err) => {
                format!("Invalid URL format: {}. Please ensure the URL is properly formatted.", err)
            }
            DotrainRainlangError::GuiError(err) => err.to_readable_msg(),
            DotrainRainlangError::OrderbookYamlError(err) => err.to_readable_msg(),
            DotrainRainlangError::RaindexClientError(err) => err.to_readable_msg(),
        }
    }
}

impl From<DotrainRainlangError> for WasmEncodedError {
    fn from(value: DotrainRainlangError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[wasm_bindgen]
impl DotrainRainlang {
    #[wasm_bindgen(getter = rainlangUrl)]
    pub fn rainlang_url(&self) -> String {
        self.rainlang_url.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn rainlang(&self) -> String {
        self.rainlang.clone()
    }
    #[wasm_bindgen(getter = settingsUrl)]
    pub fn settings_url(&self) -> String {
        self.settings_url.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn settings(&self) -> String {
        self.settings.clone()
    }
    #[wasm_bindgen(getter = orderUrls)]
    pub fn order_urls(&self) -> OrderUrls {
        OrderUrls(
            self.order_urls
                .iter()
                .map(|(key, value)| (key.clone(), value.to_string()))
                .collect(),
        )
    }
    #[wasm_bindgen(getter = orders)]
    pub fn orders(&self) -> OrderUrls {
        OrderUrls(self.orders.clone())
    }
}

#[wasm_export]
impl DotrainRainlang {
    /// Creates a new DotrainRainlang instance by fetching and parsing the registry file.
    ///
    /// The registry file should contain a settings YAML URL on the first line (without a key),
    /// followed by order entries in the format "key url".
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await DotrainRainlang.new("https://example.com/registry.txt");
    /// if (result.error) {
    ///   console.error("Registry creation failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const registry = result.value;
    /// ```
    #[wasm_export(
        js_name = "new",
        preserve_js_class,
        return_description = "DotrainRainlang instance with settings and orders loaded"
    )]
    pub async fn new(
        #[wasm_export(
            js_name = "rainlangUrl",
            param_description = "URL to the rainlang file containing settings and order definitions"
        )]
        rainlang_url: String,
    ) -> Result<DotrainRainlang, DotrainRainlangError> {
        let rainlang_url = Url::parse(&rainlang_url)?;
        let (rainlang_content, settings_url, order_urls) =
            Self::fetch_and_parse_rainlang(&rainlang_url).await?;
        let settings = Self::fetch_settings(&settings_url).await?;
        let orders = Self::fetch_orders(&order_urls).await?;

        Ok(DotrainRainlang {
            rainlang_url,
            rainlang: rainlang_content,
            settings_url,
            settings,
            order_urls,
            orders,
        })
    }

    /// Validates a registry file without downloading settings or order content.
    ///
    /// Useful for lightweight format checks (e.g., user-input registry URLs) before
    /// performing a full registry load.
    #[wasm_export(
        js_name = "validate",
        return_description = "Validates the rainlang URL and format without fetching settings or orders",
        unchecked_return_type = "void"
    )]
    pub async fn validate(
        #[wasm_export(
            js_name = "rainlangUrl",
            param_description = "URL to the rainlang file containing settings and order definitions"
        )]
        rainlang_url: String,
    ) -> Result<(), DotrainRainlangError> {
        let rainlang_url = Url::parse(&rainlang_url)?;
        // Only fetch and parse the rainlang file to verify format/URLs.
        let _ = Self::fetch_and_parse_rainlang(&rainlang_url).await?;
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
    pub fn get_all_order_details(&self) -> Result<OrderDetailsResult, DotrainRainlangError> {
        let mut valid = BTreeMap::new();
        let mut invalid = BTreeMap::new();
        let settings = self.settings_sources();

        for (order_key, dotrain) in &self.orders {
            match DotrainOrderGui::get_order_details(dotrain.clone(), settings.clone()) {
                Ok(details) => {
                    valid.insert(order_key.clone(), details);
                }
                Err(err) => {
                    invalid.insert(
                        order_key.clone(),
                        WasmEncodedError {
                            msg: err.to_string(),
                            readable_msg: err.to_readable_msg(),
                        },
                    );
                }
            }
        }

        Ok(OrderDetailsResult { valid, invalid })
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
    pub fn get_order_keys(&self) -> Result<Vec<String>, DotrainRainlangError> {
        Ok(self.order_urls.keys().cloned().collect())
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
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, DotrainRainlangError> {
        let dotrain = self
            .orders
            .get(&order_key)
            .ok_or(DotrainRainlangError::OrderKeyNotFound(order_key.clone()))?;
        let settings = self.settings_sources();
        let deployment_details =
            DotrainOrderGui::get_deployment_details(dotrain.clone(), settings.clone())?;
        Ok(deployment_details)
    }

    /// Creates a DotrainOrderGui instance for a specific order and deployment.
    ///
    /// This is a convenience method that combines getting a DotrainOrder and creating a GUI.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Simple usage without state callback
    /// const result = await registry.getGui("fixed-limit", "mainnet-deployment");
    /// if (result.error) {
    ///   console.error("Failed to create GUI:", result.error.readableMsg);
    ///   return;
    /// }
    /// const gui = result.value;
    ///
    /// // Usage with state update callback for auto-saving
    /// const stateCallback = (newState) => {
    ///   localStorage.setItem('gui-state', JSON.stringify(newState));
    /// };
    /// const resultWithCallback = await registry.getGui(
    ///   "fixed-limit",
    ///   "mainnet-deployment",
    ///   undefined,
    ///   stateCallback
    /// );
    ///
    /// // Usage restoring from serialized state (with optional callback)
    /// const savedState = localStorage.getItem('gui-state');
    /// const resultFromState = await registry.getGui(
    ///   "fixed-limit",
    ///   "mainnet-deployment",
    ///   savedState,
    ///   stateCallback
    /// );
    /// ```
    #[wasm_export(
        js_name = "getGui",
        preserve_js_class,
        unchecked_return_type = "DotrainOrderGui",
        return_description = "DotrainOrderGui instance for the specified order and deployment"
    )]
    pub async fn get_gui(
        &self,
        #[wasm_export(
            js_name = "orderKey",
            param_description = "Order key to fetch the GUI for"
        )]
        order_key: String,
        #[wasm_export(
            js_name = "deploymentKey",
            param_description = "Deployment key to create the GUI for"
        )]
        deployment_key: String,
        #[wasm_export(
            js_name = "serializedState",
            param_description = "Optional serialized GUI state string used to restore form progress before falling back to deployment defaults"
        )]
        serialized_state: Option<String>,
        #[wasm_export(
            js_name = "stateUpdateCallback",
            param_description = "Optional function called on state changes. \
            After a state change (deposit, field value, vault id, select token, etc.), the callback is called with the new state. \
            This is useful for auto-saving the state of the GUI across sessions."
        )]
        state_update_callback: Option<js_sys::Function>,
    ) -> Result<DotrainOrderGui, DotrainRainlangError> {
        let dotrain = self
            .orders
            .get(&order_key)
            .ok_or(DotrainRainlangError::OrderKeyNotFound(order_key.clone()))?;
        let settings = self.settings_sources();

        let gui_result = match serialized_state {
            Some(serialized_state) => {
                match DotrainOrderGui::new_from_state(
                    dotrain.clone(),
                    settings.clone(),
                    serialized_state,
                    state_update_callback.clone(),
                )
                .await
                {
                    Ok(gui) => Ok(gui),
                    Err(_) => {
                        DotrainOrderGui::new_with_deployment(
                            dotrain.clone(),
                            settings.clone(),
                            deployment_key,
                            state_update_callback.clone(),
                        )
                        .await
                    }
                }
            }
            None => {
                DotrainOrderGui::new_with_deployment(
                    dotrain.clone(),
                    settings.clone(),
                    deployment_key,
                    state_update_callback.clone(),
                )
                .await
            }
        };

        let gui = gui_result.map_err(DotrainRainlangError::GuiError)?;
        Ok(gui)
    }

    /// Creates an OrderbookYaml instance from the registry's shared settings.
    ///
    /// This method provides access to the OrderbookYaml SDK, allowing you to query tokens,
    /// networks, orderbooks, and other configuration from the shared settings YAML.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const yamlResult = registry.getOrderbookYaml();
    /// if (yamlResult.error) {
    ///   console.error("Failed to get OrderbookYaml:", yamlResult.error.readableMsg);
    ///   return;
    /// }
    /// const orderbookYaml = yamlResult.value;
    /// ```
    #[wasm_export(
        js_name = "getOrderbookYaml",
        preserve_js_class,
        unchecked_return_type = "OrderbookYaml",
        return_description = "OrderbookYaml instance from registry settings"
    )]
    pub fn get_orderbook_yaml(&self) -> Result<OrderbookYaml, DotrainRainlangError> {
        let yaml = OrderbookYaml::new(vec![self.settings.clone()], None)?;
        Ok(yaml)
    }
}

#[cfg(target_family = "wasm")]
#[wasm_export]
impl DotrainRainlang {
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
    ) -> Result<RaindexClient, DotrainRainlangError> {
        let client = RaindexClient::new(
            vec![self.settings.clone()],
            None,
            query_callback,
            wipe_callback,
            status_callback,
        )
        .await?;
        Ok(client)
    }
}

#[cfg(not(target_family = "wasm"))]
impl DotrainRainlang {
    pub async fn get_raindex_client(
        &self,
        db_path: Option<std::path::PathBuf>,
    ) -> Result<RaindexClient, DotrainRainlangError> {
        let client = RaindexClient::new(vec![self.settings.clone()], None, db_path).await?;
        Ok(client)
    }
}

impl DotrainRainlang {
    fn settings_sources(&self) -> Option<Vec<String>> {
        if self.settings.is_empty() {
            None
        } else {
            Some(vec![self.settings.clone()])
        }
    }

    async fn fetch_and_parse_rainlang(
        rainlang_url: &Url,
    ) -> Result<(String, Url, HashMap<String, Url>), DotrainRainlangError> {
        let rainlang_content = Self::fetch_url_content(rainlang_url).await?;
        let (settings_url, order_urls) = Self::parse_rainlang_content(&rainlang_content)?;
        Ok((rainlang_content, settings_url, order_urls))
    }

    fn parse_rainlang_content(
        content: &str,
    ) -> Result<(Url, HashMap<String, Url>), DotrainRainlangError> {
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        if lines.is_empty() {
            return Err(DotrainRainlangError::InvalidRainlangFormat(
                "Rainlang file is empty".to_string(),
            ));
        }

        let first_line = lines[0];
        if first_line.contains(' ') {
            return Err(DotrainRainlangError::InvalidRainlangFormat(
                "First line should be a settings URL without a key".to_string(),
            ));
        }

        let settings_url = Url::parse(first_line)?;
        let mut order_urls = HashMap::new();

        for line in &lines[1..] {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(DotrainRainlangError::InvalidRainlangFormat(format!(
                    "Invalid order entry format: '{}'. Expected: 'key url'",
                    line
                )));
            }

            let key = parts[0].to_string();
            let url = Url::parse(parts[1])?;

            order_urls.insert(key, url);
        }

        Ok((settings_url, order_urls))
    }

    async fn fetch_url_content(url: &Url) -> Result<String, DotrainRainlangError> {
        let response = reqwest::get(url.as_str())
            .await
            .map_err(|e| DotrainRainlangError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DotrainRainlangError::HttpError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| DotrainRainlangError::HttpError(e.to_string()))
    }

    async fn fetch_settings(settings_url: &Url) -> Result<String, DotrainRainlangError> {
        Self::fetch_url_content(settings_url).await
    }

    async fn fetch_orders(
        order_urls: &HashMap<String, Url>,
    ) -> Result<HashMap<String, String>, DotrainRainlangError> {
        use futures::future::join_all;

        let mut futures = Vec::new();

        for (key, url) in order_urls {
            let key_clone = key.clone();
            let url_clone = url.clone();
            futures.push(async move {
                let content = reqwest::get(url_clone.as_str())
                    .await
                    .map_err(|e| DotrainRainlangError::HttpError(e.to_string()))?
                    .text()
                    .await
                    .map_err(|e| DotrainRainlangError::HttpError(e.to_string()))?;
                Ok::<(String, String), DotrainRainlangError>((key_clone, content))
            });
        }

        let results = join_all(futures).await;
        let mut orders = HashMap::new();

        for result in results {
            let (key, content) = result?;
            orders.insert(key, content);
        }

        Ok(orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use std::collections::HashMap;

    const MOCK_REGISTRY_CONTENT: &str = r#"https://example.com/settings.yaml
fixed-limit https://example.com/fixed-limit.rain
auction-dca https://example.com/auction-dca.rain"#;

    fn mock_settings_content() -> String {
        format!(
            r#"version: {version}
networks:
  flare:
    rpcs:
      - https://rpc.ankr.com/flare
    chain-id: 14
    currency: FLR
  base:
    rpcs:
      - https://mainnet.base.org
    chain-id: 8453
    currency: ETH
subgraphs:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/0.8/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn
metaboards:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn
orderbooks:
  flare:
    address: 0xCEe8Cd002F151A536394E564b84076c41bBBcD4d
    network: flare
    subgraph: flare
    deployment-block: 0
  base:
    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7
    network: base
    subgraph: base
    deployment-block: 0
rainlangs:
  flare:
    address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
    network: flare
  base:
    address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
    network: base
tokens:
  token1:
    address: 0x4200000000000000000000000000000000000042
    network: flare
  token2:
    address: 0x4200000000000000000000000000000000000042
    network: base
"#,
            version = SpecVersion::current()
        )
    }

    fn mock_dotrain_prefix() -> String {
        format!(
            r#"version: {version}
gui:"#,
            version = SpecVersion::current()
        )
    }

    const MOCK_DOTRAIN_BODY: &str = r#"
  name: Test gui
  description: Test description
  short-description: Test short description
  deployments:
    flare:
      name: Flare order name
      description: Flare order description
      deposits:
        - token: token1
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
    base:
      name: Base order name
      description: Base order description
      deposits:
        - token: token2
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          description: Test binding description
          presets:
            - value: "0xbeef"
          default: 10
scenarios:
  flare:
    rainlang: flare
    runs: 1
  base:
    rainlang: base
    runs: 1
orders:
  flare:
    orderbook: flare
    inputs:
      - token: token1
    outputs:
      - token: token1
  base:
    orderbook: base
    inputs:
      - token: token2
    outputs:
      - token: token2
deployments:
  flare:
    scenario: flare
    order: flare
  base:
    scenario: base
    order: base
"#;

    fn get_first_dotrain_content() -> String {
        format!(
            r#"{prefix}{body}
----
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#,
            prefix = mock_dotrain_prefix(),
            body = MOCK_DOTRAIN_BODY
        )
    }

    fn get_second_dotrain_content() -> String {
        format!(
            r#"{prefix}{body}
----
#calculate-io
_ _: 1 1;
#handle-io
:;
#handle-add-order
:;"#,
            prefix = mock_dotrain_prefix(),
            body = MOCK_DOTRAIN_BODY
        )
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        fn test_parse_rainlang_content() {
            let (settings_url, order_urls) =
                DotrainRainlang::parse_rainlang_content(MOCK_REGISTRY_CONTENT).unwrap();

            assert_eq!(
                settings_url.to_string(),
                "https://example.com/settings.yaml"
            );
            assert_eq!(order_urls.len(), 2);
            assert_eq!(
                order_urls.get("fixed-limit").map(|u| u.to_string()),
                Some("https://example.com/fixed-limit.rain".to_string())
            );
            assert_eq!(
                order_urls.get("auction-dca").map(|u| u.to_string()),
                Some("https://example.com/auction-dca.rain".to_string())
            );
        }

        #[wasm_bindgen_test]
        fn test_parse_invalid_registry_content() {
            let result = DotrainRainlang::parse_rainlang_content("");
            assert!(result.is_err());

            let result = DotrainRainlang::parse_rainlang_content("invalid first line");
            assert!(result.is_err());

            let result = DotrainRainlang::parse_rainlang_content(
                "https://example.com/settings.yaml\ninvalid-entry",
            );
            assert!(result.is_err());
        }

        #[wasm_bindgen_test]
        fn test_get_order_keys() {
            let registry = DotrainRainlang {
                rainlang_url: Url::parse("https://example.com/test").unwrap(),
                rainlang: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: "".to_string(),
                order_urls: vec![
                    (
                        "fixed-limit".to_string(),
                        Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                    ),
                    (
                        "auction-dca".to_string(),
                        Url::parse("https://example.com/auction-dca.rain").unwrap(),
                    ),
                ]
                .into_iter()
                .collect(),
                orders: HashMap::new(),
            };

            let keys = registry.get_order_keys().unwrap();
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&"fixed-limit".to_string()));
            assert!(keys.contains(&"auction-dca".to_string()));
        }

        #[wasm_bindgen_test]
        fn test_get_all_order_details() {
            let registry = DotrainRainlang {
                rainlang_url: Url::parse("https://example.com/test").unwrap(),
                rainlang: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: mock_settings_content(),
                order_urls: vec![
                    (
                        "fixed-limit".to_string(),
                        Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                    ),
                    (
                        "auction-dca".to_string(),
                        Url::parse("https://example.com/auction-dca.rain").unwrap(),
                    ),
                ]
                .into_iter()
                .collect(),
                orders: vec![
                    ("fixed-limit".to_string(), get_first_dotrain_content()),
                    ("auction-dca".to_string(), get_second_dotrain_content()),
                ]
                .into_iter()
                .collect(),
            };

            let result = registry.get_all_order_details();
            wasm_bindgen_test::console_log!("Result: {:?}", result);
            assert!(result.is_ok());

            let order_details = result.unwrap();
            assert_eq!(order_details.valid.len(), 2);
            assert!(order_details.invalid.is_empty());
            assert!(order_details.valid.contains_key("fixed-limit"));
            assert!(order_details.valid.contains_key("auction-dca"));

            let fixed_limit_details = order_details.valid.get("fixed-limit").unwrap();
            assert_eq!(fixed_limit_details.name, "Test gui");
            assert_eq!(fixed_limit_details.description, "Test description");
        }

        #[wasm_bindgen_test]
        fn test_get_all_order_details_with_invalid_order() {
            let registry = DotrainRainlang {
                rainlang_url: Url::parse("https://example.com/test").unwrap(),
                rainlang: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: mock_settings_content(),
                order_urls: vec![
                    (
                        "valid".to_string(),
                        Url::parse("https://example.com/valid.rain").unwrap(),
                    ),
                    (
                        "invalid".to_string(),
                        Url::parse("https://example.com/invalid.rain").unwrap(),
                    ),
                ]
                .into_iter()
                .collect(),
                orders: vec![
                    ("valid".to_string(), get_first_dotrain_content()),
                    ("invalid".to_string(), "not-a-valid-dotrain".to_string()),
                ]
                .into_iter()
                .collect(),
            };

            let result = registry.get_all_order_details().unwrap();
            assert_eq!(result.valid.len(), 1);
            assert_eq!(result.invalid.len(), 1);
            assert!(result.valid.contains_key("valid"));
            assert!(result.invalid.contains_key("invalid"));
        }

        #[wasm_bindgen_test]
        fn test_get_deployment_details() {
            let registry = DotrainRainlang {
                rainlang_url: Url::parse("https://example.com/test").unwrap(),
                rainlang: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: mock_settings_content(),
                order_urls: vec![(
                    "fixed-limit".to_string(),
                    Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                )]
                .into_iter()
                .collect(),
                orders: vec![("fixed-limit".to_string(), get_first_dotrain_content())]
                    .into_iter()
                    .collect(),
            };

            let result = registry.get_deployment_details("fixed-limit".to_string());
            assert!(result.is_ok());

            let deployment_details = result.unwrap();
            assert_eq!(deployment_details.len(), 2);
            assert!(deployment_details.contains_key("flare"));
            assert!(deployment_details.contains_key("base"));

            let flare_details = deployment_details.get("flare").unwrap();
            assert_eq!(flare_details.name, "Flare order name");
            assert_eq!(flare_details.description, "Flare order description");

            let base_details = deployment_details.get("base").unwrap();
            assert_eq!(base_details.name, "Base order name");
            assert_eq!(base_details.description, "Base order description");
        }

        #[wasm_bindgen_test]
        fn test_get_deployment_details_order_not_found() {
            let registry = DotrainRainlang {
                rainlang_url: Url::parse("https://example.com/test").unwrap(),
                rainlang: "".to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: mock_settings_content(),
                order_urls: HashMap::new(),
                orders: HashMap::new(),
            };

            let result = registry.get_deployment_details("non-existent".to_string());
            assert!(result.is_err());

            match result.err().unwrap() {
                DotrainRainlangError::OrderKeyNotFound(key) => {
                    assert_eq!(key, "non-existent");
                }
                _ => panic!("Expected OrderKeyNotFound error"),
            }
        }

        #[wasm_bindgen_test]
        fn test_getter_methods() {
            let registry = DotrainRainlang {
                rainlang_url: Url::parse("https://example.com/registry.txt").unwrap(),
                rainlang: MOCK_REGISTRY_CONTENT.to_string(),
                settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
                settings: mock_settings_content(),
                order_urls: vec![(
                    "fixed-limit".to_string(),
                    Url::parse("https://example.com/fixed-limit.rain").unwrap(),
                )]
                .into_iter()
                .collect(),
                orders: vec![("fixed-limit".to_string(), get_first_dotrain_content())]
                    .into_iter()
                    .collect(),
            };

            assert_eq!(registry.rainlang_url(), "https://example.com/registry.txt");
            assert_eq!(registry.settings_url(), "https://example.com/settings.yaml");
            assert_eq!(registry.rainlang(), MOCK_REGISTRY_CONTENT);
            assert_eq!(registry.settings(), mock_settings_content());

            let order_urls_map = registry.order_urls();
            assert_eq!(order_urls_map.0.len(), 1);

            let orders_map = registry.orders();
            assert_eq!(orders_map.0.len(), 1);
        }

        #[wasm_bindgen_test]
        fn test_error_readable_messages() {
            let registry_error =
                DotrainRainlangError::RainlangFetchError("https://example.com".to_string());
            let readable = registry_error.to_readable_msg();
            assert!(readable.contains("Unable to fetch the rainlang file"));
            assert!(readable.contains("https://example.com"));

            let parse_error = DotrainRainlangError::RainlangParseError;
            let readable = parse_error.to_readable_msg();
            assert!(readable.contains("rainlang file format is invalid"));

            let not_found_error = DotrainRainlangError::OrderKeyNotFound("test-order".to_string());
            let readable = not_found_error.to_readable_msg();
            assert!(readable.contains("order key 'test-order' was not found"));
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use super::*;
        use httpmock::MockServer;

        #[tokio::test]
        async fn test_new_constructor() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\nfirst-order {}/first-order.rain\nsecond-order {}/second-order.rain",
                server.url(""),
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content.clone());
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(mock_settings_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/first-order.rain");
                then.status(200).body(get_first_dotrain_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/second-order.rain");
                then.status(200).body(get_second_dotrain_content());
            });

            let registry = DotrainRainlang::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            assert_eq!(
                registry.rainlang_url(),
                format!("{}/registry.txt", server.url(""))
            );
            assert_eq!(registry.rainlang(), test_registry_content);
            assert_eq!(
                registry.settings_url(),
                format!("{}/settings.yaml", server.url(""))
            );
            assert_eq!(registry.settings(), mock_settings_content());
            assert_eq!(registry.order_urls.len(), 2);
            assert_eq!(registry.orders.len(), 2);
            assert!(registry.order_urls.contains_key("first-order"));
            assert!(registry.order_urls.contains_key("second-order"));
            assert!(registry.orders.contains_key("first-order"));
            assert!(registry.orders.contains_key("second-order"));

            let first_order_content = registry.orders.get("first-order").unwrap();
            let second_order_content = registry.orders.get("second-order").unwrap();
            assert_ne!(first_order_content, second_order_content);
            assert!(first_order_content.contains("_ _: 0 0;"));
            assert!(second_order_content.contains("_ _: 1 1;"));
        }

        #[tokio::test]
        async fn test_fetch_and_parse_rainlang() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\norder1 {}/order1.rain\norder2 {}/order2.rain",
                server.url(""),
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content.clone());
            });

            let rainlang_url = Url::parse(&format!("{}/registry.txt", server.url(""))).unwrap();
            let (registry_content, settings_url, order_urls) =
                DotrainRainlang::fetch_and_parse_rainlang(&rainlang_url)
                    .await
                    .unwrap();

            assert_eq!(registry_content, test_registry_content);
            assert_eq!(
                settings_url.to_string(),
                format!("{}/settings.yaml", server.url(""))
            );
            assert_eq!(order_urls.len(), 2);
            assert!(order_urls.contains_key("order1"));
            assert!(order_urls.contains_key("order2"));
        }

        #[tokio::test]
        async fn test_validate_success() {
            let server = MockServer::start_async().await;
            let test_registry_content = format!(
                "{}/settings.yaml\norder1 {}/order1.rain",
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content.clone());
            });

            let result =
                DotrainRainlang::validate(format!("{}/registry.txt", server.url(""))).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_validate_invalid_registry() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/invalid-registry.txt");
                then.status(200).body("invalid format");
            });

            let result =
                DotrainRainlang::validate(format!("{}/invalid-registry.txt", server.url(""))).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_fetch_url_content_success() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/test.txt");
                then.status(200).body("test content");
            });

            let url = Url::parse(&format!("{}/test.txt", server.url(""))).unwrap();
            let content = DotrainRainlang::fetch_url_content(&url).await.unwrap();

            assert_eq!(content, "test content");
        }

        #[tokio::test]
        async fn test_fetch_url_content_http_error() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/error.txt");
                then.status(500);
            });

            let url = Url::parse(&format!("{}/error.txt", server.url(""))).unwrap();
            let result = DotrainRainlang::fetch_url_content(&url).await;

            assert!(result.is_err());
            match result.err().unwrap() {
                DotrainRainlangError::HttpError(msg) => {
                    assert!(msg.contains("HTTP 500"));
                }
                _ => panic!("Expected HttpError"),
            }
        }

        #[tokio::test]
        async fn test_fetch_settings() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body("test settings content");
            });

            let url = Url::parse(&format!("{}/settings.yaml", server.url(""))).unwrap();
            let settings = DotrainRainlang::fetch_settings(&url).await.unwrap();

            assert_eq!(settings, "test settings content");
        }

        #[tokio::test]
        async fn test_fetch_orders() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/order1.rain");
                then.status(200).body(get_first_dotrain_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/order2.rain");
                then.status(200).body(get_second_dotrain_content());
            });

            let order_urls: HashMap<String, Url> = vec![
                (
                    "order1".to_string(),
                    Url::parse(&format!("{}/order1.rain", server.url(""))).unwrap(),
                ),
                (
                    "order2".to_string(),
                    Url::parse(&format!("{}/order2.rain", server.url(""))).unwrap(),
                ),
            ]
            .into_iter()
            .collect();

            let orders = DotrainRainlang::fetch_orders(&order_urls).await.unwrap();

            assert_eq!(orders.len(), 2);
            assert_eq!(orders.get("order1").unwrap(), &get_first_dotrain_content());
            assert_eq!(orders.get("order2").unwrap(), &get_second_dotrain_content());

            let order1_content = orders.get("order1").unwrap();
            let order2_content = orders.get("order2").unwrap();
            assert_ne!(order1_content, order2_content);
            assert!(order1_content.contains("_ _: 0 0;"));
            assert!(order2_content.contains("_ _: 1 1;"));
        }

        #[tokio::test]
        async fn test_invalid_registry_format() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/invalid-registry.txt");
                then.status(200)
                    .body("invalid format without proper structure");
            });

            let result =
                DotrainRainlang::new(format!("{}/invalid-registry.txt", server.url(""))).await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_empty_registry_file() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/empty-registry.txt");
                then.status(200).body("");
            });

            let result =
                DotrainRainlang::new(format!("{}/empty-registry.txt", server.url(""))).await;
            assert!(result.is_err());

            match result.err().unwrap() {
                DotrainRainlangError::InvalidRainlangFormat(msg) => {
                    assert!(msg.contains("Rainlang file is empty"));
                }
                _ => panic!("Expected InvalidRainlangFormat error for empty rainlang"),
            }
        }

        #[tokio::test]
        async fn test_registry_with_only_settings() {
            let server = MockServer::start_async().await;

            let settings_only_content = format!("{}/settings.yaml", server.url(""));

            server.mock(|when, then| {
                when.method("GET").path("/settings-only-registry.txt");
                then.status(200).body(settings_only_content);
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(mock_settings_content());
            });

            let registry =
                DotrainRainlang::new(format!("{}/settings-only-registry.txt", server.url("")))
                    .await
                    .unwrap();

            assert_eq!(
                registry.settings_url(),
                format!("{}/settings.yaml", server.url(""))
            );
            assert_eq!(registry.settings(), mock_settings_content());
            assert_eq!(registry.order_urls.len(), 0);
            assert_eq!(registry.orders.len(), 0);

            let keys = registry.get_order_keys().unwrap();
            assert!(keys.is_empty());
        }

        #[tokio::test]
        async fn test_get_gui() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\nfirst-order {}/first-order.rain\nsecond-order {}/second-order.rain",
                server.url(""),
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content);
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(mock_settings_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/first-order.rain");
                then.status(200).body(get_first_dotrain_content());
            });

            server.mock(|when, then| {
                when.method("GET").path("/second-order.rain");
                then.status(200).body(get_second_dotrain_content());
            });

            let registry = DotrainRainlang::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            assert_eq!(registry.order_urls.len(), 2);
            assert!(registry.order_urls.contains_key("first-order"));
            assert!(registry.order_urls.contains_key("second-order"));
            assert_eq!(registry.orders.len(), 2);

            let gui1 = registry
                .get_gui("first-order".to_string(), "flare".to_string(), None, None)
                .await
                .unwrap();

            let deployment_details1 = gui1.get_current_deployment().unwrap();
            assert_eq!(deployment_details1.name, "Flare order name");
            assert_eq!(deployment_details1.description, "Flare order description");

            let default_serialized_state = gui1.serialize_state().unwrap();

            let gui2 = registry
                .get_gui("second-order".to_string(), "base".to_string(), None, None)
                .await
                .unwrap();

            let deployment_details2 = gui2.get_current_deployment().unwrap();
            assert_eq!(deployment_details2.name, "Base order name");
            assert_eq!(deployment_details2.description, "Base order description");

            let mut gui_with_state = registry
                .get_gui("first-order".to_string(), "flare".to_string(), None, None)
                .await
                .unwrap();
            gui_with_state
                .set_field_value("test-binding".to_string(), "42".to_string())
                .unwrap();
            let saved_state = gui_with_state.serialize_state().unwrap();

            let restored_gui = registry
                .get_gui(
                    "first-order".to_string(),
                    "flare".to_string(),
                    Some(saved_state.clone()),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(restored_gui.serialize_state().unwrap(), saved_state);

            let fallback_gui = registry
                .get_gui(
                    "first-order".to_string(),
                    "flare".to_string(),
                    Some("not-a-valid-state".to_string()),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                fallback_gui.serialize_state().unwrap(),
                default_serialized_state
            );

            let result = registry
                .get_gui(
                    "non-existent-order".to_string(),
                    "flare".to_string(),
                    None,
                    None,
                )
                .await;
            assert!(result.is_err());
            match result.err().unwrap() {
                DotrainRainlangError::OrderKeyNotFound(key) => {
                    assert_eq!(key, "non-existent-order");
                }
                _ => panic!("Expected OrderKeyNotFound error"),
            }
        }

        fn mock_settings_with_tokens() -> String {
            format!(
                r#"version: {}
networks:
  mainnet:
    rpcs:
      - https://mainnet.infura.io
    chain-id: 1
    currency: ETH
tokens:
  weth:
    network: mainnet
    address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
    decimals: 18
    label: Wrapped Ether
    symbol: WETH
  usdc:
    network: mainnet
    address: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    decimals: 6
    label: USD Coin
    symbol: USDC
orderbooks:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
    network: mainnet
rainlangs:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
    network: mainnet
"#,
                SpecVersion::current()
            )
        }

        const MOCK_DOTRAIN_SIMPLE: &str = r#"gui:
  name: Test Order
  description: Test description
  deployments:
    mainnet:
      name: Mainnet Order
      description: Mainnet deployment
      deposits:
        - token: weth
          presets:
            - "0"
      fields:
        - binding: test-binding
          name: Test binding
          presets:
            - value: "0xbeef"
scenarios:
  mainnet:
    rainlang: mainnet
    runs: 1
orders:
  mainnet:
    orderbook: mainnet
    inputs:
      - token: weth
    outputs:
      - token: usdc
deployments:
  mainnet:
    scenario: mainnet
    order: mainnet
---
#calculate-io
_ _: 0 0;
#handle-io
:;
"#;

        #[tokio::test]
        async fn test_get_orderbook_yaml_returns_valid_instance() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\ntest-order {}/order.rain",
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content.clone());
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(mock_settings_with_tokens());
            });

            server.mock(|when, then| {
                when.method("GET").path("/order.rain");
                then.status(200).body(MOCK_DOTRAIN_SIMPLE);
            });

            let registry = DotrainRainlang::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            let orderbook_yaml = registry.get_orderbook_yaml();
            assert!(orderbook_yaml.is_ok());
        }

        #[tokio::test]
        async fn test_get_raindex_client_returns_valid_instance() {
            let server = MockServer::start_async().await;

            let test_registry_content = format!(
                "{}/settings.yaml\ntest-order {}/order.rain",
                server.url(""),
                server.url("")
            );

            server.mock(|when, then| {
                when.method("GET").path("/registry.txt");
                then.status(200).body(test_registry_content.clone());
            });

            server.mock(|when, then| {
                when.method("GET").path("/settings.yaml");
                then.status(200).body(mock_settings_with_tokens());
            });

            server.mock(|when, then| {
                when.method("GET").path("/order.rain");
                then.status(200).body(MOCK_DOTRAIN_SIMPLE);
            });

            let registry = DotrainRainlang::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            let raindex_client = registry.get_raindex_client(None).await;
            assert!(raindex_client.is_ok());
        }
    }
}
