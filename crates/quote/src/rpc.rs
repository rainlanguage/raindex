use crate::{
    error::{Error, FailedQuote},
    quote::{QuoteResult, QuoteTarget},
};
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, Bytes};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::serde::WithOtherFields;
use alloy::sol_types::SolCall;
use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    transports::RpcError,
};
use futures::future::join_all;
use rain_error_decoding::{AbiDecodedErrorType, ErrorRegistry};
use rain_orderbook_bindings::provider::{mk_read_provider, ReadProvider};
use rain_orderbook_bindings::IRaindexV6::quote2Call;
use rain_orderbook_bindings::OrderBook::multicallCall;
use url::Url;

const DEFAULT_QUOTE_CHUNK_SIZE: usize = 16;

fn normalize_chunk_size(chunk_size: Option<usize>) -> usize {
    chunk_size.unwrap_or(DEFAULT_QUOTE_CHUNK_SIZE).max(1)
}

/// Classify a chunk-level failure into per-target `QuoteResult`s when the
/// bisection has narrowed down to a single target (or cannot narrow further).
fn single_quote_failure(err: &Error) -> QuoteResult {
    Err(FailedQuote::CorruptReturnData(format!(
        "Single quote failed after chunk split: {err}"
    )))
}

/// Issue a single `orderbook.multicall([quote2, quote2, ...])` RPC call with
/// `from = counterparty` so each inner `quote2` sees `msg.sender = counterparty`.
///
/// OrderBookV6 inherits OpenZeppelin's `Multicall` which `delegatecall`s each
/// element back into the orderbook itself. `delegatecall` preserves the outer
/// `msg.sender`, so the calldata built here gives API-gated strategies (which
/// read `order-counterparty()` in `calculate-io`) the correct taker address.
///
/// All `quote_targets` passed here MUST share the same `orderbook` address —
/// that grouping is the caller's responsibility (see `batch_quote`).
///
/// Behavior:
/// - Success path: returns a `Vec<QuoteResult>` aligned 1:1 with `quote_targets`.
///   Per-target `exists == false` becomes `FailedQuote::NonExistent`.
/// - Revert path: OZ `Multicall` bubbles the FIRST reverting inner call's
///   revert data. There is no per-element error isolation, so this function
///   returns `Err(Error::...)` with a decoded `FailedQuote` surface. The
///   caller (`quote_chunk_with_probe_and_split`) is responsible for bisecting
///   to attribute the revert to a specific target.
async fn quote_chunk_once(
    quote_targets: &[QuoteTarget],
    provider: &ReadProvider,
    block_number: Option<u64>,
    counterparty: Address,
    registry: Option<&dyn ErrorRegistry>,
) -> Result<Vec<QuoteResult>, Error> {
    debug_assert!(!quote_targets.is_empty());
    let orderbook = quote_targets[0].orderbook;
    debug_assert!(quote_targets.iter().all(|t| t.orderbook == orderbook));

    let inner_calls: Vec<Bytes> = quote_targets
        .iter()
        .map(|t| {
            Bytes::from(
                quote2Call {
                    quoteConfig: t.quote_config.clone(),
                }
                .abi_encode(),
            )
        })
        .collect();

    let calldata = multicallCall {
        data: inner_calls.clone(),
    }
    .abi_encode();

    let tx = TransactionRequest::default()
        .with_to(orderbook)
        .with_from(counterparty)
        .with_input(calldata);
    let tx = WithOtherFields::new(tx);

    let block = block_number
        .map(|n| BlockId::Number(BlockNumberOrTag::Number(n)))
        .unwrap_or(BlockId::latest());

    match provider.call(tx).block(block).await {
        Ok(bytes) => {
            let decoded = multicallCall::abi_decode_returns(&bytes).map_err(|e| {
                Error::AlloySolTypesError(alloy::sol_types::Error::Other(
                    format!("failed to decode multicall results: {e}").into(),
                ))
            })?;

            if decoded.len() != quote_targets.len() {
                return Err(Error::AlloySolTypesError(alloy::sol_types::Error::Other(
                    format!(
                        "multicall length mismatch: expected {}, got {}",
                        quote_targets.len(),
                        decoded.len()
                    )
                    .into(),
                )));
            }

            let mut results: Vec<QuoteResult> = Vec::with_capacity(decoded.len());
            for element in decoded {
                match quote2Call::abi_decode_returns(&element) {
                    Ok(ret) => {
                        if ret.exists {
                            results.push(Ok(ret.into()));
                        } else {
                            results.push(Err(FailedQuote::NonExistent));
                        }
                    }
                    Err(e) => results.push(Err(FailedQuote::CorruptReturnData(format!(
                        "Failed to decode quote2 return: {e}"
                    )))),
                }
            }
            Ok(results)
        }
        Err(RpcError::ErrorResp(err_resp)) => {
            if let Some(revert) = err_resp.as_revert_data() {
                let decoded = match AbiDecodedErrorType::selector_registry_abi_decode(
                    revert.as_ref(),
                    registry,
                )
                .await
                {
                    Ok(abi_err) => FailedQuote::RevertError(abi_err),
                    Err(err) => FailedQuote::RevertErrorDecodeFailed(err),
                };
                // Wrap the per-target failure as a chunk-level error so the
                // bisection logic can decide whether to split or to attribute
                // it to a single target.
                Err(Error::ChunkReverted(Box::new(decoded)))
            } else {
                Err(Error::ChunkReverted(Box::new(
                    FailedQuote::CorruptReturnData(format!(
                        "RPC error without revert data: {err_resp}"
                    )),
                )))
            }
        }
        Err(e) => Err(Error::TransportError(format!("{e}"))),
    }
}

