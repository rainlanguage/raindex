use crate::erc20::{Error as TokenError, TokenInfo, ERC20};
use crate::local_db::{FetchConfig, LocalDbError};
use crate::retry::{retry_with_backoff, RetryError};
use crate::rpc_client::RpcClient;
use alloy::primitives::Address;
use futures::StreamExt;

fn should_retry_token_error(err: &TokenError) -> bool {
    matches!(
        err,
        TokenError::ReadProviderError(_)
            | TokenError::ContractCallError(_)
            | TokenError::MulticallError(_)
    )
}

pub async fn fetch_erc20_metadata_concurrent(
    rpc_client: &RpcClient,
    missing_addrs: Vec<Address>,
    config: &FetchConfig,
) -> Result<Vec<(Address, TokenInfo)>, LocalDbError> {
    let concurrency = config.max_concurrent_requests();
    let max_attempts = config.max_retry_attempts();
    let retry_delay = config.retry_delay_ms();

    let results: Vec<Result<(Address, TokenInfo), LocalDbError>> =
        futures::stream::iter(missing_addrs.into_iter().map(|addr| async move {
            let erc20 = ERC20::new(rpc_client.rpc_urls().to_vec(), addr);
            let result = retry_with_backoff(
                || {
                    let erc20 = erc20.clone();
                    async move { erc20.token_info(None).await }
                },
                max_attempts,
                retry_delay,
                0,
                should_retry_token_error,
                |_| false,
            )
            .await
            .map_err(|e| match e {
                RetryError::InvalidMaxAttempts => LocalDbError::InvalidRetryMaxAttemps,
                RetryError::Operation(inner) => LocalDbError::ERC20Error(inner),
            })?;
            Ok((addr, result))
        }))
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let mut successes: Vec<(Address, TokenInfo)> = Vec::new();
    for r in results {
        match r {
            Ok(pair) => successes.push(pair),
            Err(e) => return Err(e),
        }
    }
    Ok(successes)
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use crate::local_db::token_fetch::fetch_erc20_metadata_concurrent;
        use crate::local_db::{FetchConfig, LocalDbError};
        use crate::rpc_client::RpcClient;
        use alloy::primitives::Address;
        use rain_orderbook_test_fixtures::LocalEvm;
        use url::Url;

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn test_fetch_erc20_metadata_concurrent_success() {
            let local_evm = LocalEvm::new_with_tokens(1).await;
            let rpcs = vec![Url::parse(&local_evm.url()).unwrap()];
            let token = local_evm.tokens[0].clone();
            let addrs = vec![*token.address()];

            let rpc_client = RpcClient::new_with_urls(rpcs).unwrap();
            let out = fetch_erc20_metadata_concurrent(&rpc_client, addrs, &FetchConfig::default())
                .await
                .unwrap();
            assert_eq!(out.len(), 1);
            assert_eq!(out[0].1.decimals, 18);
            assert_eq!(out[0].1.name, "Token1");
            assert_eq!(out[0].1.symbol, "TOKEN1");
        }

        #[tokio::test]
        async fn test_fetch_erc20_metadata_concurrent_failure_retries() {
            let rpcs = vec![Url::parse("http://127.0.0.1:1").unwrap()];
            let addrs = vec![Address::ZERO];

            let rpc_client = RpcClient::new_with_urls(rpcs).unwrap();
            let res =
                fetch_erc20_metadata_concurrent(&rpc_client, addrs, &FetchConfig::default()).await;
            assert!(res.is_err());
            match res.err().unwrap() {
                LocalDbError::ERC20Error(_) => {}
                other => panic!("Expected ERC20Error, got {other:?}"),
            }
        }
    }
}
