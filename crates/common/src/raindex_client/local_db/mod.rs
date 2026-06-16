use super::{RaindexClient, RaindexError};
pub use crate::local_db::pipeline::SyncPhase;
use crate::local_db::query::{
    FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
};
use crate::local_db::{LocalDbError, OrderbookIdentifier};
use executor::JsCallbackExecutor;
use pipeline::runner::scheduler;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, future::Future, pin::Pin, rc::Rc};
use tsify::Tsify;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

pub mod executor;
pub mod orders;
pub mod pipeline;
pub mod query;
pub mod transactions;
pub mod vaults;

type ExecuteBatchFn =
    dyn Fn(
        &SqlStatementBatch,
    ) -> Pin<Box<dyn Future<Output = Result<(), LocalDbQueryError>> + 'static>>;

type QueryTextFn =
    dyn Fn(
        &SqlStatement,
    ) -> Pin<Box<dyn Future<Output = Result<String, LocalDbQueryError>> + 'static>>;

type QueryJsonFn =
    dyn Fn(
        &SqlStatement,
    ) -> Pin<Box<dyn Future<Output = Result<Value, LocalDbQueryError>> + 'static>>;

type WipeAndRecreateFn =
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), LocalDbQueryError>> + 'static>>;

#[derive(Clone)]
pub(crate) struct LocalDb {
    execute_batch_fn: Rc<ExecuteBatchFn>,
    query_text_fn: Rc<QueryTextFn>,
    query_json_fn: Rc<QueryJsonFn>,
    wipe_and_recreate_fn: Rc<WipeAndRecreateFn>,
}

impl LocalDb {
    pub(crate) fn new<E>(executor: E) -> Self
    where
        E: LocalDbQueryExecutor + Sync + 'static,
    {
        let exec = Rc::new(executor);

        let execute_batch_fn: Rc<ExecuteBatchFn> = {
            let exec = Rc::clone(&exec);
            Rc::new(move |batch: &SqlStatementBatch| {
                let exec = Rc::clone(&exec);
                let batch = batch.clone();
                Box::pin(async move { exec.execute_batch(&batch).await })
            })
        };

        let query_text_fn: Rc<QueryTextFn> = {
            let exec = Rc::clone(&exec);
            Rc::new(move |stmt: &SqlStatement| {
                let exec = Rc::clone(&exec);
                let stmt = stmt.clone();
                Box::pin(async move { exec.query_text(&stmt).await })
            })
        };

        let query_json_fn: Rc<QueryJsonFn> = {
            let exec = Rc::clone(&exec);
            Rc::new(move |stmt: &SqlStatement| {
                let exec = Rc::clone(&exec);
                let stmt = stmt.clone();
                Box::pin(async move { exec.query_json::<Value>(&stmt).await })
            })
        };

        let wipe_and_recreate_fn: Rc<WipeAndRecreateFn> = {
            let exec = Rc::clone(&exec);
            Rc::new(move || {
                let exec = Rc::clone(&exec);
                Box::pin(async move { exec.wipe_and_recreate().await })
            })
        };

        Self {
            execute_batch_fn,
            query_text_fn,
            query_json_fn,
            wipe_and_recreate_fn,
        }
    }

    pub(crate) fn from_js_callback(
        query_callback: js_sys::Function,
        wipe_callback: Option<js_sys::Function>,
    ) -> Self {
        Self::new(JsCallbackExecutor::new(query_callback, wipe_callback))
    }
}

impl fmt::Debug for LocalDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalDb").finish()
    }
}

#[async_trait::async_trait(?Send)]
impl LocalDbQueryExecutor for LocalDb {
    async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
        (self.execute_batch_fn)(batch).await
    }

    async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson,
    {
        let value = (self.query_json_fn)(stmt).await?;
        serde_json::from_value(value)
            .map_err(|err| LocalDbQueryError::deserialization(err.to_string()))
    }

    async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
        (self.query_text_fn)(stmt).await
    }

    async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
        (self.wipe_and_recreate_fn)().await
    }
}

