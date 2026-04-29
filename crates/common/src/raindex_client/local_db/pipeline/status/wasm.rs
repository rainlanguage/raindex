use crate::local_db::pipeline::{StatusBus, SyncPhase};
use crate::local_db::{LocalDbError, RaindexIdentifier};
use crate::raindex_client::local_db::{RaindexSyncStatus, SchedulerState};
use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_utils::prelude::*;

thread_local! {
    static STATUS_CALLBACK: RefCell<Option<Rc<Function>>> = const { RefCell::new(None) };
    static SCHEDULER_STATE: RefCell<SchedulerState> = const { RefCell::new(SchedulerState::Leader) };
}

pub fn set_status_callback(callback: Option<Rc<Function>>) {
    STATUS_CALLBACK.with(|c| {
        *c.borrow_mut() = callback;
    });
}

pub fn set_scheduler_state(state: SchedulerState) {
    SCHEDULER_STATE.with(|s| {
        *s.borrow_mut() = state;
    });
}

pub fn get_scheduler_state() -> SchedulerState {
    SCHEDULER_STATE.with(|s| *s.borrow())
}

fn emit_to_callback(status: RaindexSyncStatus) {
    STATUS_CALLBACK.with(|c| {
        if let Some(callback) = c.borrow().as_ref() {
            if let Ok(value) = serde_wasm_bindgen::to_value(&status) {
                let _ = callback.call1(&JsValue::NULL, &value);
            }
        }
    });
}

#[derive(Debug, Clone, Default)]
pub struct ClientStatusBus {
    raindex_id: Option<RaindexIdentifier>,
}

impl ClientStatusBus {
    pub fn new() -> Self {
        Self { raindex_id: None }
    }

    pub fn with_ob_id(raindex_id: RaindexIdentifier) -> Self {
        Self {
            raindex_id: Some(raindex_id),
        }
    }

    fn emit(&self, status: RaindexSyncStatus) {
        emit_to_callback(status);
    }

    pub fn emit_active(&self) {
        let Some(raindex_id) = &self.raindex_id else {
            return;
        };

        let scheduler_state = get_scheduler_state();
        self.emit(RaindexSyncStatus::active(
            raindex_id.clone(),
            scheduler_state,
        ));
    }

