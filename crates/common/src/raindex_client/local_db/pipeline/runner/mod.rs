pub mod config;
pub mod environment;
pub mod leadership;
pub mod scheduler;

use crate::local_db::{
    pipeline::runner::TargetStage,
    pipeline::{
        adapters::apply::{ApplyPipeline, DefaultApplyPipeline},
        adapters::{
            bootstrap::BootstrapPipeline, events::DefaultEventsPipeline,
            tokens::DefaultTokensPipeline, window::DefaultWindowPipeline,
        },
        runner::{
            environment::RunnerEnvironment,
            remotes::lookup_manifest_entry,
            utils::{
                build_runner_targets, parse_runner_settings, ParsedRunnerSettings, RunnerTarget,
            },
            RunOutcome, RunReport, TargetFailure, TargetSuccess,
        },
        EventsPipeline, StatusBus, TokensPipeline, WindowPipeline,
    },
    query::LocalDbQueryExecutor,
    LocalDbError, OrderbookIdentifier,
};
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
#[cfg(target_family = "wasm")]
use crate::raindex_client::local_db::pipeline::status::ClientStatusBus;
#[cfg(not(target_family = "wasm"))]
use crate::raindex_client::local_db::pipeline::status::TracingStatusBus;
use alloy::primitives::Address;
use config::NetworkRunnerConfig;
use environment::default_environment;
use futures::future::join_all;
use leadership::{DefaultLeadership, Leadership, LeadershipGuard};
use rain_orderbook_app_settings::remote::manifest::ManifestMap;

pub struct ClientRunner<B, W, E, T, A, S, L> {
    network_key: Option<String>,
    chain_id: Option<u32>,
    settings: ParsedRunnerSettings,
    base_targets: Vec<RunnerTarget>,
    manifest_map: ManifestMap,
    manifests_loaded: bool,
    has_provisioned_dumps: bool,
    environment: RunnerEnvironment<B, W, E, T, A, S>,
    leadership: L,
    leadership_guard: Option<LeadershipGuard>,
}

impl<B, W, E, T, A, S, L> ClientRunner<B, W, E, T, A, S, L>
where
    B: BootstrapPipeline + 'static,
    W: WindowPipeline + 'static,
    E: EventsPipeline + 'static,
    T: TokensPipeline + 'static,
    A: ApplyPipeline + 'static,
    S: StatusBus + 'static,
    L: Leadership + 'static,
{
    pub fn with_environment(
        settings_yaml: String,
        environment: RunnerEnvironment<B, W, E, T, A, S>,
        leadership: L,
    ) -> Result<Self, LocalDbError> {
        let settings = parse_runner_settings(&settings_yaml)?;
        let base_targets = build_runner_targets(&settings.orderbooks, &settings.syncs)?;

        Ok(Self {
            network_key: None,
            chain_id: None,
            settings,
            base_targets,
            manifest_map: ManifestMap::new(),
            manifests_loaded: false,
            has_provisioned_dumps: false,
            environment,
            leadership,
            leadership_guard: None,
        })
    }

    pub fn from_config(
        config: NetworkRunnerConfig,
        environment: RunnerEnvironment<B, W, E, T, A, S>,
        leadership: L,
    ) -> Result<Self, LocalDbError> {
        let base_targets = config.build_targets()?;

        Ok(Self {
            network_key: Some(config.network_key),
            chain_id: Some(config.chain_id),
            settings: config.settings,
            base_targets,
            manifest_map: ManifestMap::new(),
            manifests_loaded: false,
            has_provisioned_dumps: false,
            environment,
            leadership,
            leadership_guard: None,
        })
    }

    pub fn network_key(&self) -> Option<&str> {
        self.network_key.as_deref()
    }

    pub fn chain_id(&self) -> Option<u32> {
        self.chain_id
    }

    pub async fn run<DB>(&mut self, db: &DB) -> Result<RunOutcome, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        if self.leadership_guard.is_none() {
            match self.leadership.acquire().await? {
                Some(guard) => self.leadership_guard = Some(guard),
                None => return Ok(RunOutcome::NotLeader),
            }
        }

        if !self.manifests_loaded {
            self.manifest_map = match self
                .environment
                .fetch_manifests(&self.settings.orderbooks)
                .await
            {
                Ok(map) => map,
                Err(error) => {
                    return Ok(RunOutcome::Report(RunReport {
                        successes: Vec::new(),
                        failures: vec![TargetFailure {
                            ob_id: OrderbookIdentifier::new(0, Address::ZERO),
                            orderbook_key: None,
                            stage: TargetStage::ManifestFetch,
                            error,
                        }],
                    }));
                }
            };
            self.manifests_loaded = true;
        }

        let mut targets = self.base_targets.clone();
        let needs_provisioning = !self.has_provisioned_dumps;

        if needs_provisioning {
            let (provisioned, mut provisioning_failures) = self.provision_dumps(targets).await;
            let had_provisioning_failures = !provisioning_failures.is_empty();
            targets = provisioned;

            let RunReport {
                successes,
                failures: mut run_failures,
            } = self.execute_targets(db, targets).await?;

            provisioning_failures.append(&mut run_failures);

            if !had_provisioning_failures {
                self.has_provisioned_dumps = true;
            }

            return Ok(RunOutcome::Report(RunReport {
                successes,
                failures: provisioning_failures,
            }));
        }

        let report = self.execute_targets(db, targets).await?;
        Ok(RunOutcome::Report(report))
    }

    async fn provision_dumps(
        &self,
        targets: Vec<RunnerTarget>,
    ) -> (Vec<RunnerTarget>, Vec<TargetFailure>) {
        let manifest_map = &self.manifest_map;
        let environment = self.environment.clone();
        let futures = targets.into_iter().map(move |mut target| {
            let environment = environment.clone();
            async move {
                if let Some(entry) = lookup_manifest_entry(manifest_map, &target) {
                    let dump_sql = environment.download_dump(&entry.dump_url).await;
                    return match dump_sql {
                        Ok(sql) => {
                            target.inputs.dump_str = Some(sql);
                            target.inputs.manifest_end_block = entry.end_block;
                            Ok(target)
                        }
                        Err(error) => Err(TargetFailure {
                            ob_id: target.inputs.ob_id.clone(),
                            orderbook_key: Some(target.orderbook_key.clone()),
                            stage: TargetStage::DumpDownload,
                            error,
                        }),
                    };
                }
                Ok(target)
            }
        });

        let results = join_all(futures).await;
        let mut provisioned = Vec::new();
        let mut failures = Vec::new();
        for result in results {
            match result {
                Ok(target) => provisioned.push(target),
                Err(failure) => failures.push(failure),
            }
        }

        (provisioned, failures)
    }

    async fn execute_targets<DB>(
        &self,
        db: &DB,
        targets: Vec<RunnerTarget>,
    ) -> Result<RunReport, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        if targets.is_empty() {
            return Ok(RunReport::default());
        }

        let environment = self.environment.clone();
        let futures = targets.into_iter().map(move |target| {
            let environment = environment.clone();
            async move {
                let ob_id = target.inputs.ob_id.clone();
                let engine = match environment.build_engine(&target) {
                    Ok(engine) => engine.into_engine(),
                    Err(error) => {
                        return Err(TargetFailure {
                            ob_id,
                            orderbook_key: Some(target.orderbook_key.clone()),
                            stage: TargetStage::EngineBuild,
                            error,
                        })
                    }
                };

                match engine.run(db, &target.inputs).await {
                    Ok(outcome) => Ok(TargetSuccess { outcome }),
                    Err(error) => Err(TargetFailure {
                        ob_id,
                        orderbook_key: Some(target.orderbook_key.clone()),
                        stage: TargetStage::EngineRun,
                        error,
                    }),
                }
            }
        });

        let results = join_all(futures).await;
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for result in results {
            match result {
                Ok(success) => successes.push(success),
                Err(failure) => failures.push(failure),
            }
        }

        Ok(RunReport {
            successes,
            failures,
        })
    }
}

