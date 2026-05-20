use crate::local_db::{
    pipeline::adapters::bootstrap::{BootstrapConfig, BootstrapPipeline, BootstrapState},
    query::{
        fetch_target_watermark::{fetch_target_watermark_stmt, TargetWatermarkRow},
        LocalDbQueryExecutor,
    },
    LocalDbError, RaindexIdentifier,
};
use alloy::primitives::Address;

#[derive(Debug, Default, Clone, Copy)]
pub struct ClientBootstrapAdapter;

impl ClientBootstrapAdapter {
    pub fn new() -> Self {
        Self {}
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
                db.execute_batch(dump_stmt).await?;
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
                    db.execute_batch(dump_stmt).await?;
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

        let BootstrapState {
            has_required_tables,
            ..
        } = self
            .inspect_state(db, &RaindexIdentifier::new(0, Address::ZERO))
            .await?;

        if !has_required_tables {
            self.reset_db(db, db_schema_version).await?;
        }

        match self.ensure_schema(db, db_schema_version).await {
            Ok(_) => {}
            Err(LocalDbError::MissingDbMetadataRow)
            | Err(LocalDbError::SchemaVersionMismatch { .. }) => {
                self.reset_db(db, db_schema_version).await?;
            }
            Err(err) => return Err(err),
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
    use crate::local_db::query::clear_tables::clear_tables_stmt;
    use crate::local_db::query::create_tables::create_tables_stmt;
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use crate::local_db::query::create_views::create_views_batch;
    use crate::local_db::query::fetch_db_metadata::{fetch_db_metadata_stmt, DbMetadataRow};
    use crate::local_db::query::fetch_tables::{fetch_tables_stmt, TableResponse};
    use crate::local_db::query::fetch_target_watermark::{
        fetch_target_watermark_stmt, TargetWatermarkRow,
    };
    use crate::local_db::query::insert_db_metadata::insert_db_metadata_stmt;
    use crate::local_db::query::FromDbJson;
    use crate::local_db::query::{
        LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    };
    use alloy::primitives::{Address, Bytes};
    use async_trait::async_trait;
    use raindex_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
    use serde_json::json;
    use std::str::FromStr;

    const TEST_BLOCK_NUMBER_THRESHOLD: u32 = 10_000;

    #[derive(Default)]
    struct MockDb {
        json_map: HashMap<String, String>,
        text_map: HashMap<String, String>,
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
        fn calls(&self) -> Vec<String> {
            self.calls_text.lock().unwrap().clone()
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
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
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
        assert_eq!(calls[0], clear_tables_stmt().sql().to_string());
        assert_eq!(calls[1], create_tables_stmt().sql().to_string());
        assert_eq!(
            calls[2],
            insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()
        );
        assert_eq!(&calls[3..], expected_views.as_slice());
    }

    #[tokio::test]
    async fn runner_run_resets_on_missing_db_metadata() {
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

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_db_metadata_stmt(), json!([])) // triggers reset
            // inspect_state will look for watermark since table exists
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]))
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
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
        assert!(calls.contains(&clear_tables_stmt().sql().to_string()));
        assert!(calls.contains(&create_tables_stmt().sql().to_string()));
        assert!(calls.contains(&insert_db_metadata_stmt(DB_SCHEMA_VERSION).sql().to_string()));
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
        );
    }

    #[tokio::test]
    async fn runner_run_resets_on_schema_mismatch() {
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

        let mismatched_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION + 1,
            created_at: None,
            updated_at: None,
        };

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]))
            .with_json(&fetch_db_metadata_stmt(), json!([mismatched_row]))
            .with_text(&clear_tables_stmt(), "ok")
            .with_text(&create_tables_stmt(), "ok")
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
        assert!(calls.contains(&clear_tables_stmt().sql().to_string()));
        assert!(calls.contains(&create_tables_stmt().sql().to_string()));
        assert!(
            expected_views.iter().all(|stmt| calls.contains(stmt)),
            "missing view creation statements"
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
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]));

        adapter
            .runner_run(&db, Some(DB_SCHEMA_VERSION))
            .await
            .unwrap();

        assert!(
            db.calls().is_empty(),
            "no text queries should be called when schema is ok"
        );
    }

    #[tokio::test]
    async fn runner_run_propagates_unexpected_ensure_schema_error() {
        let adapter = ClientBootstrapAdapter::new();
        let tables_json = required_tables_json();

        let db = MockDb::default()
            .with_healthy_integrity()
            .with_json(&fetch_tables_stmt(), tables_json)
            .with_json(&fetch_target_watermark_stmt(&runner_ob_id()), json!([]));

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
            .with_text(&dump_stmt, "ok");

        adapter.engine_run(&db, &cfg).await.unwrap();

        assert_eq!(db.calls(), vec![dump_stmt.sql().to_string()]);
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

        adapter.engine_run(&db, &cfg).await.unwrap();

        let calls = db.calls();
        let mut expected: Vec<String> = clear_batch
            .statements()
            .iter()
            .map(|stmt| stmt.sql().to_string())
            .collect();
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
            db.text_map
                .insert(clear_tables_stmt().sql().to_string(), "ok".to_string());
            db.text_map
                .insert(create_tables_stmt().sql().to_string(), "ok".to_string());
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
        assert!(
            calls.contains(&clear_tables_stmt().sql().to_string()),
            "should have cleared tables"
        );
        assert!(
            calls.contains(&create_tables_stmt().sql().to_string()),
            "should have created tables"
        );
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
            db.text_map
                .insert(clear_tables_stmt().sql().to_string(), "ok".to_string());
            db.text_map
                .insert(create_tables_stmt().sql().to_string(), "ok".to_string());
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
        assert!(
            calls.contains(&clear_tables_stmt().sql().to_string()),
            "should have cleared tables when integrity check errors"
        );
        assert!(
            calls.contains(&create_tables_stmt().sql().to_string()),
            "should have created tables"
        );
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
