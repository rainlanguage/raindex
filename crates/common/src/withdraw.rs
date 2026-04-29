#[cfg(not(target_family = "wasm"))]
use crate::transaction::TransactionArgs;
use crate::transaction::WritableTransactionExecuteError;
use alloy::primitives::{Address, B256};
use alloy::sol_types::SolCall;
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::{WriteTransaction, WriteTransactionStatus};
use serde::{Deserialize, Serialize};

use rain_math_float::Float;
use rain_orderbook_bindings::IRaindexV6::withdraw4Call;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct WithdrawArgs {
    pub token: Address,
    pub vault_id: B256,
    pub target_amount: Float,
}

impl From<WithdrawArgs> for withdraw4Call {
    fn from(val: WithdrawArgs) -> Self {
        withdraw4Call {
            token: val.token,
            vaultId: val.vault_id,
            targetAmount: val.target_amount.get_inner(),
            tasks: vec![],
        }
    }
}

impl WithdrawArgs {
    /// Execute OrderbookV3 withdraw call
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<withdraw4Call>)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let (ledger_client, _) = transaction_args.clone().try_into_ledger_client().await?;

        let withdraw_call: withdraw4Call = self.clone().into();
        let params = transaction_args.try_into_write_contract_parameters(
            withdraw_call,
            transaction_args.orderbook_address,
        )?;

        WriteTransaction::new(ledger_client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_withdraw_calldata(&self) -> Result<Vec<u8>, WritableTransactionExecuteError> {
        let withdraw_call: withdraw4Call = self.clone().into();
        Ok(withdraw_call.abi_encode())
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::primitives::{address, U256};
    use std::str::FromStr;

    #[test]
    fn test_withdraw_args_into() {
        let amount = Float::parse("100".to_string()).unwrap();
        let args = WithdrawArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap(),
            vault_id: B256::from(U256::from(42)),
            target_amount: amount,
        };

        let withdraw_call: withdraw4Call = args.into();
        assert_eq!(
            withdraw_call.token,
            address!("1234567890abcdef1234567890abcdef12345678")
        );
        assert_eq!(withdraw_call.vaultId, B256::from(U256::from(42)));
        assert_eq!(withdraw_call.targetAmount, amount.get_inner());
    }

    #[tokio::test]
    async fn test_get_withdraw_calldata() {
        let amount = Float::parse("100".to_string()).unwrap();
        let args = WithdrawArgs {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vault_id: B256::from(U256::from(42)),
            target_amount: amount,
        };
        let calldata = args.get_withdraw_calldata().await.unwrap();

        let expected_calldata = withdraw4Call {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vaultId: B256::from(U256::from(42)),
            targetAmount: amount.get_inner(),
            tasks: vec![],
        }
        .abi_encode();

        assert_eq!(calldata, expected_calldata);
        assert_eq!(calldata.len(), 164);
    }

    #[test]
    fn test_withdraw_call_try_into_write_contract_parameters() {
        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            orderbook_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(200),
            max_fee_per_gas: Some(100),
        };

        let amount = Float::parse("456".to_string()).unwrap().get_inner();
        let withdraw_call = withdraw4Call {
            token: Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
            vaultId: B256::from(U256::from(123)),
            targetAmount: amount,
            tasks: vec![],
        };
        let params = args
            .try_into_write_contract_parameters(withdraw_call.clone(), Address::ZERO)
            .unwrap();
        assert_eq!(params.address, Address::ZERO);
        assert_eq!(params.call, withdraw_call);
        assert_eq!(params.max_priority_fee_per_gas, Some(200));
        assert_eq!(params.max_fee_per_gas, Some(100));
    }
}
