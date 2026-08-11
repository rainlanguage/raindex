use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::sol_types::SolValue;
use futures::{stream, StreamExt};
use once_cell::sync::{Lazy, OnceCell};
use raindex_bindings::IRaindexV6::{OrderV4, SignedContextV1};
use raindex_subgraph_client::types::common::SgOrder;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    future::Future,
    num::NonZeroUsize,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};
use tokio::sync::Semaphore;
use url::Url;

/// Default maximum number of concurrent oracle HTTP requests per caller and
/// oracle origin.
///
/// Eight allows meaningful overlap for quote pages while keeping pressure on
/// each oracle service conservative. Native clients share each origin's limit
/// across the process; WASM clients share it within one module instance.
pub(crate) const ORACLE_REQUEST_CONCURRENCY_LIMIT: usize = 8;
const ORACLE_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

static ORACLE_HTTP_CLIENT: OnceCell<Client> = OnceCell::new();
type OracleRequestSemaphores = Mutex<HashMap<String, Weak<Semaphore>>>;
static ORACLE_REQUEST_SEMAPHORES: Lazy<Arc<OracleRequestSemaphores>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Validate that an oracle URL is safe to POST to.
/// Only http and https schemes are allowed to prevent SSRF.
fn validate_oracle_url(url: &str) -> Result<Url, OracleError> {
    let parsed =
        Url::parse(url).map_err(|e| OracleError::InvalidUrl(format!("Cannot parse URL: {e}")))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        scheme => Err(OracleError::InvalidUrl(format!(
            "Unsupported scheme '{scheme}', only http and https are allowed"
        ))),
    }
}

/// Error types for oracle fetching
#[derive(Debug, thiserror::Error)]
pub enum OracleError {
    #[error("HTTP request to {endpoint} failed: {source}")]
    RequestFailed {
        endpoint: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Failed to build oracle HTTP client: {0}")]
    ClientBuildFailed(#[source] reqwest::Error),

    #[error("Invalid oracle URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid oracle response: {0}")]
    InvalidResponse(String),
}

fn request_failed(endpoint: &str, source: reqwest::Error) -> OracleError {
    OracleError::RequestFailed {
        endpoint: endpoint.to_owned(),
        source: source.without_url(),
    }
}

fn sanitized_endpoint(url: &Url) -> String {
    let mut endpoint = url.clone();
    let _ = endpoint.set_username("");
    let _ = endpoint.set_password(None);
    endpoint.set_query(None);
    endpoint.set_fragment(None);
    endpoint.into()
}

/// JSON response format from an oracle endpoint.
/// Maps directly to `SignedContextV1` in the raindex contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    /// The signer address (EIP-191 signer of the context data)
    pub signer: Address,
    /// The signed context data as bytes32[] values
    pub context: Vec<FixedBytes<32>>,
    /// The EIP-191 signature over keccak256(abi.encodePacked(context))
    pub signature: Bytes,
}

impl From<OracleResponse> for SignedContextV1 {
    fn from(resp: OracleResponse) -> Self {
        SignedContextV1 {
            signer: resp.signer,
            context: resp.context,
            signature: resp.signature,
        }
    }
}

/// A reusable oracle HTTP client backed by a shared connection pool and
/// per-origin concurrency limit.
#[derive(Clone)]
pub(crate) struct OracleClient {
    client: Client,
    request_semaphores: Arc<OracleRequestSemaphores>,
    // Consumed by the concurrent quote sweep in the immediately upstack PR.
    #[allow(dead_code)]
    concurrency_limit: NonZeroUsize,
}

/// One independent oracle POST prepared for bounded execution.
// Consumed by the concurrent quote sweep in the immediately upstack PR.
#[allow(dead_code)]
pub(crate) struct OracleRequest {
    url: Url,
    body: Vec<u8>,
}

#[allow(dead_code)]
impl OracleRequest {
    pub(crate) fn new(url: String, body: Vec<u8>) -> Result<Self, OracleError> {
        Ok(Self {
            url: validate_oracle_url(&url)?,
            body,
        })
    }
}

impl OracleClient {
    /// Build an oracle client with the default caller concurrency limit.
    pub(crate) fn new() -> Result<Self, OracleError> {
        Self::with_concurrency_limit(
            NonZeroUsize::new(ORACLE_REQUEST_CONCURRENCY_LIMIT)
                .expect("the default oracle request concurrency limit is non-zero"),
        )
    }

