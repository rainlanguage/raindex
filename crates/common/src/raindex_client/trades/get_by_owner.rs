use super::LocalDbTrades;
use super::RaindexTradesListResult;
use super::SubgraphTrades;
use super::*;
use crate::local_db::is_chain_supported_local_db;
use crate::raindex_client::types::{OrderbookIdentifierParams, PaginationParams, TimeFilter};
use alloy::primitives::Address;
use std::str::FromStr;

#[wasm_export]
impl RaindexClient {
    #[wasm_export(
        js_name = "getTradesForOwner",
        return_description = "Paginated trades list with total count",
        unchecked_return_type = "RaindexTradesListResult",
        preserve_js_class
    )]
    pub async fn get_trades_for_owner(
        &self,
        #[wasm_export(
            js_name = "orderbookIdentifier",
            param_description = "Orderbook identifier (chain ID and orderbook address)"
        )]
        orderbook_identifier: OrderbookIdentifierParams,
        #[wasm_export(
            js_name = "owner",
            param_description = "Owner address to filter trades by",
            unchecked_param_type = "Address"
        )]
        owner: String,
        #[wasm_export(
            js_name = "pagination",
            param_description = "Pagination parameters (page and pageSize)"
        )]
        pagination: PaginationParams,
        #[wasm_export(
            js_name = "timeFilter",
            param_description = "Time filter parameters (start and end timestamps)"
        )]
        time_filter: TimeFilter,
    ) -> Result<RaindexTradesListResult, RaindexError> {
        let ob_id = orderbook_identifier.try_into_ob_id()?;
        let owner = Address::from_str(&owner)?;

        if is_chain_supported_local_db(ob_id.chain_id) {
            if let Some(local_db) = self.local_db() {
                let local_source = LocalDbTrades::new(&local_db);
                let trades = local_source
                    .get_by_owner(&ob_id, owner, &pagination, &time_filter)
                    .await?;
                let total_count = local_source
                    .count_by_owner(&ob_id, owner, &time_filter)
                    .await?;
                return Ok(RaindexTradesListResult {
                    trades,
                    total_count,
                });
            }
        }

        let sg_source = SubgraphTrades::new(self);
        let trades = sg_source
            .get_by_owner(&ob_id, owner, &pagination, &time_filter)
            .await?;
        let total_count = sg_source
            .count_by_owner(&ob_id, owner, &time_filter)
            .await?;
        Ok(RaindexTradesListResult {
            trades,
            total_count,
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::super::super::super::*;
        use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
        use crate::raindex_client::types::{
            OrderbookIdentifierParams, PaginationParams, TimeFilter,
        };
        use alloy::primitives::{Address, Bytes, U256};
        use httpmock::MockServer;
        use rain_orderbook_subgraph_client::utils::float::*;
        use serde_json::{json, Value};
        use std::str::FromStr;

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

        fn empty_trades_response() -> Value {
            json!({
                "data": {
                    "trades": []
                }
            })
        }

        #[tokio::test]
        async fn test_get_trades_for_owner_found() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesListQuery");
                then.status(200)
                    .json_body_obj(&sample_owner_trades_response());
            });
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

            let result = raindex_client
                .get_trades_for_owner(
                    OrderbookIdentifierParams {
                        chain_id: 1,
                        orderbook_address: CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    },
                    "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string(),
                    PaginationParams::default(),
                    TimeFilter::default(),
                )
                .await
                .unwrap();

            assert_eq!(result.trades().len(), 1);
            assert_eq!(result.total_count(), 1);

            let trade = &result.trades()[0];
            assert_eq!(trade.id(), Bytes::from_str("0xabc1").unwrap());
            assert_eq!(trade.timestamp(), U256::from(1700000000u64));
            assert_eq!(
                trade.orderbook(),
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_trades_for_owner_empty() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesListQuery");
                then.status(200).json_body_obj(&empty_trades_response());
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

            let result = raindex_client
                .get_trades_for_owner(
                    OrderbookIdentifierParams {
                        chain_id: 1,
                        orderbook_address: CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    },
                    "0x0000000000000000000000000000000000000099".to_string(),
                    PaginationParams::default(),
                    TimeFilter::default(),
                )
                .await
                .unwrap();

            assert!(result.trades().is_empty());
            assert_eq!(result.total_count(), 0);
        }

        #[tokio::test]
        async fn test_get_trades_for_owner_network_error() {
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
                .get_trades_for_owner(
                    OrderbookIdentifierParams {
                        chain_id: 1,
                        orderbook_address: CHAIN_ID_1_ORDERBOOK_ADDRESS.to_string(),
                    },
                    "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11".to_string(),
                    PaginationParams::default(),
                    TimeFilter::default(),
                )
                .await;

            assert!(result.is_err());
        }
    }
}
