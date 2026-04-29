use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::sol_types::SolValue;
use rain_orderbook_bindings::IRaindexV6::{OrderV4, SignedContextV1};
use rain_orderbook_subgraph_client::types::common::SgOrder;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

/// Validate that an oracle URL is safe to POST to.
/// Only http and https schemes are allowed to prevent SSRF.
fn validate_oracle_url(url: &str) -> Result<(), OracleError> {
    let parsed =
        Url::parse(url).map_err(|e| OracleError::InvalidUrl(format!("Cannot parse URL: {e}")))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(OracleError::InvalidUrl(format!(
            "Unsupported scheme '{scheme}', only http and https are allowed"
        ))),
    }
}

/// Error types for oracle fetching
#[derive(Debug, thiserror::Error)]
pub enum OracleError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid oracle URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid oracle response: {0}")]
    InvalidResponse(String),
}

/// JSON response format from an oracle endpoint.
/// Maps directly to `SignedContextV1` in the orderbook contract.
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
    validate_oracle_url(url)?;
    let builder = Client::builder();
    #[cfg(not(target_family = "wasm"))]
    let builder = builder.timeout(std::time::Duration::from_secs(10));
    let client = builder.build()?;

    // For single requests, we still expect a JSON array response but with one item
    let response: Vec<OracleResponse> = client
        .post(url)
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if response.len() != 1 {
        return Err(OracleError::InvalidResponse(format!(
            "Expected 1 response, got {}",
            response.len()
        )));
    }

    Ok(response.into_iter().next().unwrap().into())
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
    validate_oracle_url(url)?;
    let builder = Client::builder();
    #[cfg(not(target_family = "wasm"))]
    let builder = builder.timeout(std::time::Duration::from_secs(10));
    let client = builder.build()?;

    let response: Vec<OracleResponse> = client
        .post(url)
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if response.len() != expected_count {
        return Err(OracleError::InvalidResponse(format!(
            "Expected {} oracle responses, got {}",
            expected_count,
            response.len()
        )));
    }

    Ok(response.into_iter().map(|resp| resp.into()).collect())
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
    use rain_orderbook_bindings::IRaindexV6::{EvaluableV4, OrderV4, IOV2};

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