/// Execute `quote_chunk_once` for a single-orderbook group. If it fails at the
/// chunk level (OZ Multicall bubbles the first revert), recursively bisect the
/// group to isolate which target is responsible. All targets in `quote_targets`
/// MUST share the same orderbook.
async fn quote_chunk_with_probe_and_split(
    quote_targets: &[QuoteTarget],
    provider: &ReadProvider,
    block_number: Option<u64>,
    counterparty: Address,
    registry: Option<&dyn ErrorRegistry>,
) -> Result<Vec<QuoteResult>, Error> {
    let initial_err = match quote_chunk_once(
        quote_targets,
        provider,
        block_number,
        counterparty,
        registry,
    )
    .await
    {
        Ok(results) => return Ok(results),
        Err(err) => err,
    };

    if quote_targets.len() <= 1 {
        // Singleton batch already reverted — attribute the revert to that one
        // target using the decoded `FailedQuote` payload.
        return Ok(vec![chunk_err_to_quote_result(&initial_err)]);
    }

    // Bisect unconditionally. A previous version short-circuited here with a
    // pair of "probe" singletons, on the theory that if both probes reverted
    // the whole chunk probably reverts uniformly. That optimisation was wrong
    // for OZ multicall: OZ bubbles the *first* reverting delegatecall, so a
    // chunk-level revert tells us nothing about targets beyond the offending
    // one. With heterogeneous chunks (e.g. a mix of stale-price and fresh
    // orders), the probes easily land on stale targets and every surviving
    // order in the chunk gets silently dropped. Always bisect; the RPC cost
    // of isolating a single reverting target is bounded by log2(chunk_size).

    let mut resolved: Vec<Option<QuoteResult>> = Vec::with_capacity(quote_targets.len());
    resolved.resize_with(quote_targets.len(), || None);
    let mut pending = vec![(0usize, quote_targets.len())];

    while let Some((start, end)) = pending.pop() {
        let chunk = &quote_targets[start..end];
        match quote_chunk_once(chunk, provider, block_number, counterparty, registry).await {
            Ok(results) => {
                for (offset, result) in results.into_iter().enumerate() {
                    resolved[start + offset] = Some(result);
                }
            }
            Err(err) => {
                if chunk.len() == 1 {
                    resolved[start] = Some(chunk_err_to_quote_result(&err));
                } else {
                    let mid = start + (chunk.len() / 2);
                    pending.push((mid, end));
                    pending.push((start, mid));
                }
            }
        }
    }

    Ok(resolved
        .into_iter()
        .map(|v| v.unwrap_or_else(|| single_quote_failure(&initial_err)))
        .collect())
}

