use crate::local_db::{
    pipeline::adapters::bootstrap::{BootstrapConfig, BootstrapPipeline, BootstrapState},
    query::{
        create_tables::{required_table_schema, REQUIRED_TABLES},
        create_views::create_views_batch,
        fetch_table_columns::{fetch_table_schema_columns_stmt, TableSchemaColumnResponse},
        fetch_tables::{fetch_tables_stmt, TableResponse},
        fetch_target_watermark::{fetch_target_watermark_stmt, TargetWatermarkRow},
        LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    },
    LocalDbError, RaindexIdentifier,
};
use std::collections::{HashMap, HashSet};

const BOOTSTRAP_CACHE_SIZE_SQL: &str = "PRAGMA cache_size = -25000";

#[derive(Debug, Default, Clone, Copy)]
pub struct ClientBootstrapAdapter;

impl ClientBootstrapAdapter {
    pub fn new() -> Self {
        Self {}
    }

    async fn fetch_existing_tables<DB>(&self, db: &DB) -> Result<HashSet<String>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let existing: Vec<TableResponse> = db.query_json(&fetch_tables_stmt()).await?;
        Ok(existing
            .into_iter()
            .map(|t| t.name.to_ascii_lowercase())
            .collect())
    }

    fn has_required_tables(existing: &HashSet<String>) -> bool {
        REQUIRED_TABLES
            .iter()
            .all(|&t| existing.contains(&t.to_ascii_lowercase()))
    }

    async fn missing_required_schema<DB>(
        &self,
        db: &DB,
        existing_tables: &HashSet<String>,
    ) -> Result<Vec<String>, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let required_schema = required_table_schema();
        let missing_tables = required_schema
            .iter()
            .filter(|table| !existing_tables.contains(&table.name))
            .map(|table| table.name.clone())
            .collect::<Vec<_>>();
        if !missing_tables.is_empty() {
            return Ok(missing_tables);
        }

        let statement = fetch_table_schema_columns_stmt(
            required_schema.iter().map(|table| table.name.as_str()),
        );
        let actual_columns: Vec<TableSchemaColumnResponse> = db.query_json(&statement).await?;
        let columns_by_table = actual_columns.into_iter().fold(
            HashMap::<String, HashSet<String>>::new(),
            |mut columns_by_table, column| {
                columns_by_table
                    .entry(column.table_name.to_ascii_lowercase())
                    .or_default()
                    .insert(column.name.to_ascii_lowercase());
                columns_by_table
            },
        );

        Ok(required_schema
            .iter()
            .flat_map(|table| {
                table.columns.iter().filter_map(|column| {
                    let present = columns_by_table
                        .get(&table.name)
                        .is_some_and(|columns| columns.contains(column));
                    (!present).then(|| format!("{}.{}", table.name, column))
                })
            })
            .collect())
    }

    async fn has_required_schema_shape<DB>(
        &self,
        db: &DB,
        existing_tables: &HashSet<String>,
    ) -> Result<bool, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        Ok(self
            .missing_required_schema(db, existing_tables)
            .await?
            .is_empty())
    }

    fn check_threshold(
        &self,
        latest_block: u64,
        last_synced_block: Option<u64>,
        block_number_threshold: u32,
    ) -> Result<(), LocalDbError> {
        if let Some(last_block) = last_synced_block {
            let threshold = u64::from(block_number_threshold);
            if latest_block.saturating_sub(last_block) > threshold {
                return Err(LocalDbError::BlockSyncThresholdExceeded {
                    latest_block,
                    last_indexed_block: last_block,
                    threshold,
                });
            }
        }

        Ok(())
    }

    async fn is_fresh_db<E: LocalDbQueryExecutor + ?Sized>(
        self,
        db: &E,
        raindex_id: &RaindexIdentifier,
    ) -> Result<bool, LocalDbError> {
        let rows: Vec<TargetWatermarkRow> = db
            .query_json(&fetch_target_watermark_stmt(raindex_id))
            .await?;
        Ok(rows.is_empty())
    }

    async fn apply_dump<DB>(
        &self,
        db: &DB,
        dump_stmt: &SqlStatementBatch,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        db.query_text(&SqlStatement::new(BOOTSTRAP_CACHE_SIZE_SQL))
            .await?;
        db.execute_batch(dump_stmt).await?;
        Ok(())
    }

    /// Validate a snapshot installed by the caller without scanning or
    /// mutating the whole database. The snapshot transport is responsible for
    /// byte-level authentication; this verifies Raindex's schema contract.
    /// It deliberately performs no writes so target coverage can be validated
    /// before any replaceable objects are refreshed.
    pub async fn validate_preinstalled<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let existing_tables = self.fetch_existing_tables(db).await?;
        let missing_tables = REQUIRED_TABLES
            .iter()
            .filter(|table| !existing_tables.contains(&table.to_ascii_lowercase()))
            .copied()
            .collect::<Vec<_>>();
        if !missing_tables.is_empty() {
            return Err(LocalDbError::InvalidPreinstalledSnapshot {
                reason: format!("missing required tables: {}", missing_tables.join(", ")),
            });
        }

        self.ensure_schema(db, db_schema_version).await?;
        let missing_schema = self.missing_required_schema(db, &existing_tables).await?;
        if !missing_schema.is_empty() {
            return Err(LocalDbError::InvalidPreinstalledSnapshot {
                reason: format!("missing required columns: {}", missing_schema.join(", ")),
            });
        }

        Ok(())
    }

    /// Refresh replaceable objects after all read-only preinstalled snapshot
    /// checks, including configured target coverage, have succeeded.
    pub async fn refresh_preinstalled_views<DB>(&self, db: &DB) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        db.execute_batch(&create_views_batch()).await?;
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl BootstrapPipeline for ClientBootstrapAdapter {
    async fn engine_run<DB>(&self, db: &DB, config: &BootstrapConfig) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let BootstrapState {
            last_synced_block, ..
        } = self.inspect_state(db, &config.raindex_id).await?;

        if let Some(dump_stmt) = config.dump_stmt.as_ref() {
            if self.is_fresh_db(db, &config.raindex_id).await? {
                self.apply_dump(db, dump_stmt).await?;
                return Ok(());
            }

            match self.check_threshold(
                config.latest_block,
                last_synced_block,
                config.block_number_threshold,
            ) {
                Ok(_) => {}
                Err(_) => {
                    self.clear_raindex_data(db, &config.raindex_id).await?;
                    self.apply_dump(db, dump_stmt).await?;
                }
            }
        }

        Ok(())
    }

    async fn runner_run<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        let is_healthy = self.check_integrity(db).await.unwrap_or(false);
        if !is_healthy {
            db.wipe_and_recreate().await?;
            self.reset_db(db, db_schema_version).await?;
            return Ok(());
        }

        let existing_tables = self.fetch_existing_tables(db).await?;

        if !existing_tables.contains("db_metadata") {
            self.reset_db(db, db_schema_version).await?;
            return Ok(());
        }

        if let Err(err) = self.ensure_schema(db, db_schema_version).await {
            if matches!(
                err,
                LocalDbError::MissingDbMetadataRow | LocalDbError::SchemaVersionMismatch { .. }
            ) {
                self.reset_db(db, db_schema_version).await?;
                return Ok(());
            }

            return Err(err);
        }

        if !Self::has_required_tables(&existing_tables)
            || !self.has_required_schema_shape(db, &existing_tables).await?
        {
            self.reset_db(db, db_schema_version).await?;
        } else {
            db.execute_batch(&create_views_batch()).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::*;
    use crate::local_db::query::clear_raindex_data::clear_raindex_data_batch;
    use crate::local_db::query::clear_tables::{clear_tables_batch, vacuum_stmt};
    use crate::local_db::query::create_tables::create_tables_batch;
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use crate::local_db::query::create_views::create_views_batch;
    use crate::local_db::query::fetch_db_metadata::{fetch_db_metadata_stmt, DbMetadataRow};
    use crate::local_db::query::fetch_tables::{fetch_tables_stmt, TableResponse};
    use crate::local_db::query::fetch_target_watermark::{
        fetch_configured_target_watermarks_stmt, fetch_target_watermark_stmt,
        PreinstalledTargetWatermarkRow, TargetWatermarkRow,
    };
    use crate::local_db::query::insert_db_metadata::insert_db_metadata_stmt;
    use crate::local_db::query::FromDbJson;
    use crate::local_db::query::{
        LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    };
    use alloy::primitives::{Address, Bytes};
    use async_trait::async_trait;
    use raindex_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
    use raindex_app_settings::spec_version::SpecVersion;
    use serde_json::json;
    use std::str::FromStr;

    const TEST_BLOCK_NUMBER_THRESHOLD: u32 = 10_000;

    fn preinstalled_settings(
        deployment_block: u64,
    ) -> crate::local_db::pipeline::runner::utils::ParsedRunnerSettings {
        crate::local_db::pipeline::runner::utils::parse_runner_settings(&format!(
            r#"
version: {}
networks:
  test:
    rpcs:
      - https://rpc.example/test
    chain-id: 1
subgraphs:
  test: https://subgraph.example/test
local-db-sync:
  test:
    batch-size: 10
    max-concurrent-batches: 1
    retry-attempts: 1
    retry-delay-ms: 1
    rate-limit-delay-ms: 1
    finality-depth: 1
    bootstrap-block-threshold: 100
    sync-interval-ms: 5000
raindexes:
  test:
    address: 0x1111111111111111111111111111111111111111
    network: test
    subgraph: test
    deployment-block: {deployment_block}
"#,
            SpecVersion::current()
        ))
        .expect("valid remote-free settings")
    }

    fn configured_watermarks_stmt() -> SqlStatement {
        fetch_configured_target_watermarks_stmt(&[RaindexIdentifier::new(
            1,
            Address::repeat_byte(0x11),
        )])
    }

    #[derive(Default)]
    struct MockDb {
        json_map: HashMap<String, String>,
        text_map: HashMap<String, String>,
        calls_json: Mutex<Vec<String>>,
        calls_text: Mutex<Vec<String>>,
    }

    impl MockDb {
        fn with_json(mut self, stmt: &SqlStatement, value: serde_json::Value) -> Self {
            self.json_map
                .insert(stmt.sql().to_string(), value.to_string());
            self
        }
        fn with_text(mut self, stmt: &SqlStatement, value: &str) -> Self {
            self.text_map
                .insert(stmt.sql().to_string(), value.to_string());
            self
        }
        fn with_batch(self, batch: &SqlStatementBatch) -> Self {
            batch
                .statements()
                .iter()
                .fold(self, |db, stmt| db.with_text(stmt, "ok"))
        }
        fn with_reset_batches(self) -> Self {
            self.with_batch(&clear_tables_batch())
                .with_text(&vacuum_stmt(), "ok")
                .with_batch(&create_tables_batch())
        }
        fn calls(&self) -> Vec<String> {
            self.calls_text.lock().unwrap().clone()
        }
        fn json_calls(&self) -> Vec<String> {
            self.calls_json.lock().unwrap().clone()
        }

        fn with_views(self) -> Self {
            create_views_batch()
                .statements()
                .iter()
                .fold(self, |db, stmt| db.with_text(stmt, "ok"))
        }

        fn with_healthy_integrity(self) -> Self {
            use crate::local_db::query::integrity_check::{
                integrity_check_stmt, IntegrityCheckRow,
            };
            let row = IntegrityCheckRow {
                quick_check: "ok".to_string(),
            };
            self.with_json(&integrity_check_stmt(), json!([row]))
        }

        fn with_required_schema_columns(self) -> Self {
            let schema = required_table_schema();
            let rows = schema
                .iter()
                .flat_map(|table| {
                    table.columns.iter().map(|name| TableSchemaColumnResponse {
                        table_name: table.name.clone(),
                        name: name.clone(),
                    })
                })
                .collect::<Vec<_>>();
            self.with_json(
                &fetch_table_schema_columns_stmt(schema.iter().map(|table| table.name.as_str())),
                json!(rows),
            )
        }

        fn with_required_schema_columns_missing(self, table_name: &str, column_name: &str) -> Self {
            let schema = required_table_schema();
            let rows = schema
                .iter()
                .flat_map(|table| {
                    table
                        .columns
                        .iter()
                        .filter(|&name| table.name != table_name || name != column_name)
                        .map(|name| TableSchemaColumnResponse {
                            table_name: table.name.clone(),
                            name: name.clone(),
                        })
                })
                .collect::<Vec<_>>();
            self.with_json(
                &fetch_table_schema_columns_stmt(schema.iter().map(|table| table.name.as_str())),
                json!(rows),
            )
        }
    }

    #[cfg_attr(target_family = "wasm", async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait)]
    impl LocalDbQueryExecutor for MockDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            for stmt in batch {
                let _ = self.query_text(stmt).await?;
            }
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            let sql = stmt.sql();
            self.calls_json.lock().unwrap().push(sql.to_string());
            let Some(body) = self.json_map.get(sql) else {
                return Err(LocalDbQueryError::database("no json for sql"));
            };
            serde_json::from_str::<T>(body)
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            let sql = stmt.sql();
            self.calls_text.lock().unwrap().push(sql.to_string());
            let Some(body) = self.text_map.get(sql) else {
                return Err(LocalDbQueryError::database("no text for sql"));
            };
            Ok(body.clone())
        }

        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            Err(LocalDbQueryError::not_implemented("wipe_and_recreate"))
        }
    }

    fn sample_ob_id() -> RaindexIdentifier {
        RaindexIdentifier {
            chain_id: 1,
            raindex_address: Address::ZERO,
        }
    }

    fn runner_ob_id() -> RaindexIdentifier {
        RaindexIdentifier::new(0, Address::ZERO)
    }

    fn cfg_with_dump(latest_block: u64) -> BootstrapConfig {
        BootstrapConfig {
            raindex_id: sample_ob_id(),
            dump_stmt: Some(SqlStatementBatch::from(vec![SqlStatement::new(
                "--dump-sql",
            )])),
            latest_block,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
            deployment_block: 1,
        }
    }

    fn required_tables_json() -> serde_json::Value {
        serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn table_names_json(names: &[&str]) -> serde_json::Value {
        serde_json::to_value(
            names
                .iter()
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn required_tables_without_db_metadata_json() -> serde_json::Value {
        serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .filter(|&&name| name != "db_metadata")
                .map(|&name| TableResponse {
                    name: name.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn reset_batch_sqls() -> Vec<String> {
        let mut statements = clear_tables_batch().statements().to_vec();
        statements.push(vacuum_stmt());
        statements.extend(create_tables_batch().statements().iter().cloned());
        statements
            .iter()
            .map(|stmt| stmt.sql().to_string())
            .collect()
    }

    fn assert_reset_batches_were_called(calls: &[String]) {
        let expected = reset_batch_sqls();
        for sql in expected {
            assert!(calls.contains(&sql), "missing reset SQL: {sql}");
        }
    }

    fn insert_reset_text_map(text_map: &mut HashMap<String, String>) {
        for sql in reset_batch_sqls() {
            text_map.insert(sql, "ok".to_string());
        }
    }

    fn watermark_row(last_block: u64) -> TargetWatermarkRow {
        TargetWatermarkRow {
            chain_id: sample_ob_id().chain_id,
            raindex_address: sample_ob_id().raindex_address,
            last_block,
            last_hash: Bytes::from_str("0xbeef").unwrap(),
            updated_at: 1,
        }
    }

    #[tokio::test]
    async fn runner_run_resets_when_tables_missing() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = json!([]);
        let db_meta_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([db_meta_row]))
            .with_required_schema_columns()
            .with_reset_batches()
            .with_text(&insert_db_metadata_stmt(DB_SCHEMA_VERSION), "ok")
            .with_views();
        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        let expected_reset = reset_batch_sqls();
        assert_eq!(&calls[..expected_reset.len()], expected_reset.as_slice());
        assert_eq!(
            calls[expected_reset.len()],
            insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()
        );
        assert_eq!(
            &calls[expected_reset.len() + 1..],
            expected_views.as_slice()
        );
    }

    #[tokio::test]
    async fn runner_run_resets_on_missing_db_metadata() {
        let adapter = ClientBootstrapAdapter::new();

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), table_names_json(&["db_metadata"]))
            .with_json(&fetch_db_metadata_stmt(), json!([])) // triggers reset
            .with_reset_batches()
            .with_text(&insert_db_metadata_stmt(DB_SCHEMA_VERSION), "ok")
            .with_views();
        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        assert_reset_batches_were_called(&calls);
        assert!(calls.contains(&insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()));
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
        );
        assert!(
            !db.json_calls().contains(
                &fetch_target_watermark_stmt(&runner_ob_id())
                    .sql()
                    .to_string()
            ),
            "schema recovery should not inspect target watermarks before reset"
        );
    }

    #[tokio::test]
    async fn runner_run_resets_on_missing_db_metadata_table() {
        let adapter = ClientBootstrapAdapter::new();

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(
                &fetch_tables_stmt(),
                required_tables_without_db_metadata_json(),
            )
            .with_reset_batches()
            .with_text(&insert_db_metadata_stmt(DB_SCHEMA_VERSION), "ok")
            .with_views();

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        assert_reset_batches_were_called(&calls);
        assert!(calls.contains(&insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()));
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
        );
        assert!(
            !db.json_calls()
                .contains(&fetch_db_metadata_stmt().sql().to_string()),
            "missing metadata table should reset before querying db_metadata"
        );
        assert!(
            !db.json_calls().contains(
                &fetch_target_watermark_stmt(&runner_ob_id())
                    .sql()
                    .to_string()
            ),
            "missing metadata table should reset before target watermark inspection"
        );
    }

    #[tokio::test]
    async fn runner_run_resets_on_schema_mismatch() {
        let adapter = ClientBootstrapAdapter::new();

        let mismatched_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION + 1,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), table_names_json(&["db_metadata"]))
            .with_json(&fetch_db_metadata_stmt(), json!([mismatched_row]))
            .with_reset_batches()
            .with_text(&insert_db_metadata_stmt(DB_SCHEMA_VERSION), "ok")
            .with_views();

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        assert_reset_batches_were_called(&calls);
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
        );
        assert!(
            !db.json_calls().contains(
                &fetch_target_watermark_stmt(&runner_ob_id())
                    .sql()
                    .to_string()
            ),
            "schema recovery should not inspect target watermarks before reset"
        );
    }

    #[tokio::test]
    async fn runner_run_resets_old_schema_before_watermark_query() {
        let adapter = ClientBootstrapAdapter::new();
        let old_schema_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION - 1,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), table_names_json(&["db_metadata"]))
            .with_json(&fetch_db_metadata_stmt(), json!([old_schema_row]))
            .with_reset_batches()
            .with_text(&insert_db_metadata_stmt(DB_SCHEMA_VERSION), "ok")
            .with_views();

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        assert_reset_batches_were_called(&calls);
        assert!(calls.contains(&insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()));
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
        );
        assert!(
            !db.json_calls().contains(
                &fetch_target_watermark_stmt(&runner_ob_id())
                    .sql()
                    .to_string()
            ),
            "old schema should reset before preparing current-schema watermark SQL"
        );
    }

    #[tokio::test]
    async fn runner_run_is_idempotent_when_schema_ok() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = serde_json::to_value(
            REQUIRED_TABLES
                .iter()
                .map(|&t| TableResponse {
                    name: t.to_string(),
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns()
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]))
            .with_views();

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        assert_eq!(
            db.calls(),
            expected_views,
            "schema-ok bootstrap should still refresh replaceable views"
        );
    }

    #[tokio::test]
    async fn validate_preinstalled_checks_schema_without_full_integrity_scan() {
        let adapter = ClientBootstrapAdapter::new();
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns()
            .with_views();

        adapter
            .validate_preinstalled(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        assert!(!db
            .json_calls()
            .iter()
            .any(|sql| sql.to_ascii_lowercase().contains("quick_check")));
        assert!(db.calls().is_empty(), "validation must be read-only");

        adapter.refresh_preinstalled_views(&db).await.unwrap();
        assert_eq!(db.calls().len(), create_views_batch().statements().len());
    }

    #[tokio::test]
    async fn validate_preinstalled_reports_missing_metadata_as_invalid_snapshot() {
        let adapter = ClientBootstrapAdapter::new();
        let db = MockDb::default().with_json(
            &fetch_tables_stmt(),
            required_tables_without_db_metadata_json(),
        );

        let error = adapter
            .validate_preinstalled(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            LocalDbError::InvalidPreinstalledSnapshot { reason }
                if reason.contains("db_metadata")
        ));
        assert!(!db
            .json_calls()
            .contains(&fetch_db_metadata_stmt().sql().to_string()));
        assert!(
            db.calls().is_empty(),
            "validation must not mutate the snapshot"
        );
    }

    #[tokio::test]
    async fn validate_preinstalled_rejects_invalid_schema_without_resetting_it() {
        let adapter = ClientBootstrapAdapter::new();
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns_missing("target_watermarks", "raindex_address");

        let error = adapter
            .validate_preinstalled(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            LocalDbError::InvalidPreinstalledSnapshot { reason }
                if reason.contains("target_watermarks.raindex_address")
        ));
        assert!(
            db.calls().is_empty(),
            "validation must not mutate the snapshot"
        );
    }

    #[tokio::test]
    async fn rejected_target_coverage_does_not_refresh_views() {
        let adapter = ClientBootstrapAdapter::new();
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns()
            .with_views();

        adapter
            .validate_preinstalled(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let configured = crate::raindex_client::ConfiguredSnapshotTarget {
            raindex_id: RaindexIdentifier::new(1, Address::repeat_byte(0x11)),
            deployment_block: 1,
        };
        let watermarks = Vec::<PreinstalledTargetWatermarkRow>::new();
        crate::raindex_client::ensure_preinstalled_snapshot_covers_targets(
            &[configured],
            &watermarks,
        )
        .unwrap_err();

        assert!(
            db.calls().is_empty(),
            "rejected target coverage must not refresh views"
        );
    }

    #[tokio::test]
    async fn production_initialization_rejects_missing_coverage_without_writes_or_readiness() {
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns()
            .with_json(&configured_watermarks_stmt(), json!([]));
        let readiness = crate::raindex_client::local_db::SyncReadiness::new();
        let statuses = crate::raindex_client::local_db::LocalDbSyncStatusStore::new();

        crate::raindex_client::initialize_local_db_readiness(
            &db,
            &preinstalled_settings(100),
            &readiness,
            &statuses,
            crate::raindex_client::local_db::pipeline::runner::LocalDbProvisioning::PreinstalledSnapshot,
        )
        .await
        .unwrap_err();

        assert!(db.calls().is_empty());
        assert!(!readiness.is_ready(1));
    }

    #[tokio::test]
    async fn production_initialization_rejects_malformed_watermark_without_writes() {
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns()
            .with_json(
                &configured_watermarks_stmt(),
                json!([{
                    "chain_id": 1,
                    "raindex_address": "0x1111111111111111111111111111111111111111",
                    "last_block": 100,
                    "last_hash": null,
                    "updated_at": 1
                }]),
            );

        crate::raindex_client::initialize_local_db_readiness(
            &db,
            &preinstalled_settings(100),
            &crate::raindex_client::local_db::SyncReadiness::new(),
            &crate::raindex_client::local_db::LocalDbSyncStatusStore::new(),
            crate::raindex_client::local_db::pipeline::runner::LocalDbProvisioning::PreinstalledSnapshot,
        )
        .await
        .unwrap_err();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn production_initialization_rejects_watermark_before_deployment_without_writes() {
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns()
            .with_json(
                &configured_watermarks_stmt(),
                json!([PreinstalledTargetWatermarkRow {
                    chain_id: 1,
                    raindex_address: "0x1111111111111111111111111111111111111111".to_string(),
                    last_block: 99,
                    last_hash: Bytes::from(vec![0; 32]),
                    updated_at: 1,
                }]),
            );

        let error = crate::raindex_client::initialize_local_db_readiness(
            &db,
            &preinstalled_settings(100),
            &crate::raindex_client::local_db::SyncReadiness::new(),
            &crate::raindex_client::local_db::LocalDbSyncStatusStore::new(),
            crate::raindex_client::local_db::pipeline::runner::LocalDbProvisioning::PreinstalledSnapshot,
        )
        .await
        .unwrap_err();

        assert!(error.to_string().contains("precedes deployment block"));
        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn production_initialization_ignores_unrelated_malformed_watermark_rows() {
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        // The mock only registers the configured-target query. This models a
        // database that also retains unrelated rows (including NULL/malformed
        // rows): SQLite applies the identity predicate before deserializing the
        // selected configured row. Regressing to an all-row query fails here.
        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns()
            .with_json(
                &configured_watermarks_stmt(),
                json!([PreinstalledTargetWatermarkRow {
                    chain_id: 1,
                    raindex_address: "0x1111111111111111111111111111111111111111".to_string(),
                    last_block: 100,
                    last_hash: Bytes::from(vec![0; 32]),
                    updated_at: 1,
                }]),
            )
            .with_views();
        let readiness = crate::raindex_client::local_db::SyncReadiness::new();

        crate::raindex_client::initialize_local_db_readiness(
            &db,
            &preinstalled_settings(100),
            &readiness,
            &crate::raindex_client::local_db::LocalDbSyncStatusStore::new(),
            crate::raindex_client::local_db::pipeline::runner::LocalDbProvisioning::PreinstalledSnapshot,
        )
        .await
        .unwrap();

        assert_eq!(db.calls().len(), create_views_batch().statements().len());
        assert!(readiness.is_ready(1));
        let watermark_queries = db
            .json_calls()
            .into_iter()
            .filter(|sql| sql.contains("FROM target_watermarks"))
            .collect::<Vec<_>>();
        assert_eq!(watermark_queries.len(), 1);
        assert!(watermark_queries[0].contains("lower(raindex_address)"));
    }

    #[tokio::test]
    async fn runner_run_resets_when_required_column_is_missing() {
        let adapter = ClientBootstrapAdapter::new();
        let db_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), required_tables_json())
            .with_json(&fetch_db_metadata_stmt(), json!([db_row]))
            .with_required_schema_columns_missing("target_watermarks", "raindex_address")
            .with_reset_batches()
            .with_text(&insert_db_metadata_stmt(DB_SCHEMA_VERSION), "ok")
            .with_views();

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        let calls = db.calls();
        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        assert_reset_batches_were_called(&calls);
        assert!(calls.contains(&insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()));
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
        );
        assert!(
            !db.json_calls().contains(
                &fetch_target_watermark_stmt(&runner_ob_id())
                    .sql()
                    .to_string()
            ),
            "invalid schema shape should reset before target watermark inspection"
        );
    }

    #[tokio::test]
    async fn runner_run_propagates_unexpected_ensure_schema_error() {
        let adapter = ClientBootstrapAdapter::new();

        let db = MockDb::default().with_healthy_integrity();

        let err = adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap_err();

        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn engine_run_applies_dump_on_fresh_db() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let dump_stmt = SqlStatement::new("--dump-sql");
        let cfg = BootstrapConfig {
            raindex_id: sample_ob_id(),
            dump_stmt: Some(SqlStatementBatch::from(vec![dump_stmt.clone()])),
            latest_block: 100,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
            deployment_block: 1,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_target_watermark_stmt(&cfg.raindex_id), json!([]))
            .with_text(&SqlStatement::new(BOOTSTRAP_CACHE_SIZE_SQL), "ok")
            .with_text(&dump_stmt, "ok");

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert_eq!(
            db.calls(),
            vec![
                BOOTSTRAP_CACHE_SIZE_SQL.to_string(),
                dump_stmt.sql().to_string()
            ]
        );
    }

    #[tokio::test]
    async fn engine_run_clears_and_applies_dump_when_threshold_exceeded() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 50_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) + 1;
        let dump_stmt = SqlStatement::new("--dump-sql");
        let cfg = BootstrapConfig {
            raindex_id: sample_ob_id(),
            dump_stmt: Some(SqlStatementBatch::from(vec![dump_stmt.clone()])),
            latest_block: latest,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
            deployment_block: 1,
        };

        let clear_batch = clear_raindex_data_batch(&sample_ob_id());
        let mut db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.raindex_id),
                json!([watermark_row(last_synced)]),
            )
            .with_text(&dump_stmt, "dumped");

        for stmt in clear_batch.statements() {
            db = db.with_text(stmt, "cleared");
        }
        db = db.with_text(&SqlStatement::new(BOOTSTRAP_CACHE_SIZE_SQL), "ok");

        adapter.engine_run(&db, &cfg).await.unwrap();

        let calls = db.calls();
        let mut expected: Vec<String> = clear_batch
            .statements()
            .iter()
            .map(|stmt| stmt.sql().to_string())
            .collect();
        expected.push(BOOTSTRAP_CACHE_SIZE_SQL.to_string());
        expected.push(dump_stmt.sql().to_string());
        assert_eq!(calls, expected);
    }

    #[tokio::test]
    async fn engine_run_skips_actions_within_threshold() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 100_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) - 1;
        let cfg = cfg_with_dump(latest);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.raindex_id),
                json!([watermark_row(last_synced)]),
            );

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn engine_run_skips_actions_at_threshold_boundary() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 120_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD);
        let cfg = cfg_with_dump(latest);

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.raindex_id),
                json!([watermark_row(last_synced)]),
            );

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn engine_run_does_nothing_without_dump_even_when_threshold_exceeded() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 200_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) + 5;
        let cfg = BootstrapConfig {
            raindex_id: sample_ob_id(),
            dump_stmt: None,
            latest_block: latest,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
            deployment_block: 1,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.raindex_id),
                json!([watermark_row(last_synced)]),
            );

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert!(db.calls().is_empty());
    }

    #[tokio::test]
    async fn engine_run_propagates_clear_errors() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();
        let last_synced = 80_000u64;
        let latest = last_synced + u64::from(TEST_BLOCK_NUMBER_THRESHOLD) + 42;
        let dump_stmt = SqlStatement::new("--dump-sql");
        let cfg = BootstrapConfig {
            raindex_id: sample_ob_id(),
            dump_stmt: Some(SqlStatementBatch::from(vec![dump_stmt.clone()])),
            latest_block: latest,
            block_number_threshold: TEST_BLOCK_NUMBER_THRESHOLD,
            deployment_block: 1,
        };

        let db = MockDb::default()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(
                &fetch_target_watermark_stmt(&cfg.raindex_id),
                json!([watermark_row(last_synced)]),
            );

        let err = adapter.engine_run(&db, &cfg).await.unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    struct CorruptedDb {
        json_map: HashMap<String, String>,
        text_map: HashMap<String, String>,
        calls_text: Mutex<Vec<String>>,
        wipe_called: Mutex<bool>,
    }

    impl CorruptedDb {
        fn new() -> Self {
            use crate::local_db::query::integrity_check::{
                integrity_check_stmt, IntegrityCheckRow,
            };

            let mut db = Self {
                json_map: HashMap::new(),
                text_map: HashMap::new(),
                calls_text: Mutex::new(Vec::new()),
                wipe_called: Mutex::new(false),
            };
            let corrupted_row = IntegrityCheckRow {
                quick_check: "database disk image is malformed".to_string(),
            };
            db.json_map.insert(
                integrity_check_stmt().sql().to_string(),
                json!([corrupted_row]).to_string(),
            );
            insert_reset_text_map(&mut db.text_map);
            db.text_map.insert(
                insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string(),
                "ok".to_string(),
            );
            for stmt in create_views_batch().statements() {
                db.text_map.insert(stmt.sql().to_string(), "ok".to_string());
            }
            db
        }

        fn calls(&self) -> Vec<String> {
            self.calls_text.lock().unwrap().clone()
        }

        fn was_wipe_called(&self) -> bool {
            *self.wipe_called.lock().unwrap()
        }
    }

    #[cfg_attr(target_family = "wasm", async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait)]
    impl LocalDbQueryExecutor for CorruptedDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            for stmt in batch {
                let _ = self.query_text(stmt).await?;
            }
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            let sql = stmt.sql();
            let Some(body) = self.json_map.get(sql) else {
                return Err(LocalDbQueryError::database("no json for sql"));
            };
            serde_json::from_str::<T>(body)
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            let sql = stmt.sql();
            self.calls_text.lock().unwrap().push(sql.to_string());
            self.text_map
                .get(sql)
                .cloned()
                .ok_or_else(|| LocalDbQueryError::database("no text for sql"))
        }

        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            *self.wipe_called.lock().unwrap() = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn runner_run_resets_on_corrupted_database() {
        let adapter = ClientBootstrapAdapter::new();
        let db = CorruptedDb::new();

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .expect("runner_run should succeed after resetting corrupted db");

        assert!(db.was_wipe_called(), "should have called wipe_and_recreate");

        let calls = db.calls();
        let expected_views: Vec<String> = create_views_batch()
            .statements()
            .iter()
            .map(|s| s.sql().to_string())
            .collect();
        assert_reset_batches_were_called(&calls);
        assert!(
            calls.contains(&insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()),
            "should have inserted db metadata"
        );
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
        );
    }

    struct IntegrityCheckFailsDb {
        text_map: HashMap<String, String>,
        calls_text: Mutex<Vec<String>>,
        wipe_called: Mutex<bool>,
    }

    impl IntegrityCheckFailsDb {
        fn new() -> Self {
            let mut db = Self {
                text_map: HashMap::new(),
                calls_text: Mutex::new(Vec::new()),
                wipe_called: Mutex::new(false),
            };
            insert_reset_text_map(&mut db.text_map);
            db.text_map.insert(
                insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string(),
                "ok".to_string(),
            );
            for stmt in create_views_batch().statements() {
                db.text_map.insert(stmt.sql().to_string(), "ok".to_string());
            }
            db
        }

        fn calls(&self) -> Vec<String> {
            self.calls_text.lock().unwrap().clone()
        }

        fn was_wipe_called(&self) -> bool {
            *self.wipe_called.lock().unwrap()
        }
    }

    #[cfg_attr(target_family = "wasm", async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait)]
    impl LocalDbQueryExecutor for IntegrityCheckFailsDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            for stmt in batch {
                let _ = self.query_text(stmt).await?;
            }
            Ok(())
        }

        async fn query_json<T>(&self, _stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            Err(LocalDbQueryError::database(
                "malformed database schema (db_metadata)",
            ))
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            let sql = stmt.sql();
            self.calls_text.lock().unwrap().push(sql.to_string());
            self.text_map
                .get(sql)
                .cloned()
                .ok_or_else(|| LocalDbQueryError::database("no text for sql"))
        }

        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            *self.wipe_called.lock().unwrap() = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn runner_run_resets_when_integrity_check_fails_with_error() {
        let adapter = ClientBootstrapAdapter::new();
        let db = IntegrityCheckFailsDb::new();

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .expect("runner_run should succeed after resetting when integrity check errors");

        assert!(
            db.was_wipe_called(),
            "should have called wipe_and_recreate when integrity check errors"
        );

        let calls = db.calls();
        assert_reset_batches_were_called(&calls);
    }

    struct WipeFailsDb;

    #[cfg_attr(target_family = "wasm", async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait)]
    impl LocalDbQueryExecutor for WipeFailsDb {
        async fn execute_batch(&self, _batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            Ok(())
        }

        async fn query_json<T>(&self, _stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            Err(LocalDbQueryError::database(
                "malformed database schema (db_metadata)",
            ))
        }

        async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            Ok("ok".to_string())
        }

        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            Err(LocalDbQueryError::database("wipe failed"))
        }
    }

    #[tokio::test]
    async fn runner_run_propagates_wipe_error() {
        let adapter = ClientBootstrapAdapter::new();
        let db = WipeFailsDb;

        let err = adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap_err();

        match err {
            LocalDbError::LocalDbQueryError(inner) => {
                assert!(inner.to_string().contains("wipe failed"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
