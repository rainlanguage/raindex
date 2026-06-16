use crate::allowance::read_allowance;
use crate::transaction::{TransactionArgs, TransactionArgsError, WritableTransactionExecuteError};
use alloy::primitives::{Address, B256, U256};
use rain_math_float::{Float, FloatError};
use raindex_bindings::IRaindexV6::deposit4Call;
#[cfg(not(target_family = "wasm"))]
use raindex_bindings::IERC20::approveCall;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(not(target_family = "wasm"))]
use crate::write_tx::{execute_write_tx, WriteTransactionStatus};

#[derive(Error, Debug)]
pub enum DepositError {
    #[error(transparent)]
    WritableTransactionExecuteError(#[from] WritableTransactionExecuteError),

    #[error(transparent)]
    TransactionArgsError(#[from] TransactionArgsError),

    #[error(transparent)]
    FloatError(#[from] FloatError),

    #[error("vault-id 0 is vaultless and cannot be used for deposits")]
    ZeroVaultId,

    #[cfg(not(target_family = "wasm"))]
    #[error(transparent)]
    WriteTransactionError(#[from] crate::write_tx::WriteTransactionError),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DepositArgs {
    pub token: Address,
    pub vault_id: B256,
    pub amount: Float,
    pub decimals: u8,
}

impl TryFrom<DepositArgs> for deposit4Call {
    type Error = DepositError;

    fn try_from(val: DepositArgs) -> Result<Self, Self::Error> {
        if val.vault_id == B256::ZERO {
            return Err(DepositError::ZeroVaultId);
        }

        Ok(deposit4Call {
            token: val.token,
            vaultId: val.vault_id,
            depositAmount: val.amount.get_inner(),
            tasks: vec![],
        })
    }
}

impl DepositArgs {
    pub async fn read_allowance(
        &self,
        owner: Address,
        transaction_args: TransactionArgs,
    ) -> Result<U256, DepositError> {
        let res = read_allowance(
            &transaction_args.rpcs,
            self.token,
            owner,
            transaction_args.raindex_address,
        )
        .await?;
        Ok(res)
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn execute_approve<S: Fn(WriteTransactionStatus)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), DepositError> {
        let (ledger_client, address) = transaction_args.clone().try_into_ledger_client().await?;

        let current_allowance = self
            .read_allowance(address, transaction_args.clone())
            .await?;
        let current_allowance_float = Float::from_fixed_decimal(current_allowance, self.decimals)?;

        if !current_allowance_float.eq(self.amount)? {
            let approve_call = approveCall {
                spender: transaction_args.raindex_address,
                amount: self.amount.to_fixed_decimal(self.decimals)?,
            };
            let tx_request =
                transaction_args.try_into_transaction_request(approve_call, self.token)?;

            execute_write_tx(ledger_client, tx_request, 4, transaction_status_changed).await?;
        }

        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn execute_deposit<S: Fn(WriteTransactionStatus)>(
        &self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), DepositError> {
        let deposit_call: deposit4Call = self.clone().try_into()?;
        let (ledger_client, _) = transaction_args.clone().try_into_ledger_client().await?;

        let tx_request = transaction_args
            .try_into_transaction_request(deposit_call, transaction_args.raindex_address)?;

        execute_write_tx(ledger_client, tx_request, 4, transaction_status_changed).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_call_rejects_zero_vault_id() {
        let args = DepositArgs {
            token: Address::ZERO,
            vault_id: B256::ZERO,
            amount: Float::parse("1".to_string()).unwrap(),
            decimals: 18,
        };

        assert!(matches!(
            deposit4Call::try_from(args),
            Err(DepositError::ZeroVaultId)
        ));
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_execute_deposit_rejects_zero_vault_id_before_transaction_setup() {
        let args = DepositArgs {
            token: Address::ZERO,
            vault_id: B256::ZERO,
            amount: Float::parse("1".to_string()).unwrap(),
            decimals: 18,
        };

        assert!(matches!(
            args.execute_deposit(TransactionArgs::default(), |_| {})
                .await,
            Err(DepositError::ZeroVaultId)
        ));
    }
}
