use crate::local_db::pipeline::adapters::{
    apply::DefaultApplyPipeline, events::DefaultEventsPipeline, tokens::DefaultTokensPipeline,
    window::DefaultWindowPipeline,
};
use crate::local_db::pipeline::runner::environment::{
    default_dump_downloader, default_manifest_fetcher, EnginePipelines, RunnerEnvironment,
};
use crate::local_db::pipeline::runner::utils::RunnerTarget;
use crate::raindex_client::local_db::pipeline::bootstrap::ClientBootstrapAdapter;
use std::sync::Arc;

#[cfg(target_family = "wasm")]
use crate::raindex_client::local_db::pipeline::status::ClientStatusBus;

#[cfg(not(target_family = "wasm"))]
use crate::raindex_client::local_db::pipeline::status::TracingStatusBus;

#[cfg(target_family = "wasm")]
pub fn default_environment() -> RunnerEnvironment<
    ClientBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    ClientStatusBus,
> {
    RunnerEnvironment::new(
        default_manifest_fetcher(),
        default_dump_downloader(),
        Arc::new(|target: &RunnerTarget| {
            let events =
                DefaultEventsPipeline::with_regular_rpcs(target.inputs.metadata_rpcs.clone())?;
            let tokens = DefaultTokensPipeline::new(target.inputs.metadata_rpcs.clone())?;

            let status_bus = ClientStatusBus::with_ob_id(target.inputs.ob_id.clone());

            Ok(EnginePipelines::new(
                ClientBootstrapAdapter::new(),
                DefaultWindowPipeline::new(),
                events,
                tokens,
                DefaultApplyPipeline::new(),
                status_bus,
            ))
        }),
    )
}

#[cfg(not(target_family = "wasm"))]
pub fn default_environment() -> RunnerEnvironment<
    ClientBootstrapAdapter,
    DefaultWindowPipeline,
    DefaultEventsPipeline,
    DefaultTokensPipeline,
    DefaultApplyPipeline,
    TracingStatusBus,
> {
    RunnerEnvironment::new(
        default_manifest_fetcher(),
        default_dump_downloader(),
        Arc::new(|target: &RunnerTarget| {
            let events =
                DefaultEventsPipeline::with_regular_rpcs(target.inputs.metadata_rpcs.clone())?;
            let tokens = DefaultTokensPipeline::new(target.inputs.metadata_rpcs.clone())?;

            let status_bus = TracingStatusBus::with_ob_id(target.inputs.ob_id.clone());

            Ok(EnginePipelines::new(
                ClientBootstrapAdapter::new(),
                DefaultWindowPipeline::new(),
                events,
                tokens,
                DefaultApplyPipeline::new(),
                status_bus,
            ))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::fetch::FetchConfig;
    use crate::local_db::pipeline::engine::SyncInputs;
    use crate::local_db::pipeline::{FinalityConfig, SyncConfig, WindowOverrides};
    use crate::local_db::{LocalDbError, OrderbookIdentifier};
    use crate::rpc_client::RpcClientError;
    use alloy::primitives::address;
    use url::Url;

    fn sample_target(metadata_rpcs: Vec<Url>) -> RunnerTarget {
        let fetch = FetchConfig::new(1, 1, 1, 1, 0, 0).expect("fetch config");
        RunnerTarget {
            orderbook_key: "test-ob".to_string(),
            network_key: "test-network".to_string(),
            manifest_url: Url::parse("https://manifests.example/client.yaml").unwrap(),
            inputs: SyncInputs {
                ob_id: OrderbookIdentifier {
                    chain_id: 1,
                    orderbook_address: address!("00000000000000000000000000000000000000c1"),
                },
                metadata_rpcs,
                cfg: SyncConfig {
                    deployment_block: 0,
                    fetch,
                    finality: FinalityConfig { depth: 0 },
                    window_overrides: WindowOverrides::default(),
                },
                dump_str: None,
                block_number_threshold: 10000,
                manifest_end_block: 1,
            },
        }
    }

    #[test]
    fn build_engine_uses_regular_rpcs() {
        let env = default_environment();
        let target = sample_target(vec![Url::parse("https://rpc.client.example/anvil").unwrap()]);
        let engine = env.build_engine(&target).expect("engine available");

        let events_debug = format!("{:?}", engine.events);
        assert!(
            events_debug.contains("chain_id: None"),
            "expected regular RPC client debug; got {events_debug}"
        );
        assert!(
            events_debug.contains("https://rpc.client.example/***"),
            "expected redacted regular RPC URL in debug repr; got {events_debug}"
        );
    }

    #[test]
    fn build_engine_requires_metadata_rpcs() {
        let env = default_environment();
        let target = sample_target(Vec::new());
        match env.build_engine(&target) {
            Err(LocalDbError::Rpc(RpcClientError::Config { message })) => {
                assert_eq!(message, "at least one RPC URL is required");
            }
            Err(other) => panic!("unexpected error variant: {other:?}"),
            Ok(_) => panic!("expected missing RPC URLs to fail"),
        }
    }

    #[test]
    fn build_engine_preserves_rpc_order() {
        let env = default_environment();
        let target = sample_target(vec![
            Url::parse("https://alpha.client.example/rpc-one").unwrap(),
            Url::parse("https://beta.client.example/rpc-two").unwrap(),
        ]);
        let engine = env.build_engine(&target).expect("engine available");

        let events_debug = format!("{:?}", engine.events);
        assert!(
            events_debug.contains("https://alpha.client.example/***"),
            "expected first RPC URL redacted in debug; got {events_debug}"
        );
        assert!(
            events_debug.contains("https://beta.client.example/***"),
            "expected second RPC URL redacted in debug; got {events_debug}"
        );
    }
}