/// Convert a chunk-level error produced by `quote_chunk_once` into a per-target
/// `QuoteResult`. This is how we attribute a bisected revert back to a single
/// target.
fn chunk_err_to_quote_result(err: &Error) -> QuoteResult {
    match err {
        Error::ChunkReverted(failed) => match failed.as_ref() {
            FailedQuote::RevertError(abi_err) => Err(FailedQuote::RevertError(abi_err.clone())),
            FailedQuote::CorruptReturnData(msg) => Err(FailedQuote::CorruptReturnData(msg.clone())),
            FailedQuote::RevertErrorDecodeFailed(_) => Err(FailedQuote::CorruptReturnData(
                "Revert could not be decoded by the selector registry".to_string(),
            )),
            FailedQuote::NonExistent => Err(FailedQuote::NonExistent),
            #[cfg(target_family = "wasm")]
            FailedQuote::SerdeWasmBindgenError(_) => Err(FailedQuote::CorruptReturnData(
                "wasm serde error surfaced as chunk revert".to_string(),
            )),
        },
        other => single_quote_failure(other),
    }
}

/// Quotes an array of `QuoteTarget`s.
///
/// Targets are partitioned by their `orderbook` address so that each orderbook
/// gets its own `multicall(bytes[])` RPC call. Using the orderbook's own OZ
/// `Multicall` (rather than Multicall3) ensures `msg.sender` inside each
/// inner `quote2` is preserved as `counterparty` — critical for gated
/// strategies that read `order-counterparty()` inside `calculate-io`.
///
/// Within each orderbook group, targets are chunked by `chunk_size` and each
/// chunk is quoted with probe-and-split bisection: on a chunk revert (OZ
/// Multicall bubbles the first failing inner call's revert), we recurse to
/// isolate which target is responsible.
///
/// The returned `Vec<QuoteResult>` is positionally aligned with the input
/// `quote_targets`.
pub async fn batch_quote(
    quote_targets: &[QuoteTarget],
    rpcs: Vec<String>,
    block_number: Option<u64>,
    counterparty: Address,
    registry: Option<&dyn ErrorRegistry>,
    chunk_size: Option<usize>,
) -> Result<Vec<QuoteResult>, Error> {
    let rpcs = rpcs
        .into_iter()
        .map(|rpc| rpc.parse::<Url>())
        .collect::<Result<Vec<Url>, _>>()?;
    let provider = mk_read_provider(&rpcs)?;
    if quote_targets.is_empty() {
        return Ok(vec![]);
    }

    let chunk_size = normalize_chunk_size(chunk_size);

    // Group by orderbook, preserving the original index of each target so we
    // can scatter results back into the final vector in input order.
    let mut groups: Vec<(Address, Vec<(usize, QuoteTarget)>)> = Vec::new();
    for (i, target) in quote_targets.iter().enumerate() {
        if let Some(group) = groups.iter_mut().find(|(ob, _)| *ob == target.orderbook) {
            group.1.push((i, target.clone()));
        } else {
            groups.push((target.orderbook, vec![(i, target.clone())]));
        }
    }

    // For each orderbook group, run chunked bisected multicalls concurrently.
    let group_futures = groups.into_iter().map(|(_ob, indexed_targets)| {
        let provider = provider.clone();
        async move {
            let mut out: Vec<(usize, QuoteResult)> = Vec::with_capacity(indexed_targets.len());
            let (indexes, targets): (Vec<usize>, Vec<QuoteTarget>) =
                indexed_targets.into_iter().unzip();

            for chunk_start in (0..targets.len()).step_by(chunk_size) {
                let chunk_end = (chunk_start + chunk_size).min(targets.len());
                let chunk = &targets[chunk_start..chunk_end];
                let chunk_results = quote_chunk_with_probe_and_split(
                    chunk,
                    &provider,
                    block_number,
                    counterparty,
                    registry,
                )
                .await?;
                for (offset, result) in chunk_results.into_iter().enumerate() {
                    out.push((indexes[chunk_start + offset], result));
                }
            }
            Ok::<Vec<(usize, QuoteResult)>, Error>(out)
        }
    });

    let group_outputs = join_all(group_futures).await;

    let mut results: Vec<Option<QuoteResult>> = Vec::with_capacity(quote_targets.len());
    results.resize_with(quote_targets.len(), || None);
    for group_result in group_outputs {
        for (idx, r) in group_result? {
            results[idx] = Some(r);
        }
    }
    Ok(results.into_iter().map(|r| r.unwrap()).collect())
}

