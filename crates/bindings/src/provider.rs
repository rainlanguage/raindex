use alloy::network::AnyNetwork;
use alloy::providers::{
    fillers::FillProvider, utils::JoinedRecommendedFillers, ProviderBuilder, RootProvider,
};
use alloy::rpc::client::RpcClient;
use alloy::transports::http::Http;
use alloy::transports::layers::FallbackLayer;
use std::num::NonZeroUsize;
use thiserror::Error;
use tower::ServiceBuilder;
use url::Url;

pub type ReadProvider =
    FillProvider<JoinedRecommendedFillers, RootProvider<AnyNetwork>, AnyNetwork>;

#[derive(Error, Debug)]
pub enum ReadProviderError {
    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("No RPC URLs provided")]
    NoRpcs,
}

pub fn mk_read_provider(rpcs: &[Url]) -> Result<ReadProvider, ReadProviderError> {
    if rpcs.is_empty() {
        return Err(ReadProviderError::NoRpcs);
    }

    // Use one active transport per request: alloy's FallbackLayer health-routes
    // to the best-scored transport and falls back to others on error/429. With
    // `active_transport_count = rpcs.len()` it would dispatch every request to
    // ALL transports in parallel (request amplification), defeating the purpose
    // of providing multiple RPCs for load sharing.
    let fallback_layer = FallbackLayer::default()
        .with_active_transport_count(NonZeroUsize::new(1).expect("1 is non-zero"));

    let transports = rpcs
        .iter()
        .map(|rpc| Ok::<_, ReadProviderError>(Http::new(rpc.clone())))
        .collect::<Result<Vec<_>, _>>()?;

    let transport = ServiceBuilder::new()
        .layer(fallback_layer)
        .service(transports);
    let client = RpcClient::builder().transport(transport, false);
    let provider = ProviderBuilder::new_with_network::<AnyNetwork>().connect_client(client);

    Ok(provider)
}
