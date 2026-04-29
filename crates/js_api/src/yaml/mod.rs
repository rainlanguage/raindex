use std::str::FromStr;

use alloy::{hex::FromHexError, primitives::Address};
use raindex_app_settings::{
    raindex::RaindexCfg,
    remote_tokens::{ParseRemoteTokensError, RemoteTokensCfg},
    spec_version::CURRENT_SPEC_VERSION,
    yaml::{
        raindex::{RaindexYaml as RaindexYamlCfg, RaindexYamlValidation},
        YamlError, YamlParsable,
    },
};
use raindex_common::erc20::ExtendedTokenInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct RaindexYaml {
    yaml: RaindexYamlCfg,
}

#[wasm_export]
impl RaindexYaml {
    /// Creates a new RaindexYaml instance from YAML configuration sources.
    ///
    /// This constructor parses one or more YAML configuration strings to create an RaindexYaml
    /// instance that provides access to raindex configurations, network settings, tokens, and
    /// other deployment metadata. The YAML sources are merged and validated according to the
    /// [raindex specification](https://github.com/rainlanguage/specs/blob/main/raindex-yaml.md).
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Basic usage with single YAML source
    /// const yamlConfig = `
    /// version: "6"
    /// networks:
    ///   mainnet:
    ///     rpc: https://mainnet.infura.io
    ///     chain-id: 1
    /// raindexes:
    ///   my-raindex:
    ///     address: 0x1234567890abcdef1234567890abcdef12345678
    ///     network: mainnet
    /// ...
    /// `;
    ///
    /// const result = RaindexYaml.new([yamlConfig], false);
    /// if (result.error) {
    ///   console.error("Configuration error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const raindexYaml = result.value;
    /// // Do something with the raindexYaml
    /// ```
    #[wasm_export(
        js_name = "new",
        preserve_js_class,
        return_description = "Successfully parsed and configured instance"
    )]
    pub fn new(
        #[wasm_export(
            param_description = "Vector of YAML configuration strings to parse and merge"
        )]
        sources: Vec<String>,
        #[wasm_export(
            param_description = "Optional boolean to enable strict validation (defaults to false)"
        )]
        validate: Option<bool>,
    ) -> Result<RaindexYaml, RaindexYamlError> {
        let yaml = RaindexYamlCfg::new(
            sources,
            match validate {
                Some(true) => RaindexYamlValidation::full(),
                _ => RaindexYamlValidation::default(),
            },
        )?;
        Ok(Self { yaml })
    }

    #[wasm_export(
        js_name = "getCurrentSpecVersion",
        return_description = "Current spec version"
    )]
    pub fn get_current_spec_version() -> Result<String, RaindexYamlError> {
        Ok(CURRENT_SPEC_VERSION.to_string())
    }

    /// Retrieves raindex configuration by its contract address from a parsed YAML configuration.
    ///
    /// This function looks up a specific raindex configuration within a YAML configuration file
    /// using the raindex's blockchain address. It's essential for accessing raindex metadata
    /// including network configuration, subgraph endpoints, and other deployment details.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Basic usage
    /// const result = raindexYaml.getRaindexByAddress("0x1234567890abcdef1234567890abcdef12345678");
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const raindex = result.value;
    /// // Do something with the raindex
    /// ```
    #[wasm_export(
        js_name = "getRaindexByAddress",
        unchecked_return_type = "RaindexCfg",
        return_description = "Complete raindex configuration"
    )]
    pub fn get_raindex_by_address(
        &self,
        #[wasm_export(param_description = "The hexadecimal address of the raindex contract")]
        raindex_address: &str,
    ) -> Result<RaindexCfg, RaindexYamlError> {
        let address = Address::from_str(raindex_address).map_err(RaindexYamlError::FromHexError)?;
        Ok(self.yaml.get_raindex_by_address(address)?)
    }

    /// Retrieves all tokens from the YAML configuration, including remote tokens.
    ///
    /// This async function fetches all tokens defined in the YAML configuration.
    /// If `using-tokens-from` URLs are configured, it will fetch and merge remote
    /// tokens from those URLs before returning the complete token list.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await raindexYaml.getTokens();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const tokens = result.value;
    /// // Each token has: key, address, decimals, name, symbol, chainId
    /// tokens.forEach(token => {
    ///   console.log(`${token.symbol} on chain ${token.chainId}`);
    /// });
    /// ```
    #[wasm_export(
        js_name = "getTokens",
        unchecked_return_type = "ExtendedTokenInfo[]",
        return_description = "Array of token information"
    )]
    pub async fn get_tokens(&mut self) -> Result<Vec<ExtendedTokenInfo>, RaindexYamlError> {
        if let Some(remote_tokens_cfg) = self.yaml.get_remote_tokens()? {
            let networks = self.yaml.get_networks()?;
            let remote_tokens = RemoteTokensCfg::fetch_tokens(&networks, remote_tokens_cfg).await?;
            self.yaml.cache.update_remote_tokens(remote_tokens);
        }

        let tokens = self.yaml.get_tokens()?;

        let mut token_infos: Vec<ExtendedTokenInfo> = Vec::new();
        for token in tokens.values() {
            token_infos.push(ExtendedTokenInfo::from_token_cfg(token).await?);
        }

        Ok(token_infos)
    }
}