#[cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy::json_abi::Error as AlloyError;
    use alloy::primitives::Bytes;
    use alloy::sol_types::{SolCall, SolValue};
    use httpmock::{Method::POST, MockServer};
    use rain_error_decoding::ErrorRegistry;
    use rain_math_float::Float;
    use rain_orderbook_bindings::IRaindexV6::{quote2Call, quote2Return};
    use serde_json::json;

    #[test]
    fn test_normalize_chunk_size_defaults_to_16() {
        assert_eq!(normalize_chunk_size(None), 16);
    }

    #[test]
    fn test_normalize_chunk_size_coerces_zero_to_one() {
        assert_eq!(normalize_chunk_size(Some(0)), 1);
    }

    // Encode a single `quote2Return` into its ABI-encoded form (this is what
    // ends up as one element of the outer `bytes[]` returned by OZ Multicall).
    fn encode_quote2_return_bytes(exists: bool, output_max: Float, io_ratio: Float) -> Bytes {
        Bytes::from(quote2Call::abi_encode_returns(&quote2Return {
            exists,
            outputMax: output_max.get_inner(),
            ioRatio: io_ratio.get_inner(),
        }))
    }

    // Encode a `multicall(bytes[]) -> bytes[]` return value as the outer
    // eth_call return bytes, given per-inner encoded returns.
    fn encode_multicall_return(inner: Vec<Bytes>) -> String {
        let encoded = <Vec<Bytes> as SolValue>::abi_encode(&inner);
        alloy::hex::encode_prefixed(encoded)
    }

    fn mock_eth_call_static(server: &MockServer, path: &str, result_hex: &str) {
        let result = result_hex.to_string();
        server.mock(|when, then| {
            when.method(POST).path(path);
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": result,
            }));
        });
    }

    fn mock_eth_call_revert(server: &MockServer, path: &str, revert_hex: &str) {
        let data = revert_hex.to_string();
        server.mock(|when, then| {
            when.method(POST).path(path);
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": 3,
                    "message": "execution reverted",
                    "data": data,
                }
            }));
        });
    }

    #[tokio::test]
    async fn test_batch_quote_empty_returns_empty() {
        let result = batch_quote(
            &[],
            vec!["http://localhost:1".to_string()],
            None,
            Address::ZERO,
            None,
            None,
        )
        .await
        .unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_batch_quote_success() {
        let rpc_server = MockServer::start_async().await;
        let one = Float::parse("1".to_string()).unwrap();
        let two = Float::parse("2".to_string()).unwrap();

        // One target → one multicall call with a single inner return.
        let inner = vec![encode_quote2_return_bytes(true, one, two)];
        mock_eth_call_static(&rpc_server, "/rpc", &encode_multicall_return(inner));

        let quote_targets = vec![QuoteTarget::default()];
        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            Address::ZERO,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 1);
        let q = result[0].as_ref().unwrap();
        assert!(q.max_output.eq(one).unwrap());
        assert!(q.ratio.eq(two).unwrap());
    }

    #[tokio::test]
    async fn test_batch_quote_multi_target_success() {
        let rpc_server = MockServer::start_async().await;
        let one = Float::parse("1".to_string()).unwrap();
        let two = Float::parse("2".to_string()).unwrap();
        let three = Float::parse("3".to_string()).unwrap();
        let four = Float::parse("4".to_string()).unwrap();

        let inner = vec![
            encode_quote2_return_bytes(true, one, two),
            encode_quote2_return_bytes(true, three, four),
        ];
        mock_eth_call_static(&rpc_server, "/rpc", &encode_multicall_return(inner));

        let quote_targets = vec![QuoteTarget::default(), QuoteTarget::default()];
        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            Address::ZERO,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 2);
        let q0 = result[0].as_ref().unwrap();
        let q1 = result[1].as_ref().unwrap();
        assert!(q0.max_output.eq(one).unwrap());
        assert!(q0.ratio.eq(two).unwrap());
        assert!(q1.max_output.eq(three).unwrap());
        assert!(q1.ratio.eq(four).unwrap());
    }

    #[tokio::test]
    async fn test_batch_quote_non_existent_order() {
        let rpc_server = MockServer::start_async().await;
        let zero = Float::parse("0".to_string()).unwrap();

        let inner = vec![encode_quote2_return_bytes(false, zero, zero)];
        mock_eth_call_static(&rpc_server, "/rpc", &encode_multicall_return(inner));

        let quote_targets = vec![QuoteTarget::default()];
        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            Address::ZERO,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Err(FailedQuote::NonExistent)));
    }

    #[tokio::test]
    async fn test_batch_quote_revert_decoded_via_registry() {
        let rpc_server = MockServer::start_async().await;

        // 0x734bc71c -> TokenSelfTrade(). OZ Multicall bubbles this up as the
        // outer revert data.
        mock_eth_call_revert(&rpc_server, "/rpc", "0x734bc71c");

        struct FakeRegistry;
        #[async_trait::async_trait]
        impl ErrorRegistry for FakeRegistry {
            async fn lookup(
                &self,
                selector: [u8; 4],
            ) -> Result<Vec<AlloyError>, rain_error_decoding::AbiDecodeFailedErrors> {
                if selector == [0x73, 0x4b, 0xc7, 0x1c] {
                    Ok(vec!["TokenSelfTrade()".parse().unwrap()])
                } else {
                    Ok(vec![])
                }
            }
        }

        let quote_targets = vec![QuoteTarget::default()];
        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            Address::ZERO,
            Some(&FakeRegistry),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 1);
        assert!(matches!(
            &result[0],
            Err(FailedQuote::RevertError(
                rain_error_decoding::AbiDecodedErrorType::Known { name, .. }
            )) if name == "TokenSelfTrade"
        ));
    }

    #[tokio::test]
    async fn test_batch_quote_revert_unknown_selector() {
        let rpc_server = MockServer::start_async().await;
        mock_eth_call_revert(&rpc_server, "/rpc", "0xff00ff00");

        struct EmptyRegistry;
        #[async_trait::async_trait]
        impl ErrorRegistry for EmptyRegistry {
            async fn lookup(
                &self,
                _selector: [u8; 4],
            ) -> Result<Vec<AlloyError>, rain_error_decoding::AbiDecodeFailedErrors> {
                Ok(vec![])
            }
        }

        let quote_targets = vec![QuoteTarget::default()];
        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            Address::ZERO,
            Some(&EmptyRegistry),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 1);
        assert!(matches!(
            &result[0],
            Err(FailedQuote::RevertError(
                rain_error_decoding::AbiDecodedErrorType::Unknown(_)
            ))
        ));
    }

    #[tokio::test]
    async fn test_batch_quote_invalid_rpc_url_errors() {
        let quote_targets = vec![QuoteTarget::default()];
        let err = batch_quote(
            &quote_targets,
            vec!["this should break".to_string()],
            None,
            Address::ZERO,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            Error::UrlParseError(url::ParseError::RelativeUrlWithoutBase)
        ));
    }

    #[tokio::test]
    async fn test_batch_quote_counterparty_is_sent_as_from() {
        let rpc_server = MockServer::start_async().await;
        let cp = "0xaaaaaaaaaabbbbbbbbbbccccccccccdddddddddd"
            .parse::<Address>()
            .unwrap();

        let one = Float::parse("1".to_string()).unwrap();
        let two = Float::parse("2".to_string()).unwrap();
        let inner = vec![encode_quote2_return_bytes(true, one, two)];
        let ret_hex = encode_multicall_return(inner);

        // alloy serialises `from` lowercased in JSON-RPC params; asserting the
        // body contains the hex-encoded counterparty verifies threading.
        let expected_from = format!("{cp:#x}");
        let mock = rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc").body_contains(&expected_from);
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": ret_hex,
            }));
        });

        let quote_targets = vec![QuoteTarget::default()];
        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            cp,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].is_ok());
        mock.assert();
    }

    // When every target in a chunk reverts with the same error, the
    // bisection isolates each singleton and attributes the revert per target.
    #[tokio::test]
    async fn test_batch_quote_chunk_uniform_revert_bisects_to_singletons() {
        let rpc_server = MockServer::start_async().await;

        // Every eth_call to this orderbook reverts with TokenSelfTrade.
        mock_eth_call_revert(&rpc_server, "/rpc", "0x734bc71c");

        struct FakeRegistry;
        #[async_trait::async_trait]
        impl ErrorRegistry for FakeRegistry {
            async fn lookup(
                &self,
                selector: [u8; 4],
            ) -> Result<Vec<AlloyError>, rain_error_decoding::AbiDecodeFailedErrors> {
                if selector == [0x73, 0x4b, 0xc7, 0x1c] {
                    Ok(vec!["TokenSelfTrade()".parse().unwrap()])
                } else {
                    Ok(vec![])
                }
            }
        }

        let quote_targets = vec![QuoteTarget::default(), QuoteTarget::default()];
        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            Address::ZERO,
            Some(&FakeRegistry),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 2);
        for r in &result {
            assert!(matches!(
                r,
                Err(FailedQuote::RevertError(
                    rain_error_decoding::AbiDecodedErrorType::Known { name, .. }
                )) if name == "TokenSelfTrade"
            ));
        }
    }
}
