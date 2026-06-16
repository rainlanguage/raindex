use super::LocalDbTrades;
use super::SubgraphTrades;
use super::*;
use crate::local_db::is_chain_supported_local_db;
use crate::local_db::OrderbookIdentifier;
use alloy::primitives::{Address, B256};
#[cfg(target_family = "wasm")]
use gloo_timers::future::TimeoutFuture;
use rain_orderbook_subgraph_client::OrderbookSubgraphClientError;
use std::str::FromStr;
#[cfg(not(target_family = "wasm"))]
use std::time::Duration;
#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;

const DEFAULT_TRADES_TX_POLL_ATTEMPTS: usize = 10;
const DEFAULT_TRADES_TX_POLL_INTERVAL_MS: u64 = 1_000;

#[cfg(target_family = "wasm")]
async fn sleep_ms(ms: u64) {
    let delay = ms.min(u32::MAX as u64) as u32;
    TimeoutFuture::new(delay).await;
}

#[cfg(not(target_family = "wasm"))]
async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(
        js_name = "getTradesForTransaction",
        return_description = "Array of trades in the transaction",
        unchecked_return_type = "RaindexTrade[]",
        preserve_js_class
    )]
    pub async fn get_trades_for_transaction_wasm_binding(
        &self,
        #[wasm_export(js_name = "chainId", param_description = "Chain ID for the network")]
        chain_id: u32,
        #[wasm_export(
            js_name = "orderbookAddress",
            param_description = "Orderbook contract address",
            unchecked_param_type = "Address"
        )]
        orderbook_address: String,
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
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let orderbook_address = Address::from_str(&orderbook_address)?;
        let tx_hash = B256::from_str(&tx_hash)?;
        self.get_trades_for_transaction(
            chain_id,
            orderbook_address,
            tx_hash,
            max_attempts.map(|v| v as usize),
            interval_ms.map(|v| v as u64),
        )
        .await
    }
}
impl RaindexClient {
    pub async fn get_trades_for_transaction(
        &self,
        chain_id: u32,
        orderbook_address: Address,
        tx_hash: B256,
        max_attempts: Option<usize>,
        interval_ms: Option<u64>,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let attempts = max_attempts
            .unwrap_or(DEFAULT_TRADES_TX_POLL_ATTEMPTS)
            .max(1);
        let interval_ms = interval_ms.unwrap_or(DEFAULT_TRADES_TX_POLL_INTERVAL_MS);
        let ob_id = OrderbookIdentifier::new(chain_id, orderbook_address);

        if is_chain_supported_local_db(chain_id) {
            if let Some(local_db) = self.local_db() {
                let local_source = LocalDbTrades::new(&local_db);
                for attempt in 1..=attempts {
                    let trades = local_source.get_by_tx_hash(&ob_id, tx_hash).await?;
                    if !trades.is_empty() {
                        return Ok(trades);
                    }
                    if attempt < attempts {
                        sleep_ms(interval_ms).await;
                    }
                }
            }
        }

        let sg_source = SubgraphTrades::new(self);
        for attempt in 1..=attempts {
            match sg_source.get_by_tx_hash(&ob_id, tx_hash).await {
                Ok(trades) => return Ok(trades),
                Err(RaindexError::OrderbookSubgraphClientError(
                    OrderbookSubgraphClientError::Empty,
                )) => {
                    if attempt < attempts {
                        sleep_ms(interval_ms).await;
                        continue;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(RaindexError::TradesIndexingTimeout { tx_hash, attempts })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::super::super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use alloy::primitives::{b256, Address, Bytes, U256};
        use httpmock::MockServer;
        use rain_orderbook_subgraph_client::utils::float::*;
        use serde_json::{json, Value};
        use std::str::FromStr;

        fn sample_trades_response() -> Value {
            json!({
                "data": {
                    "trades": [
                        {
                            "id": "0xabc1",
                            "tradeEvent": {
                                "transaction": {
                                    "id": "0x0000000000000000000000000000000000000000000000000000000000000456",
                                    "from": "0x0000000000000000000000000000000000000001",
                                    "blockNumber": "100",
                                    "timestamp": "1700000000"
                                },
                                "sender": "0x0000000000000000000000000000000000000002"
                            },
                            "outputVaultBalanceChange": {
                                "id": "0xout1",
                                "__typename": "TradeVaultBalanceChange",
                                "amount": NEG2,
                                "newVaultBalance": F0,
                                "oldVaultBalance": F0,
                                "vault": {
                                    "id": "0xv1",
                                    "vaultId": "0x01",
                                    "token": {
                                        "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                                        "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                                        "name": "Staked FLR",
                                        "symbol": "sFLR",
                                        "decimals": "18"
                                    }
                                },
                                "timestamp": "1700000000",
                                "transaction": {
                                    "id": "0x0000000000000000000000000000000000000000000000000000000000000456",
                                    "from": "0x0000000000000000000000000000000000000001",
                                    "blockNumber": "100",
                                    "timestamp": "1700000000"
                                },
                                "orderbook": { "id": "0x1234567890123456789012345678901234567890" },
                                "trade": { "tradeEvent": { "__typename": "TakeOrder" } }
                            },
                            "order": {
                                "id": "0x0000000000000000000000000000000000000001",
                                "orderHash": "0x00000000000000000000000000000000000000000000000000000000000abc01"
                            },
                            "inputVaultBalanceChange": {
                                "id": "0xin1",
                                "__typename": "TradeVaultBalanceChange",
                                "amount": F1,
                                "newVaultBalance": F0,
                                "oldVaultBalance": F0,
                                "vault": {
                                    "id": "0xv2",
                                    "vaultId": "0x02",
                                    "token": {
                                        "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "name": "Wrapped Flare",
                                        "symbol": "WFLR",
                                        "decimals": "18"
                                    }
                                },
                                "timestamp": "1700000000",
                                "transaction": {
                                    "id": "0x0000000000000000000000000000000000000000000000000000000000000456",
                                    "from": "0x0000000000000000000000000000000000000001",
                                    "blockNumber": "100",
                                    "timestamp": "1700000000"
                                },
                                "orderbook": { "id": "0x1234567890123456789012345678901234567890" },
                                "trade": { "tradeEvent": { "__typename": "TakeOrder" } }
                            },
                            "timestamp": "1700000000",
                            "orderbook": { "id": "0x1234567890123456789012345678901234567890" }
                        }
                    ]
                }
            })
        }

        fn empty_trades_response() -> Value {
            json!({
                "data": {
                    "trades": []
                }
            })
        }

        #[tokio::test]
        async fn test_get_trades_for_transaction_found() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&sample_trades_response());
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();

            let trades = raindex_client
                .get_trades_for_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000456"),
                    None,
                    None,
                )
                .await
                .unwrap();

            assert_eq!(trades.len(), 1);
            let trade = &trades[0];
            assert_eq!(
                trade.transaction().id(),
                b256!("0x0000000000000000000000000000000000000000000000000000000000000456")
            );
            assert_eq!(
                trade.order_hash(),
                Bytes::from_str(
                    "0x00000000000000000000000000000000000000000000000000000000000abc01"
                )
                .unwrap()
            );
            assert_eq!(
                trade.orderbook(),
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap()
            );
            assert_eq!(trade.timestamp(), U256::from(1700000000u64));
            assert_eq!(
                trade.output_vault_balance_change().token().symbol(),
                Some("sFLR".to_string())
            );
            assert_eq!(
                trade.input_vault_balance_change().token().symbol(),
                Some("WFLR".to_string())
            );
        }

        #[tokio::test]
        async fn test_get_trades_for_transaction_empty() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&empty_trades_response());
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();

            let result = raindex_client
                .get_trades_for_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000789"),
                    Some(1),
                    Some(10),
                )
                .await
                .unwrap_err();