#[derive(Error, Debug)]
pub enum RaindexYamlError {
    #[error("Raindex yaml error: {0}")]
    YamlError(#[from] YamlError),
    #[error("Invalid address: {0}")]
    FromHexError(#[from] FromHexError),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error(transparent)]
    ParseRemoteTokensError(#[from] ParseRemoteTokensError),
    #[error(transparent)]
    ERC20Error(#[from] raindex_common::erc20::Error),
}

impl RaindexYamlError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            RaindexYamlError::YamlError(err) =>
                format!("There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: \"{}\"", err),
            RaindexYamlError::FromHexError(err) =>
                format!("The provided address is invalid. Please ensure the address is in the correct hexadecimal format. Error: \"{}\"", err),
            RaindexYamlError::MissingField(field) =>
                format!("A required field is missing from the token configuration: \"{}\". Please ensure all tokens have decimals, label, and symbol defined.", field),
            RaindexYamlError::ParseRemoteTokensError(err) =>
                format!("Failed to fetch or parse remote tokens. Please check the using-tokens-from URLs are accessible and return valid token data. Error: \"{}\"", err),
            RaindexYamlError::ERC20Error(err) =>
                format!("Failed to fetch token information from the blockchain. Please check your network connection and RPC settings. Error: \"{}\"", err),
        }
    }
}

impl From<RaindexYamlError> for JsValue {
    fn from(value: RaindexYamlError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}
impl From<RaindexYamlError> for WasmEncodedError {
    fn from(value: RaindexYamlError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use raindex_app_settings::spec_version::SpecVersion;
    use wasm_bindgen_test::wasm_bindgen_test;

    pub fn get_yaml() -> String {
        format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
        secondary: https://api.thegraph.com/subgraphs/name/abc
    metaboards:
        board1: https://meta.example.com/board1
        board2: https://meta.example.com/board2
    raindexes:
        raindex1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
            subgraph: mainnet
            local-db-remote: remote
            label: Primary Raindex
            deployment-block: 12345
    tokens:
        token1:
            network: mainnet
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
    rainlangs:
        deployer1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
    accounts:
        admin: 0x4567890123abcdef
        user: 0x5678901234abcdef
    sentry: true
    "#,
            spec_version = SpecVersion::current()
        )
    }

    #[wasm_bindgen_test]
    fn test_get_current_spec_version() {
        let version = RaindexYaml::get_current_spec_version().unwrap();
        assert_eq!(version, SpecVersion::current().to_string());
    }

    #[wasm_bindgen_test]
    fn test_raindex_yaml() {
        let raindex_yaml = RaindexYaml::new(vec![get_yaml()], None).unwrap();
        let raindex = raindex_yaml
            .get_raindex_by_address("0x0000000000000000000000000000000000000002")
            .unwrap();

        assert_eq!(
            raindex.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(raindex.key, "raindex1");
        assert_eq!(raindex.network.key, "mainnet");
        assert_eq!(raindex.subgraph.key, "mainnet");
        assert_eq!(raindex.label, Some("Primary Raindex".to_string()));
    }

    #[wasm_bindgen_test]
    fn test_raindex_yaml_error() {
        let raindex_yaml = RaindexYaml::new(vec![get_yaml()], None).unwrap();
        let raindex = raindex_yaml.get_raindex_by_address("invalid-address");

        assert!(raindex.is_err());
        assert_eq!(
            raindex.as_ref().err().unwrap().to_string(),
            "Invalid address: odd number of digits"
        );
        assert_eq!(
            raindex.as_ref().err().unwrap().to_readable_msg(),
            "The provided address is invalid. Please ensure the address is in the correct hexadecimal format. Error: \"odd number of digits\""
        );

        let raindex =
            raindex_yaml.get_raindex_by_address("0x0000000000000000000000000000000000000000");
        assert!(raindex.is_err());
        assert_eq!(
            raindex.as_ref().err().unwrap().to_string(),
            "Raindex yaml error: raindex with address: 0x0000000000000000000000000000000000000000 not found"
        );
        assert_eq!(
            raindex.as_ref().err().unwrap().to_readable_msg(),
            "There was an error processing the YAML configuration. Please check the YAML file for any issues. Error: \"raindex with address: 0x0000000000000000000000000000000000000000 not found\""
        );
    }

    pub fn get_invalid_yaml() -> String {
        format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpc: https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
    raindexes:
        raindex1:
            address: 0x0000000000000000000000000000000000000002
            network: nonexistent-network
            subgraph: nonexistent-subgraph
            label: Primary Raindex
            deployment-block: 12345
    "#,
            spec_version = SpecVersion::current()
        )
    }

