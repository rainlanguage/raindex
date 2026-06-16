use alloy::primitives::{ruint::FromUintError, Address};
use alloy::sol_types::SolCall;
use rain_math_float::FloatError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(not(target_family = "wasm"))]
use alloy::{
    network::AnyNetwork,
    providers::{Provider, ProviderBuilder, WalletProvider},
    signers::ledger::{HDPath, LedgerError, LedgerSigner},
};
use alloy_ethers_typecast::{
    ReadableClient, ReadableClientError, WritableClientError, WriteContractParameters,
    WriteContractParametersBuilder, WriteContractParametersBuilderError,
};

#[derive(Error, Debug)]
pub enum WritableTransactionExecuteError {
    #[error(transparent)]
    WritableClient(#[from] WritableClientError),
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
    BuildParameters(#[from] WriteContractParametersBuilderError),
    #[error(transparent)]
    ChainIdParse(#[from] FromUintError<u64>),
    #[error("Chain ID is required, but set to None")]
    ChainIdNone,
    #[error(transparent)]
    ReadableClient(#[from] ReadableClientError),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TransactionArgs {
    pub orderbook_address: Address,
    pub derivation_index: Option<usize>,
    pub chain_id: Option<u64>,
    pub rpcs: Vec<String>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub max_fee_per_gas: Option<u128>,
}

impl TransactionArgs {
    pub fn try_into_write_contract_parameters<T: SolCall + Clone>(
        &self,
        call: T,
        contract: Address,
    ) -> Result<WriteContractParameters<T>, TransactionArgsError> {
        let params = WriteContractParametersBuilder::default()
            .address(contract)
            .call(call)
            .max_priority_fee_per_gas(self.max_priority_fee_per_gas)
            .max_fee_per_gas(self.max_fee_per_gas)
            .build()?;

        Ok(params)
    }

    pub async fn try_fill_chain_id(&mut self) -> Result<(), TransactionArgsError> {
        if self.chain_id.is_none() {
            let chain_id = ReadableClient::new_from_http_urls(self.rpcs.clone())?
                .get_chainid()
                .await?;

            self.chain_id = Some(chain_id);
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

        // if we are here, we have tried all rpcs and failed
        Err(err.unwrap())
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use alloy::primitives::{address, B256, U256};
    use httpmock::MockServer;

    use super::*;
    use rain_orderbook_bindings::IRaindexV6::vaultBalance2Call;

    #[test]
    fn test_try_into_write_contract_parameters_ok() {
        let args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpcs: vec!["https://mainnet.infura.io/v3/your-api-key".to_string()],
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        };

        let call = vaultBalance2Call {
            owner: Address::ZERO,
            token: Address::ZERO,
            vaultId: B256::ZERO,
        };

        let params = args
            .try_into_write_contract_parameters(call.clone(), Address::ZERO)
            .unwrap();

        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, call);
        assert_eq!(params.max_priority_fee_per_gas, None);
        assert_eq!(params.max_fee_per_gas, None);

        let args = TransactionArgs {
            orderbook_address: address!("123abcdef24Ca5003905aA834De7156C68b2E1d0"),
            derivation_index: Some(0),
            chain_id: Some(1),
            rpcs: vec!["https://mainnet.infura.io/v3/your-api-key".to_string()],
            max_priority_fee_per_gas: Some(100),
            max_fee_per_gas: Some(200),
        };

        let call = vaultBalance2Call {
            owner: address!("b20a608c624Ca5003905aA834De7156C68b2E1d0"),
            token: address!("00000000219ab540356cBB839Cbe05303d7705Fa"),
            vaultId: B256::from(U256::from(123456)),
        };

        let params = args
            .try_into_write_contract_parameters(
                call.clone(),
                address!("0000000000000000000000000123456789abcdef"),
            )
            .unwrap();

        assert_eq!(
            params.address,
            address!("0000000000000000000000000123456789abcdef")
        );
        assert_eq!(params.call, call);
        assert_eq!(params.max_priority_fee_per_gas, Some(100));
        assert_eq!(params.max_fee_per_gas, Some(200));
    }

    #[tokio::test]
    async fn test_try_fill_chain_id_ok() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc").body_contains("eth_chainId");

            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{ "jsonrpc": "2.0", "id": 1, "result": "0x1" }"#);
        });

        let mut args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpcs: vec![server.url("/rpc")],
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        };

        args.try_fill_chain_id().await.unwrap();
        assert_eq!(args.chain_id, Some(1));

        // the URL is invalid but it shouldn't be used now that chain ID is set
        args.rpcs = vec!["".to_string()];
        args.try_fill_chain_id().await.unwrap();
        assert_eq!(args.chain_id, Some(1));
    }

    #[tokio::test]
    async fn test_try_fill_chain_id_err() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc").body_contains("eth_chainId");
            then.status(500);
        });

        let mut args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpcs: vec![server.url("/rpc")],
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        };

        let err = args.try_fill_chain_id().await.unwrap_err();
        assert!(
            matches!(
                &err,
                TransactionArgsError::ReadableClient(ReadableClientError::AllProvidersFailed(ref msg))
                if msg.get(&args.rpcs[0]).is_some()
                    && matches!(
                        msg.get(&args.rpcs[0]).unwrap(),
                        ReadableClientError::ReadChainIdError(_)
                    )
            ),
            "unexpected error variant: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_try_into_ledger_client_err() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.path("/rpc").body_contains("eth_chainId");
            then.status(200)
                .body(r#"{ "jsonrpc": "2.0", "id": 1, "result": "0x1" }"#);
        });

        let args = TransactionArgs {
            orderbook_address: Address::ZERO,
            derivation_index: None,
            chain_id: None,
            rpcs: vec![server.url("/rpc")],
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        };

        let result = args.clone().try_into_ledger_client().await;
        assert!(result.is_err());
    }

    // NOTE: `alloy` ignores all ledger tests so it seems like there is no way
    // to mock a device. hence there is only a test case for a scenario that
    // should fail regardless of whether a ledger is connected or not
}
