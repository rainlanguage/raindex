use crate::{
    error::{Error, FailedQuote},
    quote::{QuoteResult, QuoteTarget},
};
use alloy::providers::{Failure, MulticallError, MulticallItem, Provider};
use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    primitives::Address,
};
use rain_error_decoding::{AbiDecodedErrorType, ErrorRegistry};
use rain_orderbook_bindings::provider::{mk_read_provider, ReadProvider};
use rain_orderbook_bindings::IRaindexV6::{quote2Call, quote2Return, IRaindexV6Instance};
use url::Url;

const DEFAULT_QUOTE_CHUNK_SIZE: usize = 16;

fn normalize_chunk_size(chunk_size: Option<usize>) -> usize {
    chunk_size.unwrap_or(DEFAULT_QUOTE_CHUNK_SIZE).max(1)
}

fn single_quote_failure(err: &Error) -> QuoteResult {
    Err(FailedQuote::CorruptReturnData(format!(
        "Single quote failed after chunk split: {err}"
    )))
}

fn probe_indexes(len: usize) -> Vec<usize> {
    let mut indexes = vec![0usize];
    let middle = len / 2;
    if middle != 0 {
        indexes.push(middle);
    }
    indexes
}

async fn quote_chunk_once(
    quote_targets: &[QuoteTarget],
    provider: &ReadProvider,
    block_number: Option<u64>,
    multicall_address: Option<Address>,
    registry: Option<&dyn ErrorRegistry>,
) -> Result<Vec<QuoteResult>, Error> {
    let mut multicall = if let Some(addr) = multicall_address {
        provider.multicall().address(addr).dynamic::<quote2Call>()
    } else {
        provider.multicall().dynamic::<quote2Call>()
    };

    if let Some(block_number) = block_number {
        multicall = multicall.block(BlockId::Number(BlockNumberOrTag::Number(block_number)));
    }

    for quote_target in quote_targets {
        let ob_instance = IRaindexV6Instance::new(quote_target.orderbook, provider.clone());
        let call = ob_instance
            .quote2(quote_target.quote_config.clone())
            .into_call(true);
        multicall = multicall.add_call_dynamic(call);
    }

    let aggregate_res: Vec<Result<quote2Return, Failure>> = match multicall.aggregate3().await {
        Ok(results) => results,
        Err(MulticallError::CallFailed(bytes)) => {
            let decoded_error =
                match AbiDecodedErrorType::selector_registry_abi_decode(bytes.as_ref(), registry)
                    .await
                {
                    Ok(err) => FailedQuote::RevertError(err),
                    Err(err) => FailedQuote::RevertErrorDecodeFailed(err),
                };
            return Ok((0..quote_targets.len())
                .map(|_| match &decoded_error {
                    FailedQuote::RevertError(abi_err) => {
                        Err(FailedQuote::RevertError(abi_err.clone()))
                    }
                    FailedQuote::RevertErrorDecodeFailed(_) => Err(FailedQuote::CorruptReturnData(
                        "Multicall failed with non-decodable error".to_string(),
                    )),
                    _ => Err(FailedQuote::CorruptReturnData(
                        "Unexpected multicall failure".to_string(),
                    )),
                })
                .collect::<Vec<QuoteResult>>());
        }
        Err(err) => return Err(Error::MulticallError(err)),
    };

    let mut results: Vec<QuoteResult> = Vec::with_capacity(aggregate_res.len());
    for res in aggregate_res {
        match res {
            Ok(ret) => {
                if ret.exists {
                    results.push(Ok(ret.into()));
                } else {
                    results.push(Err(FailedQuote::NonExistent));
                }
            }
            Err(failure) => {
                match AbiDecodedErrorType::selector_registry_abi_decode(
                    &failure.return_data,
                    registry,
                )
                .await
                {
                    Ok(e) => results.push(Err(FailedQuote::RevertError(e))),
                    Err(e) => results.push(Err(FailedQuote::RevertErrorDecodeFailed(e))),
                }
            }
        }
    }

    Ok(results)
}

