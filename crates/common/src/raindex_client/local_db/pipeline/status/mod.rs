#[cfg(not(target_family = "wasm"))]
pub mod native;
#[cfg(target_family = "wasm")]
pub mod wasm;

#[cfg(not(target_family = "wasm"))]
pub use native::*;
#[cfg(target_family = "wasm")]
pub use wasm::*;

#[cfg(test)]
mod tests {
    use crate::local_db::pipeline::SyncPhase;
    use crate::local_db::RaindexIdentifier;
    use crate::raindex_client::local_db::{LocalDbStatus, RaindexSyncStatus, SchedulerState};
    use alloy::primitives::address;

    fn test_raindex_id() -> RaindexIdentifier {
        RaindexIdentifier::new(1, address!("0000000000000000000000000000000000001234"))
    }

    #[test]
    fn sync_phase_to_message_returns_correct_strings() {
        assert_eq!(
            SyncPhase::FetchingLatestBlock.to_message(),
            "Fetching latest block"
        );
        assert_eq!(
            SyncPhase::RunningBootstrap.to_message(),
            "Running bootstrap"
        );
        assert_eq!(
            SyncPhase::ComputingSyncWindow.to_message(),
            "Computing sync window"
        );
        assert_eq!(
            SyncPhase::FetchingRaindexLogs.to_message(),
            "Fetching raindex logs"
        );
        assert_eq!(
            SyncPhase::DecodingRaindexLogs.to_message(),
            "Decoding raindex logs"
        );
        assert_eq!(
            SyncPhase::FetchingStoreLogs.to_message(),
            "Fetching interpreter store logs"
        );
        assert_eq!(
            SyncPhase::DecodingStoreLogs.to_message(),
            "Decoding interpreter store logs"
        );
        assert_eq!(
            SyncPhase::FetchingTokenMetadata.to_message(),
            "Fetching missing token metadata"
        );
        assert_eq!(
            SyncPhase::BuildingSqlBatch.to_message(),
            "Building SQL batch"
        );
        assert_eq!(
            SyncPhase::PersistingToDatabase.to_message(),
            "Persisting to database"
        );
        assert_eq!(
            SyncPhase::RunningPostSyncExport.to_message(),
            "Running post-sync export"
        );
        assert_eq!(SyncPhase::Idle.to_message(), "No work for current window");
    }

    #[test]
    fn raindex_sync_status_syncing_sets_correct_fields() {
        let raindex_id = test_raindex_id();
        let status = RaindexSyncStatus::syncing(raindex_id.clone(), SyncPhase::FetchingLatestBlock);

        assert_eq!(status.raindex_id, raindex_id);
        assert_eq!(status.status, LocalDbStatus::Syncing);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(
            status.phase_message,
            Some("Fetching latest block".to_string())
        );
        assert!(status.error.is_none());
    }

    #[test]
    fn raindex_sync_status_active_with_leader_sets_correct_fields() {
        let raindex_id = test_raindex_id();
        let status = RaindexSyncStatus::active(raindex_id.clone(), SchedulerState::Leader);

        assert_eq!(status.raindex_id, raindex_id);
        assert_eq!(status.status, LocalDbStatus::Active);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert!(status.phase_message.is_none());
        assert!(status.error.is_none());
    }

    #[test]
    fn raindex_sync_status_active_with_not_leader_sets_correct_fields() {
        let raindex_id = test_raindex_id();
        let status = RaindexSyncStatus::active(raindex_id.clone(), SchedulerState::NotLeader);

        assert_eq!(status.raindex_id, raindex_id);
        assert_eq!(status.status, LocalDbStatus::Active);
        assert_eq!(status.scheduler_state, SchedulerState::NotLeader);
        assert!(status.phase_message.is_none());
        assert!(status.error.is_none());
    }

    #[test]
    fn raindex_sync_status_failure_sets_correct_fields() {
        let raindex_id = test_raindex_id();
        let error_msg = "RPC connection failed".to_string();
        let status = RaindexSyncStatus::failure(raindex_id.clone(), error_msg.clone());

        assert_eq!(status.raindex_id, raindex_id);
        assert_eq!(status.status, LocalDbStatus::Failure);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert!(status.phase_message.is_none());
        assert_eq!(status.error, Some(error_msg));
    }

    #[test]
    fn raindex_sync_status_new_with_all_fields() {
        let raindex_id = test_raindex_id();
        let status = RaindexSyncStatus::new(
            raindex_id.clone(),
            LocalDbStatus::Syncing,
            SchedulerState::Leader,
            Some("Custom phase".to_string()),
            Some("Custom error".to_string()),
        );

        assert_eq!(status.raindex_id, raindex_id);
        assert_eq!(status.status, LocalDbStatus::Syncing);
        assert_eq!(status.scheduler_state, SchedulerState::Leader);
        assert_eq!(status.phase_message, Some("Custom phase".to_string()));
        assert_eq!(status.error, Some("Custom error".to_string()));
    }
}
