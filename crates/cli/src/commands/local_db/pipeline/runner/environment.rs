use rain_orderbook_common::local_db::pipeline::{
    adapters::{
        apply::DefaultApplyPipeline, events::DefaultEventsPipeline, tokens::DefaultTokensPipeline,
        window::DefaultWindowPipeline,
    },
    runner::{
        environment::{
            default_dump_downloader, default_manifest_fetcher, EnginePipelines, RunnerEnvironment,
        },
        utils::RunnerTarget,
    },
};
use std::sync::Arc;

use crate::commands::local_db::pipeline::{
    bootstrap::ProducerBootstrapAdapter,
    status::{DebugStatus, ProducerStatusBus},
};

pub fn default_environment(
    hypersync_token: String,
    debug_status: DebugStatus,
) -> RunnerEnvironment<
    ProducerBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    ProducerStatusBus,
> {
    RunnerEnvironment::new(
        default_manifest_fetcher(),
        default_dump_downloader(),
        Arc::new(move |target: &RunnerTarget| {
            let events = DefaultEventsPipeline::with_hyperrpc(
                target.inputs.ob_id.chain_id,
                hypersync_token.clone(),
            )?;
            let tokens = DefaultTokensPipeline::new(target.inputs.metadata_rpcs.clone())?;
            let status = ProducerStatusBus::new(
                debug_status,
                target.orderbook_key.clone(),
                target.inputs.ob_id.clone(),
            );

            Ok(EnginePipelines::new(
                ProducerBootstrapAdapter::new(),
                DefaultWindowPipeline::new(),
                events,
                tokens,
                DefaultApplyPipeline::new(),
                status,
            ))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;
    use rain_orderbook_common::local_db::fetch::FetchConfig;
    use rain_orderbook_common::local_db::pipeline::engine::SyncInputs;
    use rain_orderbook_common::local_db::pipeline::{FinalityConfig, SyncConfig, WindowOverrides};
    use rain_orderbook_common::local_db::{LocalDbError, OrderbookIdentifier};
    use rain_orderbook_common::rpc_client::RpcClientError;
    use url::Url;

    fn sample_target(chain_id: u32) -> RunnerTarget {
        let fetch = FetchConfig::new(1, 1, 1, 1, 0, 0).expect("fetch config");
        RunnerTarget {
            orderbook_key: "test-book".to_string(),
            manifest_url: Url::parse("https://manifests.example/default.yaml").unwrap(),
            network_key: "anvil".to_string(),
            inputs: SyncInputs {
                ob_id: OrderbookIdentifier::new(
                    chain_id,
                    address!("00000000000000000000000000000000000000a1"),
                ),
                metadata_rpcs: vec![Url::parse("https://rpc.example/anvil").unwrap()],
                cfg: SyncConfig {
                    deployment_block: 100,
                    fetch,
                    finality: FinalityConfig { depth: 12 },
                    window_overrides: WindowOverrides::default(),
                },
                dump_str: None,
                block_number_threshold: 10000,
                manifest_end_block: 1,
            },
        }
    }

    #[test]
    fn build_engine_configures_hyperrpc_for_supported_chain() {
        let env = default_environment("super-secret-token".to_string(), DebugStatus::Disabled);
        let target = sample_target(42161);
        let engine = env.build_engine(&target).expect("engine available");

        let events_debug = format!("{:?}", engine.events);
        assert!(
            events_debug.contains("chain_id: Some(42161)"),
            "expected HyperRPC client for chain 42161: {events_debug}"
        );
        assert!(
            events_debug.contains("https://arbitrum.rpc.hypersync.xyz"),
            "expected HyperRPC base URL in debug repr: {events_debug}"
        );
    }

    #[test]
    fn build_engine_rejects_unsupported_chain() {
        let env = default_environment("token".to_string(), DebugStatus::Disabled);
        let target = sample_target(1);
        match env.build_engine(&target) {
            Err(LocalDbError::Rpc(RpcClientError::UnsupportedChainId { chain_id })) => {
                assert_eq!(chain_id, 1);
            }
            Err(other) => panic!("unexpected error variant: {other:?}"),
            Ok(_) => panic!("expected unsupported chain to fail"),
        }
    }
}