    #[wasm_bindgen_test]
    fn test_raindex_yaml_invalid_with_validation_enabled() {
        let result = RaindexYaml::new(vec![get_invalid_yaml()], Some(true));
        match result {
            Ok(_) => panic!("Expected validation error with invalid YAML"),
            Err(err) => {
                assert!(err.to_string().contains("Raindex yaml error"));
                assert!(err
                    .to_readable_msg()
                    .contains("There was an error processing the YAML configuration"));
            }
        }
    }

    #[wasm_bindgen_test]
    async fn test_get_tokens_local_only() {
        let mut raindex_yaml = RaindexYaml::new(vec![get_yaml()], None).unwrap();
        let tokens = raindex_yaml.get_tokens().await.unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].key, "token1");
        assert_eq!(tokens[0].chain_id, 1);
        assert_eq!(tokens[0].decimals, 18);
        assert_eq!(tokens[0].symbol, "WETH");
        assert_eq!(tokens[0].name, "Wrapped Ether");
        assert_eq!(
            tokens[0].address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
    }

    pub fn get_yaml_multiple_networks() -> String {
        format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
        polygon:
            rpcs:
                - https://polygon-rpc.com
            chain-id: 137
            label: Polygon
            network-id: 137
            currency: MATIC
    tokens:
        weth:
            network: mainnet
            address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
        usdc-polygon:
            network: polygon
            address: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
            decimals: 6
            label: USD Coin (PoS)
            symbol: USDC
    "#,
            spec_version = SpecVersion::current()
        )
    }

    #[wasm_bindgen_test]
    async fn test_get_tokens_multiple_networks() {
        let mut raindex_yaml = RaindexYaml::new(vec![get_yaml_multiple_networks()], None).unwrap();
        let tokens = raindex_yaml.get_tokens().await.unwrap();

        assert_eq!(tokens.len(), 2);

        let mainnet_token = tokens.iter().find(|t| t.chain_id == 1);
        assert!(mainnet_token.is_some());
        let mainnet_token = mainnet_token.unwrap();
        assert_eq!(mainnet_token.symbol, "WETH");
        assert_eq!(mainnet_token.decimals, 18);

        let polygon_token = tokens.iter().find(|t| t.chain_id == 137);
        assert!(polygon_token.is_some());
        let polygon_token = polygon_token.unwrap();
        assert_eq!(polygon_token.symbol, "USDC");
        assert_eq!(polygon_token.decimals, 6);
    }

