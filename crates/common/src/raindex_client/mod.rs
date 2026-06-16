use crate::local_db::{query::LocalDbQueryError, LocalDbError};
use crate::raindex_client::local_db::{pipeline::runner::scheduler::SchedulerHandle, LocalDb};
use crate::{
    add_order::AddOrderArgsError, deposit::DepositError, dotrain_order::DotrainOrderError,
    meta::TryDecodeRainlangSourceError, transaction::WritableTransactionExecuteError,
    utils::amount_formatter::AmountFormatterError,
};
use alloy::{
    hex::FromHexError,
    primitives::{
        ruint::{FromUintError, ParseError},
        Address, ParseSignedError, B256,
    },
};
use rain_math_float::FloatError;
use rain_orderbook_app_settings::{
    network::NetworkCfg,
    yaml::{
        orderbook::{OrderbookYaml, OrderbookYamlValidation},
        YamlError, YamlParsable,
    },
};
use rain_orderbook_subgraph_client::{
    types::order_detail_traits::OrderDetailError, MultiSubgraphArgs, OrderbookSubgraphClient,
    OrderbookSubgraphClientError,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use std::{collections::BTreeMap, fmt, num::ParseIntError, str::FromStr};
use thiserror::Error;
use tsify::Tsify;
use url::Url;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, wasm_export};

pub mod add_orders;
pub mod local_db;
pub mod order_quotes;
pub mod orderbook_yaml;
pub mod orders;
pub mod remove_orders;
pub mod take_orders;
pub mod trades;
pub mod transactions;
pub mod types;
pub mod vaults;
pub mod vaults_list;

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct ChainIds(#[tsify(type = "number[]")] pub Vec<u32>);
impl_wasm_traits!(ChainIds);

/// RaindexClient provides a simplified interface for querying orderbook data across
/// multiple networks with automatic configuration management.
///
/// This client abstracts away complex network-specific configurations by parsing YAML
/// configuration files that define networks, tokens, orderbooks, and subgraph endpoints.
/// It enables querying orderbook data either from specific chains or across all
/// configured networks with automatic fallback mechanisms.
///
/// The client handles:
/// - YAML configuration parsing and validation
/// - Network-to-subgraph URL mapping
/// - Multi-network query coordination
/// - Chain ID resolution to network configurations
///
/// ## Examples
///
/// ```javascript
/// const result = await RaindexClient.new([yamlConfig]);
/// if (result.error) {
///   console.error('Failed to create client:', result.error.readableMsg);
/// } else {
///   const client = result.value;
///   // Query orders across all networks or specific chains
/// }
///
/// // Create client with multiple YAML files for modular configuration
/// const result = await RaindexClient.new([
///   yamlConfig1,
///   yamlConfig2,
///   yamlConfig3,
/// ]);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct RaindexClient {
    orderbook_yaml: OrderbookYaml,
    #[serde(skip_serializing, skip_deserializing)]
    local_db: Rc<RefCell<Option<LocalDb>>>,
    #[serde(skip_serializing, skip_deserializing)]
    local_db_scheduler: Rc<RefCell<Option<SchedulerHandle>>>,
}

