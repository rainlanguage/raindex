use async_trait::async_trait;
#[cfg(target_family = "wasm")]
use std::sync::Arc;

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

    /// Execute a data-only dump. Backends can override this to avoid building
    /// one owned statement for every line of the dump.
    #[cfg(target_family = "wasm")]
    async fn execute_sql_dump(&self, dump: Arc<String>) -> Result<(), LocalDbQueryError> {
        let statements: Vec<_> = dump.lines().map(SqlStatement::new).collect();
        self.execute_batch(&SqlStatementBatch::from(statements))
            .await
    }

    async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson;

    async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError>;

    async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError>;
}