#[cfg(target_family = "wasm")]
impl
    ClientRunner<
        ClientBootstrapAdapter,
        DefaultWindowPipeline,
        DefaultEventsPipeline,
        DefaultTokensPipeline,
        DefaultApplyPipeline,
        ClientStatusBus,
        DefaultLeadership,
    >
{
    pub fn new(settings_yaml: String) -> Result<Self, LocalDbError> {
        let environment = default_environment();
        Self::with_environment(settings_yaml, environment, DefaultLeadership::new())
    }
}

#[cfg(not(target_family = "wasm"))]
impl
    ClientRunner<
        ClientBootstrapAdapter,
        DefaultWindowPipeline,
        DefaultEventsPipeline,
        DefaultTokensPipeline,
        DefaultApplyPipeline,
        TracingStatusBus,
        DefaultLeadership,
    >
{
    pub fn new(settings_yaml: String) -> Result<Self, LocalDbError> {
        let environment = default_environment();
        Self::with_environment(settings_yaml, environment, DefaultLeadership::new())
    }
}

#[cfg(test)]
mod tests {
    use super::leadership::LeadershipGuard;
    use super::*;
    use crate::local_db::decode::{DecodedEvent, DecodedEventData};
    use crate::local_db::fetch::FetchConfig;
    use crate::local_db::pipeline::adapters::apply::ApplyPipelineTargetInfo;
    use crate::local_db::pipeline::adapters::bootstrap::{BootstrapConfig, BootstrapState};
    use crate::local_db::pipeline::runner::environment::{
        default_dump_downloader, DumpFuture, EnginePipelines, ManifestFuture,
    };
    use crate::local_db::pipeline::runner::utils::RunnerTarget;
    use crate::local_db::pipeline::{
        EventsPipeline, StatusBus, SyncConfig, SyncOutcome, SyncPhase, TokensPipeline,
        WindowPipeline,
    };
    use crate::local_db::query::create_tables::REQUIRED_TABLES;
    use crate::local_db::query::fetch_db_metadata::{fetch_db_metadata_stmt, DbMetadataRow};
    use crate::local_db::query::fetch_store_addresses::fetch_store_addresses_stmt;
    use crate::local_db::query::fetch_tables::{fetch_tables_stmt, TableResponse};
    use crate::local_db::query::fetch_target_watermark::fetch_target_watermark_stmt;
    use crate::local_db::query::{FromDbJson, LocalDbQueryError, SqlStatement, SqlStatementBatch};
    use crate::local_db::{LocalDbError, OrderbookIdentifier};
    use crate::rpc_client::LogEntryResponse;
    use alloy::primitives::{address, b256, Address, Bytes, B256};
    use async_trait::async_trait;
    use rain_orderbook_app_settings::local_db_manifest::{
        LocalDbManifest, ManifestNetwork, ManifestOrderbook, DB_SCHEMA_VERSION, MANIFEST_VERSION,
    };
    use rain_orderbook_app_settings::orderbook::OrderbookCfg;
    use rain_orderbook_app_settings::remote::manifest::ManifestMap;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use serde::Serialize;
    use serde_json::{json, Value};
    use std::collections::{HashMap, VecDeque};
    use std::str::FromStr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use url::Url;

    const CHAIN_ID: u32 = 42161;
    const NETWORK_KEY: &str = "anvil";
    const ORDERBOOK_KEY_A: &str = "ob-a";
    const ORDERBOOK_KEY_B: &str = "ob-b";

    const ORDERBOOK_A: Address = address!("00000000000000000000000000000000000000a1");
    const ORDERBOOK_B: Address = address!("00000000000000000000000000000000000000b2");

    #[derive(Clone, Default)]
    struct AlwaysLeadership;

