use crate::local_db::pipeline::runner::utils::{
    build_runner_targets, ParsedRunnerSettings, RunnerTarget,
};
use crate::local_db::LocalDbError;
use rain_orderbook_app_settings::local_db_sync::LocalDbSyncCfg;
use rain_orderbook_app_settings::orderbook::OrderbookCfg;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NetworkRunnerConfig {
    pub network_key: String,
    pub chain_id: u32,
    pub settings: ParsedRunnerSettings,
}

impl NetworkRunnerConfig {
    pub fn from_global_settings(
        global: &ParsedRunnerSettings,
        network_key: &str,
    ) -> Result<Self, LocalDbError> {
        let filtered_orderbooks: HashMap<String, OrderbookCfg> = global
            .orderbooks
            .iter()
            .filter(|(_, ob)| ob.network.key == network_key)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if filtered_orderbooks.is_empty() {
            return Err(LocalDbError::CustomError(format!(
                "no orderbooks found for network: {}",
                network_key
            )));
        }

        let chain_id = filtered_orderbooks
            .values()
            .next()
            .map(|ob| ob.network.chain_id)
            .ok_or_else(|| {
                LocalDbError::CustomError(format!(
                    "could not determine chain_id for network: {}",
                    network_key
                ))
            })?;

        let filtered_syncs: HashMap<String, LocalDbSyncCfg> = global
            .syncs
            .iter()
            .filter(|(k, _)| *k == network_key)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Self {
            network_key: network_key.to_string(),
            chain_id,
            settings: ParsedRunnerSettings {
                orderbooks: filtered_orderbooks,
                syncs: filtered_syncs,
            },
        })
    }

    pub fn build_targets(&self) -> Result<Vec<RunnerTarget>, LocalDbError> {
        build_runner_targets(&self.settings.orderbooks, &self.settings.syncs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::pipeline::runner::utils::parse_runner_settings;
    use rain_orderbook_app_settings::spec_version::SpecVersion;

    fn sample_settings_yaml() -> String {
        format!(
            r#"
version: {}
networks:
  network-a:
    rpcs:
      - https://rpc.network/a
    chain-id: 1
  network-b:
    rpcs:
      - https://rpc.network/b
    chain-id: 2
subgraphs:
  network-a: https://subgraph.network/a
  network-b: https://subgraph.network/b
local-db-remotes:
  remote-a: https://remotes.example.com/a.yaml
  remote-b: https://remotes.example.com/b.yaml
local-db-sync:
  network-a:
    batch-size: 10
    max-concurrent-batches: 5
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 50
    finality-depth: 12
    bootstrap-block-threshold: 10000
    sync-interval-ms: 5000
  network-b:
    batch-size: 20
    max-concurrent-batches: 2
    retry-attempts: 4
    retry-delay-ms: 200
    rate-limit-delay-ms: 100
    finality-depth: 24
    bootstrap-block-threshold: 5000
    sync-interval-ms: 5000
orderbooks:
  ob-a:
    address: 0x00000000000000000000000000000000000000a1
    network: network-a
    subgraph: network-a
    local-db-remote: remote-a
    deployment-block: 111
  ob-b:
    address: 0x00000000000000000000000000000000000000b2
    network: network-b
    subgraph: network-b
    local-db-remote: remote-b
    deployment-block: 222
  ob-c:
    address: 0x00000000000000000000000000000000000000c3
    network: network-a
    subgraph: network-a
    local-db-remote: remote-a
    deployment-block: 333
"#,
            SpecVersion::current()
        )
    }

    #[test]
    fn from_global_settings_filters_orderbooks_for_network() {
        let global = parse_runner_settings(&sample_settings_yaml()).expect("valid yaml");
        let config =
            NetworkRunnerConfig::from_global_settings(&global, "network-a").expect("config ok");

        assert_eq!(config.network_key, "network-a");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.settings.orderbooks.len(), 2);
        assert!(config.settings.orderbooks.contains_key("ob-a"));
        assert!(config.settings.orderbooks.contains_key("ob-c"));
        assert!(!config.settings.orderbooks.contains_key("ob-b"));
    }

    #[test]
    fn from_global_settings_filters_syncs_for_network() {
        let global = parse_runner_settings(&sample_settings_yaml()).expect("valid yaml");
        let config =
            NetworkRunnerConfig::from_global_settings(&global, "network-b").expect("config ok");

        assert_eq!(config.settings.syncs.len(), 1);
        assert!(config.settings.syncs.contains_key("network-b"));
        assert!(!config.settings.syncs.contains_key("network-a"));
    }

    #[test]
    fn from_global_settings_errors_for_unknown_network() {
        let global = parse_runner_settings(&sample_settings_yaml()).expect("valid yaml");
        let err = NetworkRunnerConfig::from_global_settings(&global, "network-unknown")
            .expect_err("should error");

        assert!(matches!(err, LocalDbError::CustomError(_)));
    }

    #[test]
    fn build_targets_returns_targets_for_network_only() {
        let global = parse_runner_settings(&sample_settings_yaml()).expect("valid yaml");
        let config =
            NetworkRunnerConfig::from_global_settings(&global, "network-a").expect("config ok");

        let targets = config.build_targets().expect("targets ok");
        assert_eq!(targets.len(), 2);
        assert!(targets.iter().all(|t| t.network_key == "network-a"));
    }

    #[test]
    fn build_targets_single_orderbook_network() {
        let global = parse_runner_settings(&sample_settings_yaml()).expect("valid yaml");
        let config =
            NetworkRunnerConfig::from_global_settings(&global, "network-b").expect("config ok");

        let targets = config.build_targets().expect("targets ok");
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].orderbook_key, "ob-b");
    }
}
