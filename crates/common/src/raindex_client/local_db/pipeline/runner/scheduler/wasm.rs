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
use crate::local_db::LocalDbError;
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
use crate::raindex_client::local_db::pipeline::status::{
    set_scheduler_state, set_status_callback, ClientStatusBus,
};
use crate::raindex_client::local_db::{LocalDb, NetworkSyncStatus, SchedulerState, SyncReadiness};
use gloo_timers::future::TimeoutFuture;
use js_sys::Function;
use rain_orderbook_app_settings::local_db_manifest::DB_SCHEMA_VERSION;
use rain_orderbook_app_settings::network::NetworkCfg;
use std::cell::Cell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use wasm_bindgen_utils::prelude::wasm_bindgen_futures::spawn_local;
use wasm_bindgen_utils::prelude::*;

type DefaultClientRunner = ClientRunner<
    ClientBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    ClientStatusBus,
    DefaultLeadership,
>;

trait SchedulerRunner {
    fn run_once<'a>(
        &'a mut self,
        db_executor: &'a LocalDb,
    ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>>;

    fn chain_id(&self) -> Option<u32>;
}

impl SchedulerRunner for DefaultClientRunner {
    fn run_once<'a>(
        &'a mut self,
        db_executor: &'a LocalDb,
    ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>> {
        Box::pin(async move { self.run(db_executor).await })
    }

    fn chain_id(&self) -> Option<u32> {
        self.chain_id()
    }
}

#[derive(Debug)]
pub struct SchedulerHandle {
    stop_flag: Rc<Cell<bool>>,
    networks: Vec<NetworkCfg>,
}

impl SchedulerHandle {
    pub fn stop(&self) {
        self.stop_flag.set(true);
    }

    pub fn networks(&self) -> &[NetworkCfg] {
        &self.networks
    }

    pub fn network_keys(&self) -> Vec<String> {
        self.networks.iter().map(|n| n.key.clone()).collect()
    }
}

pub(crate) fn start(
    settings: ParsedRunnerSettings,
    db: LocalDb,
    status_callback: Option<Function>,
    sync_readiness: SyncReadiness,
) -> Result<SchedulerHandle, LocalDbError> {
    let mut networks_map: HashMap<String, NetworkCfg> = HashMap::new();
    for ob in settings.orderbooks.values() {
        networks_map
            .entry(ob.network.key.clone())
            .or_insert_with(|| (*ob.network).clone());
    }
    let mut networks: Vec<NetworkCfg> = networks_map.into_values().collect();
    networks.sort_by(|a, b| a.key.cmp(&b.key));

    if networks.is_empty() {
        return Err(LocalDbError::CustomError(
            "no networks found in settings".to_string(),
        ));
    }

    let stop_flag = Rc::new(Cell::new(false));
    let callback = status_callback.map(Rc::new);

    let bootstrap = ClientBootstrapAdapter::new();
    let db_clone = db.clone();
    let stop_flag_init = Rc::clone(&stop_flag);
    let networks_for_spawn = networks.clone();
    let settings_clone = settings.clone();

    spawn_local(async move {
        if stop_flag_init.get() {
            return;
        }

        if let Err(err) = bootstrap
            .runner_run(&db_clone, Some(DB_SCHEMA_VERSION))
            .await
        {
            for network in &networks_for_spawn {
                emit_network_status(
                    callback.as_deref(),
                    NetworkSyncStatus::failure(network.chain_id, err.to_readable_msg()),
                );
            }
            return;
        }

        set_status_callback(callback.clone());

        for network in &networks_for_spawn {
            let config =
                match NetworkRunnerConfig::from_global_settings(&settings_clone, &network.key) {
                    Ok(config) => config,
                    Err(err) => {
                        emit_network_status(
                            callback.as_deref(),
                            NetworkSyncStatus::failure(network.chain_id, err.to_readable_msg()),
                        );
                        continue;
                    }
                };

            let leadership = DefaultLeadership::with_network_key(network.key.clone());
            let environment = default_environment();

            let runner = match ClientRunner::from_config(config.clone(), environment, leadership) {
                Ok(r) => r,
                Err(err) => {
                    emit_network_status(
                        callback.as_deref(),
                        NetworkSyncStatus::failure(network.chain_id, err.to_readable_msg()),
                    );
                    continue;
                }
            };

            let interval_ms = match settings_clone.syncs.get(&network.key) {
                Some(sync_cfg) => sync_cfg.sync_interval_ms as u32,
                None => {
                    emit_network_status(
                        callback.as_deref(),
                        NetworkSyncStatus::failure(
                            network.chain_id,
                            format!(
                                "missing local-db-sync settings for network '{}'",
                                network.key
                            ),
                        ),
                    );
                    continue;
                }
            };

            spawn_network_loop(
                runner,
                db_clone.clone(),
                callback.clone(),
                Rc::clone(&stop_flag_init),
                interval_ms,
                sync_readiness.clone(),
            );
        }
    });

    Ok(SchedulerHandle {
        stop_flag,
        networks,
    })
}

