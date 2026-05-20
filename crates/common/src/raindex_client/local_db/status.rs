use crate::local_db::pipeline::runner::utils::ParsedRunnerSettings;
use crate::local_db::pipeline::SyncPhase;
use crate::local_db::RaindexIdentifier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
pub struct RaindexSyncStatus {
    pub raindex_id: RaindexIdentifier,
    pub status: LocalDbStatus,
    pub scheduler_state: SchedulerState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl_wasm_traits!(RaindexSyncStatus);

impl RaindexSyncStatus {
    pub fn new(
        raindex_id: RaindexIdentifier,
        status: LocalDbStatus,
        scheduler_state: SchedulerState,
        phase_message: Option<String>,
        error: Option<String>,
    ) -> Self {
        Self {
            raindex_id,
            status,
            scheduler_state,
            phase_message,
            error,
        }
    }

    pub fn syncing(raindex_id: RaindexIdentifier, phase: SyncPhase) -> Self {
        Self::new(
            raindex_id,
            LocalDbStatus::Syncing,
            SchedulerState::Leader,
            Some(phase.to_message().to_string()),
            None,
        )
    }

    pub fn active(raindex_id: RaindexIdentifier, scheduler_state: SchedulerState) -> Self {
        Self::new(
            raindex_id,
            LocalDbStatus::Active,
            scheduler_state,
            None,
            None,
        )
    }

