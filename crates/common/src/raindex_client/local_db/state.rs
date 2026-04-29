use super::LocalDb;
#[cfg(not(target_family = "wasm"))]
use crate::raindex_client::local_db::pipeline::runner::scheduler::NativeSyncHandle;
#[cfg(target_family = "wasm")]
use crate::raindex_client::local_db::pipeline::runner::scheduler::SchedulerHandle;
use raindex_app_settings::network::NetworkCfg;
use raindex_app_settings::yaml::raindex::RaindexYaml;
#[cfg(target_family = "wasm")]
use std::cell::RefCell;
use std::collections::HashSet;
#[cfg(target_family = "wasm")]
use std::rc::Rc;
#[cfg(not(target_family = "wasm"))]
use std::sync::{Arc, Mutex};

pub(crate) enum QuerySource {
    LocalDb(LocalDb),
    Subgraph,
}

pub(crate) type ClassifiedChains = (Option<LocalDb>, Vec<u32>, Vec<u32>);

#[derive(Clone, Debug)]
pub(crate) struct LocalDbState {
    #[cfg(target_family = "wasm")]
    pub(crate) db: Rc<RefCell<Option<LocalDb>>>,
    #[cfg(not(target_family = "wasm"))]
    pub(crate) db: Arc<Mutex<Option<LocalDb>>>,
    #[cfg(target_family = "wasm")]
    pub(crate) scheduler: Rc<RefCell<Option<SchedulerHandle>>>,
    #[cfg(not(target_family = "wasm"))]
    pub(crate) scheduler: Arc<Mutex<Option<NativeSyncHandle>>>,
    pub(crate) sync_readiness: SyncReadiness,
    pub(crate) sync_configured_chains: HashSet<u32>,
}

impl Default for LocalDbState {
    fn default() -> Self {
        Self {
            #[cfg(target_family = "wasm")]
            db: Rc::new(RefCell::new(None)),
            #[cfg(not(target_family = "wasm"))]
            db: Arc::new(Mutex::new(None)),
            #[cfg(target_family = "wasm")]
            scheduler: Rc::new(RefCell::new(None)),
            #[cfg(not(target_family = "wasm"))]
            scheduler: Arc::new(Mutex::new(None)),
            sync_readiness: SyncReadiness::new(),
            sync_configured_chains: HashSet::new(),
        }
    }
}

#[cfg(target_family = "wasm")]
impl LocalDbState {
    pub(crate) fn new(
        db: Option<LocalDb>,
        scheduler: Rc<RefCell<Option<SchedulerHandle>>>,
        sync_readiness: SyncReadiness,
        sync_configured_chains: HashSet<u32>,
    ) -> Self {
        Self {
            db: Rc::new(RefCell::new(db)),
            scheduler,
            sync_readiness,
            sync_configured_chains,
        }
    }

    fn db(&self) -> Option<LocalDb> {
        self.db.borrow().as_ref().cloned()
    }
}

#[cfg(not(target_family = "wasm"))]
impl LocalDbState {
    pub(crate) fn new(
        db: Option<LocalDb>,
        scheduler: Arc<Mutex<Option<NativeSyncHandle>>>,
        sync_readiness: SyncReadiness,
        sync_configured_chains: HashSet<u32>,
    ) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            scheduler,
            sync_readiness,
            sync_configured_chains,
        }
    }

    fn db(&self) -> Option<LocalDb> {
        self.db
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
    }
}

impl LocalDbState {
    pub(crate) fn compute_chain_ids(yaml: &RaindexYaml) -> HashSet<u32> {
        let syncs = match yaml.get_local_db_syncs() {
            Ok(s) => s,
            Err(_) => return HashSet::new(),
        };
        let networks = match yaml.get_networks() {
            Ok(n) => n,
            Err(_) => return HashSet::new(),
        };
        let mut ids = HashSet::new();
        for sync_key in syncs.keys() {
            if let Some(network) = networks.get(sync_key) {
                ids.insert(network.chain_id);
            }
        }
        ids
    }

    pub(crate) fn query_source(&self, chain_id: u32) -> QuerySource {
        if let Some(db) = self.db() {
            if self.sync_configured_chains.contains(&chain_id)
                && self.sync_readiness.is_ready(chain_id)
            {
                return QuerySource::LocalDb(db);
            }
        }
        QuerySource::Subgraph
    }