#[wasm_export]
impl RaindexClient {
    /// Constructor that creates and returns RaindexClient instance directly
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Single YAML file
    /// const result = await RaindexClient.new([yamlConfig]);
    /// if (result.error) {
    ///   console.error("Init failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const client = result.value;
    ///
    /// // Multiple YAML files (for modular configuration)
    /// const result = await RaindexClient.new([networksYaml, orderbooksYaml, tokensYaml]);
    /// ```
    #[wasm_export(
        js_name = "new",
        return_description = "Initialized client instance for further operations",
        preserve_js_class
    )]
    pub fn new(
        #[wasm_export(
            js_name = "obYamls",
            param_description = "List of YAML configuration strings. \
            The YAML files must match the [orderbook yaml spec](https://github.com/rainlanguage/specs/blob/main/ob-yaml.md)
            "
        )]
        ob_yamls: Vec<String>,
        validate: Option<bool>,
    ) -> Result<RaindexClient, RaindexError> {
        let orderbook_yaml = OrderbookYaml::new(
            ob_yamls,
            match validate {
                Some(true) => OrderbookYamlValidation::full(),
                _ => OrderbookYamlValidation::default(),
            },
        )?;
        Ok(RaindexClient {
            orderbook_yaml,
            local_db: Rc::new(RefCell::new(None)),
            local_db_scheduler: Rc::new(RefCell::new(None)),
        })
    }

    #[wasm_export(js_name = "setDbCallback", unchecked_return_type = "void")]
    pub fn set_local_db_callback(
        &self,
        #[wasm_export(
            js_name = "queryCallback",
            param_description = "JavaScript function to execute local database queries"
        )]
        query_callback: js_sys::Function,
        #[wasm_export(
            js_name = "wipeCallback",
            param_description = "Optional JavaScript function to wipe and recreate the database"
        )]
        wipe_callback: Option<js_sys::Function>,
    ) -> Result<(), RaindexError> {
        let mut slot = self.local_db.borrow_mut();
        *slot = Some(LocalDb::from_js_callback(query_callback, wipe_callback));
        Ok(())
    }

    fn resolve_networks(
        &self,
        chain_ids: Option<Vec<u32>>,
    ) -> Result<Vec<NetworkCfg>, RaindexError> {
        match chain_ids {
            Some(ids) if !ids.is_empty() => {
                let mut networks = Vec::with_capacity(ids.len());
                for id in ids {
                    networks.push(self.orderbook_yaml.get_network_by_chain_id(id)?);
                }
                Ok(networks)
            }
            Some(_) | None => {
                let all_nets = self.orderbook_yaml.get_networks()?;
                let networks = all_nets.values().cloned().collect();
                Ok(networks)
            }
        }
    }

    fn get_multi_subgraph_args(
        &self,
        chain_ids: Option<Vec<u32>>,
    ) -> Result<BTreeMap<u32, Vec<MultiSubgraphArgs>>, RaindexError> {
        let networks = self.resolve_networks(chain_ids)?;
        let mut result: BTreeMap<u32, Vec<MultiSubgraphArgs>> = BTreeMap::new();
        for network in networks {
            let orderbooks = self
                .orderbook_yaml
                .get_orderbooks_by_network_key(&network.key)?;
            for orderbook in orderbooks {
                result
                    .entry(network.chain_id)
                    .or_default()
                    .push(MultiSubgraphArgs {
                        url: orderbook.subgraph.url.clone(),
                        name: network.label.clone().unwrap_or(network.key.clone()),
                    });
            }
        }

        if result.is_empty() {
            return Err(RaindexError::NoNetworksConfigured);
        }
        Ok(result)
    }

    #[wasm_export(skip)]
    pub fn get_orderbook_client(
        &self,
        orderbook_address: Address,
    ) -> Result<OrderbookSubgraphClient, RaindexError> {
        let orderbook = self
            .orderbook_yaml
            .get_orderbook_by_address(orderbook_address)?;
        Ok(OrderbookSubgraphClient::new(orderbook.subgraph.url.clone()))
    }

    fn get_rpc_urls_for_chain(&self, chain_id: u32) -> Result<Vec<Url>, RaindexError> {
        let network = self.orderbook_yaml.get_network_by_chain_id(chain_id)?;
        Ok(network.rpcs.clone())
    }

    fn local_db(&self) -> Option<LocalDb> {
        self.local_db.borrow().as_ref().cloned()
    }
}

#[derive(Debug, Clone)]
pub struct SerdeWasmBindgenErrorWrapper {
    message: String,
}

impl fmt::Display for SerdeWasmBindgenErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SerdeWasmBindgenErrorWrapper {}

