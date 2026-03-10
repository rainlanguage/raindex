use super::*;
use crate::local_db::query::fetch_owner_trades::FetchOwnerTradesArgs;
use crate::local_db::query::fetch_owner_trades_count::{
    extract_trade_count, FetchOwnerTradesCountArgs,
};
use crate::raindex_client::local_db::query::fetch_owner_trades::fetch_owner_trades;
use crate::raindex_client::local_db::query::fetch_owner_trades_count::fetch_owner_trades_count;
use alloy::primitives::Address;
use rain_orderbook_subgraph_client::MultiOrderbookSubgraphClient;
use std::str::FromStr;

#[wasm_export]
impl RaindexClient {
    #[wasm_export(
        js_name = "getTradesForOwner",
        return_description = "Trades list result with total count and per-pair summary",
        unchecked_return_type = "RaindexTradesListResult",
        preserve_js_class
    )]
    pub async fn get_trades_for_owner_wasm_binding(
        &self,
        #[wasm_export(
            js_name = "chainIds",
            param_description = "Optional chain IDs to filter networks (queries all if not specified)"
        )]
        chain_ids: Option<ChainIds>,
        #[wasm_export(
            js_name = "orderbookAddresses",
            param_description = "Optional orderbook addresses to filter results"
        )]
        orderbook_addresses: Option<Vec<String>>,
        #[wasm_export(
            js_name = "owner",
            param_description = "Owner address",
            unchecked_param_type = "Address"
        )]
        owner: String,
        #[wasm_export(
            js_name = "startTimestamp",
            param_description = "Optional start time filter (Unix timestamp in seconds)"
        )]
        start_timestamp: Option<u64>,
        #[wasm_export(
            js_name = "endTimestamp",
            param_description = "Optional end time filter (Unix timestamp in seconds)"
        )]
        end_timestamp: Option<u64>,
        #[wasm_export(
            js_name = "page",
            param_description = "Optional page number (defaults to all results)"
        )]
        page: Option<u16>,
    ) -> Result<RaindexTradesListResult, RaindexError> {
        let owner = Address::from_str(&owner)?;
        let orderbook_addresses = orderbook_addresses
            .map(|addresses| {
                addresses
                    .into_iter()
                    .map(|address| Address::from_str(&address))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        self.get_trades_for_owner(
            chain_ids,
            orderbook_addresses,
            owner,
            start_timestamp,
            end_timestamp,
            page,
        )
        .await
    }
}
impl RaindexClient {
    pub async fn get_trades_for_owner(
        &self,
        chain_ids: Option<ChainIds>,
        orderbook_addresses: Option<Vec<Address>>,
        owner: Address,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
        page: Option<u16>,
    ) -> Result<RaindexTradesListResult, RaindexError> {
        let ids = chain_ids.map(|ChainIds(ids)| ids);
        let (local_db, local_ids, sg_ids) = self.classify_chains(ids)?;
        let orderbook_addresses_for_local_db = orderbook_addresses.clone().unwrap_or_default();

        let mut all_trades = Vec::new();
        let mut total_count: Option<u64> = None;

        if let Some(db) = local_db.filter(|_| !local_ids.is_empty()) {
            let trades = fetch_owner_trades(
                &db,
                FetchOwnerTradesArgs {
                    owner,
                    chain_ids: local_ids.clone(),
                    orderbook_addresses: orderbook_addresses_for_local_db.clone(),
                    start_timestamp,
                    end_timestamp,
                    page,
                },
            )
            .await?;
            let raindex_trades: Vec<RaindexTrade> = trades
                .into_iter()
                .map(RaindexTrade::try_from_local_db_trade)
                .collect::<Result<_, _>>()?;

            if page.is_some() {
                let count_rows = fetch_owner_trades_count(
                    &db,
                    FetchOwnerTradesCountArgs {
                        owner,
                        chain_ids: local_ids,
                        orderbook_addresses: orderbook_addresses_for_local_db,
                        start_timestamp,
                        end_timestamp,
                    },
                )
                .await?;
                total_count = Some(extract_trade_count(&count_rows));
            }

            all_trades.extend(raindex_trades);
        }

        if !sg_ids.is_empty() {
            let multi_subgraph_args = self.get_multi_subgraph_args(Some(sg_ids))?;
            let orderbook_in = orderbook_addresses
                .as_deref()
                .filter(|addresses| !addresses.is_empty())
                .map(|addresses| {
                    addresses
                        .iter()
                        .map(|address| address.to_string().to_lowercase())
                        .collect::<Vec<_>>()
                });
            if !multi_subgraph_args.is_empty() {
                let name_to_chain_id: std::collections::HashMap<&str, u32> = multi_subgraph_args
                    .iter()
                    .flat_map(|(chain_id, args)| {
                        args.iter().map(|arg| (arg.name.as_str(), *chain_id))
                    })
                    .collect();
                let client = MultiOrderbookSubgraphClient::new(
                    multi_subgraph_args.values().flatten().cloned().collect(),
                );
                let sg_trades = client
                    .trades_by_owner(
                        owner.to_string().to_lowercase(),
                        start_timestamp,
                        end_timestamp,
                        orderbook_in,
                    )
                    .await;
                for trade_with_name in sg_trades {
                    let chain_id = name_to_chain_id
                        .get(trade_with_name.subgraph_name.as_str())
                        .copied()
                        .ok_or(RaindexError::SubgraphNotFound(
                            trade_with_name.subgraph_name.clone(),
                            trade_with_name.trade.id.0.clone(),
                        ))?;
                    let trade = RaindexTrade::try_from_sg_trade(chain_id, trade_with_name.trade)?;
                    all_trades.push(trade);
                }
            }
        }

        let final_total_count = total_count.unwrap_or(all_trades.len() as u64);
        let summary = if page.is_some() {
            None
        } else {
            Some(RaindexPairSummary::from_trades(&all_trades)?)
        };

        Ok(RaindexTradesListResult {
            trades: all_trades,
            total_count: final_total_count,
            summary,
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    mod wasm {
        use crate::raindex_client::tests::{
            get_local_db_test_yaml, new_test_client_with_db_callback,
        };
        use crate::raindex_client::trades::test_helpers::{
            build_local_trade_fixture, make_local_db_trades_callback,
        };
        use crate::raindex_client::ChainIds;
        use alloy::primitives::{address, b256};
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_local_db_path() {
            let tx_hash =
                b256!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let fixture = build_local_trade_fixture(tx_hash, 1, 4);
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

            let callback = make_local_db_trades_callback(
                vec![fixture.order.clone()],
                vec![fixture.input_vault.clone(), fixture.output_vault.clone()],
                vec![fixture.trade.clone()],
                4,
            );
            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let result = client
                .get_trades_for_owner(Some(ChainIds(vec![42161])), None, owner, None, None, None)
                .await
                .unwrap();

            assert_eq!(result.total_count(), 1);
            assert!(result.summary().is_some());
            let trades = result.trades();
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].chain_id(), 42161);
        }

        #[wasm_bindgen_test]
        async fn test_local_db_with_pagination() {
            let tx_hash =
                b256!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let fixture = build_local_trade_fixture(tx_hash, 1, 4);
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

            let callback = make_local_db_trades_callback(
                vec![fixture.order.clone()],
                vec![fixture.input_vault.clone(), fixture.output_vault.clone()],
                vec![fixture.trade.clone()],
                4,
            );
            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let result = client
                .get_trades_for_owner(
                    Some(ChainIds(vec![42161])),
                    None,
                    owner,
                    None,
                    None,
                    Some(1),
                )
                .await
                .unwrap();

            assert_eq!(result.total_count(), 4);
            assert!(result.summary().is_none());
            assert_eq!(result.trades().len(), 1);
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use crate::raindex_client::tests::get_test_yaml;
        use crate::raindex_client::{ChainIds, RaindexClient};
        use alloy::primitives::{address, Bytes};
        use httpmock::MockServer;
        use rain_orderbook_subgraph_client::utils::float::*;
        use serde_json::json;
        use std::str::FromStr;

        fn get_trade_json() -> serde_json::Value {
            json!({
                "id": "0x0123",
                "tradeEvent": {
                    "transaction": {
                        "id": "0x0000000000000000000000000000000000000000000000000000000000000abc",
                        "from": "0x0000000000000000000000000000000000000000",
                        "blockNumber": "100",
                        "timestamp": "1700000000"
                    },
                    "sender": "0xsender1"
                },
                "outputVaultBalanceChange": {
                    "id": "0xovbc1",
                    "__typename": "TradeVaultBalanceChange",
                    "amount": NEG2,
                    "newVaultBalance": F0,
                    "oldVaultBalance": F2,
                    "vault": {
                        "id": "0xvault_out",
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
                        "from": "0x0000000000000000000000000000000000000000",
                        "blockNumber": "100",
                        "timestamp": "1700000000"
                    },
                    "orderbook": {
                        "id": "0x1234567890123456789012345678901234567890"
                    },
                    "trade": {
                        "tradeEvent": {
                            "__typename": "TakeOrder"
                        }
                    }
                },
                "order": {
                    "id": "0x0123",
                    "orderHash": "0x0000000000000000000000000000000000000000000000000000000000000123",
                    "owner": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                },
                "inputVaultBalanceChange": {
                    "id": "0xivbc1",
                    "__typename": "TradeVaultBalanceChange",
                    "amount": F1,
                    "newVaultBalance": F1,
                    "oldVaultBalance": F0,
                    "vault": {
                        "id": "0xvault_in",
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
                        "from": "0x0000000000000000000000000000000000000000",
                        "blockNumber": "100",
                        "timestamp": "1700000000"
                    },
                    "orderbook": {
                        "id": "0x1234567890123456789012345678901234567890"
                    },
                    "trade": {
                        "tradeEvent": {
                            "__typename": "TakeOrder"
                        }
                    }
                },
                "timestamp": "1700000000",
                "orderbook": {
                    "id": "0x1234567890123456789012345678901234567890"
                }
            })
        }

        #[tokio::test]
        async fn test_returns_trades() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesListQuery");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [get_trade_json()] }
                }));
            });

            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let client = RaindexClient::new(
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

            let result = client
                .get_trades_for_owner(Some(ChainIds(vec![1])), None, owner, None, None, None)
                .await
                .unwrap();

            assert_eq!(result.total_count(), 1);
            assert!(result.summary().is_some());
            let trades = result.trades();
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].id(), Bytes::from_str("0x0123").unwrap());
            assert_eq!(trades[0].chain_id(), 1);
        }

        #[tokio::test]
        async fn test_empty_result() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesListQuery");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [] }
                }));
            });

            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let client = RaindexClient::new(
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

            let result = client
                .get_trades_for_owner(Some(ChainIds(vec![1])), None, owner, None, None, None)
                .await
                .unwrap();

            assert_eq!(result.total_count(), 0);
            assert!(result.trades().is_empty());
            assert!(result.summary().is_some());
        }

        #[tokio::test]
        async fn test_with_pagination_skips_summary() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgOwnerTradesListQuery");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [get_trade_json()] }
                }));
            });

            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let client = RaindexClient::new(
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

            let result = client
                .get_trades_for_owner(Some(ChainIds(vec![1])), None, owner, None, None, Some(1))
                .await
                .unwrap();

            assert!(!result.trades().is_empty());
            assert!(result.summary().is_none());
        }
    }
}
