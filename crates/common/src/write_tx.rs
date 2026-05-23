use alloy::network::{AnyNetwork, AnyReceiptEnvelope};
use alloy::providers::{Provider, WalletProvider};
use alloy::rpc::types::{Log, TransactionReceipt, TransactionRequest};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum WriteTransactionStatus {
    PendingPrepare(Box<TransactionRequest>),
    PendingSign(Box<TransactionRequest>),
    Sending,
    Confirmed(Box<TransactionReceipt<AnyReceiptEnvelope<Log>>>),
}

#[derive(Debug, Error)]
pub enum WriteTransactionError {
    #[error("Failed to send transaction: {0}")]
    Send(String),
    #[error("Failed to confirm transaction: {0}")]
    Confirmation(String),
    #[error("Transaction reverted")]
    Reverted,
}

pub async fn execute_write_tx<
    P: Provider<AnyNetwork> + WalletProvider<AnyNetwork> + Clone,
    F: Fn(WriteTransactionStatus),
>(
    provider: P,
    tx_request: TransactionRequest,
    confirmations: u8,
    status_changed: F,
) -> Result<(), WriteTransactionError> {
    status_changed(WriteTransactionStatus::PendingPrepare(Box::new(
        tx_request.clone(),
    )));
    status_changed(WriteTransactionStatus::PendingSign(Box::new(
        tx_request.clone(),
    )));

    let pending_tx = provider
        .send_transaction(tx_request)
        .await
        .map_err(|e| WriteTransactionError::Send(e.to_string()))?
        .with_required_confirmations(u64::from(confirmations));

    status_changed(WriteTransactionStatus::Sending);

    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| WriteTransactionError::Confirmation(e.to_string()))?;

    if !receipt.inner.inner.status() {
        return Err(WriteTransactionError::Reverted);
    }

    status_changed(WriteTransactionStatus::Confirmed(Box::new(receipt.inner)));
    Ok(())
}
