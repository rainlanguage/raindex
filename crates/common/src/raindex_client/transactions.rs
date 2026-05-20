use std::str::FromStr;

use super::*;
use crate::local_db::RaindexIdentifier;
use crate::raindex_client::local_db::transactions::LocalDbTransactions;
use crate::raindex_client::QuerySource;
use alloy::primitives::{Address, B256, U256};
#[cfg(target_family = "wasm")]
use gloo_timers::future::TimeoutFuture;
use raindex_subgraph_client::types::{common::SgTransaction, Id};
use raindex_subgraph_client::RaindexSubgraphClientError;
use serde::{Deserialize, Serialize};
#[cfg(not(target_family = "wasm"))]
use std::time::Duration;
#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

const DEFAULT_TRANSACTION_POLL_ATTEMPTS: usize = 10;
const DEFAULT_TRANSACTION_POLL_INTERVAL_MS: u64 = 1_000;

#[cfg(target_family = "wasm")]
async fn sleep_ms(ms: u64) {
    let delay = ms.min(u32::MAX as u64) as u32;
    TimeoutFuture::new(delay).await;
}

#[cfg(not(target_family = "wasm"))]
async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexTransaction {
    id: B256,
    from: Address,
    block_number: U256,
    timestamp: U256,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexTransaction {
    #[wasm_bindgen(getter, unchecked_return_type = "Hex")]
    pub fn id(&self) -> String {
        self.id.to_string()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn from(&self) -> String {
        self.from.to_string()
    }
    #[wasm_bindgen(getter = blockNumber)]
    pub fn block_number(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.block_number.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.timestamp.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexTransaction {
    pub fn id(&self) -> B256 {
        self.id
    }
    pub fn from(&self) -> Address {
        self.from
    }
    pub fn block_number(&self) -> U256 {
        self.block_number
    }
    pub fn timestamp(&self) -> U256 {
        self.timestamp
    }
}

impl RaindexTransaction {
    pub(crate) fn from_local_parts(
        tx_hash: B256,
        from: Address,
        block_number: u64,
        timestamp: u64,
    ) -> Result<Self, RaindexError> {
        Ok(Self {
            id: tx_hash,
            from,
            block_number: U256::from(block_number),
            timestamp: U256::from(timestamp),
        })
    }
}

#[wasm_export]
impl RaindexClient {
    /// Fetches transaction details for a given transaction hash
    ///
    /// Retrieves basic transaction information including sender, block number,
    /// and timestamp. Polls either local DB or subgraph based on the query source.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getTransaction(
    ///   1,
    ///   "0x1234567890123456789012345678901234567890",
    ///   "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    /// );
    /// if (result.error) {
    ///   console.error("Transaction not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const transaction = result.value;
    /// // Do something with the transaction
    /// ```
    #[wasm_export(
        js_name = "getTransaction",
        return_description = "Transaction details",
        unchecked_return_type = "RaindexTransaction"
    )]
    pub async fn get_transaction_wasm_binding(
        &self,
        #[wasm_export(js_name = "chainId", param_description = "Chain ID for the network")]
        chain_id: u32,
        #[wasm_export(
            js_name = "raindexAddress",
            param_description = "Raindex contract address",
            unchecked_param_type = "Address"
        )]
        raindex_address: String,
        #[wasm_export(
            js_name = "txHash",
            param_description = "Transaction hash",
            unchecked_param_type = "Hex"
        )]
        tx_hash: String,
        #[wasm_export(
            js_name = "maxAttempts",
            param_description = "Optional maximum polling attempts before timing out"
        )]
        max_attempts: Option<u32>,
        #[wasm_export(
            js_name = "intervalMs",
            param_description = "Optional polling interval in milliseconds"
        )]
        interval_ms: Option<u32>,
    ) -> Result<RaindexTransaction, RaindexError> {
        let raindex_address = Address::from_str(&raindex_address)?;
        let tx_hash = B256::from_str(&tx_hash)?;
        self.get_transaction(
            chain_id,
            raindex_address,
            tx_hash,
            max_attempts.map(|v| v as usize),
            interval_ms.map(|v| v as u64),
        )
        .await
    }
}
impl RaindexClient {
    pub async fn get_transaction(
        &self,
        chain_id: u32,
        raindex_address: Address,
        tx_hash: B256,
        max_attempts: Option<usize>,
        interval_ms: Option<u64>,
    ) -> Result<RaindexTransaction, RaindexError> {
        let attempts = max_attempts
            .unwrap_or(DEFAULT_TRANSACTION_POLL_ATTEMPTS)
            .max(1);
        let interval_ms = interval_ms.unwrap_or(DEFAULT_TRANSACTION_POLL_INTERVAL_MS);

        match self.query_source(chain_id) {
            QuerySource::LocalDb(local_db) => {
                let local_source = LocalDbTransactions::new(&local_db);
                let raindex_id = RaindexIdentifier::new(chain_id, raindex_address);

                for attempt in 1..=attempts {
                    if let Some(tx) = local_source.get_by_tx_hash(&raindex_id, tx_hash).await? {
                        return Ok(tx);
                    }
                    if attempt < attempts {
                        sleep_ms(interval_ms).await;
                    }
                }

                Err(RaindexError::TransactionIndexingTimeout { tx_hash, attempts })
            }
            QuerySource::Subgraph => {
                let client = self.get_raindex_subgraph_client(raindex_address)?;
                for attempt in 1..=attempts {
                    match client
                        .transaction_detail(Id::new(tx_hash.to_string()))
                        .await
                    {
                        Ok(transaction) => {
                            return transaction.try_into();
                        }
                        Err(RaindexSubgraphClientError::Empty) => {
                            if attempt < attempts {
                                sleep_ms(interval_ms).await;
                                continue;
                            }
                        }
                        Err(e) => return Err(e.into()),
                    }
                }

                Err(RaindexError::TransactionIndexingTimeout { tx_hash, attempts })
            }
        }
    }
}