    pub fn emit_failure(&self, error: String) {
        let Some(raindex_id) = &self.raindex_id else {
            return;
        };

        self.emit(RaindexSyncStatus::failure(raindex_id.clone(), error));
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for ClientStatusBus {
    async fn send(&self, phase: SyncPhase) -> Result<(), LocalDbError> {
        let scheduler_state = get_scheduler_state();
        if scheduler_state == SchedulerState::NotLeader {
            return Ok(());
        }

        let Some(raindex_id) = &self.raindex_id else {
            return Ok(());
        };

        let status = RaindexSyncStatus::syncing(raindex_id.clone(), phase);
        self.emit(status);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::pipeline::SyncPhase;
    use crate::local_db::RaindexIdentifier;
    use crate::raindex_client::local_db::SchedulerState;
    use alloy::primitives::address;

    fn test_ob_id() -> RaindexIdentifier {
        RaindexIdentifier::new(1, address!("0000000000000000000000000000000000001234"))
    }

    #[test]
    fn client_status_bus_default_has_no_ob_id() {
        let bus = ClientStatusBus::new();
        assert!(bus.raindex_id.is_none());
    }

    #[test]
    fn client_status_bus_with_ob_id_stores_identifier() {
        let raindex_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(raindex_id.clone());
        assert_eq!(bus.raindex_id, Some(raindex_id));
    }

    #[tokio::test]
    async fn send_does_not_panic_without_ob_id() {
        let bus = ClientStatusBus::new();
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_skips_when_not_leader() {
        let raindex_id = test_ob_id();
        set_scheduler_state(SchedulerState::NotLeader);
        let bus = ClientStatusBus::with_ob_id(raindex_id);
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
        set_scheduler_state(SchedulerState::Leader);
    }

    #[tokio::test]
    async fn send_returns_ok_when_leader_with_ob_id() {
        set_scheduler_state(SchedulerState::Leader);
        let raindex_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(raindex_id);
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }

    #[test]
    fn emit_active_does_not_panic_without_ob_id() {
        let bus = ClientStatusBus::new();
        bus.emit_active();
    }

    #[test]
    fn emit_failure_does_not_panic_without_ob_id() {
        let bus = ClientStatusBus::new();
        bus.emit_failure("test error".to_string());
    }

    #[test]
    fn emit_active_with_ob_id_does_not_panic() {
        set_scheduler_state(SchedulerState::Leader);
        let raindex_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(raindex_id);
        bus.emit_active();
    }

    #[test]
    fn emit_failure_with_ob_id_does_not_panic() {
        let raindex_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(raindex_id);
        bus.emit_failure("test error".to_string());
    }

    #[test]
    fn set_and_get_scheduler_state_roundtrips() {
        set_scheduler_state(SchedulerState::Leader);
        assert_eq!(get_scheduler_state(), SchedulerState::Leader);

        set_scheduler_state(SchedulerState::NotLeader);
        assert_eq!(get_scheduler_state(), SchedulerState::NotLeader);

        set_scheduler_state(SchedulerState::Leader);
        assert_eq!(get_scheduler_state(), SchedulerState::Leader);
    }
}

#[cfg(all(test, feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use crate::local_db::pipeline::{StatusBus, SyncPhase};
    use crate::local_db::RaindexIdentifier;
    use crate::raindex_client::local_db::LocalDbStatus;
    use crate::raindex_client::local_db::{RaindexSyncStatus, SchedulerState};
    use alloy::primitives::address;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn test_ob_id() -> RaindexIdentifier {
        RaindexIdentifier::new(1, address!("0000000000000000000000000000000000001234"))
    }

    fn create_recording_callback(
        recorded: Rc<RefCell<Vec<RaindexSyncStatus>>>,
    ) -> Rc<js_sys::Function> {
        let closure = Closure::wrap(Box::new(move |value: JsValue| {
            if let Ok(status) = serde_wasm_bindgen::from_value::<RaindexSyncStatus>(value) {
                recorded.borrow_mut().push(status);
            }
        }) as Box<dyn FnMut(JsValue)>);
        let function: js_sys::Function = closure.as_ref().clone().unchecked_into();
        closure.forget();
        Rc::new(function)
    }

    #[wasm_bindgen_test]
    async fn emit_to_callback_invokes_js_function_with_correct_payload() {
        let recorded = Rc::new(RefCell::new(Vec::new()));
        let callback = create_recording_callback(Rc::clone(&recorded));

        set_status_callback(Some(callback));
        set_scheduler_state(SchedulerState::Leader);

        let raindex_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(raindex_id.clone());
        bus.send(SyncPhase::FetchingLatestBlock).await.unwrap();

        set_status_callback(None);

        let emissions = recorded.borrow();
        assert_eq!(emissions.len(), 1, "expected exactly one emission");

        let emitted = &emissions[0];
        assert_eq!(emitted.raindex_id, raindex_id);
        assert_eq!(emitted.status, LocalDbStatus::Syncing);
        assert_eq!(emitted.scheduler_state, SchedulerState::Leader);
        assert_eq!(
            emitted.phase_message,
            Some("Fetching latest block".to_string())
        );
    }

    #[wasm_bindgen_test]
    async fn send_does_not_emit_when_not_leader() {
        let recorded = Rc::new(RefCell::new(Vec::new()));
        let callback = create_recording_callback(Rc::clone(&recorded));

        set_status_callback(Some(callback));
        set_scheduler_state(SchedulerState::NotLeader);

        let raindex_id = test_ob_id();
        let bus = ClientStatusBus::with_ob_id(raindex_id);
        bus.send(SyncPhase::FetchingLatestBlock).await.unwrap();

        set_status_callback(None);
        set_scheduler_state(SchedulerState::Leader);

        let emissions = recorded.borrow();
        assert_eq!(
            emissions.len(),
            0,
            "expected no emissions when NotLeader, got {}",
            emissions.len()
        );
    }
}
