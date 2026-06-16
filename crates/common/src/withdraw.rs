#[cfg(not(target_family = "wasm"))]
use crate::transaction::TransactionArgs;
use crate::transaction::WritableTransactionExecuteError;
#[cfg(not(target_family = "wasm"))]
use crate::write_tx::{execute_write_tx, WriteTransactionStatus};
use alloy::primitives::{Address, B256};
use alloy::sol_types::SolCall;
use serde::{Deserialize, Serialize};

use rain_math_float::Float;
use raindex_bindings::IRaindexV6::withdraw4Call;
use std::convert::TryFrom;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct WithdrawArgs {
    pub token: Address,
    pub vault_id: B256,
    pub target_amount: Float,
}

impl TryFrom<WithdrawArgs> for withdraw4Call {
    type Error = WritableTransactionExecuteError;

    fn try_from(val: WithdrawArgs) -> Result<Self, Self::Error> {
        if val.vault_id == B256::ZERO {
            return Err(WritableTransactionExecuteError::InvalidArgs(
                "vault-id 0 is vaultless and cannot be used for withdrawals".to_string(),
            ));
        }

        Ok(withdraw4Call {
            token: val.token,
            vaultId: val.vault_id,
            targetAmount: val.target_amount.get_inner(),
            tasks: vec![],
        })
    }
}

impl WithdrawArgs {
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), WritableTransactionExecuteError> {
        let withdraw_call: withdraw4Call = self.clone().try_into()?;
        let (ledger_client, _) = transaction_args.clone().try_into_ledger_client().await?;

        let tx_request = transaction_args
            .try_into_transaction_request(withdraw_call, transaction_args.raindex_address)?;

        execute_write_tx(ledger_client, tx_request, 4, transaction_status_changed).await?;

        Ok(())
    }

    pub async fn get_withdraw_calldata(&self) -> Result<Vec<u8>, WritableTransactionExecuteError> {
        let withdraw_call: withdraw4Call = self.clone().try_into()?;
        Ok(withdraw_call.abi_encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_withdraw_call_rejects_zero_vault_id() {
        let args = WithdrawArgs {
            token: Address::ZERO,
            vault_id: B256::ZERO,
            target_amount: Float::parse("1".to_string()).unwrap(),
        };

        let err = withdraw4Call::try_from(args).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Invalid input args: vault-id 0 is vaultless and cannot be used for withdrawals"
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_execute_rejects_zero_vault_id_before_transaction_setup() {
        let args = WithdrawArgs {
            token: Address::ZERO,
            vault_id: B256::ZERO,
            target_amount: Float::parse("1".to_string()).unwrap(),
        };

        let err = args
            .execute(TransactionArgs::default(), |_| {})
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "Invalid input args: vault-id 0 is vaultless and cannot be used for withdrawals"
        );
    }
}