    pub fn failure(raindex_id: RaindexIdentifier, error: String) -> Self {
        Self::new(
            raindex_id,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSyncStatusSnapshot {
    pub chain_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_key: Option<String>,
    pub status: LocalDbStatus,
    pub scheduler_state: SchedulerState,
    pub raindex_count: usize,
    pub ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl_wasm_traits!(NetworkSyncStatusSnapshot);

impl NetworkSyncStatusSnapshot {
    pub fn new(chain_id: u32, network_key: Option<String>) -> Self {
        Self {
            chain_id,
            network_key,
            status: LocalDbStatus::Syncing,
            scheduler_state: SchedulerState::Leader,
            raindex_count: 0,
            ready: false,
            error: None,
        }
    }

    pub fn apply_status(&mut self, status: NetworkSyncStatus) {
        self.status = status.status;
        self.scheduler_state = status.scheduler_state;
        self.error = status.error;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct RaindexSyncStatusSnapshot {
    pub raindex_id: RaindexIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raindex_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_key: Option<String>,
    pub status: LocalDbStatus,
    pub scheduler_state: SchedulerState,
    pub ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_block: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
impl_wasm_traits!(RaindexSyncStatusSnapshot);

impl RaindexSyncStatusSnapshot {
    pub fn new(
        raindex_id: RaindexIdentifier,
        raindex_key: Option<String>,
        network_key: Option<String>,
    ) -> Self {
        Self {
            raindex_id,
            raindex_key,
            network_key,
            status: LocalDbStatus::Syncing,
            scheduler_state: SchedulerState::Leader,
            ready: false,
            phase_message: None,
            last_synced_block: None,
            updated_at: None,
            error: None,
        }
    }

    pub fn apply_status(&mut self, status: RaindexSyncStatus) {
        self.status = status.status;
        self.scheduler_state = status.scheduler_state;
        self.phase_message = status.phase_message;
        self.error = status.error;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbSyncSnapshot {
    pub configured: bool,
    pub healthy: bool,
    pub status: LocalDbStatus,
    pub scheduler_state: SchedulerState,
    pub networks: Vec<NetworkSyncStatusSnapshot>,
    pub raindexes: Vec<RaindexSyncStatusSnapshot>,
}
impl_wasm_traits!(LocalDbSyncSnapshot);

impl LocalDbSyncSnapshot {
    pub fn not_configured() -> Self {
        Self {
            configured: false,
            healthy: true,
            status: LocalDbStatus::Active,
            scheduler_state: SchedulerState::Leader,
            networks: Vec::new(),
            raindexes: Vec::new(),
        }
    }

    pub fn from_parts(
        mut networks: Vec<NetworkSyncStatusSnapshot>,
        mut raindexes: Vec<RaindexSyncStatusSnapshot>,
    ) -> Self {
        networks.sort_by_key(|network| network.chain_id);
        raindexes.sort_by(|a, b| {
            (a.raindex_id.chain_id, a.raindex_id.raindex_address)
                .cmp(&(b.raindex_id.chain_id, b.raindex_id.raindex_address))
        });

        let configured = !networks.is_empty() || !raindexes.is_empty();
        let all_statuses = networks
            .iter()
            .map(|network| network.status)
            .chain(raindexes.iter().map(|raindex| raindex.status))
            .collect::<Vec<_>>();

        let has_failure = all_statuses.contains(&LocalDbStatus::Failure);
        let has_syncing = all_statuses.contains(&LocalDbStatus::Syncing);
        let scheduler_state = networks
            .iter()
            .find_map(|network| {
                (network.scheduler_state == SchedulerState::NotLeader)
                    .then_some(SchedulerState::NotLeader)
            })
            .unwrap_or(SchedulerState::Leader);

        let status = if !configured {
            LocalDbStatus::Active
        } else if has_failure {
            LocalDbStatus::Failure
        } else if has_syncing {
            LocalDbStatus::Syncing
        } else {
            LocalDbStatus::Active
        };

        Self {
            configured,
            healthy: !has_failure,
            status,
            scheduler_state,
            networks,
            raindexes,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalDbSyncSnapshotStore {
    networks: HashMap<u32, NetworkSyncStatusSnapshot>,
    raindexes: HashMap<RaindexIdentifier, RaindexSyncStatusSnapshot>,
}

impl LocalDbSyncSnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_network(&mut self, chain_id: u32, network_key: Option<String>) {
        self.networks
            .entry(chain_id)
            .or_insert_with(|| NetworkSyncStatusSnapshot::new(chain_id, network_key));
    }

    pub fn seed_raindex(
        &mut self,
        raindex_id: RaindexIdentifier,
        raindex_key: Option<String>,
        network_key: Option<String>,
    ) {
        let chain_id = raindex_id.chain_id;
        self.raindexes.entry(raindex_id.clone()).or_insert_with(|| {
            RaindexSyncStatusSnapshot::new(raindex_id, raindex_key, network_key)
        });
        if let Some(network) = self.networks.get_mut(&chain_id) {
            network.raindex_count = self
                .raindexes
                .values()
                .filter(|raindex| raindex.raindex_id.chain_id == chain_id)
                .count();
        }
    }

    pub fn update_network(&mut self, status: NetworkSyncStatus) {
        self.networks
            .entry(status.chain_id)
            .or_insert_with(|| NetworkSyncStatusSnapshot::new(status.chain_id, None))
            .apply_status(status);
    }

    pub fn update_raindex(&mut self, status: RaindexSyncStatus) {
        self.raindexes
            .entry(status.raindex_id.clone())
            .or_insert_with(|| {
                RaindexSyncStatusSnapshot::new(status.raindex_id.clone(), None, None)
            })
            .apply_status(status);
    }

    pub fn mark_ready(&mut self, chain_id: u32) {
        if let Some(network) = self.networks.get_mut(&chain_id) {
            network.ready = true;
            if network.status != LocalDbStatus::Failure {
                network.status = LocalDbStatus::Active;
            }
        }
        for raindex in self.raindexes.values_mut() {
            if raindex.raindex_id.chain_id == chain_id {
                raindex.ready = true;
                if raindex.status != LocalDbStatus::Failure {
                    raindex.status = LocalDbStatus::Active;
                    raindex.phase_message = None;
                }
            }
        }
    }

    pub fn snapshot(&self) -> LocalDbSyncSnapshot {
        LocalDbSyncSnapshot::from_parts(
            self.networks.values().cloned().collect(),
            self.raindexes.values().cloned().collect(),
        )
    }
}

type StoreRef = Arc<Mutex<LocalDbSyncSnapshotStore>>;

#[derive(Debug, Clone)]
pub struct LocalDbSyncStatusStore {
    inner: StoreRef,
}

impl Default for LocalDbSyncStatusStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalDbSyncStatusStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LocalDbSyncSnapshotStore::new())),
        }
    }

    fn with_store<T>(&self, f: impl FnOnce(&mut LocalDbSyncSnapshotStore) -> T) -> T {
        let mut store = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut store)
    }

    pub fn reset(&self) {
        self.with_store(|store| {
            *store = LocalDbSyncSnapshotStore::new();
        });
    }

    pub fn seed(&self, settings: &ParsedRunnerSettings) {
        self.with_store(|store| {
            *store = LocalDbSyncSnapshotStore::new();
            let mut raindexes = settings.raindexes.iter().collect::<Vec<_>>();
            raindexes.sort_by(|a, b| a.0.cmp(b.0));
            for (raindex_key, raindex) in raindexes {
                if !settings.syncs.contains_key(&raindex.network.key) {
                    continue;
                }
                let chain_id = raindex.network.chain_id;
                let network_key = raindex.network.key.clone();
                store.seed_network(chain_id, Some(network_key.clone()));
                store.seed_raindex(
                    RaindexIdentifier::new(chain_id, raindex.address),
                    Some(raindex_key.clone()),
                    Some(network_key),
                );
            }
        });
    }

    pub fn record_network_status(&self, status: NetworkSyncStatus) {
        self.with_store(|store| {
            store.update_network(status);
        });
    }

    pub fn record_raindex_status(&self, status: RaindexSyncStatus) {
        self.with_store(|store| {
            store.update_raindex(status);
        });
    }

    pub fn record_chain_ready(&self, chain_id: u32) {
        self.with_store(|store| {
            store.mark_ready(chain_id);
        });
    }

    pub fn snapshot(&self) -> LocalDbSyncSnapshot {
        self.with_store(|store| store.snapshot())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raindex_sync_status_serializes_with_camel_case() {
        use crate::local_db::pipeline::SyncPhase;
        use alloy::primitives::address;

        let raindex_id = crate::local_db::RaindexIdentifier::new(
            42161,
            address!("0000000000000000000000000000000000001234"),
        );
        let status = RaindexSyncStatus::syncing(raindex_id, SyncPhase::FetchingLatestBlock);
        let json = serde_json::to_string(&status).unwrap();

        assert!(
            json.contains("\"raindexId\":{"),
            "expected raindexId as nested object in JSON: {}",
            json
        );
        assert!(
            json.contains("\"chainId\":42161"),
            "expected chainId in raindexId in JSON: {}",
            json
        );
        assert!(
            json.contains("\"raindexAddress\":"),
            "expected raindexAddress in raindexId in JSON: {}",
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
            !json.contains("raindex_address"),
            "should not have snake_case raindex_address: {}",
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
    fn raindex_sync_status_deserializes_from_json() {
        let json = r#"{
            "raindexId": {"chainId": 137, "raindexAddress": "0x0000000000000000000000000000000000001234"},
            "status": "syncing",
            "schedulerState": "leader",
            "phaseMessage": "Fetching latest block"
        }"#;

        let status: RaindexSyncStatus = serde_json::from_str(json).unwrap();

        assert_eq!(status.raindex_id.chain_id, 137);
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

    #[test]
    fn sync_snapshot_reports_not_configured_when_empty() {
        let snapshot = LocalDbSyncSnapshot::from_parts(Vec::new(), Vec::new());

        assert!(!snapshot.configured);
        assert!(snapshot.healthy);
        assert_eq!(snapshot.status, LocalDbStatus::Active);
        assert!(snapshot.networks.is_empty());
        assert!(snapshot.raindexes.is_empty());
    }

    #[test]
    fn sync_snapshot_failure_dominates_over_syncing() {
        use alloy::primitives::address;

        let network = NetworkSyncStatusSnapshot {
            chain_id: 1,
            network_key: Some("mainnet".to_string()),
            status: LocalDbStatus::Syncing,
            scheduler_state: SchedulerState::Leader,
            raindex_count: 1,
            ready: false,
            error: None,
        };
        let raindex = RaindexSyncStatusSnapshot {
            raindex_id: crate::local_db::RaindexIdentifier::new(
                1,
                address!("0000000000000000000000000000000000001234"),
            ),
            raindex_key: Some("ob".to_string()),
            network_key: Some("mainnet".to_string()),
            status: LocalDbStatus::Failure,
            scheduler_state: SchedulerState::Leader,
            ready: false,
            phase_message: None,
            last_synced_block: Some(100),
            updated_at: Some("2026-05-01 10:00:00".to_string()),
            error: Some("boom".to_string()),
        };

        let snapshot = LocalDbSyncSnapshot::from_parts(vec![network], vec![raindex]);

        assert!(snapshot.configured);
        assert!(!snapshot.healthy);
        assert_eq!(snapshot.status, LocalDbStatus::Failure);
    }

    #[test]
    fn sync_snapshot_store_marks_chain_ready() {
        use alloy::primitives::address;

        let mut store = LocalDbSyncSnapshotStore::new();
        let raindex_id = crate::local_db::RaindexIdentifier::new(
            42161,
            address!("0000000000000000000000000000000000001234"),
        );
        store.seed_network(42161, Some("arbitrum".to_string()));
        store.seed_raindex(
            raindex_id.clone(),
            Some("main-ob".to_string()),
            Some("arbitrum".to_string()),
        );

        let syncing = store.snapshot();
        assert_eq!(syncing.status, LocalDbStatus::Syncing);
        assert!(!syncing.networks[0].ready);
        assert!(!syncing.raindexes[0].ready);
        assert_eq!(syncing.networks[0].raindex_count, 1);

        store.mark_ready(42161);
        let ready = store.snapshot();

        assert_eq!(ready.status, LocalDbStatus::Active);
        assert!(ready.networks[0].ready);
        assert!(ready.raindexes[0].ready);
        assert_eq!(ready.raindexes[0].raindex_id, raindex_id);
    }

    #[test]
    fn sync_snapshot_store_updates_live_network_and_raindex_statuses() {
        use crate::local_db::pipeline::SyncPhase;
        use alloy::primitives::address;

        let mut store = LocalDbSyncSnapshotStore::new();
        let raindex_id = crate::local_db::RaindexIdentifier::new(
            1,
            address!("0000000000000000000000000000000000000001"),
        );
        store.seed_network(1, Some("mainnet".to_string()));
        store.seed_raindex(
            raindex_id.clone(),
            Some("main-ob".to_string()),
            Some("mainnet".to_string()),
        );

        store.update_network(NetworkSyncStatus::active(1, SchedulerState::NotLeader));
        store.update_raindex(RaindexSyncStatus::syncing(
            raindex_id.clone(),
            SyncPhase::FetchingRaindexLogs,
        ));

        let snapshot = store.snapshot();
        assert_eq!(snapshot.status, LocalDbStatus::Syncing);
        assert_eq!(snapshot.scheduler_state, SchedulerState::NotLeader);
        assert_eq!(snapshot.networks[0].status, LocalDbStatus::Active);
        assert_eq!(
            snapshot.raindexes[0].phase_message,
            Some("Fetching raindex logs".to_string())
        );

        store.update_raindex(RaindexSyncStatus::failure(
            raindex_id,
            "rpc failed".to_string(),
        ));
        let failed = store.snapshot();
        assert_eq!(failed.status, LocalDbStatus::Failure);
        assert!(!failed.healthy);
        assert_eq!(failed.raindexes[0].error, Some("rpc failed".to_string()));
    }

    #[test]
    fn sync_status_store_clones_share_state_but_new_stores_are_isolated() {
        let first = LocalDbSyncStatusStore::new();
        let first_clone = first.clone();
        let second = LocalDbSyncStatusStore::new();

        first.record_network_status(NetworkSyncStatus::syncing(1));

        let cloned_snapshot = first_clone.snapshot();
        assert!(cloned_snapshot.configured);
        assert_eq!(cloned_snapshot.networks.len(), 1);
        assert_eq!(cloned_snapshot.networks[0].chain_id, 1);
        assert_eq!(cloned_snapshot.status, LocalDbStatus::Syncing);

        let second_snapshot = second.snapshot();
        assert!(!second_snapshot.configured);
        assert!(second_snapshot.networks.is_empty());
    }
}
