use crate::raindex_client::RaindexError as RaindexClientError;
use crate::raindex_order_builder::{RaindexOrderBuilder, RaindexOrderBuilderError};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use raindex_app_settings::order_builder::NameAndDescriptionCfg;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use url::Url;

/// A registry system for managing dotrain order configurations with layered content merging.
///
/// The `RaindexRegistry` provides a centralized way to fetch, parse, and manage dotrain order
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
/// 4. **Create Builder** → Instantiate RaindexOrderBuilder with merged content
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RaindexRegistry {
    /// The URL of the registry file containing the list of orders and settings.
    ///
    /// This is the original URL passed to the constructor and serves as the entry point
    /// for fetching the registry configuration. The registry file should contain a
    /// settings URL on the first line followed by order entries.
    registry_url: Url,

    /// The raw content of the registry file as fetched from `registry_url`.
    ///
    /// Contains the complete registry file content including the settings URL and
    /// all order entries. This is stored for reference and debugging purposes.
    /// Format: settings URL on first line, then "key url" pairs for each order.
    registry: String,

    /// The URL of the shared settings YAML file extracted from the first line of the registry.
    ///
    /// This URL points to a YAML file containing shared configuration such as:
    /// - Network configurations (RPCs, chain IDs)
    /// - Subgraph endpoints
    /// - Raindex contract addresses
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
    /// for quick access. It gets merged with `settings` content when creating builders.
    orders: HashMap<String, String>,
}

/// Order metadata grouped by validity, as produced by [`RaindexRegistry::get_all_order_details`].
#[derive(Debug)]
pub struct OrderDetailsResult {
    pub valid: BTreeMap<String, NameAndDescriptionCfg>,
    pub invalid: BTreeMap<String, RaindexRegistryError>,
}

#[derive(Error, Debug)]
pub enum RaindexRegistryError {
    #[error("Failed to fetch registry from URL: {0}")]
    RegistryFetchError(String),
    #[error("Failed to parse registry content")]
    RegistryParseError,
    #[error("Failed to fetch settings from URL: {0}")]
    SettingsFetchError(String),
    #[error("Failed to fetch order content from URL: {0}")]
    OrderFetchError(String),
    #[error("Order key not found: {0}")]
    OrderKeyNotFound(String),
    #[error("Invalid registry format: {0}")]
    InvalidRegistryFormat(String),
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("Invalid data URI: {0}")]
    DataUriError(String),
    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    BuilderError(#[from] RaindexOrderBuilderError),
    #[error(transparent)]
    RaindexClientError(#[from] RaindexClientError),
}

impl RaindexRegistryError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            RaindexRegistryError::RegistryFetchError(url) => {
                format!("Unable to fetch the registry file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            RaindexRegistryError::RegistryParseError => {
                "The registry file format is invalid. Please ensure it follows the expected format with settings URL on the first line and order entries on subsequent lines.".to_string()
            }
            RaindexRegistryError::SettingsFetchError(url) => {
                format!("Unable to fetch the settings file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            RaindexRegistryError::OrderFetchError(url) => {
                format!("Unable to fetch the order file from {}. Please check your internet connection and ensure the URL is accessible.", url)
            }
            RaindexRegistryError::OrderKeyNotFound(key) => {
                format!("The order key '{}' was not found in the registry. Please check the available order keys.", key)
            }
            RaindexRegistryError::InvalidRegistryFormat(msg) => {
                format!("Invalid registry format: {}", msg)
            }
            RaindexRegistryError::HttpError(msg) => {
                format!("Network error: {}", msg)
            }
            RaindexRegistryError::DataUriError(msg) => {
                format!("Invalid data URI: {}", msg)
            }
            RaindexRegistryError::UrlParseError(err) => {
                format!("Invalid URL format: {}. Please ensure the URL is properly formatted.", err)
            }
            RaindexRegistryError::BuilderError(err) => err.to_readable_msg(),
            RaindexRegistryError::RaindexClientError(err) => err.to_readable_msg(),
        }
    }
}