impl From<LocalDbError> for WasmEncodedError {
    fn from(value: LocalDbError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "lowercase")]
pub enum LocalDbStatus {
    Active,
    Syncing,
    Failure,
}
impl_wasm_traits!(LocalDbStatus);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum SchedulerState {
    Leader,
    NotLeader,
}
impl_wasm_traits!(SchedulerState);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookSyncStatus {
    pub ob_id: OrderbookIdentifier,
    pub status: LocalDbStatus,
    pub scheduler_state: SchedulerState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl_wasm_traits!(OrderbookSyncStatus);

impl OrderbookSyncStatus {
    pub fn new(
        ob_id: OrderbookIdentifier,
        status: LocalDbStatus,
        scheduler_state: SchedulerState,
        phase_message: Option<String>,
        error: Option<String>,
    ) -> Self {
        Self {
            ob_id,
            status,
            scheduler_state,
            phase_message,
            error,
        }
    }

    pub fn syncing(ob_id: OrderbookIdentifier, phase: SyncPhase) -> Self {
        Self::new(
            ob_id,
            LocalDbStatus::Syncing,
            SchedulerState::Leader,
            Some(phase.to_message().to_string()),
            None,
        )
    }

    pub fn active(ob_id: OrderbookIdentifier, scheduler_state: SchedulerState) -> Self {
        Self::new(ob_id, LocalDbStatus::Active, scheduler_state, None, None)
    }

    pub fn failure(ob_id: OrderbookIdentifier, error: String) -> Self {
        Self::new(
            ob_id,
            LocalDbStatus::Failure,
            SchedulerState::Leader,
            None,
            Some(error),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSyncStatus {
    pub chain_id: u32,
    pub status: LocalDbStatus,
    pub scheduler_state: SchedulerState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl_wasm_traits!(NetworkSyncStatus);

impl NetworkSyncStatus {
    pub fn new(
        chain_id: u32,
        status: LocalDbStatus,
        scheduler_state: SchedulerState,
        error: Option<String>,
    ) -> Self {
        Self {
            chain_id,
            status,
            scheduler_state,
            error,
        }
    }

    pub fn active(chain_id: u32, scheduler_state: SchedulerState) -> Self {
        Self::new(chain_id, LocalDbStatus::Active, scheduler_state, None)
    }

    pub fn syncing(chain_id: u32) -> Self {
        Self::new(
            chain_id,
            LocalDbStatus::Syncing,
            SchedulerState::Leader,
            None,
        )
    }

    pub fn failure(chain_id: u32, error: String) -> Self {
        Self::new(
            chain_id,
            LocalDbStatus::Failure,
            SchedulerState::Leader,
            Some(error),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbStatusSnapshot {
    pub status: LocalDbStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl_wasm_traits!(LocalDbStatusSnapshot);

impl LocalDbStatusSnapshot {
    pub fn new(status: LocalDbStatus, error: Option<String>) -> Self {
        Self { status, error }
    }

    pub fn active() -> Self {
        Self::new(LocalDbStatus::Active, None)
    }

    pub fn syncing() -> Self {
        Self::new(LocalDbStatus::Syncing, None)
    }

    pub fn failure(error: String) -> Self {
        Self::new(LocalDbStatus::Failure, Some(error))
    }
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(js_name = "startLocalDbScheduler", unchecked_return_type = "void")]
    pub async fn start_local_db_scheduler(
        &self,
        #[wasm_export(
            js_name = "settingsYaml",
            param_description = "Full settings YAML string used by the client runner"
        )]
        settings_yaml: String,
        #[wasm_export(
            js_name = "statusCallback",
            param_description = "Optional callback invoked with the current local DB status"
        )]
        status_callback: Option<js_sys::Function>,
    ) -> Result<(), RaindexError> {
        let local_db = {
            let slot = self.local_db.borrow();
            slot.clone()
                .ok_or_else(|| RaindexError::JsError("Local DB not set".to_string()))?
        };

        let scheduler_cell = Rc::clone(&self.local_db_scheduler);
        let existing = {
            let mut slot = scheduler_cell.borrow_mut();
            slot.take()
        };

        if let Some(handle) = existing {
            handle.stop();
        }

        let handle = scheduler::start(settings_yaml, local_db, status_callback)?;
        {
            let mut slot = scheduler_cell.borrow_mut();
            *slot = Some(handle);
        }
        Ok(())
    }

    #[wasm_export(js_name = "stopLocalDbScheduler", unchecked_return_type = "void")]
    pub fn stop_local_db_scheduler(&self) -> Result<(), RaindexError> {
        let scheduler_cell = Rc::clone(&self.local_db_scheduler);
        let handle = {
            let mut slot = scheduler_cell.borrow_mut();
            slot.take()
        };

        if let Some(handle) = handle {
            handle.stop();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::{
        FromDbJson, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    };
    use serde::Deserialize;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
    struct TestRow {
        id: u32,
    }

    #[derive(Clone)]
    struct RecordingExec {
        calls: Arc<Mutex<Vec<&'static str>>>,
        json: String,
        text: String,
    }

    impl RecordingExec {
        fn new(json: impl Into<String>, text: impl Into<String>) -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                json: json.into(),
                text: text.into(),
            }
        }
    }

    #[async_trait::async_trait(?Send)]
    impl LocalDbQueryExecutor for RecordingExec {
        async fn execute_batch(&self, _batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            self.calls.lock().unwrap().push("batch");
            Ok(())
        }

        async fn query_json<T>(&self, _stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: FromDbJson,
        {
            self.calls.lock().unwrap().push("json");
            serde_json::from_str(&self.json)
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
        }

        async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            self.calls.lock().unwrap().push("text");
            Ok(self.text.clone())
        }

        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            self.calls.lock().unwrap().push("wipe");
            Ok(())
        }
    }

    #[tokio::test]
    async fn local_db_delegates_to_executor() {
        let exec = RecordingExec::new(r#"[{"id":1}]"#, "ok");
        let db = LocalDb::new(exec.clone());

        db.execute_batch(&SqlStatementBatch::new()).await.unwrap();
        let rows: Vec<TestRow> = db.query_json(&SqlStatement::new("SELECT 1")).await.unwrap();
        let text = db.query_text(&SqlStatement::new("SELECT 2")).await.unwrap();

        assert_eq!(rows, vec![TestRow { id: 1 }]);
        assert_eq!(text, "ok");

        let calls = exec.calls.lock().unwrap().clone();
        assert_eq!(calls, vec!["batch", "json", "text"]);
    }

    #[tokio::test]
    async fn local_db_delegates_wipe_and_recreate_to_executor() {
        let exec = RecordingExec::new("[]", "ok");
        let db = LocalDb::new(exec.clone());

        db.wipe_and_recreate().await.unwrap();

        let calls = exec.calls.lock().unwrap().clone();
        assert_eq!(calls, vec!["wipe"]);
    }

    #[test]
    fn debug_impl_is_stable() {
        let exec = RecordingExec::new("[]", "ok");
        let db = LocalDb::new(exec);
        assert!(format!("{:?}", db).contains("LocalDb"));
    }

    #[test]
    fn orderbook_sync_status_serializes_with_camel_case() {
        use crate::local_db::pipeline::SyncPhase;
        use alloy::primitives::address;

        let ob_id = crate::local_db::OrderbookIdentifier::new(
            42161,
            address!("0000000000000000000000000000000000001234"),
        );
        let status = OrderbookSyncStatus::syncing(ob_id, SyncPhase::FetchingLatestBlock);
        let json = serde_json::to_string(&status).unwrap();

        assert!(
            json.contains("\"obId\":{"),
            "expected obId as nested object in JSON: {}",
            json
        );
        assert!(
            json.contains("\"chainId\":42161"),
            "expected chainId in obId in JSON: {}",
            json
        );
        assert!(
            json.contains("\"orderbookAddress\":"),
            "expected orderbookAddress in obId in JSON: {}",
            json
        );
        assert!(
            json.contains("\"schedulerState\":\"leader\""),
            "expected schedulerState in JSON: {}",
            json
        );
        assert!(
            json.contains("\"phaseMessage\":\"Fetching latest block\""),
            "expected phaseMessage in JSON: {}",
            json
        );
        assert!(
            !json.contains("chain_id"),
            "should not have snake_case chain_id: {}",
            json
        );
        assert!(
            !json.contains("orderbook_address"),
            "should not have snake_case orderbook_address: {}",
            json
        );
    }

    #[test]
    fn network_sync_status_serializes_with_camel_case() {
        let status = NetworkSyncStatus::syncing(42161);
        let json = serde_json::to_string(&status).unwrap();

        assert!(
            json.contains("\"chainId\":42161"),
            "expected chainId in JSON: {}",
            json
        );
        assert!(
            json.contains("\"schedulerState\":\"leader\""),
            "expected schedulerState in JSON: {}",
            json
        );
        assert!(
            !json.contains("chain_id"),
            "should not have snake_case chain_id: {}",
            json
        );
    }

    #[test]
    fn network_sync_status_active_with_leader_sets_correct_fields() {
        let status = NetworkSyncStatus::active(137, SchedulerState::Leader);

        assert_eq!(status.chain_id, 137);
        assert_eq!(status.status, LocalDbStatus::Active);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert!(status.error.is_none());
    }

    #[test]
    fn network_sync_status_active_with_not_leader_sets_correct_fields() {
        let status = NetworkSyncStatus::active(137, SchedulerState::NotLeader);

        assert_eq!(status.chain_id, 137);
        assert_eq!(status.status, LocalDbStatus::Active);
        assert_eq!(status.scheduler_state, SchedulerState::NotLeader);
        assert!(status.error.is_none());
    }

    #[test]
    fn network_sync_status_syncing_sets_correct_fields() {
        let status = NetworkSyncStatus::syncing(42161);

        assert_eq!(status.chain_id, 42161);
        assert_eq!(status.status, LocalDbStatus::Syncing);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert!(status.error.is_none());
    }

    #[test]
    fn network_sync_status_failure_sets_correct_fields() {
        let error_msg = "RPC timeout".to_string();
        let status = NetworkSyncStatus::failure(8453, error_msg.clone());

        assert_eq!(status.chain_id, 8453);
        assert_eq!(status.status, LocalDbStatus::Failure);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(status.error, Some(error_msg));
    }

    #[test]
    fn network_sync_status_new_with_all_fields() {
        let status = NetworkSyncStatus::new(
            137,
            LocalDbStatus::Failure,
            SchedulerState::Leader,
            Some("custom error".to_string()),
        );

        assert_eq!(status.chain_id, 137);
        assert_eq!(status.status, LocalDbStatus::Failure);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(status.error, Some("custom error".to_string()));
    }

    #[test]
    fn network_sync_status_does_not_have_network_key_field() {
        let status = NetworkSyncStatus::syncing(42161);
        let json = serde_json::to_string(&status).unwrap();

        assert!(
            !json.contains("networkKey"),
            "should not have networkKey field: {}",
            json
        );
        assert!(
            !json.contains("network_key"),
            "should not have network_key field: {}",
            json
        );
    }

    #[test]
    fn orderbook_sync_status_deserializes_from_json() {
        let json = r#"{
            "obId": {"chainId": 137, "orderbookAddress": "0x0000000000000000000000000000000000001234"},
            "status": "syncing",
            "schedulerState": "leader",
            "phaseMessage": "Fetching latest block"
        }"#;

        let status: OrderbookSyncStatus = serde_json::from_str(json).unwrap();

        assert_eq!(status.ob_id.chain_id, 137);
        assert_eq!(status.status, LocalDbStatus::Syncing);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(
            status.phase_message,
            Some("Fetching latest block".to_string())
        );
        assert!(status.error.is_none());
    }

    #[test]
    fn network_sync_status_deserializes_from_json() {
        let json = r#"{
            "chainId": 42161,
            "status": "failure",
            "schedulerState": "leader",
            "error": "Connection refused"
        }"#;

        let status: NetworkSyncStatus = serde_json::from_str(json).unwrap();

        assert_eq!(status.chain_id, 42161);
        assert_eq!(status.status, LocalDbStatus::Failure);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(status.error, Some("Connection refused".to_string()));
    }

    #[test]
    fn local_db_status_snapshot_factory_methods() {
        let active = LocalDbStatusSnapshot::active();
        assert_eq!(active.status, LocalDbStatus::Active);
        assert!(active.error.is_none());

        let syncing = LocalDbStatusSnapshot::syncing();
        assert_eq!(syncing.status, LocalDbStatus::Syncing);
        assert!(syncing.error.is_none());

        let failure = LocalDbStatusSnapshot::failure("test error".to_string());
        assert_eq!(failure.status, LocalDbStatus::Failure);
        assert_eq!(failure.error, Some("test error".to_string()));
    }
}

#[cfg(all(test, target_family = "wasm", feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use gloo_timers::future::TimeoutFuture;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_app_settings::yaml::{
        orderbook::{OrderbookYaml, OrderbookYamlValidation},
        YamlParsable,
    };
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::{
        prelude::{serde_wasm_bindgen, JsValue},
        result::{WasmEncodedError, WasmEncodedResult},
    };

    wasm_bindgen_test_configure!(run_in_browser);

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
    bootstrap-block-threshold: 1000
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

    fn build_client() -> RaindexClient {
        let orderbook_yaml = OrderbookYaml::new(
            vec![single_orderbook_settings_yaml()],
            OrderbookYamlValidation::default(),
        )
        .expect("valid orderbook yaml");

        RaindexClient {
            orderbook_yaml,
            local_db: Rc::new(RefCell::new(None)),
            local_db_scheduler: Rc::new(RefCell::new(None)),
        }
    }

    fn success_callback() -> js_sys::Function {
        let result = WasmEncodedResult::Success::<String> {
            value: "[]".to_string(),
            error: None,
        };
        let js_value = serde_wasm_bindgen::to_value(&result).unwrap();
        js_sys::Function::new_no_args(&format!(
            "return Promise.resolve({})",
            js_sys::JSON::stringify(&js_value)
                .unwrap()
                .as_string()
                .unwrap()
        ))
    }

    fn healthy_db_callback() -> js_sys::Function {
        js_sys::Function::new_with_args(
            "sql",
            r#"
            var value = '[]';
            if (sql && sql.toLowerCase().includes('quick_check')) {
                value = '[{"quick_check":"ok"}]';
            }
            return Promise.resolve({ value: value, error: null });
            "#,
        )
    }

    fn recording_status_callback(
        store: Rc<RefCell<Vec<LocalDbStatusSnapshot>>>,
    ) -> js_sys::Function {
        let closure = Closure::wrap(Box::new(move |value: JsValue| {
            if let Ok(snapshot) = serde_wasm_bindgen::from_value::<LocalDbStatusSnapshot>(value) {
                store.borrow_mut().push(snapshot);
            }
        }) as Box<dyn FnMut(JsValue)>);

        let function: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();
        function
    }

    #[wasm_bindgen_test]
    async fn local_db_from_js_callback_executes_queries() {
        let client = build_client();
        client
            .set_local_db_callback(success_callback(), None)
            .expect("callback set");
        let db = client.local_db().expect("local db set");

        let stmt = SqlStatement::new("SELECT 1");
        let rows: Vec<String> = db.query_json(&stmt).await.unwrap();
        assert!(rows.is_empty());

        let text = db.query_text(&stmt).await.unwrap();
        assert_eq!(text, "[]");
    }

    #[wasm_bindgen_test]
    async fn local_db_from_js_callback_surfaces_errors() {
        let error = WasmEncodedResult::Err::<String> {
            value: None,
            error: WasmEncodedError {
                msg: "boom".to_string(),
                readable_msg: "boom readable".to_string(),
            },
        };
        let js_value = serde_wasm_bindgen::to_value(&error).unwrap();
        let callback = js_sys::Function::new_no_args(&format!(
            "return Promise.resolve({})",
            js_sys::JSON::stringify(&js_value)
                .unwrap()
                .as_string()
                .unwrap()
        ));

        let db = LocalDb::from_js_callback(callback, None);
        let stmt = SqlStatement::new("SELECT 1");
        let err = db.query_text(&stmt).await.unwrap_err();
        assert!(matches!(err, LocalDbQueryError::Database { .. }));
    }

    #[wasm_bindgen_test]
    async fn start_scheduler_without_callback_returns_error() {
        let client = build_client();
        let result = client
            .start_local_db_scheduler(single_orderbook_settings_yaml(), None)
            .await;
        assert!(matches!(result, Err(RaindexError::JsError(_))));
        assert!(client.local_db_scheduler.borrow().is_none());
    }

    #[wasm_bindgen_test]
    async fn start_and_stop_scheduler_updates_handle_state() {
        let client = build_client();
        client
            .set_local_db_callback(healthy_db_callback(), None)
            .expect("callback set");

        client
            .start_local_db_scheduler(single_orderbook_settings_yaml(), None)
            .await
            .expect("scheduler starts");
        assert!(client.local_db_scheduler.borrow().is_some());

        TimeoutFuture::new(0).await;

        client.stop_local_db_scheduler().expect("scheduler stops");
        assert!(client.local_db_scheduler.borrow().is_none());
    }

    #[wasm_bindgen_test]
    async fn restarting_scheduler_replaces_handle() {
        let client = build_client();
        client
            .set_local_db_callback(healthy_db_callback(), None)
            .expect("callback set");

        client
            .start_local_db_scheduler(single_orderbook_settings_yaml(), None)
            .await
            .expect("first scheduler starts");

        TimeoutFuture::new(0).await;

        let first_handle_ptr = client
            .local_db_scheduler
            .borrow()
            .as_ref()
            .map(|handle| handle.stop_flag_ptr())
            .expect("first handle exists");

        let statuses = Rc::new(RefCell::new(Vec::new()));
        let status_callback = recording_status_callback(Rc::clone(&statuses));

        client
            .start_local_db_scheduler(single_orderbook_settings_yaml(), Some(status_callback))
            .await
            .expect("second scheduler starts");

        TimeoutFuture::new(0).await;
        TimeoutFuture::new(1000).await;

        let second_handle_ptr = client
            .local_db_scheduler
            .borrow()
            .as_ref()
            .map(|handle| handle.stop_flag_ptr())
            .expect("second handle exists");

        assert_ne!(first_handle_ptr, second_handle_ptr);
        let recorded = statuses.borrow();
        assert!(recorded
            .iter()
            .any(|snapshot| snapshot.status == LocalDbStatus::Syncing));

        client.stop_local_db_scheduler().expect("scheduler stops");
    }

    #[wasm_bindgen_test]
    async fn start_scheduler_propagates_errors_and_leaves_handle_empty() {
        let client = build_client();
        client
            .set_local_db_callback(success_callback(), None)
            .expect("callback set");

        let result = client
            .start_local_db_scheduler("not yaml".to_owned(), None)
            .await;
        assert!(matches!(result, Err(RaindexError::LocalDbError(_))));
        assert!(client.local_db_scheduler.borrow().is_none());
    }

    #[wasm_bindgen_test]
    async fn stop_scheduler_is_idempotent() {
        let client = build_client();
        client
            .set_local_db_callback(healthy_db_callback(), None)
            .expect("callback set");

        client
            .start_local_db_scheduler(single_orderbook_settings_yaml(), None)
            .await
            .expect("scheduler starts");

        TimeoutFuture::new(0).await;

        client
            .stop_local_db_scheduler()
            .expect("first stop succeeds");
        client
            .stop_local_db_scheduler()
            .expect("second stop succeeds");
        assert!(client.local_db_scheduler.borrow().is_none());
    }
}
