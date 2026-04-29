use crate::local_db::pipeline::SyncPhase;
use crate::local_db::OrderbookIdentifier;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

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

#[cfg(test)]
mod tests {
    use super::*;

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