    /// Build an oracle client with a caller-specific concurrency limit.
    ///
    /// Requests also share the default per-origin ceiling with other clients.
    pub(crate) fn with_concurrency_limit(
        concurrency_limit: NonZeroUsize,
    ) -> Result<Self, OracleError> {
        let client = ORACLE_HTTP_CLIENT
            .get_or_try_init(build_http_client)
            .map_err(OracleError::ClientBuildFailed)?;
        Ok(Self {
            client: client.clone(),
            request_semaphores: ORACLE_REQUEST_SEMAPHORES.clone(),
            concurrency_limit,
        })
    }

    fn request_semaphore(&self, url: &Url) -> Arc<Semaphore> {
        let origin = url.origin().ascii_serialization();
        let mut semaphores = self
            .request_semaphores
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        semaphores.retain(|_, semaphore| semaphore.strong_count() > 0);

        semaphores
            .get(&origin)
            .and_then(Weak::upgrade)
            .unwrap_or_else(|| {
                let semaphore = Arc::new(Semaphore::new(ORACLE_REQUEST_CONCURRENCY_LIMIT));
                semaphores.insert(origin, Arc::downgrade(&semaphore));
                semaphore
            })
    }

    async fn with_request_permit<F, T>(&self, url: &Url, request: F) -> T
    where
        F: Future<Output = T>,
    {
        let request_semaphore = self.request_semaphore(url);
        let _permit = request_semaphore
            .acquire()
            .await
            .expect("oracle request semaphores are never closed");
        request.await
    }

    async fn fetch_responses(
        &self,
        url: &Url,
        body: Vec<u8>,
    ) -> Result<Vec<OracleResponse>, OracleError> {
        let endpoint = sanitized_endpoint(url);
        self.with_request_permit(url, async {
            let response = self
                .client
                .post(url.clone())
                .header("Content-Type", "application/octet-stream")
                .body(body)
                .timeout(ORACLE_REQUEST_TIMEOUT)
                .send()
                .await
                .map_err(|source| request_failed(&endpoint, source))?
                .error_for_status()
                .map_err(|source| request_failed(&endpoint, source))?;

            response
                .json()
                .await
                .map_err(|source| request_failed(&endpoint, source))
        })
        .await
    }

    /// Fetch signed context for one pair.
    pub(crate) async fn fetch_signed_context(
        &self,
        url: &str,
        body: Vec<u8>,
    ) -> Result<SignedContextV1, OracleError> {
        let url = validate_oracle_url(url)?;
        self.fetch_signed_context_at(&url, body).await
    }

    async fn fetch_signed_context_at(
        &self,
        url: &Url,
        body: Vec<u8>,
    ) -> Result<SignedContextV1, OracleError> {
        let response = self.fetch_responses(url, body).await?;

        let [response]: [OracleResponse; 1] =
            response
                .try_into()
                .map_err(|response: Vec<OracleResponse>| {
                    OracleError::InvalidResponse(format!(
                        "Expected 1 response, got {}",
                        response.len()
                    ))
                })?;
        Ok(response.into())
    }

    /// Fetch signed contexts for several independent pair requests.
    ///
    /// Results remain aligned with `requests`; a failed request is returned in
    /// only its own slot. Requests run through the caller and per-origin
    /// concurrency limits.
    // Consumed by the concurrent quote sweep in the immediately upstack PR.
    #[allow(dead_code)]
    pub(crate) async fn fetch_signed_contexts(
        &self,
        requests: Vec<OracleRequest>,
    ) -> Vec<Result<SignedContextV1, OracleError>> {
        let requests = requests.into_iter().map(|request| async move {
            self.fetch_signed_context_at(&request.url, request.body)
                .await
        });

        collect_bounded(requests, self.concurrency_limit).await
    }