impl RaindexRegistry {
    /// Creates a new RaindexRegistry instance by fetching and parsing the registry file.
    ///
    /// The registry file should contain a settings YAML URL on the first line (without a key),
    /// followed by order entries in the format "key url".
    pub async fn new(registry_url: String) -> Result<RaindexRegistry, RaindexRegistryError> {
        let registry_url = Url::parse(&registry_url)?;
        let (registry_content, settings_url, order_urls) =
            Self::fetch_and_parse_registry(&registry_url).await?;
        let settings = Self::fetch_settings(&settings_url).await?;
        let orders = Self::fetch_orders(&order_urls).await?;

        Ok(RaindexRegistry {
            registry_url,
            registry: registry_content,
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
    pub async fn validate(registry_url: String) -> Result<(), RaindexRegistryError> {
        let registry_url = Url::parse(&registry_url)?;
        // Only fetch and parse the registry file to verify format/URLs.
        let _ = Self::fetch_and_parse_registry(&registry_url).await?;
        Ok(())
    }

    /// The URL of the registry file as a string.
    pub fn registry_url(&self) -> String {
        self.registry_url.to_string()
    }

    /// The raw content of the registry file.
    pub fn registry(&self) -> String {
        self.registry.clone()
    }

    /// The URL of the shared settings YAML file as a string.
    pub fn settings_url(&self) -> String {
        self.settings_url.to_string()
    }

    /// The content of the shared settings YAML file.
    pub fn settings(&self) -> String {
        self.settings.clone()
    }

    /// A map of order keys to their .rain file URLs as strings.
    pub fn order_urls(&self) -> HashMap<String, String> {
        self.order_urls
            .iter()
            .map(|(key, value)| (key.clone(), value.to_string()))
            .collect()
    }

    /// A map of order keys to their raw .rain file contents.
    pub fn orders(&self) -> HashMap<String, String> {
        self.orders.clone()
    }

    /// Gets details for all orders in the registry.
    ///
    /// This method extracts name and description information for each order,
    /// useful for building the initial order selection UI. Any order that
    /// fails to parse/validate will be placed in the `invalid` map with its
    /// corresponding error.
    pub fn get_all_order_details(&self) -> OrderDetailsResult {
        let mut valid = BTreeMap::new();
        let mut invalid = BTreeMap::new();
        let settings = self.settings_sources();

        for (order_key, dotrain) in &self.orders {
            match RaindexOrderBuilder::get_order_details(dotrain.clone(), settings.clone()) {
                Ok(details) => {
                    valid.insert(order_key.clone(), details);
                }
                Err(err) => {
                    invalid.insert(order_key.clone(), RaindexRegistryError::from(err));
                }
            }
        }

        OrderDetailsResult { valid, invalid }
    }

    /// Returns a list of all order keys available in the registry.
    pub fn get_order_keys(&self) -> Vec<String> {
        self.order_urls.keys().cloned().collect()
    }

    /// Gets deployment details for a specific order.
    ///
    /// This method extracts deployment information for a given order,
    /// useful for building the deployment selection UI.
    pub fn get_deployment_details(
        &self,
        order_key: String,
    ) -> Result<BTreeMap<String, NameAndDescriptionCfg>, RaindexRegistryError> {
        let dotrain = self
            .orders
            .get(&order_key)
            .ok_or(RaindexRegistryError::OrderKeyNotFound(order_key.clone()))?;
        let settings = self.settings_sources();
        let deployment_details =
            RaindexOrderBuilder::get_deployment_details(dotrain.clone(), settings.clone())?;
        Ok(deployment_details)
    }

    /// Creates a RaindexOrderBuilder instance for a specific order and deployment.
    ///
    /// This is a convenience method that combines getting a DotrainOrder and creating a builder.
    /// When a serialized state is supplied it is restored, falling back to the deployment
    /// defaults if restoration fails.
    pub async fn get_order_builder(
        &self,
        order_key: String,
        deployment_key: String,
        serialized_state: Option<String>,
    ) -> Result<RaindexOrderBuilder, RaindexRegistryError> {
        let dotrain = self
            .orders
            .get(&order_key)
            .ok_or(RaindexRegistryError::OrderKeyNotFound(order_key.clone()))?;
        let settings = self.settings_sources();

        let result = match serialized_state {
            Some(serialized_state) => {
                match RaindexOrderBuilder::new_from_state(
                    dotrain.clone(),
                    settings.clone(),
                    serialized_state,
                )
                .await
                {
                    Ok(builder) => Ok(builder),
                    Err(_) => {
                        RaindexOrderBuilder::new_with_deployment(
                            dotrain.clone(),
                            settings.clone(),
                            deployment_key,
                        )
                        .await
                    }
                }
            }
            None => {
                RaindexOrderBuilder::new_with_deployment(
                    dotrain.clone(),
                    settings.clone(),
                    deployment_key,
                )
                .await
            }
        };

        let builder = result.map_err(RaindexRegistryError::BuilderError)?;
        Ok(builder)
    }

    /// Returns the shared settings content as a sources list, if non-empty.
    pub fn settings_sources(&self) -> Option<Vec<String>> {
        if self.settings.is_empty() {
            None
        } else {
            Some(vec![self.settings.clone()])
        }
    }

    async fn fetch_and_parse_registry(
        registry_url: &Url,
    ) -> Result<(String, Url, HashMap<String, Url>), RaindexRegistryError> {
        let registry_content = Self::fetch_url_content(registry_url).await?;
        let (settings_url, order_urls) = Self::parse_registry_content(&registry_content)?;
        Ok((registry_content, settings_url, order_urls))
    }

    fn parse_registry_content(
        content: &str,
    ) -> Result<(Url, HashMap<String, Url>), RaindexRegistryError> {
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        if lines.is_empty() {
            return Err(RaindexRegistryError::InvalidRegistryFormat(
                "Registry file is empty".to_string(),
            ));
        }

        let first_line = lines[0];
        if first_line.contains(' ') {
            return Err(RaindexRegistryError::InvalidRegistryFormat(
                "First line should be a settings URL without a key".to_string(),
            ));
        }

        let settings_url = Url::parse(first_line)?;
        let mut order_urls = HashMap::new();

        for line in &lines[1..] {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(RaindexRegistryError::InvalidRegistryFormat(format!(
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

    async fn fetch_url_content(url: &Url) -> Result<String, RaindexRegistryError> {
        if url.scheme() == "data" {
            return Self::decode_data_uri_content(url);
        }

        let response = reqwest::get(url.as_str())
            .await
            .map_err(|e| RaindexRegistryError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(RaindexRegistryError::HttpError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| RaindexRegistryError::HttpError(e.to_string()))
    }

    fn decode_data_uri_content(url: &Url) -> Result<String, RaindexRegistryError> {
        let data_uri = url
            .as_str()
            .strip_prefix("data:")
            .ok_or_else(|| RaindexRegistryError::DataUriError("missing data scheme".to_string()))?;
        let (metadata, payload) = data_uri.split_once(',').ok_or_else(|| {
            RaindexRegistryError::DataUriError("missing metadata/payload separator".to_string())
        })?;
        let is_base64 = metadata
            .split(';')
            .any(|part| part.eq_ignore_ascii_case("base64"));

        if !is_base64 {
            return Err(RaindexRegistryError::DataUriError(
                "payload must be base64 encoded".to_string(),
            ));
        }

        let bytes = BASE64_STANDARD.decode(payload).map_err(|_| {
            RaindexRegistryError::DataUriError("invalid base64 payload".to_string())
        })?;
        String::from_utf8(bytes).map_err(|_| {
            RaindexRegistryError::DataUriError("decoded payload is not valid UTF-8".to_string())
        })
    }

    async fn fetch_settings(settings_url: &Url) -> Result<String, RaindexRegistryError> {
        Self::fetch_url_content(settings_url).await
    }

    async fn fetch_orders(
        order_urls: &HashMap<String, Url>,
    ) -> Result<HashMap<String, String>, RaindexRegistryError> {
        use futures::future::join_all;

        let mut futures = Vec::new();

        for (key, url) in order_urls {
            let key_clone = key.clone();
            let url_clone = url.clone();
            futures.push(async move {
                let content = Self::fetch_url_content(&url_clone).await?;
                Ok::<(String, String), RaindexRegistryError>((key_clone, content))
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

#[cfg(not(target_family = "wasm"))]
impl RaindexRegistry {
    /// Creates a RaindexClient instance from the registry's shared settings.
    pub async fn get_raindex_client(
        &self,
        db_path: Option<std::path::PathBuf>,
    ) -> Result<crate::raindex_client::RaindexClient, RaindexRegistryError> {
        let client =
            crate::raindex_client::RaindexClient::new(vec![self.settings.clone()], None, db_path)
                .await?;
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raindex_app_settings::spec_version::SpecVersion;
    use std::collections::HashMap;

    const MOCK_REGISTRY_CONTENT: &str = r#"https://example.com/settings.yaml
fixed-limit https://example.com/fixed-limit.rain
auction-dca https://example.com/auction-dca.rain"#;

    fn base64_data_uri(media_type: &str, content: &str) -> String {
        format!(
            "data:{};base64,{}",
            media_type,
            BASE64_STANDARD.encode(content)
        )
    }

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
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/raindex-flare/0.8/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/raindex-base/0.9/gn
metaboards:
  flare: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn
rainlangs:
  flare:
    address: 0x1111111111111111111111111111111111111111
    network: flare
  base:
    address: 0x2222222222222222222222222222222222222222
    network: base
raindexes:
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
registrys:
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
builder:"#,
            version = SpecVersion::current()
        )
    }

    const MOCK_DOTRAIN_BODY: &str = r#"
  name: Test builder
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
    rainlang: flare
    raindex: flare
    inputs:
      - token: token1
    outputs:
      - token: token1
  base:
    rainlang: base
    raindex: base
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
#test-binding !
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
#test-binding !
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

    #[test]
    fn test_parse_registry_content() {
        let (settings_url, order_urls) =
            RaindexRegistry::parse_registry_content(MOCK_REGISTRY_CONTENT).unwrap();

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

    #[test]
    fn test_parse_invalid_registry_content() {
        let result = RaindexRegistry::parse_registry_content("");
        match result.unwrap_err() {
            RaindexRegistryError::InvalidRegistryFormat(msg) => {
                assert_eq!(msg, "Registry file is empty");
            }
            other => panic!("Expected InvalidRegistryFormat, got {other:?}"),
        }

        let result = RaindexRegistry::parse_registry_content("invalid first line");
        match result.unwrap_err() {
            RaindexRegistryError::InvalidRegistryFormat(msg) => {
                assert_eq!(msg, "First line should be a settings URL without a key");
            }
            other => panic!("Expected InvalidRegistryFormat, got {other:?}"),
        }

        let result = RaindexRegistry::parse_registry_content(
            "https://example.com/settings.yaml\ninvalid-entry",
        );
        match result.unwrap_err() {
            RaindexRegistryError::InvalidRegistryFormat(msg) => {
                assert_eq!(
                    msg,
                    "Invalid order entry format: 'invalid-entry'. Expected: 'key url'"
                );
            }
            other => panic!("Expected InvalidRegistryFormat, got {other:?}"),
        }

        let result = RaindexRegistry::parse_registry_content(
            "https://example.com/settings.yaml\nkey url extra",
        );
        match result.unwrap_err() {
            RaindexRegistryError::InvalidRegistryFormat(msg) => {
                assert_eq!(
                    msg,
                    "Invalid order entry format: 'key url extra'. Expected: 'key url'"
                );
            }
            other => panic!("Expected InvalidRegistryFormat, got {other:?}"),
        }
    }

    #[test]
    fn test_get_order_keys() {
        let registry = RaindexRegistry {
            registry_url: Url::parse("https://example.com/test").unwrap(),
            registry: "".to_string(),
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

        let keys = registry.get_order_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"fixed-limit".to_string()));
        assert!(keys.contains(&"auction-dca".to_string()));
    }

    #[test]
    fn test_get_all_order_details() {
        let registry = RaindexRegistry {
            registry_url: Url::parse("https://example.com/test").unwrap(),
            registry: "".to_string(),
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

        let order_details = registry.get_all_order_details();
        assert_eq!(order_details.valid.len(), 2);
        assert!(order_details.invalid.is_empty());
        assert!(order_details.valid.contains_key("fixed-limit"));
        assert!(order_details.valid.contains_key("auction-dca"));

        let fixed_limit_details = order_details.valid.get("fixed-limit").unwrap();
        assert_eq!(fixed_limit_details.name, "Test builder");
        assert_eq!(fixed_limit_details.description, "Test description");
    }

    #[test]
    fn test_get_all_order_details_with_invalid_order() {
        let registry = RaindexRegistry {
            registry_url: Url::parse("https://example.com/test").unwrap(),
            registry: "".to_string(),
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

        let result = registry.get_all_order_details();
        assert_eq!(result.valid.len(), 1);
        assert_eq!(result.invalid.len(), 1);
        assert!(result.valid.contains_key("valid"));
        assert!(result.invalid.contains_key("invalid"));
    }

    #[test]
    fn test_get_deployment_details() {
        let registry = RaindexRegistry {
            registry_url: Url::parse("https://example.com/test").unwrap(),
            registry: "".to_string(),
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

    #[test]
    fn test_get_deployment_details_order_not_found() {
        let registry = RaindexRegistry {
            registry_url: Url::parse("https://example.com/test").unwrap(),
            registry: "".to_string(),
            settings_url: Url::parse("https://example.com/settings.yaml").unwrap(),
            settings: mock_settings_content(),
            order_urls: HashMap::new(),
            orders: HashMap::new(),
        };

        let result = registry.get_deployment_details("non-existent".to_string());
        assert!(result.is_err());

        match result.err().unwrap() {
            RaindexRegistryError::OrderKeyNotFound(key) => {
                assert_eq!(key, "non-existent");
            }
            _ => panic!("Expected OrderKeyNotFound error"),
        }
    }

    #[test]
    fn test_getter_methods() {
        let registry = RaindexRegistry {
            registry_url: Url::parse("https://example.com/registry.txt").unwrap(),
            registry: MOCK_REGISTRY_CONTENT.to_string(),
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

        assert_eq!(registry.registry_url(), "https://example.com/registry.txt");
        assert_eq!(registry.settings_url(), "https://example.com/settings.yaml");
        assert_eq!(registry.registry(), MOCK_REGISTRY_CONTENT);
        assert_eq!(registry.settings(), mock_settings_content());

        let order_urls_map = registry.order_urls();
        assert_eq!(order_urls_map.len(), 1);

        let orders_map = registry.orders();
        assert_eq!(orders_map.len(), 1);
    }

    #[test]
    fn test_error_readable_messages() {
        let registry_error =
            RaindexRegistryError::RegistryFetchError("https://example.com".to_string());
        let readable = registry_error.to_readable_msg();
        assert!(readable.contains("Unable to fetch the registry file"));
        assert!(readable.contains("https://example.com"));

        let parse_error = RaindexRegistryError::RegistryParseError;
        let readable = parse_error.to_readable_msg();
        assert!(readable.contains("registry file format is invalid"));

        let not_found_error = RaindexRegistryError::OrderKeyNotFound("test-order".to_string());
        let readable = not_found_error.to_readable_msg();
        assert!(readable.contains("order key 'test-order' was not found"));

        let settings_fetch_error =
            RaindexRegistryError::SettingsFetchError("https://settings.example".to_string());
        let readable = settings_fetch_error.to_readable_msg();
        assert!(readable.contains("Unable to fetch the settings file"));
        assert!(readable.contains("https://settings.example"));

        let order_fetch_error =
            RaindexRegistryError::OrderFetchError("https://order.example".to_string());
        let readable = order_fetch_error.to_readable_msg();
        assert!(readable.contains("Unable to fetch the order file"));
        assert!(readable.contains("https://order.example"));

        let invalid_format_error =
            RaindexRegistryError::InvalidRegistryFormat("bad entry".to_string());
        let readable = invalid_format_error.to_readable_msg();
        assert_eq!(readable, "Invalid registry format: bad entry");

        let http_error = RaindexRegistryError::HttpError("HTTP 500".to_string());
        let readable = http_error.to_readable_msg();
        assert_eq!(readable, "Network error: HTTP 500");

        let data_uri_error =
            RaindexRegistryError::DataUriError("invalid base64 payload".to_string());
        let readable = data_uri_error.to_readable_msg();
        assert_eq!(readable, "Invalid data URI: invalid base64 payload");

        let url_parse_error = RaindexRegistryError::from(Url::parse("not a url").unwrap_err());
        let readable = url_parse_error.to_readable_msg();
        assert!(readable.contains("Invalid URL format"));
        assert!(readable.contains("Please ensure the URL is properly formatted"));
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

            let registry = RaindexRegistry::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            assert_eq!(
                registry.registry_url(),
                format!("{}/registry.txt", server.url(""))
            );
            assert_eq!(registry.registry(), test_registry_content);
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
        async fn test_fetch_and_parse_registry() {
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

            let registry_url = Url::parse(&format!("{}/registry.txt", server.url(""))).unwrap();
            let (registry_content, settings_url, order_urls) =
                RaindexRegistry::fetch_and_parse_registry(&registry_url)
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
                RaindexRegistry::validate(format!("{}/registry.txt", server.url(""))).await;
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
                RaindexRegistry::validate(format!("{}/invalid-registry.txt", server.url(""))).await;
            match result.unwrap_err() {
                RaindexRegistryError::InvalidRegistryFormat(msg) => {
                    assert_eq!(msg, "First line should be a settings URL without a key");
                }
                other => panic!("Expected InvalidRegistryFormat, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_fetch_url_content_success() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/test.txt");
                then.status(200).body("test content");
            });

            let url = Url::parse(&format!("{}/test.txt", server.url(""))).unwrap();
            let content = RaindexRegistry::fetch_url_content(&url).await.unwrap();

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
            let result = RaindexRegistry::fetch_url_content(&url).await;

            assert!(result.is_err());
            match result.err().unwrap() {
                RaindexRegistryError::HttpError(msg) => {
                    assert!(msg.contains("HTTP 500"));
                }
                _ => panic!("Expected HttpError"),
            }
        }

        #[tokio::test]
        async fn test_fetch_url_content_base64_data_uri() {
            let url = Url::parse(&base64_data_uri("text/plain", "test content")).unwrap();
            let content = RaindexRegistry::fetch_url_content(&url).await.unwrap();

            assert_eq!(content, "test content");
        }

        #[tokio::test]
        async fn test_fetch_url_content_malformed_data_uri() {
            let url = Url::parse("data:text/plain;base64,not-valid-base64").unwrap();
            let result = RaindexRegistry::fetch_url_content(&url).await;

            assert!(result.is_err());
            match result.err().unwrap() {
                RaindexRegistryError::DataUriError(msg) => {
                    assert!(msg.contains("invalid base64 payload"));
                    assert!(!msg.contains("not-valid-base64"));
                }
                _ => panic!("Expected DataUriError"),
            }
        }

        #[tokio::test]
        async fn test_fetch_url_content_rejects_non_base64_data_uri() {
            let url = Url::parse("data:text/plain,test%20content").unwrap();
            let result = RaindexRegistry::fetch_url_content(&url).await;

            assert!(result.is_err());
            match result.err().unwrap() {
                RaindexRegistryError::DataUriError(msg) => {
                    assert!(msg.contains("payload must be base64 encoded"));
                }
                _ => panic!("Expected DataUriError"),
            }
        }

        #[tokio::test]
        async fn test_fetch_url_content_rejects_data_uri_without_separator() {
            let url = Url::parse("data:text/plain").unwrap();
            let result = RaindexRegistry::fetch_url_content(&url).await;

            match result.unwrap_err() {
                RaindexRegistryError::DataUriError(msg) => {
                    assert_eq!(msg, "missing metadata/payload separator");
                }
                other => panic!("Expected DataUriError, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn test_fetch_url_content_rejects_data_uri_with_non_utf8_payload() {
            // 0xff 0xfe is valid base64 but not valid UTF-8 once decoded.
            let url = Url::parse(&format!(
                "data:text/plain;base64,{}",
                BASE64_STANDARD.encode([0xff, 0xfe])
            ))
            .unwrap();
            let result = RaindexRegistry::fetch_url_content(&url).await;

            match result.unwrap_err() {
                RaindexRegistryError::DataUriError(msg) => {
                    assert_eq!(msg, "decoded payload is not valid UTF-8");
                }
                other => panic!("Expected DataUriError, got {other:?}"),
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
            let settings = RaindexRegistry::fetch_settings(&url).await.unwrap();

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

            let orders = RaindexRegistry::fetch_orders(&order_urls).await.unwrap();

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
        async fn test_new_constructor_with_data_uri_registry_and_settings() {
            let server = MockServer::start_async().await;

            server.mock(|when, then| {
                when.method("GET").path("/order.rain");
                then.status(200).body(get_first_dotrain_content());
            });

            let settings_uri = base64_data_uri("application/yaml", &mock_settings_content());
            let test_registry_content = format!(
                "{}\nfirst-order {}/order.rain",
                settings_uri,
                server.url("")
            );
            let registry_uri = base64_data_uri("text/plain", &test_registry_content);

            let registry = RaindexRegistry::new(registry_uri).await.unwrap();

            assert_eq!(registry.registry(), test_registry_content);
            assert_eq!(registry.settings(), mock_settings_content());
            assert_eq!(registry.order_urls.len(), 1);
            assert_eq!(registry.orders.len(), 1);
            assert!(registry.order_urls.contains_key("first-order"));
            assert_eq!(
                registry.orders.get("first-order").unwrap(),
                &get_first_dotrain_content()
            );
        }

        #[tokio::test]
        async fn test_fetch_orders_supports_data_uri_order_content() {
            let order_urls: HashMap<String, Url> = vec![(
                "order1".to_string(),
                Url::parse(&base64_data_uri("text/plain", &get_first_dotrain_content())).unwrap(),
            )]
            .into_iter()
            .collect();

            let orders = RaindexRegistry::fetch_orders(&order_urls).await.unwrap();

            assert_eq!(orders.len(), 1);
            assert_eq!(orders.get("order1").unwrap(), &get_first_dotrain_content());
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
                RaindexRegistry::new(format!("{}/invalid-registry.txt", server.url(""))).await;
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
                RaindexRegistry::new(format!("{}/empty-registry.txt", server.url(""))).await;
            assert!(result.is_err());

            match result.err().unwrap() {
                RaindexRegistryError::InvalidRegistryFormat(msg) => {
                    assert!(msg.contains("Registry file is empty"));
                }
                _ => panic!("Expected InvalidRegistryFormat error for empty registry"),
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
                RaindexRegistry::new(format!("{}/settings-only-registry.txt", server.url("")))
                    .await
                    .unwrap();

            assert_eq!(
                registry.settings_url(),
                format!("{}/settings.yaml", server.url(""))
            );
            assert_eq!(registry.settings(), mock_settings_content());
            assert_eq!(registry.order_urls.len(), 0);
            assert_eq!(registry.orders.len(), 0);

            let keys = registry.get_order_keys();
            assert!(keys.is_empty());
        }

        #[tokio::test]
        async fn test_get_order_builder() {
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

            let registry = RaindexRegistry::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            assert_eq!(registry.order_urls.len(), 2);
            assert!(registry.order_urls.contains_key("first-order"));
            assert!(registry.order_urls.contains_key("second-order"));
            assert_eq!(registry.orders.len(), 2);

            let builder1 = registry
                .get_order_builder("first-order".to_string(), "flare".to_string(), None)
                .await
                .unwrap();

            let deployment_details1 = builder1.get_current_deployment().unwrap();
            assert_eq!(deployment_details1.name, "Flare order name");
            assert_eq!(deployment_details1.description, "Flare order description");

            let default_serialized_state = builder1.serialize_state().unwrap();

            let builder2 = registry
                .get_order_builder("second-order".to_string(), "base".to_string(), None)
                .await
                .unwrap();

            let deployment_details2 = builder2.get_current_deployment().unwrap();
            assert_eq!(deployment_details2.name, "Base order name");
            assert_eq!(deployment_details2.description, "Base order description");

            let mut builder_with_state = registry
                .get_order_builder("first-order".to_string(), "flare".to_string(), None)
                .await
                .unwrap();
            builder_with_state
                .set_field_value("test-binding".to_string(), "42".to_string())
                .unwrap();
            let saved_state = builder_with_state.serialize_state().unwrap();

            let restored_builder = registry
                .get_order_builder(
                    "first-order".to_string(),
                    "flare".to_string(),
                    Some(saved_state.clone()),
                )
                .await
                .unwrap();
            assert_eq!(restored_builder.serialize_state().unwrap(), saved_state);

            let fallback_builder = registry
                .get_order_builder(
                    "first-order".to_string(),
                    "flare".to_string(),
                    Some("not-a-valid-state".to_string()),
                )
                .await
                .unwrap();
            assert_eq!(
                fallback_builder.serialize_state().unwrap(),
                default_serialized_state
            );

            let result = registry
                .get_order_builder("non-existent-order".to_string(), "flare".to_string(), None)
                .await;
            assert!(result.is_err());
            match result.err().unwrap() {
                RaindexRegistryError::OrderKeyNotFound(key) => {
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
rainlangs:
  mainnet:
    address: 0x1111111111111111111111111111111111111111
    network: mainnet
raindexes:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
    network: mainnet
registrys:
  mainnet:
    address: 0x1234567890123456789012345678901234567890
    network: mainnet
"#,
                SpecVersion::current()
            )
        }

        const MOCK_DOTRAIN_SIMPLE: &str = r#"builder:
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
    rainlang: mainnet
    raindex: mainnet
    inputs:
      - token: weth
    outputs:
      - token: usdc
deployments:
  mainnet:
    scenario: mainnet
    order: mainnet
---
#test-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
"#;

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

            let registry = RaindexRegistry::new(format!("{}/registry.txt", server.url("")))
                .await
                .unwrap();

            let raindex_client = registry.get_raindex_client(None).await;
            assert!(raindex_client.is_ok());
        }
    }
}