    pub fn get_yaml_missing_fields() -> String {
        format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    tokens:
        incomplete:
            network: mainnet
            address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
    "#,
            spec_version = SpecVersion::current()
        )
    }

    #[wasm_bindgen_test]
    async fn test_get_tokens_missing_fields_tries_rpc() {
        let mut raindex_yaml = RaindexYaml::new(vec![get_yaml_missing_fields()], None).unwrap();
        let result = raindex_yaml.get_tokens().await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, RaindexYamlError::ERC20Error(_)),
            "Expected ERC20Error when trying to fetch missing token info from RPC, got: {:?}",
            err
        );
        assert!(err
            .to_readable_msg()
            .contains("Failed to fetch token information"));
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod non_wasm_tests {
    use super::*;
    use httpmock::MockServer;
    use raindex_app_settings::spec_version::SpecVersion;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_tokens_from_remote() {
        let server = MockServer::start_async().await;

        let remote_response = json!({
            "name": "Remote Tokens",
            "timestamp": "2021-01-01T00:00:00.000Z",
            "version": { "major": 1, "minor": 0, "patch": 0 },
            "keywords": [],
            "logoURI": "",
            "tokens": [
                {
                    "chainId": 1,
                    "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    "name": "USD Coin",
                    "symbol": "USDC",
                    "decimals": 6
                }
            ]
        });

        server.mock(|when, then| {
            when.method("GET").path("/tokens.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(remote_response.to_string());
        });

        let yaml = format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    using-tokens-from:
        - {url}/tokens.json
    "#,
            spec_version = SpecVersion::current(),
            url = server.base_url()
        );

        let mut raindex_yaml = RaindexYaml::new(vec![yaml], None).unwrap();
        let tokens = raindex_yaml.get_tokens().await.unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].symbol, "USDC");
        assert_eq!(tokens[0].chain_id, 1);
        assert_eq!(tokens[0].decimals, 6);
        assert_eq!(tokens[0].name, "USD Coin");
    }

    #[tokio::test]
    async fn test_get_tokens_mixed_local_and_remote() {
        let server = MockServer::start_async().await;

        let remote_response = json!({
            "name": "Remote",
            "timestamp": "2021-01-01T00:00:00.000Z",
            "version": { "major": 1, "minor": 0, "patch": 0 },
            "keywords": [],
            "logoURI": "",
            "tokens": [
                {
                    "chainId": 1,
                    "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    "name": "USD Coin",
                    "symbol": "USDC",
                    "decimals": 6
                }
            ]
        });

        server.mock(|when, then| {
            when.method("GET").path("/tokens.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(remote_response.to_string());
        });

        let yaml = format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    using-tokens-from:
        - {url}/tokens.json
    tokens:
        weth:
            network: mainnet
            address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
    "#,
            spec_version = SpecVersion::current(),
            url = server.base_url()
        );

        let mut raindex_yaml = RaindexYaml::new(vec![yaml], None).unwrap();
        let tokens = raindex_yaml.get_tokens().await.unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(tokens.iter().any(|t| t.symbol == "WETH"));
        assert!(tokens.iter().any(|t| t.symbol == "USDC"));
    }

    #[tokio::test]
    async fn test_get_tokens_multiple_networks_remote() {
        let server = MockServer::start_async().await;

        let remote_response = json!({
            "name": "Multi-chain",
            "timestamp": "2021-01-01T00:00:00.000Z",
            "version": { "major": 1, "minor": 0, "patch": 0 },
            "keywords": [],
            "logoURI": "",
            "tokens": [
                { "chainId": 1, "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "name": "USDC", "symbol": "USDC", "decimals": 6 },
                { "chainId": 137, "address": "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", "name": "USDC PoS", "symbol": "USDC", "decimals": 6 }
            ]
        });

        server.mock(|when, then| {
            when.method("GET").path("/tokens.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(remote_response.to_string());
        });

        let yaml = format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
        polygon:
            rpcs:
                - https://polygon-rpc.com
            chain-id: 137
    using-tokens-from:
        - {url}/tokens.json
    "#,
            spec_version = SpecVersion::current(),
            url = server.base_url()
        );

        let mut raindex_yaml = RaindexYaml::new(vec![yaml], None).unwrap();
        let tokens = raindex_yaml.get_tokens().await.unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(tokens.iter().any(|t| t.chain_id == 1));
        assert!(tokens.iter().any(|t| t.chain_id == 137));
    }

    #[tokio::test]
    async fn test_get_tokens_remote_fetch_failure() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("GET").path("/tokens.json");
            then.status(500).body("Internal Server Error");
        });

        let yaml = format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    using-tokens-from:
        - {url}/tokens.json
    "#,
            spec_version = SpecVersion::current(),
            url = server.base_url()
        );

        let mut raindex_yaml = RaindexYaml::new(vec![yaml], None).unwrap();
        let result = raindex_yaml.get_tokens().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_tokens_multiple_remote_urls() {
        let server = MockServer::start_async().await;

        let response1 = json!({
            "name": "Source 1",
            "timestamp": "2021-01-01T00:00:00.000Z",
            "version": { "major": 1, "minor": 0, "patch": 0 },
            "keywords": [],
            "logoURI": "",
            "tokens": [
                { "chainId": 1, "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "name": "USDC", "symbol": "USDC", "decimals": 6 }
            ]
        });
        let response2 = json!({
            "name": "Source 2",
            "timestamp": "2021-01-01T00:00:00.000Z",
            "version": { "major": 1, "minor": 0, "patch": 0 },
            "keywords": [],
            "logoURI": "",
            "tokens": [
                { "chainId": 1, "address": "0xdAC17F958D2ee523a2206206994597C13D831ec7", "name": "USDT", "symbol": "USDT", "decimals": 6 }
            ]
        });

        server.mock(|when, then| {
            when.method("GET").path("/tokens1.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(response1.to_string());
        });
        server.mock(|when, then| {
            when.method("GET").path("/tokens2.json");
            then.status(200)
                .header("content-type", "application/json")
                .body(response2.to_string());
        });

        let yaml = format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    using-tokens-from:
        - {url}/tokens1.json
        - {url}/tokens2.json
    "#,
            spec_version = SpecVersion::current(),
            url = server.base_url()
        );

        let mut raindex_yaml = RaindexYaml::new(vec![yaml], None).unwrap();
        let tokens = raindex_yaml.get_tokens().await.unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(tokens.iter().any(|t| t.symbol == "USDC"));
        assert!(tokens.iter().any(|t| t.symbol == "USDT"));
    }
}