async fn rpc_probe_succeeds(
    quote_targets: &[QuoteTarget],
    provider: &ReadProvider,
    block_number: Option<u64>,
    multicall_address: Option<Address>,
    registry: Option<&dyn ErrorRegistry>,
) -> bool {
    for idx in probe_indexes(quote_targets.len()) {
        if quote_chunk_once(
            &quote_targets[idx..idx + 1],
            provider,
            block_number,
            multicall_address,
            registry,
        )
        .await
        .is_ok()
        {
            return true;
        }
    }

    false
}

async fn quote_chunk_with_probe_and_split(
    quote_targets: &[QuoteTarget],
    provider: &ReadProvider,
    block_number: Option<u64>,
    multicall_address: Option<Address>,
    registry: Option<&dyn ErrorRegistry>,
) -> Result<Vec<QuoteResult>, Error> {
    let initial_err = match quote_chunk_once(
        quote_targets,
        provider,
        block_number,
        multicall_address,
        registry,
    )
    .await
    {
        Ok(results) => return Ok(results),
        Err(err) => err,
    };

    if quote_targets.len() <= 1 {
        return Err(initial_err);
    }

    if !rpc_probe_succeeds(
        quote_targets,
        provider,
        block_number,
        multicall_address,
        registry,
    )
    .await
    {
        return Err(initial_err);
    }

    let mut resolved: Vec<Option<QuoteResult>> = Vec::with_capacity(quote_targets.len());
    resolved.resize_with(quote_targets.len(), || None);
    let mut pending = vec![(0usize, quote_targets.len())];

    while let Some((start, end)) = pending.pop() {
        let chunk = &quote_targets[start..end];
        match quote_chunk_once(chunk, provider, block_number, multicall_address, registry).await {
            Ok(results) => {
                for (offset, result) in results.into_iter().enumerate() {
                    resolved[start + offset] = Some(result);
                }
            }
            Err(err) => {
                if chunk.len() == 1 {
                    resolved[start] = Some(single_quote_failure(&err));
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

/// Quotes array of given quote targets using the given rpc url
pub async fn batch_quote(
    quote_targets: &[QuoteTarget],
    rpcs: Vec<String>,
    block_number: Option<u64>,
    multicall_address: Option<Address>,
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

    let mut results = Vec::with_capacity(quote_targets.len());
    let chunk_size = normalize_chunk_size(chunk_size);
    for quote_chunk in quote_targets.chunks(chunk_size) {
        let chunk_results = quote_chunk_with_probe_and_split(
            quote_chunk,
            &provider,
            block_number,
            multicall_address,
            registry,
        )
        .await?;
        results.extend(chunk_results);
    }

    Ok(results)
}

#[cfg(not(target_family = "wasm"))]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy::json_abi::Error as AlloyError;
    use alloy::providers::bindings::IMulticall3::Result as MulticallResult;
    use alloy::providers::MulticallError;
    use alloy::sol_types::SolCall;
    use alloy::sol_types::SolValue;
    use alloy::transports::TransportError;
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

    #[test]
    fn test_probe_indexes_singleton_only_first() {
        assert_eq!(probe_indexes(1), vec![0]);
    }

    #[test]
    fn test_probe_indexes_uses_first_and_middle() {
        assert_eq!(probe_indexes(6), vec![0, 3]);
    }

    #[tokio::test]
    async fn test_batch_quote_ok() {
        let rpc_server = MockServer::start_async().await;

        let zero = Float::parse("0".to_string()).unwrap();
        let one = Float::parse("1".to_string()).unwrap();
        let two = Float::parse("2".to_string()).unwrap();

        // build call data
        let quote_targets = vec![
            QuoteTarget::default(),
            QuoteTarget::default(),
            QuoteTarget::default(),
        ];

        // build response data
        let response_data = vec![
            MulticallResult {
                success: true,
                returnData: quote2Call::abi_encode_returns(&quote2Return {
                    exists: true,
                    outputMax: one.get_inner(),
                    ioRatio: two.get_inner(),
                })
                .into(),
            },
            MulticallResult {
                success: true,
                returnData: quote2Call::abi_encode_returns(&quote2Return {
                    exists: false,
                    outputMax: zero.get_inner(),
                    ioRatio: zero.get_inner(),
                })
                .into(),
            },
            MulticallResult {
                success: false,
                returnData: vec![].into(),
            },
        ]
        .abi_encode();

        // mock rpc with call data and response data
        rpc_server.mock(|when, then| {
            when.method(POST).path("/");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": alloy::hex::encode_prefixed(response_data).as_str(),
            }));
        });

        let result = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/").to_string()],
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 3);

        assert!(result[0].as_ref().unwrap().max_output.eq(one).unwrap());
        assert!(result[0].as_ref().unwrap().ratio.eq(two).unwrap());

        assert!(matches!(result[1], Err(FailedQuote::NonExistent)));
        assert!(matches!(
            result[2],
            Err(FailedQuote::RevertErrorDecodeFailed(_))
        ));
    }

    #[tokio::test]
    async fn test_batch_quote_err() {
        let rpc_server = MockServer::start_async().await;
        let quote_targets = vec![QuoteTarget::default()];

        let err = batch_quote(
            &quote_targets,
            vec!["this should break".to_string()],
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert!(
            matches!(
                err,
                Error::UrlParseError(url::ParseError::RelativeUrlWithoutBase)
            ),
            "unexpected error: {err:?}"
        );

        rpc_server.mock(|when, then| {
            when.path("/rpc");
            then.status(500).json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32000,
                    "message": "Internal error"
                }
            }));
        });

        let err = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap_err();

        assert!(
            matches!(
                err,
                Error::MulticallError(MulticallError::TransportError(TransportError::Transport(_)))
            ),
            "unexpected error: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_batch_quote_handles_individual_call_failures_with_rain_error() {
        let rpc_server = MockServer::start_async().await;
        let quote_targets = vec![QuoteTarget::default(), QuoteTarget::default()];

        let response_data = vec![
            MulticallResult {
                success: false,
                returnData: alloy::hex!("734bc71c").to_vec().into(), // TokenSelfTrade error selector
            },
            MulticallResult {
                success: false,
                returnData: alloy::hex!("ff00ff00").to_vec().into(), // Unknown error selector
            },
        ]
        .abi_encode();

        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": alloy::hex::encode_prefixed(response_data),
            }));
        });

        struct FakeRegistry;

        #[async_trait::async_trait]
        impl ErrorRegistry for FakeRegistry {
            async fn lookup(
                &self,
                selector: [u8; 4],
            ) -> Result<Vec<AlloyError>, rain_error_decoding::AbiDecodeFailedErrors> {
                // 0x734bc71c -> "TokenSelfTrade()"
                if selector == [0x73, 0x4b, 0xc7, 0x1c] {
                    Ok(vec!["TokenSelfTrade()".parse().unwrap()])
                } else {
                    Ok(vec![]) // keep 0xdeadbeef unknown
                }
            }
        }

        let results = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            None,
            Some(&FakeRegistry),
            None,
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);

        assert!(matches!(
            &results[0],
            Err(FailedQuote::RevertError(rain_error_decoding::AbiDecodedErrorType::Known { name, .. }))
            if name == "TokenSelfTrade"
        ));

        assert!(matches!(
            &results[1],
            Err(FailedQuote::RevertError(
                rain_error_decoding::AbiDecodedErrorType::Unknown(_)
            ))
        ));
    }

    #[tokio::test]
    async fn test_batch_quote_respects_chunk_size_override() {
        let rpc_server = MockServer::start_async().await;
        let one = Float::parse("1".to_string()).unwrap();
        let two = Float::parse("2".to_string()).unwrap();
        let quote_targets = vec![QuoteTarget::default(), QuoteTarget::default()];

        let single_response_data = vec![MulticallResult {
            success: true,
            returnData: quote2Call::abi_encode_returns(&quote2Return {
                exists: true,
                outputMax: one.get_inner(),
                ioRatio: two.get_inner(),
            })
            .into(),
        }]
        .abi_encode();

        rpc_server.mock(|when, then| {
            when.method(POST).path("/rpc");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": alloy::hex::encode_prefixed(single_response_data),
            }));
        });

        let results = batch_quote(
            &quote_targets,
            vec![rpc_server.url("/rpc").to_string()],
            None,
            None,
            None,
            Some(1),
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