    #[async_trait(?Send)]
    impl Leadership for AlwaysLeadership {
        async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
            Ok(Some(LeadershipGuard::new_noop()))
        }
    }

    #[derive(Clone, Default)]
    struct CountingLeadership {
        calls: Arc<AtomicUsize>,
    }

    impl CountingLeadership {
        fn new() -> Self {
            Self::default()
        }

        fn call_count(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait(?Send)]
    impl Leadership for CountingLeadership {
        async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(Some(LeadershipGuard::new_noop()))
        }
    }

    #[derive(Debug, Clone)]
    enum LeadershipAction {
        Grant,
        Skip,
        Fail(String),
    }

    #[derive(Clone)]
    struct SequenceLeadership {
        actions: Arc<Mutex<VecDeque<LeadershipAction>>>,
    }

    impl SequenceLeadership {
        fn new(actions: Vec<LeadershipAction>) -> Self {
            Self {
                actions: Arc::new(Mutex::new(VecDeque::from(actions))),
            }
        }
    }

    #[async_trait(?Send)]
    impl Leadership for SequenceLeadership {
        async fn acquire(&self) -> Result<Option<LeadershipGuard>, LocalDbError> {
            let action = self
                .actions
                .lock()
                .expect("actions lock poisoned")
                .pop_front()
                .unwrap_or_else(|| panic!("unexpected leadership acquire call"));
            match action {
                LeadershipAction::Grant => Ok(Some(LeadershipGuard::new_noop())),
                LeadershipAction::Skip => Ok(None),
                LeadershipAction::Fail(message) => Err(LocalDbError::CustomError(message)),
            }
        }
    }

    #[derive(Clone, Default)]
    struct Telemetry {
        manifest_fetches: Arc<AtomicUsize>,
        dump_requests: Arc<Mutex<Vec<Url>>>,
        engine_runs: Arc<Mutex<Vec<String>>>,
        bootstrap_records: Arc<Mutex<Vec<BootstrapRecord>>>,
        status_messages: Arc<Mutex<Vec<(String, String)>>>,
        persist_calls: Arc<AtomicUsize>,
        builder_inits: Arc<AtomicUsize>,
    }

    #[derive(Clone, Debug)]
    struct BootstrapRecord {
        orderbook_key: String,
        dump_sql: Option<String>,
        latest_block: u64,
    }

    fn dump_sql(batch: &SqlStatementBatch) -> String {
        batch
            .statements()
            .iter()
            .map(|stmt| stmt.sql())
            .collect::<Vec<_>>()
            .join("\n")
    }

    impl Telemetry {
        fn record_manifest_fetch(&self) -> usize {
            self.manifest_fetches.fetch_add(1, Ordering::SeqCst)
        }

        fn manifest_fetch_count(&self) -> usize {
            self.manifest_fetches.load(Ordering::SeqCst)
        }

        fn record_dump(&self, url: Url) -> usize {
            let mut dumps = self.dump_requests.lock().unwrap();
            dumps.push(url);
            dumps.len()
        }

        fn dump_requests(&self) -> Vec<Url> {
            self.dump_requests.lock().unwrap().clone()
        }

        fn record_engine_run(&self, orderbook_key: &str) {
            self.engine_runs
                .lock()
                .unwrap()
                .push(orderbook_key.to_string());
        }

        fn engine_runs(&self) -> Vec<String> {
            self.engine_runs.lock().unwrap().clone()
        }

        fn record_bootstrap(
            &self,
            orderbook_key: String,
            dump_sql: Option<String>,
            latest_block: u64,
        ) {
            self.bootstrap_records
                .lock()
                .unwrap()
                .push(BootstrapRecord {
                    orderbook_key,
                    dump_sql,
                    latest_block,
                });
        }

        fn bootstrap_records(&self) -> Vec<BootstrapRecord> {
            self.bootstrap_records.lock().unwrap().clone()
        }

        fn record_status(&self, orderbook_key: &str, message: &str) {
            self.status_messages
                .lock()
                .unwrap()
                .push((orderbook_key.to_string(), message.to_string()));
        }

        fn record_persist(&self) {
            self.persist_calls.fetch_add(1, Ordering::SeqCst);
        }

        fn persist_count(&self) -> usize {
            self.persist_calls.load(Ordering::SeqCst)
        }

        fn record_builder_init(&self) {
            self.builder_inits.fetch_add(1, Ordering::SeqCst);
        }

        fn builder_inits(&self) -> usize {
            self.builder_inits.load(Ordering::SeqCst)
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum EngineBehavior {
        Success,
        ApplyFail,
    }

    #[derive(Clone, Default)]
    struct RecordingDb {
        inner: Arc<RecordingDbInner>,
    }

    #[derive(Default)]
    struct RecordingDbInner {
        json_map: Mutex<HashMap<String, JsonResponse>>,
        text_map: Mutex<HashMap<String, Result<String, LocalDbQueryError>>>,
        batch_calls: Mutex<Vec<Vec<String>>>,
        text_calls: Mutex<Vec<String>>,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    enum JsonResponse {
        Value(Value),
        Error(LocalDbQueryError),
    }

    impl RecordingDb {
        fn set_json_value<T>(&self, stmt: &SqlStatement, value: T)
        where
            T: Serialize,
        {
            self.inner
                .json_map
                .lock()
                .unwrap()
                .insert(stmt.sql().to_string(), JsonResponse::Value(json!(value)));
        }

        fn set_json_raw(&self, stmt: &SqlStatement, value: Value) {
            self.inner
                .json_map
                .lock()
                .unwrap()
                .insert(stmt.sql().to_string(), JsonResponse::Value(value));
        }

        #[allow(dead_code)]
        fn set_json_error(&self, stmt: &SqlStatement, err: LocalDbQueryError) {
            self.inner
                .json_map
                .lock()
                .unwrap()
                .insert(stmt.sql().to_string(), JsonResponse::Error(err));
        }

        fn batch_calls(&self) -> Vec<Vec<String>> {
            self.inner.batch_calls.lock().unwrap().clone()
        }
    }

    #[cfg_attr(target_family = "wasm", async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait)]
    impl LocalDbQueryExecutor for RecordingDb {
        async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            let statements: Vec<String> = batch
                .into_iter()
                .map(|stmt| stmt.sql().to_string())
                .collect();
            self.inner.batch_calls.lock().unwrap().push(statements);
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            let sql = stmt.sql().to_string();
            let response = self.inner.json_map.lock().unwrap().get(&sql).cloned();
            match response {
                Some(JsonResponse::Value(value)) => serde_json::from_value(value)
                    .map_err(|err| LocalDbQueryError::deserialization(err.to_string())),
                Some(JsonResponse::Error(err)) => Err(err),
                None => Err(LocalDbQueryError::database(format!(
                    "no json response configured for sql: {sql}"
                ))),
            }
        }

        async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            let sql = stmt.sql().to_string();
            self.inner.text_calls.lock().unwrap().push(sql.clone());
            let response = self.inner.text_map.lock().unwrap().get(&sql).cloned();
            match response {
                Some(Ok(value)) => Ok(value),
                Some(Err(err)) => Err(err),
                None => Ok("ok".to_string()),
            }
        }

        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            Err(LocalDbQueryError::not_implemented("wipe_and_recreate"))
        }
    }

    #[derive(Clone)]
    struct StubBootstrap {
        telemetry: Telemetry,
        orderbook_key: String,
    }

    impl StubBootstrap {
        fn new(telemetry: Telemetry, orderbook_key: String) -> Self {
            Self {
                telemetry,
                orderbook_key,
            }
        }
    }

    #[async_trait(?Send)]
    impl BootstrapPipeline for StubBootstrap {
        async fn ensure_schema<DB>(
            &self,
            _db: &DB,
            _db_schema_version: Option<u32>,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }

        async fn inspect_state<DB>(
            &self,
            _db: &DB,
            _ob_id: &OrderbookIdentifier,
        ) -> Result<BootstrapState, LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(BootstrapState {
                has_required_tables: true,
                last_synced_block: None,
            })
        }

        async fn reset_db<DB>(
            &self,
            _db: &DB,
            _db_schema_version: Option<u32>,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }

        async fn clear_orderbook_data<DB>(
            &self,
            _db: &DB,
            _target: &OrderbookIdentifier,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }

        async fn engine_run<DB>(
            &self,
            _db: &DB,
            config: &BootstrapConfig,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            let dump_sql = config.dump_stmt.as_ref().map(dump_sql);
            self.telemetry.record_bootstrap(
                self.orderbook_key.clone(),
                dump_sql,
                config.latest_block,
            );
            Ok(())
        }

        async fn runner_run<DB>(
            &self,
            _db: &DB,
            _db_schema_version: Option<u32>,
        ) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct StubWindow {
        start_block: u64,
        target_block: u64,
    }

    impl StubWindow {
        fn new(start_block: u64, target_block: u64) -> Self {
            Self {
                start_block,
                target_block,
            }
        }
    }

    #[async_trait(?Send)]
    impl WindowPipeline for StubWindow {
        async fn compute<DB>(
            &self,
            _db: &DB,
            _ob_id: &OrderbookIdentifier,
            _cfg: &SyncConfig,
            _latest_block: u64,
        ) -> Result<(u64, u64), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok((self.start_block, self.target_block))
        }
    }

    #[derive(Clone)]
    struct StubEvents {
        latest_block: u64,
    }

    impl StubEvents {
        fn new(latest_block: u64) -> Self {
            Self { latest_block }
        }
    }

    #[async_trait(?Send)]
    impl EventsPipeline for StubEvents {
        async fn latest_block(&self) -> Result<u64, LocalDbError> {
            Ok(self.latest_block)
        }

        async fn block_hash(&self, _block_number: u64) -> Result<B256, LocalDbError> {
            Ok(b256!(
                "0x0000000000000000000000000000000000000000000000000000000000000000"
            ))
        }

        async fn fetch_orderbook(
            &self,
            _orderbook_address: Address,
            _from_block: u64,
            _to_block: u64,
            _cfg: &FetchConfig,
        ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
            Ok(Vec::new())
        }

        async fn fetch_stores(
            &self,
            _store_addresses: &[Address],
            _from_block: u64,
            _to_block: u64,
            _cfg: &FetchConfig,
        ) -> Result<Vec<LogEntryResponse>, LocalDbError> {
            Ok(Vec::new())
        }

        fn decode(
            &self,
            _logs: &[LogEntryResponse],
        ) -> Result<Vec<DecodedEventData<DecodedEvent>>, LocalDbError> {
            Ok(Vec::new())
        }
    }

    #[derive(Clone, Copy)]
    struct StubTokens;

    #[async_trait(?Send)]
    impl TokensPipeline for StubTokens {
        async fn load_existing<DB>(
            &self,
            _db: &DB,
            _ob_id: &OrderbookIdentifier,
            _token_addrs_lower: &[Address],
        ) -> Result<
            Vec<crate::local_db::query::fetch_erc20_tokens_by_addresses::Erc20TokenRow>,
            LocalDbError,
        >
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            Ok(Vec::new())
        }

        async fn fetch_missing(
            &self,
            _missing: Vec<Address>,
            _cfg: &FetchConfig,
        ) -> Result<Vec<(Address, crate::erc20::TokenInfo)>, LocalDbError> {
            Ok(Vec::new())
        }
    }

    #[derive(Clone)]
    struct StubApply {
        telemetry: Telemetry,
        orderbook_key: String,
        fail: bool,
    }

    impl StubApply {
        fn new(telemetry: Telemetry, orderbook_key: String, fail: bool) -> Self {
            Self {
                telemetry,
                orderbook_key,
                fail,
            }
        }
    }

    #[async_trait(?Send)]
    impl ApplyPipeline for StubApply {
        fn build_batch(
            &self,
            _target_info: &ApplyPipelineTargetInfo,
            _raw_logs: &[LogEntryResponse],
            _decoded_events: &[DecodedEventData<DecodedEvent>],
            _existing_tokens: &[crate::local_db::query::fetch_erc20_tokens_by_addresses::Erc20TokenRow],
            _tokens_to_upsert: &[(Address, crate::erc20::TokenInfo)],
        ) -> Result<SqlStatementBatch, LocalDbError> {
            let mut batch = SqlStatementBatch::new();
            batch.add(SqlStatement::new("BEGIN TRANSACTION"));
            batch.add(SqlStatement::new(format!(
                "-- apply {}",
                self.orderbook_key
            )));
            batch.add(SqlStatement::new("COMMIT"));
            Ok(batch)
        }

        async fn persist<DB>(&self, db: &DB, batch: &SqlStatementBatch) -> Result<(), LocalDbError>
        where
            DB: LocalDbQueryExecutor + ?Sized,
        {
            self.telemetry.record_engine_run(&self.orderbook_key);
            if self.fail {
                return Err(LocalDbError::CustomError(format!(
                    "apply failed {}",
                    self.orderbook_key
                )));
            }
            db.execute_batch(batch).await?;
            self.telemetry.record_persist();
            Ok(())
        }
    }

    #[derive(Clone)]
    struct StubStatusBus {
        telemetry: Telemetry,
        orderbook_key: String,
    }

    impl StubStatusBus {
        fn new(telemetry: Telemetry, orderbook_key: String) -> Self {
            Self {
                telemetry,
                orderbook_key,
            }
        }
    }

    #[async_trait(?Send)]
    impl StatusBus for StubStatusBus {
        async fn send(&self, phase: SyncPhase) -> Result<(), LocalDbError> {
            self.telemetry
                .record_status(&self.orderbook_key, phase.to_message());
            Ok(())
        }
    }

    type TestRunnerEnvironment = RunnerEnvironment<
        StubBootstrap,
        StubWindow,
        StubEvents,
        StubTokens,
        StubApply,
        StubStatusBus,
    >;

    type TestEngineBuilder = Arc<
        dyn Fn(
                &RunnerTarget,
            ) -> Result<
                EnginePipelines<
                    StubBootstrap,
                    StubWindow,
                    StubEvents,
                    StubTokens,
                    StubApply,
                    StubStatusBus,
                >,
                LocalDbError,
            > + Send
            + Sync,
    >;

    fn remote_url_a() -> Url {
        Url::parse("https://manifests.example/a.yaml").unwrap()
    }

    fn remote_url_b() -> Url {
        Url::parse("https://manifests.example/b.yaml").unwrap()
    }

    const END_HASH_A: &str = "0x000000000000000000000000000000000000000000000000000000000000dead";
    const END_HASH_B: &str = "0x000000000000000000000000000000000000000000000000000000000000beef";

    fn dump_url_a() -> Url {
        Url::parse("https://dumps.example/ob-a.sql").unwrap()
    }

    fn dump_url_b() -> Url {
        Url::parse("https://dumps.example/ob-b.sql").unwrap()
    }

    fn manifest_for_a() -> ManifestMap {
        make_manifest(remote_url_a(), ORDERBOOK_A, dump_url_a(), 111, END_HASH_A)
    }

    fn manifest_for_b() -> ManifestMap {
        make_manifest(remote_url_b(), ORDERBOOK_B, dump_url_b(), 222, END_HASH_B)
    }

    fn manifest_for_both() -> ManifestMap {
        let mut map = manifest_for_a();
        map.extend(manifest_for_b());
        map
    }

    fn make_manifest(
        remote: Url,
        orderbook_address: Address,
        dump_url: Url,
        end_block: u64,
        end_hash: &str,
    ) -> ManifestMap {
        let end_block_hash = B256::from_str(end_hash).unwrap();
        let manifest = LocalDbManifest {
            manifest_version: MANIFEST_VERSION,
            db_schema_version: DB_SCHEMA_VERSION,
            networks: HashMap::from([(
                NETWORK_KEY.to_string(),
                ManifestNetwork {
                    chain_id: CHAIN_ID,
                    orderbooks: vec![ManifestOrderbook {
                        address: orderbook_address,
                        dump_url,
                        end_block,
                        end_block_hash: Bytes::copy_from_slice(end_block_hash.as_slice()),
                        end_block_time_ms: 1_000,
                    }],
                },
            )]),
        };
        HashMap::from([(remote, manifest)])
    }

    fn two_orderbooks_settings_yaml() -> String {
        format!(
            r#"
version: {version}
networks:
  anvil:
    rpcs:
      - https://rpc.example/anvil
    chain-id: 42161
subgraphs:
  anvil: https://subgraph.example/anvil
local-db-remotes:
  remote-a: https://manifests.example/a.yaml
  remote-b: https://manifests.example/b.yaml
local-db-sync:
  anvil:
    batch-size: 10
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 1
    finality-depth: 12
    bootstrap-block-threshold: 10000
    sync-interval-ms: 5000
orderbooks:
  ob-a:
    address: 0x00000000000000000000000000000000000000a1
    network: anvil
    subgraph: anvil
    local-db-remote: remote-a
    deployment-block: 123
  ob-b:
    address: 0x00000000000000000000000000000000000000b2
    network: anvil
    subgraph: anvil
    local-db-remote: remote-b
    deployment-block: 456
"#,
            version = SpecVersion::current()
        )
    }

    fn single_orderbook_settings_yaml() -> String {
        format!(
            r#"
version: {version}
networks:
  anvil:
    rpcs:
      - https://rpc.example/anvil
    chain-id: 42161
subgraphs:
  anvil: https://subgraph.example/anvil
local-db-remotes:
  remote-a: https://manifests.example/a.yaml
local-db-sync:
  anvil:
    batch-size: 10
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 1
    finality-depth: 12
    bootstrap-block-threshold: 10000
    sync-interval-ms: 5000
orderbooks:
  ob-a:
    address: 0x00000000000000000000000000000000000000a1
    network: anvil
    subgraph: anvil
    local-db-remote: remote-a
    deployment-block: 123
"#,
            version = SpecVersion::current()
        )
    }

    fn prepare_db_baseline(db: &RecordingDb) {
        let tables: Vec<TableResponse> = REQUIRED_TABLES
            .iter()
            .map(|&name| TableResponse {
                name: name.to_string(),
            })
            .collect();
        db.set_json_value(&fetch_tables_stmt(), &tables);

        let metadata_row = DbMetadataRow {
            id: 1,
            db_schema_version: DB_SCHEMA_VERSION,
            created_at: None,
            updated_at: None,
        };
        db.set_json_value(&fetch_db_metadata_stmt(), &[metadata_row]);
        db.set_json_raw(
            &fetch_target_watermark_stmt(&OrderbookIdentifier::new(0, Address::ZERO)),
            json!([]),
        );
    }

    fn prepare_db_for_targets(db: &RecordingDb, targets: &[RunnerTarget]) {
        prepare_db_baseline(db);
        for target in targets {
            db.set_json_raw(
                &fetch_target_watermark_stmt(&target.inputs.ob_id),
                json!([]),
            );
            db.set_json_raw(&fetch_store_addresses_stmt(&target.inputs.ob_id), json!([]));
        }
    }

    fn engine_builder_for_behaviors(
        telemetry: Telemetry,
        behaviors: HashMap<String, EngineBehavior>,
    ) -> TestEngineBuilder {
        let behaviors = Arc::new(behaviors);
        Arc::new(move |target: &RunnerTarget| {
            let behavior = behaviors
                .get(&target.orderbook_key)
                .copied()
                .unwrap_or(EngineBehavior::Success);
            let telemetry = telemetry.clone();
            telemetry.record_builder_init();
            let fail_apply = behavior == EngineBehavior::ApplyFail;
            let bootstrap = StubBootstrap::new(telemetry.clone(), target.orderbook_key.clone());
            let window = StubWindow::new(0, target.inputs.cfg.deployment_block);
            let events = StubEvents::new(target.inputs.cfg.deployment_block);
            let apply = StubApply::new(telemetry.clone(), target.orderbook_key.clone(), fail_apply);
            let status = StubStatusBus::new(telemetry.clone(), target.orderbook_key.clone());
            Ok(EnginePipelines::new(
                bootstrap, window, events, StubTokens, apply, status,
            ))
        })
    }

    fn build_environment(
        manifest_map: ManifestMap,
        behaviors: HashMap<String, EngineBehavior>,
        manifest_limit: usize,
        dump_limit: usize,
        telemetry: Telemetry,
    ) -> TestRunnerEnvironment {
        let manifest_arc = Arc::new(manifest_map);

        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    let prev = telemetry.record_manifest_fetch();
                    if prev >= manifest_limit {
                        panic!("manifest fetch called more than expected");
                    }
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = {
            let telemetry = telemetry.clone();
            let counter = Arc::new(AtomicUsize::new(0));
            Arc::new(move |url: &Url| {
                let telemetry = telemetry.clone();
                let counter = Arc::clone(&counter);
                let url = url.clone();
                Box::pin(async move {
                    let position = counter.fetch_add(1, Ordering::SeqCst);
                    if position >= dump_limit {
                        panic!("dump downloader invoked more than expected");
                    }
                    telemetry.record_dump(url.clone());
                    Ok(format!("-- dump for {}", url))
                }) as DumpFuture
            })
        };

        let engine_builder = engine_builder_for_behaviors(telemetry, behaviors);

        RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder)
    }

    fn unwrap_report(outcome: RunOutcome) -> RunReport {
        match outcome {
            RunOutcome::Report(report) => report,
            RunOutcome::NotLeader => panic!("expected Report, got NotLeader"),
        }
    }

    fn extract_outcomes(report: &RunReport) -> Vec<SyncOutcome> {
        report
            .successes
            .iter()
            .map(|success| success.outcome.clone())
            .collect()
    }

    fn expect_orderbooks(report: &RunReport, expected: &[Address]) {
        let outcomes = extract_outcomes(report);
        let mut addrs: Vec<Address> = outcomes.iter().map(|o| o.ob_id.orderbook_address).collect();
        addrs.sort();
        let mut expected_sorted = expected.to_vec();
        expected_sorted.sort();
        assert_eq!(addrs, expected_sorted);
    }

    #[test]
    fn with_environment_propagates_settings_parse_error() {
        let telemetry = Telemetry::default();
        let environment = build_environment(ManifestMap::new(), HashMap::new(), 1, 1, telemetry);

        let err = match ClientRunner::with_environment(
            "invalid: [yaml".to_string(),
            environment,
            AlwaysLeadership,
        ) {
            Ok(_) => panic!("invalid yaml should fail"),
            Err(err) => err,
        };

        assert!(matches!(
            err,
            LocalDbError::SettingsYaml(_) | LocalDbError::YamlScan(_)
        ));
    }

    #[test]
    fn default_environment_builds_engine_for_target() {
        let settings = single_orderbook_settings_yaml();
        let runner = ClientRunner::new(settings).expect("runner builds with default env");
        assert_eq!(runner.base_targets.len(), 1);

        let target = runner.base_targets[0].clone();
        let pipelines = runner
            .environment
            .build_engine(&target)
            .expect("default builder constructs engine");

        pipelines.into_engine();
    }

    #[tokio::test]
    async fn first_run_fetches_manifests_bootstraps_and_runs_engines() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run succeeds");
        let report = unwrap_report(outcome);
        let outcomes = extract_outcomes(&report);
        assert_eq!(outcomes.len(), 2);
        expect_orderbooks(&report, &[ORDERBOOK_A, ORDERBOOK_B]);
        assert!(runner.manifests_loaded);
        assert!(runner.has_provisioned_dumps);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.engine_runs().len(), 2);
        assert_eq!(telemetry.persist_count(), 2);
        assert_eq!(telemetry.builder_inits(), 2);
        let records = telemetry.bootstrap_records();
        assert_eq!(records.len(), 2);
        assert!(records.iter().all(|record| record
            .dump_sql
            .as_ref()
            .is_some_and(|sql| sql.starts_with("-- dump for "))));
        let latest_blocks: HashMap<String, u64> = records
            .iter()
            .map(|record| (record.orderbook_key.clone(), record.latest_block))
            .collect();
        assert_eq!(latest_blocks.get(ORDERBOOK_KEY_A), Some(&111));
        assert_eq!(latest_blocks.get(ORDERBOOK_KEY_B), Some(&222));
        assert_eq!(db.batch_calls().len(), 2);
    }

    #[tokio::test]
    async fn second_run_reuses_cached_manifest_and_skips_downloads() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db_first = RecordingDb::default();
        prepare_db_for_targets(&db_first, &runner.base_targets);
        runner.run(&db_first).await.expect("initial run succeeds");

        let db_second = RecordingDb::default();
        prepare_db_for_targets(&db_second, &runner.base_targets);
        let outcome = runner.run(&db_second).await.expect("second run succeeds");
        let report = unwrap_report(outcome);
        let outcomes = extract_outcomes(&report);

        assert_eq!(outcomes.len(), 2);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.builder_inits(), 4);
    }

    #[tokio::test]
    async fn leadership_guard_reused_across_runs() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership = CountingLeadership::new();
        let mut runner =
            ClientRunner::with_environment(settings, environment, leadership.clone()).unwrap();

        let db_first = RecordingDb::default();
        prepare_db_for_targets(&db_first, &runner.base_targets);
        runner.run(&db_first).await.expect("first run succeeds");

        let db_second = RecordingDb::default();
        prepare_db_for_targets(&db_second, &runner.base_targets);
        runner.run(&db_second).await.expect("second run succeeds");

        assert_eq!(
            leadership.call_count(),
            1,
            "leadership should be acquired only once per runner lifetime"
        );
    }

    #[tokio::test]
    async fn run_returns_noop_outcomes_when_leadership_not_acquired() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership = SequenceLeadership::new(vec![LeadershipAction::Skip]);
        let mut runner = ClientRunner::with_environment(settings, environment, leadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let outcome = runner.run(&db).await.expect("run succeeds");
        assert!(matches!(outcome, RunOutcome::NotLeader));
        assert!(!runner.manifests_loaded);
        assert!(!runner.has_provisioned_dumps);
        assert_eq!(telemetry.manifest_fetch_count(), 0);
        assert!(telemetry.dump_requests().is_empty());
        assert_eq!(telemetry.builder_inits(), 0);
    }

    #[tokio::test]
    async fn leadership_skip_then_grant_fetches_once() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership =
            SequenceLeadership::new(vec![LeadershipAction::Skip, LeadershipAction::Grant]);
        let mut runner = ClientRunner::with_environment(settings, environment, leadership).unwrap();

        let db_skip = RecordingDb::default();
        prepare_db_baseline(&db_skip);
        let outcome_skip = runner.run(&db_skip).await.expect("skip run succeeds");
        assert!(matches!(outcome_skip, RunOutcome::NotLeader));
        assert!(!runner.manifests_loaded);
        assert!(!runner.has_provisioned_dumps);

        let db_grant = RecordingDb::default();
        prepare_db_for_targets(&db_grant, &runner.base_targets);
        let outcome = runner.run(&db_grant).await.expect("second run succeeds");
        let report = unwrap_report(outcome);
        assert_eq!(report.successes.len(), 2);
        assert!(runner.manifests_loaded);
        assert!(runner.has_provisioned_dumps);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.builder_inits(), 2);
    }

    #[tokio::test]
    async fn leadership_error_is_propagated() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(manifest_for_both(), HashMap::new(), 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let leadership =
            SequenceLeadership::new(vec![LeadershipAction::Fail("no leadership".into())]);
        let mut runner = ClientRunner::with_environment(settings, environment, leadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let err = runner.run(&db).await.expect_err("run should fail");
        matches!(err, LocalDbError::CustomError(message) if message == "no leadership");
        assert!(!runner.manifests_loaded);
        assert!(!runner.has_provisioned_dumps);
        assert_eq!(telemetry.manifest_fetch_count(), 0);
        assert!(telemetry.dump_requests().is_empty());
    }

    #[tokio::test]
    async fn provisioning_downloads_only_present_manifests() {
        let telemetry = Telemetry::default();
        let environment = build_environment(
            manifest_for_a(),
            HashMap::from([(ORDERBOOK_KEY_B.to_string(), EngineBehavior::Success)]),
            1,
            1,
            telemetry.clone(),
        );
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run succeeds");
        let report = unwrap_report(outcome);
        assert_eq!(report.successes.len(), 2);
        let dumps = telemetry.dump_requests();
        assert_eq!(dumps.len(), 1);
        assert_eq!(dumps[0], dump_url_a());

        let records = telemetry.bootstrap_records();
        let record_a = records
            .iter()
            .find(|r| r.orderbook_key == ORDERBOOK_KEY_A)
            .expect("record for ob-a");
        assert!(record_a
            .dump_sql
            .as_ref()
            .is_some_and(|sql| sql.contains("-- dump for")));
        assert_eq!(record_a.latest_block, 111);

        let record_b = records
            .iter()
            .find(|r| r.orderbook_key == ORDERBOOK_KEY_B)
            .expect("record for ob-b");
        assert!(record_b.dump_sql.is_none());
        assert_eq!(record_b.latest_block, 0);
    }

    #[tokio::test]
    async fn run_returns_immediately_when_no_targets() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(ManifestMap::new(), HashMap::new(), 1, 0, telemetry.clone());
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        runner.base_targets.clear();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let outcome = runner.run(&db).await.expect("run succeeds");
        let report = unwrap_report(outcome);
        assert!(report.successes.is_empty());
        assert!(runner.has_provisioned_dumps);
        assert_eq!(telemetry.builder_inits(), 0);
        assert_eq!(telemetry.dump_requests().len(), 0);
    }

    #[tokio::test]
    async fn dump_failure_leaves_runner_unprovisioned_for_retry() {
        let telemetry = Telemetry::default();
        let manifest_map = manifest_for_a();
        let manifest_arc = Arc::new(manifest_map);
        let call_count = Arc::new(AtomicUsize::new(0));
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    telemetry.record_manifest_fetch();
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };
        let dump_downloader = {
            let call_count = Arc::clone(&call_count);
            Arc::new(move |_url: &Url| {
                let count = call_count.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if count == 0 {
                        Err(LocalDbError::CustomError("download failed".into()))
                    } else {
                        Ok("-- dump sql".to_string())
                    }
                }) as DumpFuture
            })
        };
        let engine_builder = engine_builder_for_behaviors(telemetry.clone(), HashMap::new());
        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run completes with failure");
        let report = unwrap_report(outcome);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].stage, TargetStage::DumpDownload);
        assert!(!runner.has_provisioned_dumps);
        assert!(runner.manifests_loaded);

        let outcome2 = runner.run(&db).await.expect("retry succeeds");
        let report2 = unwrap_report(outcome2);
        assert_eq!(report2.successes.len(), 1);
        assert!(runner.has_provisioned_dumps);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
    }

    #[tokio::test]
    async fn manifest_fetch_failure_leaves_state_unloaded() {
        let telemetry = Telemetry::default();
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                Box::pin(async move {
                    telemetry.record_manifest_fetch();
                    Err(LocalDbError::CustomError("manifest boom".into()))
                }) as ManifestFuture
            })
        };

        let dump_downloader = Arc::new(|_url: &Url| {
            Box::pin(async move {
                Err(LocalDbError::CustomError(
                    "dump downloader should not run".into(),
                ))
            }) as DumpFuture
        });

        let engine_builder = engine_builder_for_behaviors(telemetry.clone(), HashMap::new());
        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_baseline(&db);

        let outcome = runner
            .run(&db)
            .await
            .expect("run completes with manifest failure");
        let report = unwrap_report(outcome);
        assert!(report.successes.is_empty());
        assert_eq!(report.failures.len(), 1);
        let failure = &report.failures[0];
        assert_eq!(failure.stage, TargetStage::ManifestFetch);
        matches!(&failure.error, LocalDbError::CustomError(message) if message == "manifest boom");

        assert!(!runner.manifests_loaded);
        assert!(!runner.has_provisioned_dumps);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert!(telemetry.dump_requests().is_empty());
        assert_eq!(telemetry.builder_inits(), 0);
    }

    #[tokio::test]
    async fn dump_download_failure_is_propagated() {
        let telemetry = Telemetry::default();
        let manifest_map = manifest_for_a();
        let manifest_arc = Arc::new(manifest_map);
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    let prev = telemetry.record_manifest_fetch();
                    assert!(prev < 1, "manifest fetched too many times");
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = {
            let telemetry = telemetry.clone();
            Arc::new(move |url: &Url| {
                let telemetry = telemetry.clone();
                let url = url.clone();
                Box::pin(async move {
                    telemetry.record_dump(url.clone());
                    Err(LocalDbError::CustomError("download failed".into()))
                }) as DumpFuture
            })
        };

        let behaviors = HashMap::new();
        let engine_builder = engine_builder_for_behaviors(telemetry.clone(), behaviors);
        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);

        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run completes with failures");
        let report = unwrap_report(outcome);
        assert_eq!(report.successes.len(), 0);
        assert_eq!(report.failures.len(), 1);
        let failure = &report.failures[0];
        assert_eq!(failure.stage, TargetStage::DumpDownload);
        matches!(&failure.error, LocalDbError::CustomError(message) if message == "download failed");
        assert!(!runner.has_provisioned_dumps);
        assert!(runner.manifests_loaded);
        assert_eq!(telemetry.dump_requests().len(), 1);
    }

    #[tokio::test]
    async fn dump_failure_on_one_target_still_allows_other_success() {
        let telemetry = Telemetry::default();
        let manifest = manifest_for_both();
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::new(manifest);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    telemetry.record_manifest_fetch();
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = Arc::new(|url: &Url| {
            let url = url.clone();
            Box::pin(async move {
                if url == dump_url_a() {
                    Err(LocalDbError::CustomError("dump failed".into()))
                } else {
                    Ok(format!("-- dump for {}", url))
                }
            }) as DumpFuture
        });

        let engine_builder = engine_builder_for_behaviors(telemetry.clone(), HashMap::new());
        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);

        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner
            .run(&db)
            .await
            .expect("run completes with mixed results");
        let report = unwrap_report(outcome);
        assert_eq!(report.successes.len() + report.failures.len(), 2);
        assert!(report
            .failures
            .iter()
            .any(|failure| failure.stage == TargetStage::DumpDownload
                && failure.ob_id.orderbook_address == ORDERBOOK_A));
        assert!(report
            .successes
            .iter()
            .any(|success| success.outcome.ob_id.orderbook_address == ORDERBOOK_B));
        assert!(!runner.has_provisioned_dumps);
    }

    #[tokio::test]
    async fn engine_builder_error_is_propagated() {
        let telemetry = Telemetry::default();
        let manifest = manifest_for_both();
        let settings = two_orderbooks_settings_yaml();
        let behaviors = HashMap::from([(ORDERBOOK_KEY_B.to_string(), EngineBehavior::Success)]);

        let manifest_arc = Arc::new(manifest);
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    let prev = telemetry.record_manifest_fetch();
                    assert!(prev < 1, "manifest fetched too many times");
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = {
            let telemetry = telemetry.clone();
            let counter = Arc::new(AtomicUsize::new(0));
            Arc::new(move |url: &Url| {
                let telemetry = telemetry.clone();
                let counter = Arc::clone(&counter);
                let url = url.clone();
                Box::pin(async move {
                    let prev = counter.fetch_add(1, Ordering::SeqCst);
                    assert!(prev < 2, "dump downloaded too many times");
                    telemetry.record_dump(url.clone());
                    Ok(format!("-- dump for {}", url))
                }) as DumpFuture
            })
        };

        let engine_builder: TestEngineBuilder = {
            let telemetry = telemetry.clone();
            let behaviors = Arc::new(behaviors);
            Arc::new(move |target: &RunnerTarget| {
                if target.orderbook_key == ORDERBOOK_KEY_A {
                    return Err(LocalDbError::CustomError("builder failed".into()));
                }
                let behaviors = Arc::clone(&behaviors);
                let telemetry = telemetry.clone();
                let behavior = behaviors
                    .get(&target.orderbook_key)
                    .copied()
                    .unwrap_or(EngineBehavior::Success);
                telemetry.record_builder_init();
                let bootstrap = StubBootstrap::new(telemetry.clone(), target.orderbook_key.clone());
                let window = StubWindow::new(0, target.inputs.cfg.deployment_block);
                let events = StubEvents::new(target.inputs.cfg.deployment_block);
                let apply = StubApply::new(
                    telemetry.clone(),
                    target.orderbook_key.clone(),
                    behavior == EngineBehavior::ApplyFail,
                );
                let status = StubStatusBus::new(telemetry.clone(), target.orderbook_key.clone());
                Ok(EnginePipelines::new(
                    bootstrap, window, events, StubTokens, apply, status,
                ))
            })
        };

        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run completes with failures");
        let report = unwrap_report(outcome);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.successes.len(), 1);
        let failure = &report.failures[0];
        assert_eq!(failure.stage, TargetStage::EngineBuild);
        matches!(&failure.error, LocalDbError::CustomError(message) if message == "builder failed");
        assert!(runner.has_provisioned_dumps);
        assert_eq!(telemetry.dump_requests().len(), 2);
        let engine_runs = telemetry.engine_runs();
        assert!(
            engine_runs.iter().all(|key| key != ORDERBOOK_KEY_A),
            "engine should not run for target with builder failure"
        );
    }

    #[tokio::test]
    async fn engine_run_failure_is_propagated() {
        let telemetry = Telemetry::default();
        let mut behaviors = HashMap::new();
        behaviors.insert(ORDERBOOK_KEY_A.to_string(), EngineBehavior::ApplyFail);
        behaviors.insert(ORDERBOOK_KEY_B.to_string(), EngineBehavior::Success);
        let environment =
            build_environment(manifest_for_both(), behaviors, 1, 2, telemetry.clone());
        let settings = two_orderbooks_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();
        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run completes with failures");
        let report = unwrap_report(outcome);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.successes.len(), 1);
        let failure = &report.failures[0];
        assert_eq!(failure.stage, TargetStage::EngineRun);
        matches!(&failure.error, LocalDbError::CustomError(message) if message.starts_with("apply failed"));
        assert!(runner.has_provisioned_dumps);
        assert!(runner.manifests_loaded);
        assert_eq!(telemetry.dump_requests().len(), 2);
        assert_eq!(telemetry.manifest_fetch_count(), 1);
        assert!(telemetry
            .engine_runs()
            .iter()
            .any(|key| key == ORDERBOOK_KEY_A));
    }

    #[tokio::test]
    async fn engine_build_and_run_failures_are_both_reported() {
        let telemetry = Telemetry::default();
        let manifest = manifest_for_both();
        let settings = two_orderbooks_settings_yaml();

        let manifest_arc = Arc::new(manifest);
        let manifest_fetcher = {
            let telemetry = telemetry.clone();
            let manifest_arc = Arc::clone(&manifest_arc);
            Arc::new(move |_orderbooks: &HashMap<String, OrderbookCfg>| {
                let telemetry = telemetry.clone();
                let manifest_arc = Arc::clone(&manifest_arc);
                Box::pin(async move {
                    telemetry.record_manifest_fetch();
                    Ok((*manifest_arc).clone())
                }) as ManifestFuture
            })
        };

        let dump_downloader = default_dump_downloader();
        let engine_builder: TestEngineBuilder = {
            let telemetry = telemetry.clone();
            Arc::new(move |target: &RunnerTarget| {
                if target.orderbook_key == ORDERBOOK_KEY_A {
                    return Err(LocalDbError::CustomError("build boom".into()));
                }
                telemetry.record_builder_init();
                let bootstrap = StubBootstrap::new(telemetry.clone(), target.orderbook_key.clone());
                let window = StubWindow::new(0, target.inputs.cfg.deployment_block);
                let events = StubEvents::new(target.inputs.cfg.deployment_block);
                let apply = StubApply::new(telemetry.clone(), target.orderbook_key.clone(), true); // force engine run failure
                let status = StubStatusBus::new(telemetry.clone(), target.orderbook_key.clone());
                Ok(EnginePipelines::new(
                    bootstrap, window, events, StubTokens, apply, status,
                ))
            })
        };

        let environment = RunnerEnvironment::new(manifest_fetcher, dump_downloader, engine_builder);
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run completes with failures");
        let report = unwrap_report(outcome);
        assert_eq!(report.successes.len(), 0);
        assert_eq!(report.failures.len(), 2);
        assert!(report
            .failures
            .iter()
            .any(|f| f.ob_id.orderbook_address == ORDERBOOK_A));
        assert!(report
            .failures
            .iter()
            .any(|f| f.ob_id.orderbook_address == ORDERBOOK_B));
    }

    #[tokio::test]
    async fn existing_dump_not_overwritten_without_manifest_entry() {
        let telemetry = Telemetry::default();
        let environment =
            build_environment(ManifestMap::new(), HashMap::new(), 1, 0, telemetry.clone());
        let settings = single_orderbook_settings_yaml();
        let mut runner =
            ClientRunner::with_environment(settings, environment, AlwaysLeadership).unwrap();

        assert_eq!(runner.base_targets.len(), 1);
        runner.base_targets[0].inputs.dump_str = Some("-- preloaded dump".to_string());

        let db = RecordingDb::default();
        prepare_db_for_targets(&db, &runner.base_targets);

        let outcome = runner.run(&db).await.expect("run succeeds");
        let report = unwrap_report(outcome);
        assert_eq!(report.successes.len(), 1);
        assert!(telemetry.dump_requests().is_empty());
        let records = telemetry.bootstrap_records();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.orderbook_key, ORDERBOOK_KEY_A);
        assert_eq!(record.dump_sql.as_deref(), Some("-- preloaded dump"));
        assert_eq!(record.latest_block, 0);
    }
}