            assert!(matches!(
                result,
                RaindexError::TradesIndexingTimeout { attempts: 1, .. }
            ));
        }

        #[tokio::test]
        async fn test_get_trades_for_transaction_network_error() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(500);
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();

            let result = raindex_client
                .get_trades_for_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000999"),
                    None,
                    None,
                )
                .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_get_trades_for_transaction_polling_success() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&sample_trades_response());
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();

            let trades = raindex_client
                .get_trades_for_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000456"),
                    Some(5),
                    Some(10),
                )
                .await
                .unwrap();

            assert_eq!(trades.len(), 1);
        }

        #[tokio::test]
        async fn test_get_trades_for_transaction_timeout() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg");
                then.status(200).json_body_obj(&empty_trades_response());
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg"),
                    "http://localhost:3000",
                    "http://localhost:3000",
                    "http://localhost:3000",
                )],
                None,
            )
            .unwrap();

            let err = raindex_client
                .get_trades_for_transaction(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000456"),
                    Some(3),
                    Some(10),
                )
                .await
                .unwrap_err();

            match err {
                RaindexError::TradesIndexingTimeout { attempts, .. } => {
                    assert_eq!(attempts, 3);
                }
                other => panic!("expected TradesIndexingTimeout, got {other:?}"),
            }
        }
    }
}