fn spawn_network_loop<R>(
    runner: R,
    db: LocalDb,
    callback: Option<Rc<Function>>,
    stop_flag: Rc<Cell<bool>>,
    interval_ms: u32,
    sync_readiness: SyncReadiness,
) where
    R: SchedulerRunner + 'static,
{
    spawn_local(async move {
        run_network_loop(runner, db, callback, stop_flag, interval_ms, sync_readiness).await;
    });
}

async fn run_network_loop<R>(
    mut runner: R,
    db: LocalDb,
    callback: Option<Rc<Function>>,
    stop_flag: Rc<Cell<bool>>,
    interval_ms: u32,
    sync_readiness: SyncReadiness,
) where
    R: SchedulerRunner + 'static,
{
    let chain_id = runner.chain_id().unwrap_or(0);
    let mut was_leader_last_cycle = false;

    emit_network_status(callback.as_deref(), NetworkSyncStatus::syncing(chain_id));

    loop {
        if stop_flag.get() {
            break;
        }

        if was_leader_last_cycle {
            emit_network_status(callback.as_deref(), NetworkSyncStatus::syncing(chain_id));
        }

        match runner.run_once(&db).await {
            Ok(outcome) => match outcome {
                RunOutcome::Report(report) => {
                    was_leader_last_cycle = true;
                    set_scheduler_state(SchedulerState::Leader);

                    if report.failures.is_empty() {
                        sync_readiness.mark_ready(chain_id);
                        emit_network_status(
                            callback.as_deref(),
                            NetworkSyncStatus::active(chain_id, SchedulerState::Leader),
                        );
                    } else {
                        let first = &report.failures[0];
                        let msg = format!(
                            "ob {:#x} failed at {:?}: {}",
                            first.ob_id.orderbook_address,
                            first.stage,
                            first.error.to_readable_msg()
                        );
                        emit_network_status(
                            callback.as_deref(),
                            NetworkSyncStatus::failure(chain_id, msg),
                        );
                    }
                }
                RunOutcome::NotLeader => {
                    was_leader_last_cycle = false;
                    set_scheduler_state(SchedulerState::NotLeader);
                    emit_network_status(
                        callback.as_deref(),
                        NetworkSyncStatus::active(chain_id, SchedulerState::NotLeader),
                    );
                }
            },
            Err(err) => {
                was_leader_last_cycle = true;
                emit_network_status(
                    callback.as_deref(),
                    NetworkSyncStatus::failure(chain_id, err.to_readable_msg()),
                );
            }
        }

        if stop_flag.get() {
            break;
        }

        TimeoutFuture::new(interval_ms).await;
    }
}

fn emit_network_status(callback: Option<&Function>, status: NetworkSyncStatus) {
    if let Some(callback) = callback {
        if let Ok(value) = serde_wasm_bindgen::to_value(&status) {
            let _ = callback.call1(&JsValue::NULL, &value);
        }
    }
}

