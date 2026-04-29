use crate::transaction::{TransactionArgs, TransactionArgsError, WritableTransactionExecuteError};
use alloy::primitives::{Address, B256, U256};
use alloy_ethers_typecast::{
    ReadContractParametersBuilder, ReadContractParametersBuilderError, ReadableClient,
    ReadableClientError, WritableClientError,
};
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::{WriteTransaction, WriteTransactionStatus};
use rain_math_float::{Float, FloatError};
#[cfg(not(target_family = "wasm"))]
use rain_orderbook_bindings::IERC20::approveCall;
use rain_orderbook_bindings::{IRaindexV6::deposit4Call, IERC20::allowanceCall};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepositError {
    #[error(transparent)]
    ReadableClientError(#[from] ReadableClientError),

    #[error(transparent)]
    ReadContractParametersBuilderError(#[from] ReadContractParametersBuilderError),

    #[error(transparent)]
    WritableClientError(#[from] WritableClientError),

    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),

    #[error(transparent)]
    TransactionArgsError(#[from] TransactionArgsError),

    #[error(transparent)]
    FloatError(#[from] FloatError),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DepositArgs {
    pub token: Address,
    pub vault_id: B256,
    pub amount: Float,
    pub decimals: u8,
}

impl TryFrom<DepositArgs> for deposit4Call {
    type Error = FloatError;

    fn try_from(val: DepositArgs) -> Result<Self, Self::Error> {
        let call = deposit4Call {
            token: val.token,
            vaultId: val.vault_id,
            depositAmount: val.amount.get_inner(),
            tasks: vec![],
        };

        Ok(call)
    }
}

impl DepositArgs {
    /// Execute read IERC20 allowance call
    pub async fn read_allowance(
        &self,
        owner: Address,
        transaction_args: TransactionArgs,
    ) -> Result<U256, DepositError> {
        let readable_client = ReadableClient::new_from_http_urls(transaction_args.rpcs.clone())?;
        let parameters = ReadContractParametersBuilder::<allowanceCall>::default()
            .address(self.token)
            .call(allowanceCall {
                owner,
                spender: transaction_args.orderbook_address,
            })
            .build()?;
        let res = readable_client.read(parameters).await?;

        Ok(res)
    }

    /// Execute IERC20 approve call
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute_approve<S: Fn(WriteTransactionStatus<approveCall>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), DepositError> {
        let (ledger_client, address) = transaction_args.clone().try_into_ledger_client().await?;

        // Check allowance already granted for this token and contract
        let current_allowance = self
            .read_allowance(address, transaction_args.clone())
            .await?;
        let current_allowance_float = Float::from_fixed_decimal(current_allowance, self.decimals)?;

        // If allowance differs from desired amount, overwrite it with the target value
        if !current_allowance_float.eq(self.amount)? {
            let approve_call = approveCall {
                spender: transaction_args.orderbook_address,
                amount: self.amount.to_fixed_decimal(self.decimals)?,
            };
            let params =
                transaction_args.try_into_write_contract_parameters(approve_call, self.token)?;

            WriteTransaction::new(ledger_client, params, 4, transaction_status_changed)
                .execute()
                .await?;
        }

        Ok(())
    }

    /// Execute OrderbookV3 deposit call
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute_deposit<S: Fn(WriteTransactionStatus<deposit4Call>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), DepositError> {
        let (ledger_client, _) = transaction_args.clone().try_into_ledger_client().await?;

        let deposit_call: deposit4Call = self.clone().try_into()?;
        let params = transaction_args
            .try_into_write_contract_parameters(deposit_call, transaction_args.orderbook_address)?;

        WriteTransaction::new(ledger_client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::primitives::{address, Address, B256};
    use httpmock::MockServer;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn test_deposit_args_into() {
        let args = DepositArgs {
            token: address!("1234567890abcdef1234567890abcdef12345678"),
            vault_id: B256::from(U256::from(42)),
            amount: Float::from_fixed_decimal(U256::from(123), 6).unwrap(),
            decimals: 6,
        };

        let deposit_call: deposit4Call = args.try_into().unwrap();

        assert_eq!(
            deposit_call.token,
            address!("1234567890abcdef1234567890abcdef12345678")
        );
        assert_eq!(deposit_call.vaultId, B256::from(U256::from(42)));
        let amount = Float::parse("0.000123".to_string()).unwrap().get_inner();
        assert_eq!(deposit_call.depositAmount, amount);
    }

    #[tokio::test]
    async fn test_read_allowance() {
        let rpc_server = MockServer::start_async().await;

        rpc_server.mock(|when, then| {
            when.path("/rpc").body_contains("0xdd62ed3e");
            let value = B256::left_padding_from(&[200u8]).to_string();
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": value,
            }));
        });

        let args = DepositArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: B256::from(U256::from(42)),
            amount: Float::from_fixed_decimal(U256::from(100), 18).unwrap(),
            decimals: 18,
        };

        let res = args
            .read_allowance(
                Address::ZERO,
                TransactionArgs {
                    rpcs: vec![rpc_server.url("/rpc")],
                    orderbook_address: Address::ZERO,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(res, U256::from(200));
    }

    #[test]
    fn test_deposit_call_try_into_write_contract_parameters() {
        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            orderbook_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(200),
            max_fee_per_gas: Some(100),
        };

        let amount = Float::parse("100".to_string()).unwrap().get_inner();
        let deposit_call = deposit4Call {
            token: Address::ZERO,
            vaultId: B256::from(U256::from(42)),
            depositAmount: amount,
            tasks: vec![],
        };
        let params = args
            .try_into_write_contract_parameters(deposit_call.clone(), Address::ZERO)
            .unwrap();
        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, deposit_call);
        assert_eq!(params.max_priority_fee_per_gas, Some(200));
        assert_eq!(params.max_fee_per_gas, Some(100));
    }

    #[test]
    fn test_approve_call_try_into_write_contract_parameters() {
        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            orderbook_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(200),
            max_fee_per_gas: Some(100),
        };
        let approve_call = approveCall {
            spender: Address::ZERO,
            amount: U256::from(100),
        };
        let params = args
            .try_into_write_contract_parameters(approve_call.clone(), Address::ZERO)
            .unwrap();
        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, approve_call);
        assert_eq!(params.max_priority_fee_per_gas, Some(200));
        assert_eq!(params.max_fee_per_gas, Some(100));
    }
}
