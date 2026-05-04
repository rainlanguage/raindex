use crate::local_db::pipeline::{StatusBus, SyncPhase};
use crate::local_db::{LocalDbError, OrderbookIdentifier};
use crate::raindex_client::local_db::{LocalDbSyncStatusStore, OrderbookSyncStatus};

#[derive(Debug, Clone, Default)]
pub struct TracingStatusBus {
    ob_id: Option<OrderbookIdentifier>,
    orderbook_key: Option<String>,
    status_store: LocalDbSyncStatusStore,
}

impl TracingStatusBus {
    pub fn new() -> Self {
        Self {
            ob_id: None,
            orderbook_key: None,
            status_store: LocalDbSyncStatusStore::new(),
        }
    }

    pub fn with_ob_id(ob_id: OrderbookIdentifier) -> Self {
        Self::with_ob_id_and_store(ob_id, LocalDbSyncStatusStore::new())
    }

    pub fn with_ob_id_and_store(
        ob_id: OrderbookIdentifier,
        status_store: LocalDbSyncStatusStore,
    ) -> Self {
        Self {
            ob_id: Some(ob_id),
            orderbook_key: None,
            status_store,
        }
    }

    pub fn with_ob_id_and_key(ob_id: OrderbookIdentifier, key: String) -> Self {
        Self {
            ob_id: Some(ob_id),
            orderbook_key: Some(key),
            status_store: LocalDbSyncStatusStore::new(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl StatusBus for TracingStatusBus {
    async fn send(&self, phase: SyncPhase) -> Result<(), LocalDbError> {
        let chain_id = self.ob_id.as_ref().map(|id| id.chain_id).unwrap_or(0);
        let ob_addr = self
            .ob_id
            .as_ref()
            .map(|id| format!("{:#x}", id.orderbook_address))
            .unwrap_or_default();
        let key = self.orderbook_key.as_deref().unwrap_or("unknown");

        tracing::debug!(
            chain_id = chain_id,
            orderbook = %ob_addr,
            key = key,
            phase = %phase.to_message(),
            "sync phase"
        );

        if let Some(ob_id) = &self.ob_id {
            self.status_store
                .record_orderbook_status(OrderbookSyncStatus::syncing(ob_id.clone(), phase));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::pipeline::SyncPhase;
    use crate::local_db::OrderbookIdentifier;
    use alloy::primitives::address;

    fn test_ob_id() -> OrderbookIdentifier {
        OrderbookIdentifier::new(1, address!("0000000000000000000000000000000000001234"))
    }

    #[tokio::test]
    async fn tracing_status_bus_send_returns_ok() {
        let ob_id = test_ob_id();
        let bus = TracingStatusBus::with_ob_id(ob_id);
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn tracing_status_bus_send_without_ob_id_returns_ok() {
        let bus = TracingStatusBus::new();
        let result = bus.send(SyncPhase::FetchingLatestBlock).await;
        assert!(result.is_ok());
    }
}