#[cfg(all(test, target_family = "wasm", feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use crate::local_db::pipeline::runner::utils::parse_runner_settings;
    use crate::local_db::pipeline::runner::{RunReport, TargetFailure, TargetStage};
    use crate::local_db::OrderbookIdentifier;
    use crate::raindex_client::local_db::pipeline::status::get_scheduler_state;
    use crate::raindex_client::local_db::LocalDbStatus;
    use alloy::primitives::Address;
    use gloo_timers::future::TimeoutFuture;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn noop_callback() -> Function {
        Function::new_no_args("return undefined;")
    }

    fn noop_local_db() -> LocalDb {
        LocalDb::from_js_callback(noop_callback(), None)
    }

    impl SchedulerHandle {
        pub(crate) fn stop_flag_ptr(&self) -> *const Cell<bool> {
            Rc::as_ptr(&self.stop_flag)
        }
    }

    struct RecordingRunner {
        chain_id: u32,
        calls: Rc<Cell<usize>>,
        failures: Rc<Cell<usize>>,
        outcomes: Rc<RefCell<VecDeque<Option<bool>>>>,
    }

    impl RecordingRunner {
        fn new(
            chain_id: u32,
            calls: Rc<Cell<usize>>,
            failures: Rc<Cell<usize>>,
            outcomes: Vec<Option<bool>>,
        ) -> Self {
            Self {
                chain_id,
                calls,
                failures,
                outcomes: Rc::new(RefCell::new(VecDeque::from(outcomes))),
            }
        }
    }

    impl SchedulerRunner for RecordingRunner {
        fn run_once<'a>(
            &'a mut self,
            _db_executor: &'a LocalDb,
        ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>> {
            let calls = Rc::clone(&self.calls);
            let failures = Rc::clone(&self.failures);
            let outcomes = Rc::clone(&self.outcomes);

            Box::pin(async move {
                calls.set(calls.get() + 1);
                let outcome = outcomes.borrow_mut().pop_front().unwrap_or(Some(false));
                match outcome {
                    Some(should_fail) => {
                        if should_fail {
                            failures.set(failures.get() + 1);
                            let failure = TargetFailure {
                                ob_id: OrderbookIdentifier::new(1, Address::ZERO),
                                orderbook_key: None,
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

        fn chain_id(&self) -> Option<u32> {
            Some(self.chain_id)
        }
    }

    #[wasm_bindgen_test]
    async fn start_returns_error_for_invalid_yaml() {
        let result = parse_runner_settings("not yaml");
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn network_loop_runs_until_stopped() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner = RecordingRunner::new(
            1,
            Rc::clone(&calls),
            Rc::clone(&failures),
            vec![Some(false)],
        );
        let stop_flag = Rc::new(Cell::new(false));

        spawn_network_loop(
            runner,
            noop_local_db(),
            None,
            Rc::clone(&stop_flag),
            1,
            SyncReadiness::new(),
        );

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(3).await;

        stop_flag.set(true);
        TimeoutFuture::new(5).await;

        assert!(calls.get() >= 1);
        assert_eq!(failures.get(), 0);
    }

    #[wasm_bindgen_test]
    async fn network_loop_continues_after_failure() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner = RecordingRunner::new(
            1,
            Rc::clone(&calls),
            Rc::clone(&failures),
            vec![Some(true), Some(false)],
        );
        let stop_flag = Rc::new(Cell::new(false));

        spawn_network_loop(
            runner,
            noop_local_db(),
            None,
            Rc::clone(&stop_flag),
            1,
            SyncReadiness::new(),
        );

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(5).await;

        stop_flag.set(true);
        TimeoutFuture::new(5).await;

        assert!(failures.get() >= 1);
        assert!(calls.get() >= 2);
    }

    #[wasm_bindgen_test]
    async fn network_loop_handles_not_leader() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner = RecordingRunner::new(
            1,
            Rc::clone(&calls),
            Rc::clone(&failures),
            vec![None, Some(false)],
        );
        let stop_flag = Rc::new(Cell::new(false));

        let statuses = Rc::new(RefCell::new(Vec::new()));
        let status_callback = {
            let statuses = Rc::clone(&statuses);
            let closure = Closure::wrap(Box::new(move |value: JsValue| {
                let snapshot: NetworkSyncStatus =
                    serde_wasm_bindgen::from_value(value).expect("valid status value");
                statuses.borrow_mut().push(snapshot);
            }) as Box<dyn FnMut(JsValue)>);
            let function: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();
            function
        };

        spawn_network_loop(
            runner,
            noop_local_db(),
            Some(Rc::new(status_callback)),
            Rc::clone(&stop_flag),
            1,
            SyncReadiness::new(),
        );

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(5).await;

        stop_flag.set(true);
        TimeoutFuture::new(5).await;

        let recorded = statuses.borrow();
        assert!(
            recorded
                .iter()
                .any(|s| s.scheduler_state == SchedulerState::NotLeader),
            "expected NotLeader status"
        );
        assert_eq!(
            get_scheduler_state(),
            SchedulerState::Leader,
            "scheduler state should be Leader after recovering"
        );
    }

    #[wasm_bindgen_test]
    async fn network_loop_emits_status_transitions() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner = RecordingRunner::new(
            42,
            Rc::clone(&calls),
            Rc::clone(&failures),
            vec![Some(true), Some(false)],
        );
        let stop_flag = Rc::new(Cell::new(false));

        let statuses = Rc::new(RefCell::new(Vec::new()));
        let status_callback = {
            let statuses = Rc::clone(&statuses);
            let closure = Closure::wrap(Box::new(move |value: JsValue| {
                let snapshot: NetworkSyncStatus =
                    serde_wasm_bindgen::from_value(value).expect("valid status value");
                statuses.borrow_mut().push(snapshot);
            }) as Box<dyn FnMut(JsValue)>);
            let function: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();
            function
        };

        spawn_network_loop(
            runner,
            noop_local_db(),
            Some(Rc::new(status_callback)),
            Rc::clone(&stop_flag),
            1,
            SyncReadiness::new(),
        );

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(5).await;

        stop_flag.set(true);
        TimeoutFuture::new(5).await;

        let recorded = statuses.borrow();
        assert!(
            recorded
                .iter()
                .any(|s| s.status == LocalDbStatus::Failure && s.chain_id == 42),
            "expected failure status for chain 42"
        );
        assert!(
            recorded
                .iter()
                .any(|s| s.status == LocalDbStatus::Active && s.chain_id == 42),
            "expected active status for chain 42"
        );
        assert!(
            recorded
                .iter()
                .any(|s| s.status == LocalDbStatus::Syncing && s.chain_id == 42),
            "expected syncing status for chain 42"
        );
        assert!(
            recorded.iter().all(|s| s.chain_id == 42),
            "expected all statuses to have chain_id 42"
        );
    }

    #[wasm_bindgen_test]
    async fn network_loop_emits_syncing_immediately_on_start() {
        let calls = Rc::new(Cell::new(0));
        let failures = Rc::new(Cell::new(0));
        let runner = RecordingRunner::new(
            99,
            Rc::clone(&calls),
            Rc::clone(&failures),
            vec![Some(false)],
        );
        let stop_flag = Rc::new(Cell::new(false));

        let statuses = Rc::new(RefCell::new(Vec::new()));
        let status_callback = {
            let statuses = Rc::clone(&statuses);
            let closure = Closure::wrap(Box::new(move |value: JsValue| {
                let snapshot: NetworkSyncStatus =
                    serde_wasm_bindgen::from_value(value).expect("valid status value");
                statuses.borrow_mut().push(snapshot);
            }) as Box<dyn FnMut(JsValue)>);
            let function: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();
            function
        };

        spawn_network_loop(
            runner,
            noop_local_db(),
            Some(Rc::new(status_callback)),
            Rc::clone(&stop_flag),
            1,
            SyncReadiness::new(),
        );

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(5).await;

        stop_flag.set(true);
        TimeoutFuture::new(5).await;

        let recorded = statuses.borrow();
        assert!(
            !recorded.is_empty(),
            "expected at least one status to be emitted"
        );
        assert_eq!(
            recorded[0].status,
            LocalDbStatus::Syncing,
            "first status should be Syncing"
        );
        assert_eq!(
            recorded[0].chain_id, 99,
            "first status should have correct chain_id"
        );
    }

    struct DelayedRunner {
        chain_id: u32,
        calls: Rc<Cell<usize>>,
        delay_ms: u32,
    }

    impl DelayedRunner {
        fn new(chain_id: u32, calls: Rc<Cell<usize>>, delay_ms: u32) -> Self {
            Self {
                chain_id,
                calls,
                delay_ms,
            }
        }
    }

    impl SchedulerRunner for DelayedRunner {
        fn run_once<'a>(
            &'a mut self,
            _db_executor: &'a LocalDb,
        ) -> Pin<Box<dyn Future<Output = Result<RunOutcome, LocalDbError>> + 'a>> {
            let calls = Rc::clone(&self.calls);
            let delay_ms = self.delay_ms;

            Box::pin(async move {
                if delay_ms > 0 {
                    TimeoutFuture::new(delay_ms).await;
                }
                calls.set(calls.get() + 1);
                Ok(RunOutcome::Report(RunReport {
                    successes: vec![],
                    failures: vec![],
                }))
            })
        }

        fn chain_id(&self) -> Option<u32> {
            Some(self.chain_id)
        }
    }

    #[wasm_bindgen_test]
    fn scheduler_handle_networks_returns_correct_network_configs() {
        use crate::raindex_client::tests::get_local_db_test_yaml;

        let yaml = get_local_db_test_yaml();
        let settings = parse_runner_settings(&yaml).expect("should parse valid yaml");
        let handle = start(settings, noop_local_db(), None, SyncReadiness::new())
            .expect("should start with valid yaml");

        handle.stop();

        let networks = handle.networks();
        assert_eq!(networks.len(), 1, "expected exactly one network");
        assert_eq!(networks[0].key, "arbitrum");
        assert_eq!(networks[0].chain_id, 42161);

        let network_keys = handle.network_keys();
        assert_eq!(network_keys, vec!["arbitrum".to_string()]);
    }

    #[wasm_bindgen_test]
    async fn slow_network_does_not_block_fast_networks() {
        let slow_calls = Rc::new(Cell::new(0));
        let fast_calls = Rc::new(Cell::new(0));

        let slow_runner = DelayedRunner::new(1, Rc::clone(&slow_calls), 100);
        let fast_runner = DelayedRunner::new(2, Rc::clone(&fast_calls), 0);

        let stop_flag = Rc::new(Cell::new(false));

        spawn_network_loop(
            slow_runner,
            noop_local_db(),
            None,
            Rc::clone(&stop_flag),
            1,
            SyncReadiness::new(),
        );
        spawn_network_loop(
            fast_runner,
            noop_local_db(),
            None,
            Rc::clone(&stop_flag),
            1,
            SyncReadiness::new(),
        );

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(50).await;

        let fast_count_midway = fast_calls.get();
        let slow_count_midway = slow_calls.get();

        TimeoutFuture::new(100).await;

        stop_flag.set(true);
        TimeoutFuture::new(10).await;

        assert!(
            fast_count_midway > slow_count_midway,
            "fast network should have more cycles than slow network midway: fast={}, slow={}",
            fast_count_midway,
            slow_count_midway
        );
        assert!(
            fast_calls.get() >= 3,
            "fast network should complete multiple cycles: got {}",
            fast_calls.get()
        );
        assert!(
            slow_calls.get() >= 1,
            "slow network should complete at least one cycle: got {}",
            slow_calls.get()
        );
    }
}