    /// Fetch signed context for a batch body supported by a known-compatible
    /// endpoint.
    pub(crate) async fn fetch_signed_context_batch(
        &self,
        url: &str,
        body: Vec<u8>,
        expected_count: usize,
    ) -> Result<Vec<SignedContextV1>, OracleError> {
        let url = validate_oracle_url(url)?;
        let response = self.fetch_responses(&url, body).await?;

        if response.len() != expected_count {
            return Err(OracleError::InvalidResponse(format!(
                "Expected {} oracle responses, got {}",
                expected_count,
                response.len()
            )));
        }

        Ok(response.into_iter().map(Into::into).collect())
    }
}

fn build_http_client() -> Result<Client, reqwest::Error> {
    Client::builder().build()
}

// Consumed by the concurrent quote sweep in the immediately upstack PR.
#[allow(dead_code)]
async fn collect_bounded<F, T>(futures: impl IntoIterator<Item = F>, limit: NonZeroUsize) -> Vec<T>
where
    F: Future<Output = T>,
{
    let mut results = stream::iter(
        futures
            .into_iter()
            .enumerate()
            .map(|(index, future)| async move { (index, future.await) }),
    )
    .buffer_unordered(limit.get())
    .collect::<Vec<_>>()
    .await;
    results.sort_unstable_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

/// Encode the POST body for a single oracle request.
///
/// The body is `abi.encode(OrderV4, uint256 inputIOIndex, uint256 outputIOIndex, address counterparty)`.
pub fn encode_oracle_body(
    order: &OrderV4,
    input_io_index: u32,
    output_io_index: u32,
    counterparty: Address,
) -> Vec<u8> {
    (
        order.clone(),
        U256::from(input_io_index),
        U256::from(output_io_index),
        counterparty,
    )
        .abi_encode()
}

/// Encode the POST body for a batch oracle request.
///
/// The body is `abi.encode((OrderV4, uint256 inputIOIndex, uint256 outputIOIndex, address counterparty)[])`.
pub fn encode_oracle_body_batch(requests: Vec<(&OrderV4, u32, u32, Address)>) -> Vec<u8> {
    let tuples: Vec<_> = requests
        .into_iter()
        .map(|(order, input_io_index, output_io_index, counterparty)| {
            (
                order.clone(),
                U256::from(input_io_index),
                U256::from(output_io_index),
                counterparty,
            )
        })
        .collect();

    tuples.abi_encode()
}

/// Fetch signed context from an oracle endpoint via POST (single request).
///
/// The endpoint receives an ABI-encoded body containing the order details
/// that will be used for calculateOrderIO:
/// `abi.encode(OrderV4, uint256 inputIOIndex, uint256 outputIOIndex, address counterparty)`
///
/// The endpoint must respond with a JSON array containing exactly one `OracleResponse`.
pub async fn fetch_signed_context(
    url: &str,
    body: Vec<u8>,
) -> Result<SignedContextV1, OracleError> {
    OracleClient::new()?.fetch_signed_context(url, body).await
}

/// Fetch signed context from an oracle endpoint via POST (batch request).
///
/// The endpoint receives an ABI-encoded body containing an array of order details:
/// `abi.encode((OrderV4, uint256 inputIOIndex, uint256 outputIOIndex, address counterparty)[])`
///
/// The endpoint must respond with a JSON array of `OracleResponse` objects.
/// The response array length must match the request array length.
pub async fn fetch_signed_context_batch(
    url: &str,
    body: Vec<u8>,
    expected_count: usize,
) -> Result<Vec<SignedContextV1>, OracleError> {
    OracleClient::new()?
        .fetch_signed_context_batch(url, body, expected_count)
        .await
}

/// Extract the oracle URL from an SgOrder's meta, if present.
///
/// Parses the meta bytes and looks for a `RaindexSignedContextOracleV1` entry.
/// Returns `None` if meta is absent, unparseable, or doesn't contain an oracle entry.
pub fn extract_oracle_url(order: &SgOrder) -> Option<String> {
    use rain_metadata::types::raindex_signed_context_oracle::RaindexSignedContextOracleV1;
    use rain_metadata::RainMetaDocumentV1Item;

    let meta = order.meta.as_ref()?;
    let decoded = alloy::hex::decode(&meta.0).ok()?;
    let items = RainMetaDocumentV1Item::cbor_decode(&decoded).ok()?;
    let oracle = RaindexSignedContextOracleV1::find_in_items(&items).ok()??;
    Some(oracle.url().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, FixedBytes};
    #[cfg(not(target_family = "wasm"))]
    use httpmock::{Method::POST, MockServer};
    use raindex_bindings::IRaindexV6::{EvaluableV4, OrderV4, IOV2};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn test_oracle_response_to_signed_context() {
        let ctx_val = FixedBytes::<32>::from([0x2a; 32]);
        let response = OracleResponse {
            signer: address!("0x1234567890123456789012345678901234567890"),
            context: vec![ctx_val],
            signature: Bytes::from(vec![0xaa, 0xbb, 0xcc]),
        };

        let signed: SignedContextV1 = response.into();
        assert_eq!(
            signed.signer,
            address!("0x1234567890123456789012345678901234567890")
        );
        assert_eq!(signed.context.len(), 1);
        assert_eq!(signed.context[0], ctx_val);
        assert_eq!(signed.signature, Bytes::from(vec![0xaa, 0xbb, 0xcc]));
    }

    #[test]
    fn test_encode_oracle_body_single() {
        let order = create_test_order();
        let body = encode_oracle_body(
            &order,
            1,
            2,
            address!("0x1111111111111111111111111111111111111111"),
        );
        assert!(!body.is_empty());
    }

    #[test]
    fn test_encode_oracle_body_batch() {
        let order1 = create_test_order();
        let order2 = create_test_order();

        let requests = vec![
            (
                &order1,
                1,
                2,
                address!("0x1111111111111111111111111111111111111111"),
            ),
            (
                &order2,
                3,
                4,
                address!("0x2222222222222222222222222222222222222222"),
            ),
        ];

        let body = encode_oracle_body_batch(requests);
        assert!(!body.is_empty());

        // Batch encoding should be different from single encoding
        let single_body = encode_oracle_body(
            &order1,
            1,
            2,
            address!("0x1111111111111111111111111111111111111111"),
        );
        assert_ne!(body, single_body);
    }

    #[tokio::test]
    async fn test_fetch_signed_context_invalid_url() {
        let result = fetch_signed_context("not-a-url", vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_signed_context_unreachable() {
        let result = fetch_signed_context("http://127.0.0.1:1/oracle", vec![]).await;
        assert!(result.is_err());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_fetch_signed_context_error_redacts_url() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/oracle");
                then.status(503);
            })
            .await;
        let secret = "do-not-log-this-api-key";
        let url = format!("{}?api_key={secret}", server.url("/oracle"));

        let error = fetch_signed_context(&url, vec![]).await.unwrap_err();
        let message = error.to_string();
        let parsed_url = Url::parse(&url).unwrap();

        assert!(message.contains("503"));
        assert!(message.contains(parsed_url.host_str().unwrap()));
        assert!(message.contains("/oracle"));
        assert!(!message.contains(secret));
        assert!(!message.contains(&url));
        mock.assert_async().await;
    }

    #[test]
    fn test_oracle_request_validates_url_at_construction() {
        assert!(OracleRequest::new("not-a-url".to_owned(), vec![]).is_err());
        assert!(
            OracleRequest::new("https://oracle.example.com/context".to_owned(), vec![]).is_ok()
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_fetch_signed_contexts_preserves_request_positions_and_failures() {
        let server = MockServer::start_async().await;
        let first_response = OracleResponse {
            signer: address!("0x1111111111111111111111111111111111111111"),
            context: vec![FixedBytes::with_last_byte(1)],
            signature: vec![0xaa].into(),
        };
        let third_response = OracleResponse {
            signer: address!("0x3333333333333333333333333333333333333333"),
            context: vec![FixedBytes::with_last_byte(3)],
            signature: vec![0xcc].into(),
        };
        let first_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/first");
                then.status(200)
                    .json_body_obj(&vec![first_response.clone()]);
            })
            .await;
        let failing_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/failing");
                then.status(503);
            })
            .await;
        let third_mock = server
            .mock_async(|when, then| {
                when.method(POST).path("/third");
                then.status(200)
                    .json_body_obj(&vec![third_response.clone()]);
            })
            .await;
        let requests = ["/first", "/failing", "/third"]
            .into_iter()
            .map(|path| OracleRequest::new(server.url(path), vec![]).unwrap())
            .collect();

        let results = OracleClient::new()
            .unwrap()
            .fetch_signed_contexts(requests)
            .await;

        assert_eq!(results.len(), 3);
        assert_eq!(
            results[0].as_ref().unwrap().signer,
            address!("0x1111111111111111111111111111111111111111")
        );
        assert!(results[1].is_err());
        assert_eq!(
            results[2].as_ref().unwrap().signer,
            address!("0x3333333333333333333333333333333333333333")
        );
        first_mock.assert_async().await;
        failing_mock.assert_async().await;
        third_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_fetch_signed_context_batch_invalid_url() {
        let result = fetch_signed_context_batch("not-a-url", vec![], 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_signed_context_batch_unreachable() {
        let result = fetch_signed_context_batch("http://127.0.0.1:1/oracle", vec![], 0).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_oracle_url_rejects_non_http() {
        assert!(validate_oracle_url("ftp://example.com").is_err());
        assert!(validate_oracle_url("javascript:alert(1)").is_err());
        assert!(validate_oracle_url("file:///etc/passwd").is_err());
        assert!(validate_oracle_url("data:text/html,<h1>hi</h1>").is_err());
    }

    #[test]
    fn test_validate_oracle_url_accepts_http() {
        assert!(validate_oracle_url("http://localhost:8080/oracle").is_ok());
        assert!(validate_oracle_url("https://oracle.example.com/context").is_ok());
    }

    #[tokio::test]
    async fn test_collect_bounded_preserves_positions_and_limits_concurrency() {
        const REQUEST_COUNT: usize = 17;
        const TEST_LIMIT: usize = 3;

        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(Semaphore::new(0));
        let requests = (0..REQUEST_COUNT).map(|index| {
            let active = active.clone();
            let maximum = maximum.clone();
            let started = started.clone();
            let gate = gate.clone();
            async move {
                let active_now = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum.fetch_max(active_now, Ordering::SeqCst);
                let started_now = started.fetch_add(1, Ordering::SeqCst) + 1;
                if started_now == TEST_LIMIT {
                    gate.add_permits(REQUEST_COUNT);
                }

                let _permit = gate.acquire().await.unwrap();
                active.fetch_sub(1, Ordering::SeqCst);

                if index == 5 {
                    Err("expected failure")
                } else {
                    Ok(index)
                }
            }
        });

        let results = collect_bounded(requests, NonZeroUsize::new(TEST_LIMIT).unwrap()).await;

        assert_eq!(maximum.load(Ordering::SeqCst), TEST_LIMIT);
        assert_eq!(started.load(Ordering::SeqCst), REQUEST_COUNT);
        assert_eq!(results.len(), REQUEST_COUNT);
        assert_eq!(results[0], Ok(0));
        assert_eq!(results[5], Err("expected failure"));
        assert_eq!(results[6], Ok(6));
        assert_eq!(results[16], Ok(16));
    }

    #[tokio::test]
    async fn test_collect_bounded_does_not_block_behind_slow_first_request() {
        const REQUEST_COUNT: usize = 8;
        const TEST_LIMIT: usize = 3;

        let slow_request_gate = Arc::new(Semaphore::new(0));
        let later_request_started = Arc::new(Semaphore::new(0));
        let requests = (0..REQUEST_COUNT).map(|index| {
            let slow_request_gate = slow_request_gate.clone();
            let later_request_started = later_request_started.clone();
            async move {
                if index == 0 {
                    let _permit = slow_request_gate.acquire().await.unwrap();
                } else if index == TEST_LIMIT {
                    later_request_started.add_permits(1);
                }
                index
            }
        });
        let results = collect_bounded(requests, NonZeroUsize::new(TEST_LIMIT).unwrap());
        tokio::pin!(results);

        tokio::select! {
            permit = later_request_started.acquire() => {
                assert!(permit.is_ok(), "later request signal must remain open");
            }
            _ = &mut results => {
                panic!("all requests completed while the first request was still gated");
            }
        }

        slow_request_gate.add_permits(1);
        assert_eq!(results.await, (0..REQUEST_COUNT).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn test_oracle_clients_share_per_origin_request_limit() {
        const REQUEST_COUNT: usize = 17;

        let global_client = OracleClient::new().unwrap();
        let second_global_client = OracleClient::new().unwrap();
        assert!(Arc::ptr_eq(
            &global_client.request_semaphores,
            &second_global_client.request_semaphores
        ));

        let same_origin = Url::parse("https://oracle.example.com/first").unwrap();
        let same_origin_other_path = Url::parse("https://oracle.example.com/second").unwrap();
        let other_origin = Url::parse("https://other.example.com/oracle").unwrap();
        let first_semaphore = global_client.request_semaphore(&same_origin);
        let same_origin_semaphore = second_global_client.request_semaphore(&same_origin_other_path);
        let other_origin_semaphore = global_client.request_semaphore(&other_origin);
        assert!(Arc::ptr_eq(&first_semaphore, &same_origin_semaphore));
        assert!(!Arc::ptr_eq(&first_semaphore, &other_origin_semaphore));

        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(Semaphore::new(0));
        let requests = (0..REQUEST_COUNT).map(|index| {
            let client = if index % 2 == 0 {
                global_client.clone()
            } else {
                second_global_client.clone()
            };
            let same_origin = same_origin.clone();
            let active = active.clone();
            let maximum = maximum.clone();
            let started = started.clone();
            let gate = gate.clone();
            async move {
                client
                    .with_request_permit(&same_origin, async move {
                        let active_now = active.fetch_add(1, Ordering::SeqCst) + 1;
                        maximum.fetch_max(active_now, Ordering::SeqCst);
                        let started_now = started.fetch_add(1, Ordering::SeqCst) + 1;
                        if started_now == ORACLE_REQUEST_CONCURRENCY_LIMIT {
                            gate.add_permits(REQUEST_COUNT);
                        }

                        let _permit = gate.acquire().await.unwrap();
                        active.fetch_sub(1, Ordering::SeqCst);
                    })
                    .await;
            }
        });

        collect_bounded(requests, NonZeroUsize::new(REQUEST_COUNT).unwrap()).await;

        assert_eq!(
            maximum.load(Ordering::SeqCst),
            ORACLE_REQUEST_CONCURRENCY_LIMIT
        );
        assert_eq!(started.load(Ordering::SeqCst), REQUEST_COUNT);
    }

    #[test]
    fn test_oracle_client_accepts_caller_concurrency_limit() {
        let limit = NonZeroUsize::new(3).unwrap();
        let client = OracleClient::with_concurrency_limit(limit).unwrap();

        assert_eq!(client.concurrency_limit, limit);
    }

    fn create_test_order() -> OrderV4 {
        OrderV4 {
            owner: address!("0x0000000000000000000000000000000000000000"),
            evaluable: EvaluableV4 {
                interpreter: address!("0x0000000000000000000000000000000000000000"),
                store: address!("0x0000000000000000000000000000000000000000"),
                bytecode: Bytes::new(),
            },
            validInputs: vec![IOV2 {
                token: address!("0x0000000000000000000000000000000000000000"),
                vaultId: FixedBytes::<32>::ZERO,
            }],
            validOutputs: vec![IOV2 {
                token: address!("0x0000000000000000000000000000000000000000"),
                vaultId: FixedBytes::<32>::ZERO,
            }],
            nonce: FixedBytes::<32>::ZERO,
        }
    }
}
