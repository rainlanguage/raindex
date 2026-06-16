use alloy::primitives::{ruint::FromUintError, Address};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::SolCall;
use alloy::transports::TransportError;
use rain_math_float::FloatError;
use raindex_bindings::provider::{mk_read_provider, ReadProviderError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[cfg(not(target_family = "wasm"))]
use alloy::{
    network::AnyNetwork,
    providers::{ProviderBuilder, WalletProvider},
    signers::ledger::{HDPath, LedgerError, LedgerSigner},
};

#[cfg(not(target_family = "wasm"))]
use crate::write_tx::WriteTransactionError;

#[derive(Error, Debug)]
pub enum WritableTransactionExecuteError {
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    WriteTransaction(#[from] WriteTransactionError),
    #[error(transparent)]
    TransactionArgs(#[from] TransactionArgsError),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
    #[error(transparent)]
    FloatError(#[from] FloatError),
}

#[derive(Error, Debug)]
pub enum TransactionArgsError {
    #[error(transparent)]
    ChainIdParse(#[from] FromUintError<u64>),
    #[error("Chain ID is required, but set to None")]
    ChainIdNone,
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    ReadProvider(#[from] ReadProviderError),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
    #[error("ABI decode error: {0}")]
    AbiDecode(String),
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TransactionArgs {
    pub raindex_address: Address,
    pub derivation_index: Option<usize>,
    pub chain_id: Option<u64>,
    pub rpcs: Vec<String>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub max_fee_per_gas: Option<u128>,
}

impl TransactionArgs {
    pub fn try_into_transaction_request<T: SolCall>(
        &self,
        call: T,
        contract: Address,
    ) -> Result<TransactionRequest, TransactionArgsError> {
        let mut tx = TransactionRequest::default()
            .to(contract)
            .input(call.abi_encode().into());
        if let Some(max_priority) = self.max_priority_fee_per_gas {
            tx = tx.max_priority_fee_per_gas(max_priority);
        }
        if let Some(max_fee) = self.max_fee_per_gas {
            tx = tx.max_fee_per_gas(max_fee);
        }
        Ok(tx)
    }

    pub async fn try_fill_chain_id(&mut self) -> Result<(), TransactionArgsError> {
        if self.chain_id.is_none() {
            self.chain_id = Some(read_chain_id(&self.rpcs).await?);
        }
        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn try_into_ledger_client(
        self,
    ) -> Result<
        (
            impl Provider<AnyNetwork> + WalletProvider<AnyNetwork> + Clone + std::fmt::Debug,
            Address,
        ),
        TransactionArgsError,
    > {
        let mut err: Option<TransactionArgsError> = None;
        if self.rpcs.is_empty() {
            return Err(TransactionArgsError::InvalidArgs(
                "rpcs cannot be empty".into(),
            ));
        }

        let derivation_index = self.derivation_index.unwrap_or(0);

        for rpc in self.rpcs.clone() {
            let signer =
                LedgerSigner::new(HDPath::LedgerLive(derivation_index), self.chain_id).await;

            match signer {
                Ok(signer) => {
                    let address = signer.get_address().await?;

                    let url: url::Url = rpc.parse()?;
                    let provider = ProviderBuilder::new_with_network::<AnyNetwork>()
                        .wallet(signer)
                        .connect_http(url);

                    return Ok((provider, address));
                }
                Err(e) => {
                    err = Some(TransactionArgsError::Ledger(e));
                }
            }
        }

        Err(err.unwrap())
    }
}

pub async fn read_chain_id(rpcs: &[String]) -> Result<u64, TransactionArgsError> {
    let urls = rpcs
        .iter()
        .map(|s| s.parse::<Url>())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = mk_read_provider(&urls)?;
    Ok(provider.get_chain_id().await?)
}

pub async fn read_block_number(rpcs: &[String]) -> Result<u64, TransactionArgsError> {
    let urls = rpcs
        .iter()
        .map(|s| s.parse::<Url>())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = mk_read_provider(&urls)?;
    Ok(provider.get_block_number().await?)
}

pub async fn read_call<C: SolCall>(
    rpcs: &[String],
    contract: Address,
    call: C,
) -> Result<C::Return, TransactionArgsError> {
    let urls = rpcs
        .iter()
        .map(|s| s.parse::<Url>())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = mk_read_provider(&urls)?;
    let tx = TransactionRequest::default()
        .to(contract)
        .input(call.abi_encode().into());
    let bytes = provider.call(tx.into()).await?;
    C::abi_decode_returns(&bytes).map_err(|e| TransactionArgsError::AbiDecode(e.to_string()))
}
