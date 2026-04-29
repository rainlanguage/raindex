use raindex_common::local_db::{
    pipeline::{StatusBus, SyncPhase},
    LocalDbError, RaindexIdentifier,
};
use tracing::info;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DebugStatus {
    #[default]
    Disabled,
    Enabled,
}

impl From<bool> for DebugStatus {
    fn from(value: bool) -> Self {
        if value {
            DebugStatus::Enabled
        } else {
            DebugStatus::Disabled
        }
    }
}

#[derive(Clone)]
pub struct ProducerStatusBus {
    debug: DebugStatus,
    raindex_key: String,
    raindex_id: RaindexIdentifier,
}

impl ProducerStatusBus {
    pub fn new(debug: DebugStatus, raindex_key: String, raindex_id: RaindexIdentifier) -> Self {
        Self {
            debug,
            raindex_key,
            raindex_id,
        }
    }
}

impl Default for ProducerStatusBus {
    fn default() -> Self {
        Self {
            debug: DebugStatus::Disabled,
            raindex_key: "<unknown>".to_string(),
            raindex_id: RaindexIdentifier::new(0, Default::default()),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for ProducerStatusBus {
    async fn send(&self, phase: SyncPhase) -> Result<(), LocalDbError> {
        if self.debug == DebugStatus::Enabled {
            info!(
                target: "local_db_status",
                chain_id = self.raindex_id.chain_id,
                raindex = %self.raindex_id.raindex_address,
                raindex_key = %self.raindex_key,
                "{}",
                phase.to_message()
            );
        }
        Ok(())
    }
}
