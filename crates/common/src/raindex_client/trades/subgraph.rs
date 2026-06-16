use super::RaindexTrade;
use super::*;
use crate::local_db::OrderbookIdentifier;
use crate::raindex_client::types::{PaginationParams, TimeFilter};
use alloy::primitives::{Address, B256};
use rain_orderbook_subgraph_client::types::common::SgBytes;
use rain_orderbook_subgraph_client::types::Id;
use rain_orderbook_subgraph_client::SgPaginationArgs;

const DEFAULT_PAGE_SIZE: u16 = 100;

pub(crate) struct SubgraphTrades<'a> {
    client: &'a RaindexClient,
}

impl<'a> SubgraphTrades<'a> {
    pub(crate) fn new(client: &'a RaindexClient) -> Self {
        Self { client }
    }

    pub async fn get_by_tx_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        tx_hash: B256,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let client = self.client.get_orderbook_client(ob_id.orderbook_address)?;
        let sg_trades = client
            .transaction_trades(Id::new(tx_hash.to_string()))
            .await?;
        sg_trades
            .into_iter()
            .map(|t| RaindexTrade::try_from_sg_trade(ob_id.chain_id, t))
            .collect()
    }

    pub async fn get_by_owner(
        &self,
        ob_id: &OrderbookIdentifier,
        owner: Address,
        pagination: &PaginationParams,
        time_filter: &TimeFilter,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let client = self.client.get_orderbook_client(ob_id.orderbook_address)?;
        let owner_bytes = SgBytes(format!("{:#x}", owner));
        let page_num = pagination.page.unwrap_or(1);
        let sg_trades = client
            .owner_trades_list(
                owner_bytes,
                SgPaginationArgs {
                    page: page_num,
                    page_size: pagination.page_size.unwrap_or(DEFAULT_PAGE_SIZE),
                },
                time_filter.start,
                time_filter.end,
            )
            .await?;
        sg_trades
            .into_iter()
            .map(|t| RaindexTrade::try_from_sg_trade(ob_id.chain_id, t))
            .collect()
    }

    pub async fn count_by_owner(
        &self,
        ob_id: &OrderbookIdentifier,
        owner: Address,
        time_filter: &TimeFilter,
    ) -> Result<u64, RaindexError> {
        let client = self.client.get_orderbook_client(ob_id.orderbook_address)?;
        let owner_bytes = SgBytes(format!("{:#x}", owner));
        Ok(client
            .owner_trades_count(owner_bytes, time_filter.start, time_filter.end)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::super::*;
        use crate::local_db::OrderbookIdentifier;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use crate::raindex_client::types::{PaginationParams, TimeFilter};
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

        fn sample_owner_trades_response() -> Value {
            json!({
                "data": {
                    "trades": [
                        {
                            "id": "0xabc1",
                            "tradeEvent": {
                                "transaction": {
                                    "id": "0x0000000000000000000000000000000000000000000000000000000000000abc",
                                    "from": "0x0000000000000000000000000000000000000001",
                                    "blockNumber": "200",
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
                                    "id": "0x0000000000000000000000000000000000000000000000000000000000000abc",
                                    "from": "0x0000000000000000000000000000000000000001",
                                    "blockNumber": "200",
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
                                    "id": "0x0000000000000000000000000000000000000000000000000000000000000abc",
                                    "from": "0x0000000000000000000000000000000000000001",
                                    "blockNumber": "200",
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

        #[tokio::test]
        async fn test_get_by_tx_hash_returns_trades() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgTransactionTradesQuery");
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

            let ob_id = OrderbookIdentifier::new(
                1,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
            );
            let sg_source = SubgraphTrades::new(&raindex_client);
            let trades = sg_source
                .get_by_tx_hash(
                    &ob_id,
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000456"),
                )
                .await
                .unwrap();

            assert_eq!(trades.len(), 1);
            let trade = &trades[0];
            assert_eq!(trade.id(), Bytes::from_str("0xabc1").unwrap());
            assert_eq!(trade.timestamp(), U256::from(1700000000u64));
            assert_eq!(
                trade.orderbook(),
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_by_tx_hash_empty_propagates_error() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgTransactionTradesQuery");
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

            let ob_id = OrderbookIdentifier::new(
                1,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
            );
            let sg_source = SubgraphTrades::new(&raindex_client);
            let result = sg_source
                .get_by_tx_hash(
                    &ob_id,
                    b256!("0x0000000000000000000000000000000000000000000000000000000000000789"),
                )
                .await;

            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_get_by_owner_returns_trades() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesListQuery");
                then.status(200)
                    .json_body_obj(&sample_owner_trades_response());
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

            let ob_id = OrderbookIdentifier::new(
                1,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
            );
            let sg_source = SubgraphTrades::new(&raindex_client);
            let trades = sg_source
                .get_by_owner(
                    &ob_id,
                    Address::from_str("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11").unwrap(),
                    &PaginationParams::default(),
                    &TimeFilter::default(),
                )
                .await
                .unwrap();

            assert_eq!(trades.len(), 1);
            let trade = &trades[0];
            assert_eq!(trade.id(), Bytes::from_str("0xabc1").unwrap());
            assert_eq!(trade.timestamp(), U256::from(1700000000u64));
        }

        #[tokio::test]
        async fn test_get_by_owner_empty() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesListQuery");
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

            let ob_id = OrderbookIdentifier::new(
                1,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
            );
            let sg_source = SubgraphTrades::new(&raindex_client);
            let trades = sg_source
                .get_by_owner(
                    &ob_id,
                    Address::from_str("0x0000000000000000000000000000000000000099").unwrap(),
                    &PaginationParams::default(),
                    &TimeFilter::default(),
                )
                .await
                .unwrap();

            assert!(trades.is_empty());
        }

        #[tokio::test]
        async fn test_count_by_owner() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesCountQuery");
                then.status(200)
                    .json_body_obj(&sample_owner_trades_response());
            });
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesCountQuery");
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

            let ob_id = OrderbookIdentifier::new(
                1,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
            );
            let sg_source = SubgraphTrades::new(&raindex_client);
            let count = sg_source
                .count_by_owner(
                    &ob_id,
                    Address::from_str("0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11").unwrap(),
                    &TimeFilter::default(),
                )
                .await
                .unwrap();

            assert_eq!(count, 1);
        }
    }
}