    pub(crate) fn classify_chains(&self, networks: &[NetworkCfg]) -> ClassifiedChains {
        let mut local_db: Option<LocalDb> = None;
        let mut local_ids = Vec::new();
        let mut sg_ids = Vec::new();
        for net in networks {
            match self.query_source(net.chain_id) {
                QuerySource::LocalDb(db) => {
                    if local_db.is_none() {
                        local_db = Some(db);
                    }
                    local_ids.push(net.chain_id);
                }
                QuerySource::Subgraph => sg_ids.push(net.chain_id),
            }
        }
        (local_db, local_ids, sg_ids)
    }
}

impl Drop for LocalDbState {
    fn drop(&mut self) {
        #[cfg(target_family = "wasm")]
        {
            if Rc::strong_count(&self.scheduler) == 1 {
                if let Some(handle) = self.scheduler.borrow_mut().take() {
                    handle.stop();
                }
            }
        }
        #[cfg(not(target_family = "wasm"))]
        {
            if Arc::strong_count(&self.scheduler) == 1 {
                let mut guard = self
                    .scheduler
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Some(handle) = guard.take() {
                    handle.stop();
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SyncReadiness {
    #[cfg(target_family = "wasm")]
    ready: Rc<RefCell<HashSet<u32>>>,
    #[cfg(not(target_family = "wasm"))]
    ready: std::sync::Arc<std::sync::RwLock<HashSet<u32>>>,
}

impl SyncReadiness {
    pub fn new() -> Self {
        Self {
            #[cfg(target_family = "wasm")]
            ready: Rc::new(RefCell::new(HashSet::new())),
            #[cfg(not(target_family = "wasm"))]
            ready: std::sync::Arc::new(std::sync::RwLock::new(HashSet::new())),
        }
    }

    pub fn mark_ready(&self, chain_id: u32) {
        #[cfg(target_family = "wasm")]
        self.ready.borrow_mut().insert(chain_id);
        #[cfg(not(target_family = "wasm"))]
        self.ready
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(chain_id);
    }

    pub fn is_ready(&self, chain_id: u32) -> bool {
        #[cfg(target_family = "wasm")]
        return self.ready.borrow().contains(&chain_id);
        #[cfg(not(target_family = "wasm"))]
        return self
            .ready
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains(&chain_id);
    }
}

impl Default for SyncReadiness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::{
        FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch,
    };
    use raindex_app_settings::spec_version::SpecVersion;
    use raindex_app_settings::yaml::raindex::{RaindexYaml, RaindexYamlValidation};
    use raindex_app_settings::yaml::YamlParsable;

    struct NoopExec;

    #[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
    impl LocalDbQueryExecutor for NoopExec {
        async fn execute_batch(&self, _: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            Ok(())
        }
        async fn query_json<T: FromDbJson>(
            &self,
            _: &SqlStatement,
        ) -> Result<T, LocalDbQueryError> {
            serde_json::from_str("[]")
                .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
        }
        async fn query_text(&self, _: &SqlStatement) -> Result<String, LocalDbQueryError> {
            Ok(String::new())
        }
        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            Ok(())
        }
    }

    fn dummy_db() -> LocalDb {
        LocalDb::new(NoopExec)
    }

    fn net(chain_id: u32) -> NetworkCfg {
        let mut n = NetworkCfg::dummy();
        n.chain_id = chain_id;
        n
    }

    fn state_with_db(configured: &[u32], ready: &[u32]) -> LocalDbState {
        let readiness = SyncReadiness::new();
        for &id in ready {
            readiness.mark_ready(id);
        }
        LocalDbState::new(
            Some(dummy_db()),
            #[cfg(target_family = "wasm")]
            Rc::new(RefCell::new(None)),
            #[cfg(not(target_family = "wasm"))]
            Arc::new(Mutex::new(None)),
            readiness,
            configured.iter().copied().collect(),
        )
    }

    #[test]
    fn sync_readiness_starts_empty() {
        let r = SyncReadiness::new();
        assert!(!r.is_ready(1));
        assert!(!r.is_ready(42161));
    }

    #[test]
    fn sync_readiness_default_equals_new() {
        let r = SyncReadiness::default();
        assert!(!r.is_ready(1));
    }

    #[test]
    fn sync_readiness_mark_and_check() {
        let r = SyncReadiness::new();
        r.mark_ready(42161);
        assert!(r.is_ready(42161));
        assert!(!r.is_ready(137));
    }

    #[test]
    fn sync_readiness_multiple_chains() {
        let r = SyncReadiness::new();
        r.mark_ready(42161);
        r.mark_ready(137);
        r.mark_ready(8453);
        assert!(r.is_ready(42161));
        assert!(r.is_ready(137));
        assert!(r.is_ready(8453));
        assert!(!r.is_ready(1));
    }

    #[test]
    fn sync_readiness_mark_idempotent() {
        let r = SyncReadiness::new();
        r.mark_ready(42161);
        r.mark_ready(42161);
        assert!(r.is_ready(42161));
    }

    #[test]
    fn default_state_has_no_db() {
        let state = LocalDbState::default();
        assert!(state.db().is_none());
        assert!(state.sync_configured_chains.is_empty());
    }

    #[test]
    fn query_source_subgraph_when_no_db() {
        let state = LocalDbState::default();
        assert!(matches!(state.query_source(42161), QuerySource::Subgraph));
    }

    #[test]
    fn query_source_subgraph_when_chain_not_configured() {
        let state = state_with_db(&[137], &[137]);
        assert!(matches!(state.query_source(42161), QuerySource::Subgraph));
    }

    #[test]
    fn query_source_subgraph_when_chain_not_ready() {
        let state = state_with_db(&[42161], &[]);
        assert!(matches!(state.query_source(42161), QuerySource::Subgraph));
    }

    #[test]
    fn query_source_local_db_when_configured_and_ready() {
        let state = state_with_db(&[42161], &[42161]);
        assert!(matches!(state.query_source(42161), QuerySource::LocalDb(_)));
    }

    #[test]
    fn classify_chains_all_subgraph_when_no_db() {
        let state = LocalDbState::default();
        let networks = vec![net(42161), net(137)];
        let (db, local, sg) = state.classify_chains(&networks);
        assert!(db.is_none());
        assert!(local.is_empty());
        assert_eq!(sg, vec![42161, 137]);
    }

    #[test]
    fn classify_chains_splits_correctly() {
        let state = state_with_db(&[42161, 8453], &[42161, 8453]);
        let networks = vec![net(42161), net(137), net(8453)];
        let (db, local, sg) = state.classify_chains(&networks);
        assert!(db.is_some());
        assert_eq!(local, vec![42161, 8453]);
        assert_eq!(sg, vec![137]);
    }

    #[test]
    fn classify_chains_all_local() {
        let state = state_with_db(&[42161, 137], &[42161, 137]);
        let networks = vec![net(42161), net(137)];
        let (db, local, sg) = state.classify_chains(&networks);
        assert!(db.is_some());
        assert_eq!(local, vec![42161, 137]);
        assert!(sg.is_empty());
    }

    #[test]
    fn classify_chains_empty_networks() {
        let state = state_with_db(&[42161], &[42161]);
        let (db, local, sg) = state.classify_chains(&[]);
        assert!(db.is_none());
        assert!(local.is_empty());
        assert!(sg.is_empty());
    }

    #[test]
    fn compute_chain_ids_from_yaml() {
        let yaml_str = format!(
            r#"
version: {version}
networks:
  anvil:
    rpcs:
      - https://rpc.example/anvil
    chain-id: 42161
  polygon:
    rpcs:
      - https://rpc.example/polygon
    chain-id: 137
subgraphs:
  anvil: https://subgraph.example/anvil
  polygon: https://subgraph.example/polygon
local-db-sync:
  anvil:
    batch-size: 10
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 1
    finality-depth: 12
    bootstrap-block-threshold: 1000
    sync-interval-ms: 5000
raindexes:
  raindex-a:
    address: 0x00000000000000000000000000000000000000a1
    network: anvil
    subgraph: anvil
    deployment-block: 123
"#,
            version = SpecVersion::current()
        );
        let yaml =
            RaindexYaml::new(vec![yaml_str], RaindexYamlValidation::default()).expect("valid yaml");
        let ids = LocalDbState::compute_chain_ids(&yaml);
        assert_eq!(ids, HashSet::from([42161]));
        assert!(!ids.contains(&137));
    }

    #[test]
    fn compute_chain_ids_no_sync_config() {
        let yaml_str = format!(
            r#"
version: {version}
networks:
  anvil:
    rpcs:
      - https://rpc.example/anvil
    chain-id: 42161
subgraphs:
  anvil: https://subgraph.example/anvil
raindexes:
  raindex-a:
    address: 0x00000000000000000000000000000000000000a1
    network: anvil
    subgraph: anvil
    deployment-block: 123
"#,
            version = SpecVersion::current()
        );
        let yaml =
            RaindexYaml::new(vec![yaml_str], RaindexYamlValidation::default()).expect("valid yaml");
        let ids = LocalDbState::compute_chain_ids(&yaml);
        assert!(ids.is_empty());
    }
}
