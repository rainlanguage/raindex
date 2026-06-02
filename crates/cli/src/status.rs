use raindex_common::write_tx::WriteTransactionStatus;
use tracing::info;

pub fn display_write_transaction_status(status: WriteTransactionStatus) {
    match status {
        WriteTransactionStatus::PendingPrepare(_) => {
            info!("⏳  Preparing transaction. Please wait.");
        }
        WriteTransactionStatus::PendingSign(_) => {
            info!("🖋   Please sign the transaction on your Ledger device.");
        }
        WriteTransactionStatus::Sending => {
            info!("⏳  Awaiting transaction confirmation. Please wait.");
        }
        WriteTransactionStatus::Confirmed(receipt) => {
            info!("✅  Transaction confirmed: {:?}", receipt.transaction_hash);
        }
    }
}
