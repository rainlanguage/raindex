use super::super::config::NetworkRunnerConfig;
use super::super::environment::default_environment;
use super::super::leadership::DefaultLeadership;
use super::super::ClientRunner;
use crate::local_db::pipeline::adapters::bootstrap::BootstrapPipeline;
use crate::local_db::pipeline::adapters::{
    apply::DefaultApplyPipeline, events::DefaultEventsPipeline, tokens::DefaultTokensPipeline,
    window::DefaultWindowPipeline,
};
use crate::local_db::pipeline::runner::utils::ParsedRunnerSettings;
use crate::local_db::pipeline::runner::RunOutcome;
use crate::local_db::query::LocalDbQueryExecutor;
use crate::local_db::LocalDbError;
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
use crate::raindex_client::local_db::pipeline::status::TracingStatusBus;
use crate::raindex_client::local_db::{LocalDb, SyncReadiness};
use raindex_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
use raindex_app_settings::network::NetworkCfg;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

type NativeClientRunner = ClientRunner<
    ClientBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    TracingStatusBus,
    DefaultLeadership,
>;

trait NativeRunner {
    fn run_once<'a, DB: LocalDbQueryExecutor + ?Sized>(
        &'a mut self,
        db: &'a DB,
    ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>>;
}

impl NativeRunner for NativeClientRunner {
    fn run_once<'a, DB: LocalDbQueryExecutor + ?Sized>(
        &'a mut self,
        db: &'a DB,
    ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>> {
        Box::pin(async move { self.run(db).await })
    }
}

pub struct NativeSyncHandle {
    stop_flag: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for NativeSyncHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSyncHandle")
            .field("stopped", &self.stop_flag.load(Ordering::SeqCst))
            .finish()
    }
}

impl NativeSyncHandle {
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    pub fn stop_and_join(mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn start(
    settings: ParsedRunnerSettings,
    db_path: PathBuf,
    sync_readiness: SyncReadiness,
) -> Result<NativeSyncHandle, LocalDbError> {
    let mut networks_map: HashMap<String, NetworkCfg> = HashMap::new();
    for raindex_cfg in settings.raindexes.values() {
        networks_map
            .entry(raindex_cfg.network.key.clone())
            .or_insert_with(|| (*raindex_cfg.network).clone());
    }
    let mut networks: Vec<NetworkCfg> = networks_map.into_values().collect();
    networks.sort_by(|a, b| a.key.cmp(&b.key));

    if networks.is_empty() {
        return Err(LocalDbError::CustomError(
            "no networks found in settings".to_string(),
        ));
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    let thread_handle = thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime for native sync");

        rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            local_set
                .run_until(async move {
                    let executor = crate::local_db::executor::RusqliteExecutor::new(&db_path);
                    let db = LocalDb::new(executor);

                    let bootstrap = ClientBootstrapAdapter::new();
                    if let Err(err) = bootstrap.runner_run(&db, Some(DB_SCHEMA_VERSION)).await {
                        tracing::error!(error = %err, "native sync bootstrap failed");
                        return;
                    }

                    for network in &networks {
                        if stop_flag_clone.load(Ordering::SeqCst) {
                            return;
                        }

                        let config = match NetworkRunnerConfig::from_global_settings(
                            &settings,
                            &network.key,
                        ) {
                            Ok(config) => config,
                            Err(err) => {
                                tracing::error!(
                                    network = %network.key,
                                    chain_id = network.chain_id,
                                    error = %err,
                                    "failed to create network runner config"
                                );
                                continue;
                            }
                        };

                        let interval_ms = match settings.syncs.get(&network.key) {
                            Some(sync_cfg) => sync_cfg.sync_interval_ms,
                            None => {
                                tracing::error!(
                                    network = %network.key,
                                    chain_id = network.chain_id,
                                    "missing local-db-sync settings for network"
                                );
                                continue;
                            }
                        };

                        let leadership = DefaultLeadership::with_network_key(network.key.clone());
                        let environment = default_environment();

                        let runner = match NativeClientRunner::from_config(
                            config,
                            environment,
                            leadership,
                        ) {
                            Ok(r) => r,
                            Err(err) => {
                                tracing::error!(
                                    network = %network.key,
                                    chain_id = network.chain_id,
                                    error = %err,
                                    "failed to create native runner"
                                );
                                continue;
                            }
                        };

                        let db_clone = db.clone();
                        let stop = Arc::clone(&stop_flag_clone);

                        tokio::task::spawn_local(run_network_loop(
                            runner,
                            db_clone,
                            stop,
                            interval_ms,
                            network.key.clone(),
                            network.chain_id,
                            sync_readiness.clone(),
                        ));
                    }

                    loop {
                        if stop_flag_clone.load(Ordering::SeqCst) {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                })
                .await;
        });
    });

    Ok(NativeSyncHandle {
        stop_flag,
        thread_handle: Some(thread_handle),
    })
}

async fn run_network_loop<R: NativeRunner>(
    mut runner: R,
    db: LocalDb,
    stop_flag: Arc<AtomicBool>,
    interval_ms: u64,
    network_key: String,
    chain_id: u32,
    sync_readiness: SyncReadiness,
) {
    tracing::info!(network = %network_key, chain_id, "starting native sync loop");

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        match runner.run_once(&db).await {
            Ok(outcome) => match outcome {
                RunOutcome::Report(report) => {
                    if report.failures.is_empty() {
                        sync_readiness.mark_ready(chain_id);
                        tracing::debug!(
                            network = %network_key,
                            chain_id,
                            successes = report.successes.len(),
                            "sync cycle completed"
                        );
                    } else {
                        for failure in &report.failures {
                            tracing::warn!(
                                network = %network_key,
                                chain_id,
                                raindex = %format!("{:#x}", failure.raindex_id.raindex_address),
                                stage = ?failure.stage,
                                error = %failure.error,
                                "sync target failed"
                            );
                        }
                    }
                }
                RunOutcome::NotLeader => {
                    tracing::debug!(
                        network = %network_key,
                        chain_id,
                        "not leader, skipping sync cycle"
                    );
                }
            },
            Err(err) => {
                tracing::error!(
                    network = %network_key,
                    chain_id,
                    error = %err,
                    "sync cycle error"
                );
            }
        }

        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
    }