impl TryFrom<SgTransaction> for RaindexTransaction {
    type Error = RaindexError;
    fn try_from(transaction: SgTransaction) -> Result<Self, Self::Error> {
        Ok(Self {
            id: B256::from_str(&transaction.id.0)?,
            from: Address::from_str(&transaction.from.0)?,
            block_number: U256::from_str(&transaction.block_number.0)?,
            timestamp: U256::from_str(&transaction.timestamp.0)?,
        })
    }
}

#[cfg(test)]
mod test_helpers {
    #[cfg(not(target_family = "wasm"))]
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_RAINDEX_ADDRESS};
        use alloy::primitives::b256;
        use httpmock::MockServer;
        use serde_json::{json, Value};

        fn sample_transaction_response() -> Value {
            json!({
                "data": {
                    "transaction": {
                        "id": "0x0000000000000000000000000000000000000000000000000000000000000123",
                        "from": "0x1000000000000000000000000000000000000000",
                        "blockNumber": "12345",
                        "timestamp": "1734054063"
                    }
                }
            })
        }

        fn empty_transaction_response() -> Value {
            json!({
                "data": {
                    "transaction": null
                }
            })
        }

        #[tokio::test]
        async fn test_get_transaction() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200)
                    .json_body_obj(&sample_transaction_response());
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let tx = raindex_client
                .get_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_RAINDEX_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                    None,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                tx.id(),
                b256!("0x0000000000000000000000000000000000000000000000000000000000000123")
            );
            assert_eq!(
                tx.from(),
                Address::from_str("0x1000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(tx.block_number(), U256::from_str("12345").unwrap());
            assert_eq!(tx.timestamp(), U256::from_str("1734054063").unwrap());
        }

        #[tokio::test]
        async fn test_get_transaction_with_polling_success() {
            let sg_server = MockServer::start_async().await;
            let _mock = sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200)
                    .json_body_obj(&sample_transaction_response());
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let tx = raindex_client
                .get_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_RAINDEX_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                    Some(DEFAULT_TRANSACTION_POLL_ATTEMPTS),
                    Some(10),
                )
                .await
                .unwrap();

            assert_eq!(
                tx.id(),
                b256!("0x0000000000000000000000000000000000000000000000000000000000000123")
            );
        }

        #[tokio::test]
        async fn test_get_transaction_timeout() {
            let sg_server = MockServer::start_async().await;
            let _empty = sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200)
                    .json_body_obj(&empty_transaction_response());
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let err = raindex_client
                .get_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_RAINDEX_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
                    Some(3),
                    Some(10),
                )
                .await
                .unwrap_err();

            match err {
                RaindexError::TransactionIndexingTimeout { attempts, .. } => {
                    assert_eq!(attempts, 3);
                }
                other => panic!("expected timeout error, got {other:?}"),
            }
        }
    }
}
