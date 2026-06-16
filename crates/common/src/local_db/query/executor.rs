use async_trait::async_trait;

use super::{FromDbJson, LocalDbQueryError, SqlStatement, SqlStatementBatch};

/// Backend-neutral executor for running SQL against the local DB backend.
///
/// Implementations provide text and JSON query methods that map backend
/// errors into `LocalDbQueryError` and deserialize JSON into target types
/// via the `FromDbJson` bound.
#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
pub trait LocalDbQueryExecutor {
    async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError>;

    async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson;

    async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError>;

    async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError>;
}
