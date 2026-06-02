use crate::{erc20, utils::amount_formatter};
use alloy::eips::{BlockId, RpcBlockHash};
use alloy::network::{AnyNetwork, TransactionBuilder};
use alloy::primitives::{address, Address, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::serde::WithOtherFields;
use alloy::sol_types::{Revert, SolCall, SolError};
use alloy::transports::TransportError;
use rain_error_decoding::{AbiDecodeFailedErrors, AbiDecodedErrorType};
use raindex_bindings::provider::{mk_read_provider, ReadProvider, ReadProviderError};
use raindex_bindings::IERC4626::IERC4626Instance;
use raindex_bindings::{
    IERC20Metadata::decimalsCall as erc20DecimalsCall,
    IMulticall3::{aggregate3Call, Call3, Result as Multicall3Result},
    IERC4626::{assetCall, convertToAssetsCall, decimalsCall as erc4626DecimalsCall},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[cfg(not(target_family = "wasm"))]
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_MULTICALL3_ADDRESS: Address =
    address!("cA11bde05977b3631167028862bE2a173976CA11");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ERC4626ShareAssetConversion {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub share_token_address: Address,
    pub share_token_decimals: u8,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub asset_address: Address,
    pub asset_decimals: u8,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub shares: U256,
    pub shares_display: String,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub assets: U256,
    pub assets_display: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ERC4626ShareAssetConversion);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ERC4626BatchVault {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub vault_address: Address,
    #[cfg_attr(target_family = "wasm", tsify(optional, type = "string"))]
    pub shares: Option<U256>,
    #[cfg_attr(target_family = "wasm", tsify(optional, type = "string"))]
    pub expected_asset_address: Option<Address>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ERC4626BatchVault);

impl ERC4626BatchVault {
    pub fn new(vault_address: Address) -> Self {
        Self {
            vault_address,
            shares: None,
            expected_asset_address: None,
        }
    }

    pub fn with_shares(mut self, shares: U256) -> Self {
        self.shares = Some(shares);
        self
    }

    pub fn with_expected_asset_address(mut self, expected_asset_address: Address) -> Self {
        self.expected_asset_address = Some(expected_asset_address);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ERC4626BatchItem {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub vault_address: Address,
    pub success: bool,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub data: Option<ERC4626ShareAssetConversion>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub expected_asset_matches: Option<bool>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub error: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ERC4626BatchItem);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ERC4626BatchResponse {
    pub block_number: u64,
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub block_hash: B256,
    pub block_timestamp: Option<u64>,
    pub captured_at: u64,
    pub items: Vec<ERC4626BatchItem>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ERC4626BatchResponse);

#[derive(Debug, Clone, PartialEq, Eq)]
struct BatchState {
    vault_address: Address,
    expected_asset_address: Option<Address>,
    share_decimals: Option<u8>,
    asset_address: Option<Address>,
    asset_decimals: Option<u8>,
    shares: Option<U256>,
    assets: Option<U256>,
    error: Option<String>,
}

impl BatchState {
    fn new(input: &ERC4626BatchVault) -> Self {
        Self {
            vault_address: input.vault_address,
            expected_asset_address: input.expected_asset_address,
            share_decimals: None,
            asset_address: None,
            asset_decimals: None,
            shares: input.shares,
            assets: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase1Call {
    ShareDecimals(usize),
    Asset(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase2Call {
    AssetDecimals(usize),
    ConvertToAssets(usize),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlockResponse {
    number: U256,
    hash: B256,
    timestamp: U256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapturedBlock {
    number: u64,
    hash: B256,
    timestamp: Option<u64>,
}

#[derive(Debug, Error)]
enum ERC4626BatchItemError {
    #[error("{label} call reverted: {reason}")]
    StandardRevert { label: &'static str, reason: String },
    #[error("{label} call reverted: {source}")]
    CallReverted {
        label: &'static str,
        source: AbiDecodedErrorType,
    },
    #[error("{label} call failed with undecodable revert data {return_data}: {source}")]
    CallRevertDecodeFailed {
        label: &'static str,
        return_data: alloy::primitives::Bytes,
        source: AbiDecodeFailedErrors,
    },
    #[error("{label} return decode failed: {source}")]
    ReturnDecodeFailed {
        label: &'static str,
        source: alloy::sol_types::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ERC4626 {
    pub rpcs: Vec<Url>,
    pub vault_address: Address,
}

impl ERC4626 {
    pub fn new(rpcs: Vec<Url>, vault_address: Address) -> Self {
        Self {
            rpcs,
            vault_address,
        }
    }

    /// Reads share-to-asset conversions for many ERC4626 vault/share tokens.
    ///
    /// Contract reads are pinned to the captured latest block hash. Per-vault
    /// ERC4626/ERC20 subcall failures are returned in `items[n].error`; this
    /// function returns `Err` only for batch infrastructure failures such as
    /// provider construction, latest block capture, transport failure, or a
    /// malformed Multicall3 aggregate response.
    pub async fn batch_share_ratios(
        rpcs: Vec<Url>,
        vaults: Vec<ERC4626BatchVault>,
        multicall3_address: Option<Address>,
    ) -> Result<ERC4626BatchResponse, Error> {
        let captured_at = captured_at_unix_timestamp();
        let provider = mk_read_provider(&rpcs)?;
        let block = get_latest_block(&provider).await?;
        let block_id = BlockId::Hash(RpcBlockHash::from(block.hash));
        let multicall3_address = multicall3_address.unwrap_or(DEFAULT_MULTICALL3_ADDRESS);

        let mut states: Vec<BatchState> = vaults.iter().map(BatchState::new).collect();

        if !states.is_empty() {
            read_phase1(&provider, block_id, multicall3_address, &mut states).await?;
            read_phase2(&provider, block_id, multicall3_address, &mut states).await?;
        }

        Ok(ERC4626BatchResponse {
            block_number: block.number,
            block_hash: block.hash,
            block_timestamp: block.timestamp,
            captured_at,
            items: states.into_iter().map(BatchState::into_item).collect(),
        })
    }

    fn get_instance(&self) -> Result<IERC4626Instance<ReadProvider, AnyNetwork>, Error> {
        let provider = mk_read_provider(&self.rpcs)?;
        let vault = IERC4626Instance::new(self.vault_address, provider);
        Ok(vault)
    }

    pub async fn share_decimals(&self) -> Result<u8, Error> {
        let vault = self.get_instance()?;
        let decimals = vault.decimals().call().await;

        match decimals {
            Ok(decimals) => Ok(decimals),
            Err(err) => Err(handle_alloy_err(err, "Share decimals reverted").await),
        }
    }

    pub async fn asset(&self) -> Result<Address, Error> {
        let vault = self.get_instance()?;
        let asset = vault.asset().call().await;

        match asset {
            Ok(asset) => Ok(asset),
            Err(err) => Err(handle_alloy_err(err, "Asset query reverted").await),
        }
    }

    pub async fn underlying_asset(&self) -> Result<Address, Error> {
        self.asset().await
    }

    pub async fn asset_decimals(&self) -> Result<u8, Error> {
        let asset = self.asset().await?;
        let erc20 = erc20::ERC20::new(self.rpcs.clone(), asset);
        Ok(erc20.decimals().await?)
    }

    pub async fn underlying_asset_decimals(&self) -> Result<u8, Error> {
        self.asset_decimals().await
    }

    pub async fn convert_to_assets(&self, shares: U256) -> Result<U256, Error> {
        let vault = self.get_instance()?;
        let assets = vault.convertToAssets(shares).call().await;

        match assets {
            Ok(assets) => Ok(assets),
            Err(err) => Err(handle_alloy_err(err, "Convert to assets reverted").await),
        }
    }

    pub async fn convert_to_shares(&self, assets: U256) -> Result<U256, Error> {
        let vault = self.get_instance()?;
        let shares = vault.convertToShares(assets).call().await;

        match shares {
            Ok(shares) => Ok(shares),
            Err(err) => Err(handle_alloy_err(err, "Convert to shares reverted").await),
        }
    }

    pub async fn share_ratio(&self) -> Result<ERC4626ShareAssetConversion, Error> {
        let share_decimals = self.share_decimals().await?;
        let one_share = one_token_amount(share_decimals)?;
        self.convert_shares_to_assets_with_share_decimals(share_decimals, one_share)
            .await
    }

    pub async fn convert_shares_to_assets(
        &self,
        shares: U256,
    ) -> Result<ERC4626ShareAssetConversion, Error> {
        let share_decimals = self.share_decimals().await?;
        self.convert_shares_to_assets_with_share_decimals(share_decimals, shares)
            .await
    }

    async fn convert_shares_to_assets_with_share_decimals(
        &self,
        share_decimals: u8,
        shares: U256,
    ) -> Result<ERC4626ShareAssetConversion, Error> {
        let asset_address = self.asset().await?;
        let erc20 = erc20::ERC20::new(self.rpcs.clone(), asset_address);
        let asset_decimals = erc20.decimals().await?;
        let assets = self.convert_to_assets(shares).await?;

        self.build_conversion(
            share_decimals,
            asset_address,
            asset_decimals,
            shares,
            assets,
        )
    }

    pub async fn convert_assets_to_shares(
        &self,
        assets: U256,
    ) -> Result<ERC4626ShareAssetConversion, Error> {
        let share_decimals = self.share_decimals().await?;
        let asset_address = self.asset().await?;
        let erc20 = erc20::ERC20::new(self.rpcs.clone(), asset_address);
        let asset_decimals = erc20.decimals().await?;
        let shares = self.convert_to_shares(assets).await?;

        self.build_conversion(
            share_decimals,
            asset_address,
            asset_decimals,
            shares,
            assets,
        )
    }

    fn build_conversion(
        &self,
        share_decimals: u8,
        asset_address: Address,
        asset_decimals: u8,
        shares: U256,
        assets: U256,
    ) -> Result<ERC4626ShareAssetConversion, Error> {
        build_conversion(
            self.vault_address,
            share_decimals,
            asset_address,
            asset_decimals,
            shares,
            assets,
        )
    }
}

impl BatchState {
    fn set_error(&mut self, error: impl Into<String>) {
        if self.error.is_none() {
            self.error = Some(error.into());
        }
    }

    fn into_item(self) -> ERC4626BatchItem {
        let expected_asset_matches = self
            .expected_asset_address
            .zip(self.asset_address)
            .map(|(expected, actual)| expected == actual);

        let mut error = self.error;
        if expected_asset_matches == Some(false) && error.is_none() {
            if let (Some(expected), Some(actual)) =
                (self.expected_asset_address, self.asset_address)
            {
                error = Some(format!(
                    "Expected asset address mismatch: expected {expected}, got {actual}"
                ));
            }
        }

        let data = match (
            self.share_decimals,
            self.asset_address,
            self.asset_decimals,
            self.shares,
            self.assets,
        ) {
            (
                Some(share_decimals),
                Some(asset_address),
                Some(asset_decimals),
                Some(shares),
                Some(assets),
            ) => {
                match build_conversion(
                    self.vault_address,
                    share_decimals,
                    asset_address,
                    asset_decimals,
                    shares,
                    assets,
                ) {
                    Ok(conversion) => Some(conversion),
                    Err(err) => {
                        if error.is_none() {
                            error = Some(err.to_string());
                        }
                        None
                    }
                }
            }
            _ => None,
        };

        let error = match (error, data.is_none()) {
            (Some(error), _) => Some(error),
            (None, true) => Some("Incomplete ERC4626 batch result".to_string()),
            (None, false) => None,
        };

        ERC4626BatchItem {
            vault_address: self.vault_address,
            success: error.is_none(),
            data,
            expected_asset_matches,
            error,
        }
    }
}

async fn read_phase1(
    provider: &ReadProvider,
    block_id: BlockId,
    multicall3_address: Address,
    states: &mut [BatchState],
) -> Result<(), Error> {
    let mut calls = Vec::with_capacity(states.len() * 2);
    let mut call_map = Vec::with_capacity(states.len() * 2);

    for (index, state) in states.iter().enumerate() {
        calls.push(Call3 {
            target: state.vault_address,
            allowFailure: true,
            callData: erc4626DecimalsCall {}.abi_encode().into(),
        });
        call_map.push(Phase1Call::ShareDecimals(index));

        calls.push(Call3 {
            target: state.vault_address,
            allowFailure: true,
            callData: assetCall {}.abi_encode().into(),
        });
        call_map.push(Phase1Call::Asset(index));
    }

    let results = call_multicall3(provider, block_id, multicall3_address, calls).await?;
    if results.len() != call_map.len() {
        return Err(Error::MalformedMulticallResponse {
            expected: call_map.len(),
            actual: results.len(),
        });
    }

    for (result, call) in results.into_iter().zip(call_map) {
        match call {
            Phase1Call::ShareDecimals(index) => {
                match decode_multicall_return::<erc4626DecimalsCall>(&result, "share decimals")
                    .await
                {
                    Ok(decimals) => states[index].share_decimals = Some(decimals),
                    Err(error) => states[index].set_error(error.to_string()),
                }
            }
            Phase1Call::Asset(index) => {
                match decode_multicall_return::<assetCall>(&result, "asset").await {
                    Ok(asset) => states[index].asset_address = Some(asset),
                    Err(error) => states[index].set_error(error.to_string()),
                }
            }
        }
    }

    for state in states.iter_mut() {
        if state.error.is_none() {
            if let (Some(share_decimals), None) = (state.share_decimals, state.shares) {
                match one_token_amount(share_decimals) {
                    Ok(shares) => state.shares = Some(shares),
                    Err(err) => state.set_error(err.to_string()),
                }
            }
        }
    }

    Ok(())
}

async fn read_phase2(
    provider: &ReadProvider,
    block_id: BlockId,
    multicall3_address: Address,
    states: &mut [BatchState],
) -> Result<(), Error> {
    let mut calls = Vec::new();
    let mut call_map = Vec::new();

    for (index, state) in states.iter().enumerate() {
        if state.error.is_some() {
            continue;
        }

        let (Some(asset_address), Some(shares)) = (state.asset_address, state.shares) else {
            continue;
        };

        calls.push(Call3 {
            target: asset_address,
            allowFailure: true,
            callData: erc20DecimalsCall {}.abi_encode().into(),
        });
        call_map.push(Phase2Call::AssetDecimals(index));

        calls.push(Call3 {
            target: state.vault_address,
            allowFailure: true,
            callData: convertToAssetsCall { shares }.abi_encode().into(),
        });
        call_map.push(Phase2Call::ConvertToAssets(index));
    }

    if calls.is_empty() {
        return Ok(());
    }

    let results = call_multicall3(provider, block_id, multicall3_address, calls).await?;
    if results.len() != call_map.len() {
        return Err(Error::MalformedMulticallResponse {
            expected: call_map.len(),
            actual: results.len(),
        });
    }

    for (result, call) in results.into_iter().zip(call_map) {
        match call {
            Phase2Call::AssetDecimals(index) => {
                match decode_multicall_return::<erc20DecimalsCall>(&result, "asset decimals").await
                {
                    Ok(decimals) => states[index].asset_decimals = Some(decimals),
                    Err(error) => states[index].set_error(error.to_string()),
                }
            }
            Phase2Call::ConvertToAssets(index) => {
                match decode_multicall_return::<convertToAssetsCall>(&result, "convert to assets")
                    .await
                {
                    Ok(assets) => states[index].assets = Some(assets),
                    Err(error) => states[index].set_error(error.to_string()),
                }
            }
        }
    }

    Ok(())
}

async fn call_multicall3(
    provider: &ReadProvider,
    block_id: BlockId,
    multicall3_address: Address,
    calls: Vec<Call3>,
) -> Result<Vec<Multicall3Result>, Error> {
    let calldata = aggregate3Call { calls }.abi_encode();
    let tx = TransactionRequest::default()
        .with_to(multicall3_address)
        .with_input(calldata);
    let tx = WithOtherFields::new(tx);
    let bytes = provider.call(tx).block(block_id).await?;
    aggregate3Call::abi_decode_returns(&bytes).map_err(|source| Error::SolTypesError {
        msg: "Failed to decode Multicall3 aggregate3 return".to_string(),
        source,
    })
}

async fn decode_multicall_return<C: SolCall>(
    result: &Multicall3Result,
    label: &'static str,
) -> Result<C::Return, ERC4626BatchItemError> {
    if !result.success {
        if let Ok(revert) = Revert::abi_decode(result.returnData.as_ref()) {
            return Err(ERC4626BatchItemError::StandardRevert {
                label,
                reason: revert.reason,
            });
        }

        return match AbiDecodedErrorType::selector_registry_abi_decode(
            result.returnData.as_ref(),
            None,
        )
        .await
        {
            Ok(source) => Err(ERC4626BatchItemError::CallReverted { label, source }),
            Err(source) => Err(ERC4626BatchItemError::CallRevertDecodeFailed {
                label,
                return_data: result.returnData.clone(),
                source,
            }),
        };
    }

    C::abi_decode_returns(&result.returnData)
        .map_err(|source| ERC4626BatchItemError::ReturnDecodeFailed { label, source })
}

async fn get_latest_block(provider: &ReadProvider) -> Result<CapturedBlock, Error> {
    let params = serde_json::json!(["latest", false]);

    let block = provider
        .client()
        .request::<_, Option<BlockResponse>>("eth_getBlockByNumber", params)
        .await?
        .ok_or(Error::LatestBlockNotFound)?;

    Ok(CapturedBlock {
        number: block
            .number
            .try_into()
            .map_err(|_| Error::BlockNumberOverflow {
                number: block.number,
            })?,
        hash: block.hash,
        timestamp: block.timestamp.try_into().ok(),
    })
}

#[cfg(not(target_family = "wasm"))]
fn captured_at_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(target_family = "wasm")]
fn captured_at_unix_timestamp() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

fn one_token_amount(decimals: u8) -> Result<U256, Error> {
    U256::from(10)
        .checked_pow(U256::from(decimals))
        .ok_or(Error::UnsupportedDecimals { decimals })
}

fn build_conversion(
    share_token_address: Address,
    share_decimals: u8,
    asset_address: Address,
    asset_decimals: u8,
    shares: U256,
    assets: U256,
) -> Result<ERC4626ShareAssetConversion, Error> {
    Ok(ERC4626ShareAssetConversion {
        share_token_address,
        share_token_decimals: share_decimals,
        asset_address,
        asset_decimals,
        shares,
        shares_display: amount_formatter::format_amount_u256(shares, share_decimals)?,
        assets,
        assets_display: amount_formatter::format_amount_u256(assets, asset_decimals)?,
    })
}

const ERROR_MESSAGE: &str = "Failed to get ERC4626 vault information: ";

#[derive(Debug, Error)]
pub enum Error {
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    AbiDecodedErrorType {
        msg: String,
        #[source]
        source: AbiDecodedErrorType,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    AbiDecodeError {
        msg: String,
        #[source]
        source: AbiDecodeFailedErrors,
    },
    #[error("{ERROR_MESSAGE} {msg} - {source}")]
    SolTypesError {
        msg: String,
        #[source]
        source: alloy::sol_types::Error,
    },
    #[error(transparent)]
    ReadProviderError(#[from] ReadProviderError),
    #[error(transparent)]
    TransportError(#[from] TransportError),
    #[error("Contract call failed: {0}")]
    ContractCallError(#[from] alloy::contract::Error),
    #[error(transparent)]
    ERC20Error(#[from] erc20::Error),
    #[error(transparent)]
    AmountFormatterError(#[from] amount_formatter::AmountFormatterError),
    #[error("Malformed Multicall3 response: expected {expected} results, got {actual}")]
    MalformedMulticallResponse { expected: usize, actual: usize },
    #[error("Latest block not found")]
    LatestBlockNotFound,
    #[error("Block number does not fit u64: {number}")]
    BlockNumberOverflow { number: U256 },
    #[error("Unsupported token decimals for one-token conversion: {decimals}")]
    UnsupportedDecimals { decimals: u8 },
}

async fn handle_alloy_err(err: alloy::contract::Error, msg: &str) -> Error {
    if let Some(revert_data) = err.as_revert_data() {
        let err =
            AbiDecodedErrorType::selector_registry_abi_decode(revert_data.as_ref(), None).await;

        match err {
            Ok(err) => {
                return Error::AbiDecodedErrorType {
                    msg: msg.to_string(),
                    source: err,
                };
            }
            Err(e) => {
                return Error::AbiDecodeError {
                    msg: msg.to_string(),
                    source: e,
                };
            }
        }
    }

    Error::ContractCallError(err)
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::{
        hex::encode_prefixed,
        primitives::Bytes,
        sol_types::{Revert, SolCall, SolError},
    };
    use httpmock::MockServer;
    use raindex_bindings::{
        IERC20Metadata::decimalsCall as erc20DecimalsCall,
        IERC4626::{
            assetCall, convertToAssetsCall, convertToSharesCall,
            decimalsCall as erc4626DecimalsCall,
        },
    };
    use serde_json::json;

    fn rpc_url(server: &MockServer) -> Url {
        Url::parse(&server.url("/rpc")).unwrap()
    }

    fn mock_block_by_number(server: &MockServer, block_number: u64, timestamp: u64) {
        let block_hash = test_block_hash(block_number);
        server.mock(move |when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains("eth_getBlockByNumber")
                .body_contains("latest");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "number": format!("0x{block_number:x}"),
                    "hash": block_hash,
                    "timestamp": format!("0x{timestamp:x}"),
                },
            }));
        });
    }

    fn test_block_hash(block_number: u64) -> B256 {
        B256::from(U256::from(block_number))
    }

    fn success_result<C: SolCall>(value: &C::Return) -> Multicall3Result {
        Multicall3Result {
            success: true,
            returnData: C::abi_encode_returns(value).into(),
        }
    }

    fn failed_result() -> Multicall3Result {
        failed_result_with(Bytes::from_static(&[0xde, 0xad, 0xbe, 0xef]))
    }

    fn failed_result_with(return_data: Bytes) -> Multicall3Result {
        Multicall3Result {
            success: false,
            returnData: return_data,
        }
    }

    fn revert_result(reason: &str) -> Multicall3Result {
        failed_result_with(Revert::from(reason).abi_encode().into())
    }

    fn mock_multicall(
        server: &MockServer,
        required_selector: impl Into<String>,
        results: Vec<Multicall3Result>,
    ) {
        let required_selector = required_selector
            .into()
            .trim_start_matches("0x")
            .to_string();
        server.mock(move |when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(required_selector.clone());
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(aggregate3Call::abi_encode_returns(&results)),
            }));
        });
    }

    #[tokio::test]
    async fn test_batch_share_ratios_multi_vault_success() {
        let server = MockServer::start_async().await;
        let vault1 = Address::repeat_byte(0x11);
        let vault2 = Address::repeat_byte(0x22);
        let asset1 = Address::repeat_byte(0xaa);
        let asset2 = Address::repeat_byte(0xbb);
        let block_number = 123u64;
        let block_timestamp = 456u64;
        let shares = U256::from(10).pow(U256::from(18));

        mock_block_by_number(&server, block_number, block_timestamp);
        mock_multicall(
            &server,
            encode_prefixed(assetCall::SELECTOR),
            vec![
                success_result::<erc4626DecimalsCall>(&18u8),
                success_result::<assetCall>(&asset1),
                success_result::<erc4626DecimalsCall>(&18u8),
                success_result::<assetCall>(&asset2),
            ],
        );
        mock_multicall(
            &server,
            encode_prefixed(convertToAssetsCall::SELECTOR),
            vec![
                success_result::<erc20DecimalsCall>(&18u8),
                success_result::<convertToAssetsCall>(&shares),
                success_result::<erc20DecimalsCall>(&6u8),
                success_result::<convertToAssetsCall>(&U256::from(1_500_000u64)),
            ],
        );

        let response = ERC4626::batch_share_ratios(
            vec![rpc_url(&server)],
            vec![
                ERC4626BatchVault::new(vault1).with_expected_asset_address(asset1),
                ERC4626BatchVault::new(vault2).with_expected_asset_address(asset2),
            ],
            None,
        )
        .await
        .unwrap();

        assert_eq!(response.block_number, block_number);
        assert_eq!(response.block_hash, test_block_hash(block_number));
        assert_eq!(response.block_timestamp, Some(block_timestamp));
        assert!(response.captured_at > 0);
        assert_eq!(response.items.len(), 2);

        let item1 = &response.items[0];
        assert!(item1.success);
        assert_eq!(item1.expected_asset_matches, Some(true));
        let data1 = item1.data.as_ref().unwrap();
        assert_eq!(data1.share_token_address, vault1);
        assert_eq!(data1.asset_address, asset1);
        assert_eq!(data1.asset_decimals, 18);
        assert_eq!(data1.shares, shares);
        assert_eq!(data1.assets, shares);
        assert_eq!(data1.assets_display, "1");

        let item2 = &response.items[1];
        assert!(item2.success);
        assert_eq!(item2.expected_asset_matches, Some(true));
        let data2 = item2.data.as_ref().unwrap();
        assert_eq!(data2.share_token_address, vault2);
        assert_eq!(data2.asset_address, asset2);
        assert_eq!(data2.asset_decimals, 6);
        assert_eq!(data2.shares, shares);
        assert_eq!(data2.assets, U256::from(1_500_000u64));
        assert_eq!(data2.assets_display, "1.5");
    }

    #[tokio::test]
    async fn test_batch_share_ratios_per_item_failure() {
        let server = MockServer::start_async().await;
        let vault1 = Address::repeat_byte(0x33);
        let vault2 = Address::repeat_byte(0x44);
        let asset1 = Address::repeat_byte(0xcc);
        let block_number = 124u64;
        let shares = U256::from(10).pow(U256::from(18));

        mock_block_by_number(&server, block_number, 457);
        mock_multicall(
            &server,
            encode_prefixed(assetCall::SELECTOR),
            vec![
                success_result::<erc4626DecimalsCall>(&18u8),
                success_result::<assetCall>(&asset1),
                success_result::<erc4626DecimalsCall>(&18u8),
                failed_result(),
            ],
        );
        mock_multicall(
            &server,
            encode_prefixed(convertToAssetsCall::SELECTOR),
            vec![
                success_result::<erc20DecimalsCall>(&18u8),
                success_result::<convertToAssetsCall>(&shares),
            ],
        );

        let response = ERC4626::batch_share_ratios(
            vec![rpc_url(&server)],
            vec![
                ERC4626BatchVault::new(vault1),
                ERC4626BatchVault::new(vault2),
            ],
            None,
        )
        .await
        .unwrap();

        assert_eq!(response.items.len(), 2);
        assert!(response.items[0].success);
        assert!(response.items[0].data.is_some());
        assert!(!response.items[1].success);
        assert!(response.items[1].data.is_none());
        assert!(response.items[1]
            .error
            .as_ref()
            .unwrap()
            .contains("asset call reverted"));
    }

    #[tokio::test]
    async fn test_batch_share_ratios_one_item_custom_shares() {
        let server = MockServer::start_async().await;
        let vault = Address::repeat_byte(0x55);
        let asset = Address::repeat_byte(0xdd);
        let block_number = 125u64;
        let shares = U256::from(2) * U256::from(10).pow(U256::from(18));
        let assets = U256::from(3) * U256::from(10).pow(U256::from(18));

        mock_block_by_number(&server, block_number, 458);
        mock_multicall(
            &server,
            encode_prefixed(assetCall::SELECTOR),
            vec![
                success_result::<erc4626DecimalsCall>(&18u8),
                success_result::<assetCall>(&asset),
            ],
        );
        mock_multicall(
            &server,
            encode_prefixed(convertToAssetsCall::SELECTOR),
            vec![
                success_result::<erc20DecimalsCall>(&18u8),
                success_result::<convertToAssetsCall>(&assets),
            ],
        );

        let response = ERC4626::batch_share_ratios(
            vec![rpc_url(&server)],
            vec![ERC4626BatchVault::new(vault).with_shares(shares)],
            Some(Address::repeat_byte(0xee)),
        )
        .await
        .unwrap();

        assert_eq!(response.items.len(), 1);
        let data = response.items[0].data.as_ref().unwrap();
        assert_eq!(data.share_token_address, vault);
        assert_eq!(data.asset_address, asset);
        assert_eq!(data.shares, shares);
        assert_eq!(data.assets, assets);
        assert_eq!(data.shares_display, "2");
        assert_eq!(data.assets_display, "3");
    }

    #[tokio::test]
    async fn test_batch_share_ratios_expected_asset_mismatch() {
        let server = MockServer::start_async().await;
        let vault = Address::repeat_byte(0x66);
        let expected_asset = Address::repeat_byte(0x77);
        let actual_asset = Address::repeat_byte(0x88);
        let shares = U256::from(10).pow(U256::from(18));

        mock_block_by_number(&server, 126, 459);
        mock_multicall(
            &server,
            encode_prefixed(assetCall::SELECTOR),
            vec![
                success_result::<erc4626DecimalsCall>(&18u8),
                success_result::<assetCall>(&actual_asset),
            ],
        );
        mock_multicall(
            &server,
            encode_prefixed(convertToAssetsCall::SELECTOR),
            vec![
                success_result::<erc20DecimalsCall>(&18u8),
                success_result::<convertToAssetsCall>(&shares),
            ],
        );

        let response = ERC4626::batch_share_ratios(
            vec![rpc_url(&server)],
            vec![ERC4626BatchVault::new(vault).with_expected_asset_address(expected_asset)],
            None,
        )
        .await
        .unwrap();

        let item = &response.items[0];
        assert!(!item.success);
        assert_eq!(item.expected_asset_matches, Some(false));
        assert!(item.data.is_some());
        assert!(item
            .error
            .as_ref()
            .unwrap()
            .contains("Expected asset address mismatch"));
    }

    #[tokio::test]
    async fn test_batch_share_ratios_phase2_failure() {
        let server = MockServer::start_async().await;
        let vault = Address::repeat_byte(0x99);
        let asset = Address::repeat_byte(0xaa);

        mock_block_by_number(&server, 127, 460);
        mock_multicall(
            &server,
            encode_prefixed(assetCall::SELECTOR),
            vec![
                success_result::<erc4626DecimalsCall>(&18u8),
                success_result::<assetCall>(&asset),
            ],
        );
        mock_multicall(
            &server,
            encode_prefixed(convertToAssetsCall::SELECTOR),
            vec![success_result::<erc20DecimalsCall>(&18u8), failed_result()],
        );

        let response = ERC4626::batch_share_ratios(
            vec![rpc_url(&server)],
            vec![ERC4626BatchVault::new(vault)],
            None,
        )
        .await
        .unwrap();

        let item = &response.items[0];
        assert!(!item.success);
        assert!(item.data.is_none());
        assert!(item
            .error
            .as_ref()
            .unwrap()
            .contains("convert to assets call reverted"));
    }

    #[tokio::test]
    async fn test_batch_share_ratios_decodes_revert_reason() {
        let server = MockServer::start_async().await;
        let vault = Address::repeat_byte(0xab);
        let asset = Address::repeat_byte(0xcd);

        mock_block_by_number(&server, 130, 463);
        mock_multicall(
            &server,
            encode_prefixed(assetCall::SELECTOR),
            vec![
                success_result::<erc4626DecimalsCall>(&18u8),
                success_result::<assetCall>(&asset),
            ],
        );
        mock_multicall(
            &server,
            encode_prefixed(convertToAssetsCall::SELECTOR),
            vec![
                success_result::<erc20DecimalsCall>(&18u8),
                revert_result("vault paused"),
            ],
        );

        let response = ERC4626::batch_share_ratios(
            vec![rpc_url(&server)],
            vec![ERC4626BatchVault::new(vault)],
            None,
        )
        .await
        .unwrap();

        let item = &response.items[0];
        assert!(!item.success);
        assert_eq!(
            item.error.as_deref(),
            Some("convert to assets call reverted: vault paused")
        );
    }

    #[tokio::test]
    async fn test_batch_share_ratios_empty_list() {
        let server = MockServer::start_async().await;
        let block_number = 128;
        let block_timestamp = 461;

        mock_block_by_number(&server, block_number, block_timestamp);

        let response = ERC4626::batch_share_ratios(vec![rpc_url(&server)], vec![], None)
            .await
            .unwrap();

        assert_eq!(response.block_number, block_number);
        assert_eq!(response.block_hash, test_block_hash(block_number));
        assert_eq!(response.block_timestamp, Some(block_timestamp));
        assert!(response.items.is_empty());
    }

    #[tokio::test]
    async fn test_batch_share_ratios_malformed_multicall_response() {
        let server = MockServer::start_async().await;
        let vault = Address::repeat_byte(0xbb);

        mock_block_by_number(&server, 129, 462);
        mock_multicall(
            &server,
            encode_prefixed(assetCall::SELECTOR),
            vec![success_result::<erc4626DecimalsCall>(&18u8)],
        );

        let err = ERC4626::batch_share_ratios(
            vec![rpc_url(&server)],
            vec![ERC4626BatchVault::new(vault)],
            None,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            Error::MalformedMulticallResponse {
                expected: 2,
                actual: 1
            }
        ));
    }

    #[tokio::test]
    async fn test_share_ratio() {
        let server = MockServer::start_async().await;
        let vault = Address::repeat_byte(0x11);
        let asset = Address::repeat_byte(0x22);
        let converted_assets = U256::from(2_500_000u64);

        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", vault))
                .body_contains(encode_prefixed(erc4626DecimalsCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(erc4626DecimalsCall::abi_encode_returns(&18u8)),
            }));
        });
        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", vault))
                .body_contains(encode_prefixed(assetCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(assetCall::abi_encode_returns(&asset)),
            }));
        });
        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", asset))
                .body_contains(encode_prefixed(erc20DecimalsCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(erc20DecimalsCall::abi_encode_returns(&6u8)),
            }));
        });
        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", vault))
                .body_contains(encode_prefixed(convertToAssetsCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(convertToAssetsCall::abi_encode_returns(&converted_assets)),
            }));
        });

        let erc4626 = ERC4626::new(vec![rpc_url(&server)], vault);
        let ratio = erc4626.share_ratio().await.unwrap();

        assert_eq!(ratio.share_token_address, vault);
        assert_eq!(ratio.share_token_decimals, 18);
        assert_eq!(ratio.asset_address, asset);
        assert_eq!(ratio.asset_decimals, 6);
        assert_eq!(ratio.shares, U256::from(10).pow(U256::from(18)));
        assert_eq!(ratio.shares_display, "1");
        assert_eq!(ratio.assets, converted_assets);
        assert_eq!(ratio.assets_display, "2.5");
    }

    #[tokio::test]
    async fn test_convert_assets_to_shares() {
        let server = MockServer::start_async().await;
        let vault = Address::repeat_byte(0x33);
        let asset = Address::repeat_byte(0x44);
        let assets = U256::from(1_500_000u64);
        let converted_shares = U256::from(3) * U256::from(10).pow(U256::from(18));

        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", vault))
                .body_contains(encode_prefixed(erc4626DecimalsCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(erc4626DecimalsCall::abi_encode_returns(&18u8)),
            }));
        });
        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", vault))
                .body_contains(encode_prefixed(assetCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(assetCall::abi_encode_returns(&asset)),
            }));
        });
        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", asset))
                .body_contains(encode_prefixed(erc20DecimalsCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(erc20DecimalsCall::abi_encode_returns(&6u8)),
            }));
        });
        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(format!("{:#x}", vault))
                .body_contains(encode_prefixed(convertToSharesCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(convertToSharesCall::abi_encode_returns(&converted_shares)),
            }));
        });

        let erc4626 = ERC4626::new(vec![rpc_url(&server)], vault);
        let conversion = erc4626.convert_assets_to_shares(assets).await.unwrap();

        assert_eq!(conversion.shares, converted_shares);
        assert_eq!(conversion.shares_display, "3");
        assert_eq!(conversion.assets, assets);
        assert_eq!(conversion.assets_display, "1.5");
    }

    #[tokio::test]
    async fn test_share_decimals_malformed_abi_response() {
        let server = MockServer::start_async().await;

        server.mock(|when, then| {
            when.method("POST")
                .path("/rpc")
                .body_contains(encode_prefixed(erc4626DecimalsCall::SELECTOR));
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x1",
            }));
        });

        let erc4626 = ERC4626::new(vec![rpc_url(&server)], Address::ZERO);
        assert!(erc4626.share_decimals().await.is_err());
    }
}