    tracing::info!(network = %network_key, chain_id, "native sync loop stopped");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::pipeline::runner::{RunReport, TargetFailure, TargetStage};
    use crate::local_db::query::{FromDbJson, LocalDbQueryError, SqlStatement, SqlStatementBatch};
    use crate::local_db::RaindexIdentifier;
    use alloy::primitives::Address;
    use std::collections::VecDeque;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    struct NoopExecutor;

    #[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
    impl LocalDbQueryExecutor for NoopExecutor {
        async fn execute_batch(&self, _: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            Ok(())
        }
        async fn query_json<T: FromDbJson>(
            &self,
            _: &SqlStatement,
        ) -> Result<T, LocalDbQueryError> {
            Err(LocalDbQueryError::database("noop"))
        }
        async fn query_text(&self, _: &SqlStatement) -> Result<String, LocalDbQueryError> {
            Ok(String::new())
        }
        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            Ok(())
        }
    }

    fn noop_local_db() -> LocalDb {
        LocalDb::new(NoopExecutor)
    }

    struct RecordingRunner {
        calls: Arc<AtomicUsize>,
        failures: Arc<AtomicUsize>,
        outcomes: Arc<Mutex<VecDeque<Option<bool>>>>,
    }

    impl RecordingRunner {
        fn new(
            calls: Arc<AtomicUsize>,
            failures: Arc<AtomicUsize>,
            outcomes: Vec<Option<bool>>,
        ) -> Self {
            Self {
                calls,
                failures,
                outcomes: Arc::new(Mutex::new(VecDeque::from(outcomes))),
            }
        }
    }

    impl NativeRunner for RecordingRunner {
        fn run_once<'a, DB: LocalDbQueryExecutor + ?Sized>(
            &'a mut self,
            _db: &'a DB,
        ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>> {
            let calls = Arc::clone(&self.calls);
            let failures = Arc::clone(&self.failures);
            let outcomes = Arc::clone(&self.outcomes);

            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                let outcome = outcomes.lock().unwrap().pop_front().unwrap_or(Some(false));
                match outcome {
                    Some(should_fail) => {
                        if should_fail {
                            failures.fetch_add(1, Ordering::SeqCst);
                            let failure = TargetFailure {
                                raindex_id: RaindexIdentifier::new(1, Address::ZERO),
                                raindex_key: None,
                                stage: TargetStage::EngineRun,
                                error: LocalDbError::CustomError("runner failure".to_string()),
                            };
                            Ok(RunOutcome::Report(RunReport {
                                successes: vec![],
                                failures: vec![failure],
                            }))
                        } else {
                            Ok(RunOutcome::Report(RunReport {
                                successes: vec![],
                                failures: vec![],
                            }))
                        }
                    }
                    None => Ok(RunOutcome::NotLeader),
                }
            })
        }
    }

    struct ErrorRunner {
        calls: Arc<AtomicUsize>,
    }

    impl NativeRunner for ErrorRunner {
        fn run_once<'a, DB: LocalDbQueryExecutor + ?Sized>(
            &'a mut self,
            _db: &'a DB,
        ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>> {
            let calls = Arc::clone(&self.calls);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(LocalDbError::CustomError("sync error".to_string()))
            })
        }
    }

    #[test]
    fn start_returns_error_for_empty_settings() {
        let settings = ParsedRunnerSettings {
            raindexes: HashMap::new(),
            syncs: HashMap::new(),
        };
        let result = start(
            settings,
            PathBuf::from("/tmp/test.db"),
            SyncReadiness::new(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn stop_flag_works() {
        let handle = NativeSyncHandle {
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        };
        assert!(!handle.stop_flag.load(Ordering::SeqCst));
        handle.stop();
        assert!(handle.stop_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn stop_and_join_with_completed_thread() {
        let handle = NativeSyncHandle {
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: Some(thread::spawn(|| {})),
        };
        handle.stop_and_join();
    }

    #[test]
    fn stop_and_join_without_thread_handle() {
        let handle = NativeSyncHandle {
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        };
        handle.stop_and_join();
    }

    #[test]
    fn debug_impl_shows_stopped_state() {
        let handle = NativeSyncHandle {
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        };
        let debug = format!("{:?}", handle);
        assert!(debug.contains("NativeSyncHandle"));
        assert!(debug.contains("stopped: false"));

        handle.stop();
        let debug = format!("{:?}", handle);
        assert!(debug.contains("stopped: true"));
    }

    #[tokio::test]
    async fn network_loop_runs_until_stopped() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let calls = Arc::new(AtomicUsize::new(0));
                let failures = Arc::new(AtomicUsize::new(0));
                let runner = RecordingRunner::new(
                    Arc::clone(&calls),
                    Arc::clone(&failures),
                    vec![Some(false)],
                );
                let stop_flag = Arc::new(AtomicBool::new(false));
                let readiness = SyncReadiness::new();

                tokio::task::spawn_local(run_network_loop(
                    runner,
                    noop_local_db(),
                    Arc::clone(&stop_flag),
                    1,
                    "test".to_string(),
                    1,
                    readiness.clone(),
                ));

                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                stop_flag.store(true, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                assert!(calls.load(Ordering::SeqCst) >= 1);
                assert_eq!(failures.load(Ordering::SeqCst), 0);
                assert!(
                    readiness.is_ready(1),
                    "chain should be marked ready after successful run"
                );
            })
            .await;
    }

    #[tokio::test]
    async fn network_loop_continues_after_failure() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let calls = Arc::new(AtomicUsize::new(0));
                let failures = Arc::new(AtomicUsize::new(0));
                let runner = RecordingRunner::new(
                    Arc::clone(&calls),
                    Arc::clone(&failures),
                    vec![Some(true), Some(false)],
                );
                let stop_flag = Arc::new(AtomicBool::new(false));
                let readiness = SyncReadiness::new();

                tokio::task::spawn_local(run_network_loop(
                    runner,
                    noop_local_db(),
                    Arc::clone(&stop_flag),
                    1,
                    "test".to_string(),
                    1,
                    readiness.clone(),
                ));

                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                stop_flag.store(true, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                assert!(failures.load(Ordering::SeqCst) >= 1);
                assert!(calls.load(Ordering::SeqCst) >= 2);
                assert!(
                    readiness.is_ready(1),
                    "chain should be marked ready after recovery"
                );
            })
            .await;
    }

    #[tokio::test]
    async fn network_loop_handles_not_leader() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let calls = Arc::new(AtomicUsize::new(0));
                let failures = Arc::new(AtomicUsize::new(0));
                let runner = RecordingRunner::new(
                    Arc::clone(&calls),
                    Arc::clone(&failures),
                    vec![None, Some(false)],
                );
                let stop_flag = Arc::new(AtomicBool::new(false));
                let readiness = SyncReadiness::new();

                tokio::task::spawn_local(run_network_loop(
                    runner,
                    noop_local_db(),
                    Arc::clone(&stop_flag),
                    1,
                    "test".to_string(),
                    1,
                    readiness.clone(),
                ));

                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                stop_flag.store(true, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                assert!(calls.load(Ordering::SeqCst) >= 2);
                assert_eq!(failures.load(Ordering::SeqCst), 0);
                assert!(
                    readiness.is_ready(1),
                    "chain should be marked ready after successful cycle"
                );
            })
            .await;
    }

    #[tokio::test]
    async fn network_loop_handles_runner_error() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let calls = Arc::new(AtomicUsize::new(0));
                let runner = ErrorRunner {
                    calls: Arc::clone(&calls),
                };
                let stop_flag = Arc::new(AtomicBool::new(false));
                let readiness = SyncReadiness::new();

                tokio::task::spawn_local(run_network_loop(
                    runner,
                    noop_local_db(),
                    Arc::clone(&stop_flag),
                    1,
                    "test".to_string(),
                    1,
                    readiness.clone(),
                ));

                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                stop_flag.store(true, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                assert!(
                    calls.load(Ordering::SeqCst) >= 2,
                    "loop should continue after errors, got {} calls",
                    calls.load(Ordering::SeqCst)
                );
                assert!(
                    !readiness.is_ready(1),
                    "chain should not be marked ready after errors"
                );
            })
            .await;
    }

    #[tokio::test]
    async fn network_loop_respects_pre_run_stop_flag() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let calls = Arc::new(AtomicUsize::new(0));
                let failures = Arc::new(AtomicUsize::new(0));
                let runner = RecordingRunner::new(
                    Arc::clone(&calls),
                    Arc::clone(&failures),
                    vec![Some(false)],
                );
                let stop_flag = Arc::new(AtomicBool::new(true));
                let readiness = SyncReadiness::new();

                tokio::task::spawn_local(run_network_loop(
                    runner,
                    noop_local_db(),
                    Arc::clone(&stop_flag),
                    1,
                    "test".to_string(),
                    1,
                    readiness.clone(),
                ));

                tokio::time::sleep(std::time::Duration::from_millis(20)).await;

                assert_eq!(calls.load(Ordering::SeqCst), 0);
                assert!(
                    !readiness.is_ready(1),
                    "chain should not be marked ready when stopped before running"
                );
            })
            .await;
    }

    #[tokio::test]
    async fn network_loop_respects_post_run_stop_flag() {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let calls = Arc::new(AtomicUsize::new(0));
                let failures = Arc::new(AtomicUsize::new(0));
                let runner = RecordingRunner::new(
                    Arc::clone(&calls),
                    Arc::clone(&failures),
                    vec![Some(false)],
                );
                let stop_flag = Arc::new(AtomicBool::new(false));
                let stop_clone = Arc::clone(&stop_flag);
                let calls_clone = Arc::clone(&calls);
                let readiness = SyncReadiness::new();

                tokio::task::spawn_local(run_network_loop(
                    runner,
                    noop_local_db(),
                    stop_clone,
                    10000,
                    "test".to_string(),
                    1,
                    readiness.clone(),
                ));

                for _ in 0..200 {
                    if calls_clone.load(Ordering::SeqCst) >= 1 {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                }

                stop_flag.store(true, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;

                assert_eq!(
                    calls.load(Ordering::SeqCst),
                    1,
                    "loop should stop after first run without waiting for interval"
                );
                assert!(
                    readiness.is_ready(1),
                    "chain should be marked ready after successful run"
                );
            })
            .await;
    }
}
