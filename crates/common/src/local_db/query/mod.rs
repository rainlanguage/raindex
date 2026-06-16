pub mod clear_orderbook_data;
pub mod clear_tables;
pub mod create_tables;
pub mod create_views;
pub mod executor;
pub mod fetch_all_tokens;
pub mod fetch_db_metadata;
pub mod fetch_erc20_tokens_by_addresses;
pub mod fetch_last_synced_block;
pub mod fetch_order_trades;
pub mod fetch_order_trades_count;
pub mod fetch_order_vaults_volume;
pub mod fetch_orders;
pub mod fetch_owner_trades;
pub mod fetch_owner_trades_count;
pub mod fetch_store_addresses;
pub mod fetch_tables;
pub mod fetch_target_watermark;
pub mod fetch_trades_by_tx;
pub mod fetch_transaction_by_hash;
pub mod fetch_vault_balance_changes;
pub mod fetch_vaults;
pub mod insert_db_metadata;
pub mod integrity_check;
pub mod sql_statement;
pub mod sql_statement_batch;
pub mod update_last_synced_block;
pub mod upsert_target_watermark;
pub mod upsert_vault_balances;

pub use executor::LocalDbQueryExecutor;
pub use sql_statement::{SqlBuildError, SqlStatement, SqlValue};
pub use sql_statement_batch::SqlStatementBatch;

use serde::de::DeserializeOwned;
use thiserror::Error;

/// Backend-neutral error representing failures when executing SQL against the
/// embedded local database. Specific backends should map their native error
/// types into this structure.
#[derive(Debug, Error, Clone)]
pub enum LocalDbQueryError {
    #[error("Database operation failed: {message}")]
    Database { message: String },

    #[error("Invalid response format from database")]
    InvalidResponse,

    #[error("Deserialization failed: {message}")]
    Deserialization { message: String },

    #[error("SQL build failed: {source}")]
    SqlBuild { source: SqlBuildError },

    #[error("Operation not implemented: {operation}")]
    NotImplemented { operation: String },
}

impl LocalDbQueryError {
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn invalid_response() -> Self {
        Self::InvalidResponse
    }

    pub fn deserialization(message: impl Into<String>) -> Self {
        Self::Deserialization {
            message: message.into(),
        }
    }

    pub fn not_implemented(operation: impl Into<String>) -> Self {
        Self::NotImplemented {
            operation: operation.into(),
        }
    }
}

impl From<SqlBuildError> for LocalDbQueryError {
    fn from(e: SqlBuildError) -> Self {
        LocalDbQueryError::SqlBuild { source: e }
    }
}

/// Helper trait for types that can be deserialized from DB JSON emitted by the
/// local database backend. Implementors must be deserializable from JSON.
pub trait FromDbJson: DeserializeOwned {}

impl<T> FromDbJson for T where T: DeserializeOwned {}