impl From<serde_wasm_bindgen::Error> for SerdeWasmBindgenErrorWrapper {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum RaindexError {
    #[error("Invalid yaml configuration")]
    InvalidYamlConfig,
    #[error("Chain ID not found: {0}")]
    ChainIdNotFound(u32),
    #[error("No networks configured")]
    NoNetworksConfigured,
    #[error("Subgraph not configured for chain ID: {0}")]
    SubgraphNotConfigured(String),
    #[error("Transaction {tx_hash:#x} was not indexed after {attempts} attempts")]
    TransactionIndexingTimeout { tx_hash: B256, attempts: usize },
    #[error("Trades for transaction {tx_hash:#x} were not indexed after {attempts} attempts")]
    TradesIndexingTimeout { tx_hash: B256, attempts: usize },
    #[error(transparent)]
    YamlError(#[from] YamlError),
    #[error(transparent)]
    SerdeError(#[from] SerdeWasmBindgenErrorWrapper),
    #[error(transparent)]
    DotrainOrderError(Box<DotrainOrderError>),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    OrderbookSubgraphClientError(#[from] OrderbookSubgraphClientError),
    #[error(transparent)]
    TryDecodeRainlangSourceError(#[from] TryDecodeRainlangSourceError),
    #[error(transparent)]
    U256ParseError(#[from] ParseError),
    #[error(transparent)]
    I256ParseError(#[from] ParseSignedError),
    #[error("JavaScript error: {0}")]
    JsError(String),
    #[error("Failed to acquire read lock")]
    ReadLockError,
    #[error("Failed to acquire write lock")]
    WriteLockError,
    #[error("Zero amount")]
    ZeroAmount,
    #[error("Negative amount")]
    NegativeAmount,
    #[error("Existing allowance")]
    ExistingAllowance,
    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),
    #[error(transparent)]
    DepositArgsError(#[from] DepositError),
    #[error("Orderbook not found for address: {0} on chain ID: {1}")]
    OrderbookNotFound(String, u32),
    #[error("Order not found for address: {0} on chain ID: {1} with hash: {2}")]
    OrderNotFound(String, u32, B256),
    #[error("Vault not found for address: {0} on chain ID: {1} with id: {2}")]
    VaultNotFound(String, u32, String),
    #[error(transparent)]
    OrderDetailError(#[from] OrderDetailError),
    #[error(transparent)]
    AddOrderArgsError(#[from] AddOrderArgsError),
    #[error(transparent)]
    OrderbookQuoteError(#[from] rain_orderbook_quote::error::Error),
    #[error("Missing subgraph {0} for order {1}")]
    SubgraphNotFound(String, String),
    #[error("Invalid vault balance change type: {0}")]
    InvalidVaultBalanceChangeType(String),
    #[error(transparent)]
    Erc20(#[from] crate::erc20::Error),
    #[error("Float error: {0}")]
    Float(#[from] FloatError),
    #[error("Failed to parse an integer: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("Failed to convert to u8: {0}")]
    TryFromUint(#[from] FromUintError<u8>),
    #[error("Missing decimals for token {0}")]
    MissingErc20Decimals(String),
    #[error(transparent)]
    AmountFormatterError(#[from] AmountFormatterError),
    #[error(transparent)]
    LocalDbError(#[from] Box<LocalDbError>),
    #[error(transparent)]
    LocalDbQueryError(#[from] LocalDbQueryError),
    #[error("Chain id: {0} is not supported for local database")]
    LocalDbUnsupportedNetwork(u32),
    #[error("No liquidity available for the requested token pair")]
    NoLiquidity,
    #[error("Insufficient liquidity: requested {requested}, available {available}")]
    InsufficientLiquidity {
        requested: String,
        available: String,
    },
    #[error("Sell token and buy token cannot be the same")]
    SameTokenPair,
    #[error("Amount must be positive")]
    NonPositiveAmount,
    #[error("Price cap cannot be negative")]
    NegativePriceCap,
    #[error(transparent)]
    RpcClientError(#[from] crate::rpc_client::RpcClientError),
    #[error("Preflight check failed: {0}")]
    PreflightError(String),
    #[error("Quote data is missing")]
    QuoteDataMissing,
    #[error("Invalid input index: {0}")]
    InvalidInputIndex(u32),
    #[error("Cannot parse metadata: {0}")]
    ParseMetaError(#[from] rain_metadata::Error),
    #[error("No metaboards configured for any chain")]
    NoMetaboardsConfigured,
    #[error("Metaboard not configured for chain ID: {0}")]
    MetaboardNotConfigured(u32),
    #[error("Metaboard subgraph error: {0}")]
    MetaboardSubgraphError(String),
    #[error("Invalid dotrain source metadata found")]
    InvalidDotrainSourceMetadata,
}

impl From<DotrainOrderError> for RaindexError {
    fn from(err: DotrainOrderError) -> Self {
        Self::DotrainOrderError(Box::new(err))
    }
}

impl From<LocalDbError> for RaindexError {
    fn from(err: LocalDbError) -> Self {
        Self::LocalDbError(Box::new(err))
    }
}

impl From<serde_wasm_bindgen::Error> for RaindexError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        RaindexError::SerdeError(err.into())
    }
}

impl RaindexError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            RaindexError::InvalidYamlConfig => {
                "The YAML configuration is invalid. Please check your configuration.".to_string()
            }
            RaindexError::ChainIdNotFound(chain_id) => format!(
                "The chain ID '{}' was not found in the configuration.",
                chain_id
            ),
            RaindexError::NoNetworksConfigured => {
                "No networks configured. Please check your configuration.".to_string()
            }
            RaindexError::SubgraphNotConfigured(chain_id) => {
                format!("No subgraph is configured for chain ID '{}'.", chain_id)
            }
            RaindexError::TransactionIndexingTimeout { tx_hash, attempts } => {
                format!(
                    "Timeout waiting for transaction {tx_hash:#x} to be indexed after {attempts} attempts."
                )
            }
            RaindexError::TradesIndexingTimeout { tx_hash, attempts } => {
                format!(
                    "Timeout waiting for trades of transaction {tx_hash:#x} to be indexed after {attempts} attempts."
                )
            }
            RaindexError::YamlError(err) => format!(
                "YAML configuration parsing failed: {}. Check file syntax and structure.",
                err
            ),
            RaindexError::SerdeError(err) => format!(
                "Data conversion failed: {}. The data format may be incompatible.",
                err
            ),
            RaindexError::DotrainOrderError(err) => format!(
                "Order configuration is invalid: {}. Please check the order parameters and format.",
                err
            ),
            RaindexError::FromHexError(err) => {
                format!(
                    "Invalid address format: {}. Please provide a valid hexadecimal address.",
                    err
                )
            }
            RaindexError::OrderbookSubgraphClientError(err) => {
                format!("Failed to query subgraph: {}. Check network connection and subgraph availability.", err)
            }
            RaindexError::TryDecodeRainlangSourceError(err) => {
                format!("Failed to decode Rainlang source: {}. The source code may be corrupted or incompatible.", err)
            }
            RaindexError::U256ParseError(err) => {
                format!(
                    "Invalid number format: {}. Please provide a valid numeric value.",
                    err
                )
            }
            RaindexError::I256ParseError(err) => {
                format!(
                    "Invalid number format: {}. Please provide a valid numeric value.",
                    err
                )
            }
            RaindexError::JsError(err) => format!("JavaScript error: {}", err),
            RaindexError::ReadLockError => {
                "Failed to read the YAML configuration due to a lock error".to_string()
            }
            RaindexError::WriteLockError => {
                "Failed to modify the YAML configuration due to a lock error".to_string()
            }
            RaindexError::ZeroAmount => "Amount cannot be zero".to_string(),
            RaindexError::NegativeAmount => "Amount cannot be negative".to_string(),
            RaindexError::WritableTransactionExecuteError(err) => {
                format!("Failed to execute transaction: {}", err)
            }
            RaindexError::ExistingAllowance => {
                "There is already an allowance for this vault".to_string()
            }
            RaindexError::DepositArgsError(err) => {
                format!("Failed to create deposit arguments: {}", err)
            }
            RaindexError::OrderbookNotFound(address, chain_id) => {
                format!(
                    "Orderbook not found for address: {} on chain ID: {}",
                    address, chain_id
                )
            }
            RaindexError::OrderNotFound(address, chain_id, order_hash) => {
                format!(
                    "Order not found for address: {} on chain ID: {} with hash: {}",
                    address, chain_id, order_hash
                )
            }
            RaindexError::VaultNotFound(address, chain_id, vault_id) => format!(
                "Vault not found for address: {} on chain ID: {} with id: {}",
                address, chain_id, vault_id
            ),
            RaindexError::OrderDetailError(err) => {
                format!("Failed to decode order detail: {}", err)
            }
            RaindexError::AddOrderArgsError(e) => {
                format!("Failed to prepare the add order calldata: {}", e)
            }
            RaindexError::OrderbookQuoteError(err) => {
                format!("Failed to get order quote: {}", err)
            }
            RaindexError::SubgraphNotFound(subgraph, order) => {
                format!(
                    "Subgraph with name '{}' not found for the order with hash '{}'",
                    subgraph, order
                )
            }
            RaindexError::InvalidVaultBalanceChangeType(typ) => {
                format!("Invalid vault balance change type: {}", typ)
            }
            RaindexError::Erc20(err) => format!("Failed to get ERC20 info: {err}"),
            RaindexError::Float(err) => format!("Float error: {err}"),
            RaindexError::ParseInt(err) => format!("Failed to parse an integer: {err}"),
            RaindexError::TryFromUint(err) => format!("Failed to convert to u8: {err}"),
            RaindexError::MissingErc20Decimals(token) => {
                format!("Missing decimal information for the token address: {token}")
            }
            RaindexError::AmountFormatterError(err) => format!("Amount formatter error: {err}"),
            RaindexError::LocalDbError(err) => {
                format!("There was an error with the local database: {err}")
            }
            RaindexError::LocalDbQueryError(err) => {
                format!("There was an error querying the local database: {err}")
            }
            RaindexError::LocalDbUnsupportedNetwork(chain_id) => {
                format!("The chain ID: {chain_id} is not supported for local database operations.")
            }
            RaindexError::NoLiquidity => {
                "No liquidity available for the requested token pair".to_string()
            }
            RaindexError::InsufficientLiquidity {
                requested,
                available,
            } => {
                format!(
                    "Insufficient liquidity: requested {}, but only {} available",
                    requested, available
                )
            }
            RaindexError::SameTokenPair => {
                "Sell token and buy token cannot be the same".to_string()
            }
            RaindexError::NonPositiveAmount => "Amount must be positive".to_string(),
            RaindexError::NegativePriceCap => "Price cap cannot be negative".to_string(),
            RaindexError::RpcClientError(err) => format!("RPC client error: {}", err),
            RaindexError::PreflightError(err) => {
                format!("Preflight check failed: {err}")
            }
            RaindexError::QuoteDataMissing => {
                "Quote data is missing. Please ensure the quote was successful.".to_string()
            }
            RaindexError::InvalidInputIndex(index) => {
                format!(
                    "Invalid input index: {}. The order does not have an input at this index.",
                    index
                )
            }
            RaindexError::ParseMetaError(err) => format!("Cannot parse metadata: {err}"),
            RaindexError::NoMetaboardsConfigured => {
                "No metaboards configured for any chain. Please check your configuration."
                    .to_string()
            }
            RaindexError::MetaboardNotConfigured(chain_id) => {
                format!("Metaboard is not configured for chain ID: {chain_id}")
            }
            RaindexError::MetaboardSubgraphError(err) => {
                format!("Failed to query metaboard subgraph: {err}")
            }
            RaindexError::InvalidDotrainSourceMetadata => {
                "Found metadata but it could not be parsed as valid dotrain source".to_string()
            }
        }
    }
}

impl From<RaindexError> for JsValue {
    fn from(value: RaindexError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

impl From<RaindexError> for WasmEncodedError {
    fn from(value: RaindexError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;

    #[cfg(not(target_family = "wasm"))]
    pub const CHAIN_ID_1_ORDERBOOK_ADDRESS: &str = "0x1234567890123456789012345678901234567890";
    pub fn get_test_yaml(subgraph1: &str, subgraph2: &str, rpc1: &str, rpc2: &str) -> String {
        format!(
            r#"
version: {spec_version}
networks:
    mainnet:
        rpcs:
            - {rpc1}
        chain-id: 1
        label: Ethereum Mainnet
        network-id: 1
        currency: ETH
    polygon:
        rpcs:
            - {rpc2}
        chain-id: 137
        label: Polygon Mainnet
        network-id: 137
        currency: MATIC
subgraphs:
    mainnet: {subgraph1}
    polygon: {subgraph2}
metaboards:
    mainnet: https://api.thegraph.com/subgraphs/name/xyz
    polygon: https://api.thegraph.com/subgraphs/name/polygon
orderbooks:
    mainnet-orderbook:
        address: 0x1234567890123456789012345678901234567890
        network: mainnet
        subgraph: mainnet
        local-db-remote: remote
        label: Primary Orderbook
        deployment-block: 12345
    polygon-orderbook:
        address: 0x0987654321098765432109876543210987654321
        network: polygon
        subgraph: polygon
        local-db-remote: remote
        deployment-block: 12345
        label: Polygon Orderbook
tokens:
    weth:
        network: mainnet
        address: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
        decimals: 18
        label: Wrapped Ether
        symbol: WETH
    usdc:
        network: polygon
        address: 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174
        decimals: 6
        label: USD Coin
        symbol: USDC
deployers:
    mainnet-deployer:
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
        network: mainnet
accounts:
    alice: 0x742d35Cc6634C0532925a3b8D4Fd2d3dB2d4D7fA
    bob: 0x8ba1f109551bD432803012645aac136c0c8D2e80
    charlie: 0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5
"#,
            spec_version = SpecVersion::current()
        )
    }

    #[cfg(target_family = "wasm")]
    pub fn new_test_client_with_db_callback(
        yamls: Vec<String>,
        query_callback: js_sys::Function,
    ) -> RaindexClient {
        let client = RaindexClient::new(yamls, None).expect("test yaml should be valid");
        client.set_local_db_callback(query_callback, None).unwrap();
        client
    }

    #[cfg(target_family = "wasm")]
    pub fn get_local_db_test_yaml() -> String {
        format!(
            r#"
version: {spec_version}
networks:
    arbitrum:
        rpcs:
            - https://arb1.example
        chain-id: 42161
        label: Arbitrum
        network-id: 42161
        currency: ETH
subgraphs:
    arbitrum: https://arb.subgraph
metaboards:
    arbitrum: https://arb.metaboard
orderbooks:
    arbitrum-orderbook:
        address: 0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB
        network: arbitrum
        subgraph: arbitrum
        local-db-remote: remote
        deployment-block: 1
tokens:
    tokena:
        network: arbitrum
        address: 0x00000000000000000000000000000000000000aa
        decimals: 18
        label: Token A
        symbol: TKNA
    tokenb:
        network: arbitrum
        address: 0x00000000000000000000000000000000000000bb
        decimals: 6
        label: Token B
        symbol: TKNB
deployers:
    arb-deployer:
        address: 0x1111111111111111111111111111111111111111
        network: arbitrum
accounts:
    test: 0x2222222222222222222222222222222222222222
"#,
            spec_version = SpecVersion::current()
        )
    }

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use rain_orderbook_app_settings::yaml::YamlError;
        use url::Url;
        use wasm_bindgen_test::wasm_bindgen_test;

        fn get_invalid_yaml() -> String {
            format!(
                r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    orderbooks:
        invalid-orderbook:
            address: 0x1234567890123456789012345678901234567890
            network: nonexistent-network
            subgraph: nonexistent-subgraph
    "#,
                spec_version = SpecVersion::current()
            )
        }

        #[wasm_bindgen_test]
        fn test_raindex_client_new_success() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();
            assert!(!client.orderbook_yaml.documents.is_empty());
        }

        #[wasm_bindgen_test]
        fn test_raindex_client_new_invalid_yaml() {
            let err = RaindexClient::new(vec![get_invalid_yaml()], Some(true)).unwrap_err();
            assert!(matches!(
                err,
                RaindexError::YamlError(YamlError::Field { .. })
            ));
            assert!(err
                .to_readable_msg()
                .contains("YAML configuration parsing failed"));
        }

        #[wasm_bindgen_test]
        fn test_raindex_client_new_empty_yaml() {
            let err = RaindexClient::new(vec!["".to_string()], None).unwrap_err();
            assert!(matches!(err, RaindexError::YamlError(YamlError::EmptyFile)));
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_single_chain() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let args = client.get_multi_subgraph_args(Some(vec![1])).unwrap();
            assert_eq!(args.len(), 1);
            assert_eq!(
                args.get(&1).unwrap()[0].url,
                Url::parse("http://localhost:3000/sg1").unwrap()
            );
            assert_eq!(args.get(&1).unwrap()[0].name, "Ethereum Mainnet");
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_all_chains() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let args = client.get_multi_subgraph_args(None).unwrap();
            assert_eq!(args.len(), 2);

            let urls: Vec<&str> = args.iter().map(|(_, arg)| arg[0].url.as_str()).collect();
            assert!(urls.contains(&"http://localhost:3000/sg1"));
            assert!(urls.contains(&"http://localhost:3000/sg2"));

            let names: Vec<&str> = args.iter().map(|(_, arg)| arg[0].name.as_str()).collect();
            assert!(names.contains(&"Ethereum Mainnet"));
            assert!(names.contains(&"Polygon Mainnet"));
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_empty_chain_ids_defaults_to_all() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let args = client.get_multi_subgraph_args(Some(vec![])).unwrap();
            assert_eq!(args.len(), 2);

            let urls: Vec<&str> = args.iter().map(|(_, arg)| arg[0].url.as_str()).collect();
            assert!(urls.contains(&"http://localhost:3000/sg1"));
            assert!(urls.contains(&"http://localhost:3000/sg2"));
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_multiple_chains() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let args = client.get_multi_subgraph_args(Some(vec![1, 137])).unwrap();
            assert_eq!(args.len(), 2);

            let args1 = args.get(&1).unwrap();
            assert_eq!(args1.len(), 1);
            assert_eq!(
                args1[0].url,
                Url::parse("http://localhost:3000/sg1").unwrap()
            );
            assert_eq!(args1[0].name, "Ethereum Mainnet");

            let args2 = args.get(&137).unwrap();
            assert_eq!(args2.len(), 1);
            assert_eq!(
                args2[0].url,
                Url::parse("http://localhost:3000/sg2").unwrap()
            );
            assert_eq!(args2[0].name, "Polygon Mainnet");
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_invalid_chain() {
            let client = RaindexClient::new(
                vec![get_test_yaml(
                    // not used
                    "http://localhost:3000/sg1",
                    "http://localhost:3000/sg2",
                    "http://localhost:3000/rpc1",
                    "http://localhost:3000/rpc2",
                )],
                None,
            )
            .unwrap();

            let err = client.get_multi_subgraph_args(Some(vec![999])).unwrap_err();
            assert!(
                matches!(err, RaindexError::YamlError(YamlError::NotFound(ref msg)) if msg.contains("network with chain-id: 999"))
            );
        }

        #[wasm_bindgen_test]
        fn test_get_multi_subgraph_args_no_networks() {
            let yaml = format!(
                r#"
    version: {spec_version}
    networks:
        isolated:
            rpcs:
                - https://isolated.rpc
            chain-id: 999
        some-network:
            rpcs:
                - https://some-network.rpc
            chain-id: 1000
    subgraphs:
        test: https://test.subgraph
    metaboards:
        test: https://test.metaboard
    tokens:
        test-token:
            network: isolated
            address: 0x1111111111111111111111111111111111111111
            decimals: 18
    deployers:
        test-deployer:
            address: 0x2222222222222222222222222222222222222222
            network: isolated
    orderbooks:
        test-orderbook:
            address: 0x1111111111111111111111111111111111111111
            network: some-network
            subgraph: test
            local-db-remote: remote
            label: Test Orderbook
            deployment-block: 12345
    "#,
                spec_version = SpecVersion::current()
            );

            let client = RaindexClient::new(vec![yaml], None).unwrap();

            let err = client.get_multi_subgraph_args(None).unwrap_err();
            assert!(matches!(
                err,
                RaindexError::YamlError(YamlError::NotFound(ref msg)) if msg.contains("orderbook with network key: isolated")
            ));
            assert!(err
                .to_readable_msg()
                .contains("orderbook with network key: isolated not found"));
        }
    }
}
