use super::*;
use crate::local_db::query::fetch_trades_by_tx::FetchTradesByTxArgs;
use crate::raindex_client::local_db::query::fetch_trades_by_tx::fetch_trades_by_tx;
use alloy::primitives::{Address, B256};
use rain_orderbook_subgraph_client::MultiOrderbookSubgraphClient;
use std::str::FromStr;

#[wasm_export]
impl RaindexClient {
    #[wasm_export(
        js_name = "getTradesForTransaction",
        return_description = "Trades list result with total count and per-pair summary",
        unchecked_return_type = "RaindexTradesListResult",
        preserve_js_class
    )]
    pub async fn get_trades_for_transaction_wasm_binding(
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
            js_name = "txHash",
            param_description = "Transaction hash",
            unchecked_param_type = "Hex"
        )]
        tx_hash: String,
    ) -> Result<RaindexTradesListResult, RaindexError> {
        let tx_hash = B256::from_str(&tx_hash)?;
        let orderbook_addresses = orderbook_addresses
            .map(|addresses| {
                addresses
                    .into_iter()
                    .map(|address| Address::from_str(&address))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        self.get_trades_for_transaction(chain_ids, orderbook_addresses, tx_hash)
            .await
    }
}
impl RaindexClient {
    pub async fn get_trades_for_transaction(
        &self,
        chain_ids: Option<ChainIds>,
        orderbook_addresses: Option<Vec<Address>>,
        tx_hash: B256,
    ) -> Result<RaindexTradesListResult, RaindexError> {
        let ids = chain_ids.map(|ChainIds(ids)| ids);
        let (local_db, local_ids, sg_ids) = self.classify_chains(ids)?;
        let orderbook_addresses_for_local_db = orderbook_addresses.clone().unwrap_or_default();

        let mut all_trades = Vec::new();

        if let Some(db) = local_db.filter(|_| !local_ids.is_empty()) {
            let trades = fetch_trades_by_tx(
                &db,
                FetchTradesByTxArgs {
                    chain_ids: local_ids,
                    orderbook_addresses: orderbook_addresses_for_local_db,
                    tx_hash,
                },
            )
            .await?;
            let raindex_trades: Vec<RaindexTrade> = trades
                .into_iter()
                .map(RaindexTrade::try_from_local_db_trade)
                .collect::<Result<_, _>>()?;
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
                    .trades_by_transaction(tx_hash.to_string(), orderbook_in)
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

        let total_count = all_trades.len() as u64;
        let summary = RaindexPairSummary::from_trades(&all_trades)?;

        Ok(RaindexTradesListResult {
            trades: all_trades,
            total_count,
            summary: Some(summary),
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
        use alloy::primitives::b256;
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_local_db_path() {
            let trade_log_index = 1u64;
            let tx_hash =
                b256!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let fixture = build_local_trade_fixture(tx_hash, trade_log_index, 4);
            let trade_id = fixture.trade.trade_id.clone();

            let callback = make_local_db_trades_callback(
                vec![fixture.order],
                vec![fixture.input_vault, fixture.output_vault],
                vec![fixture.trade],
                4,
            );
            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let result = client
                .get_trades_for_transaction(Some(ChainIds(vec![42161])), None, tx_hash)
                .await
                .unwrap();

            assert_eq!(result.total_count(), 1);
            assert!(result.summary().is_some());
            let trades = result.trades();
            assert_eq!(trades.len(), 1);

            let trade = trades.first().unwrap();
            assert_eq!(trade.id(), trade_id);
            assert_eq!(trade.chain_id(), 42161);
            assert_eq!(trade.order_hash(), fixture.order_hash.to_string());
            assert_eq!(
                trade.orderbook().to_lowercase(),
                fixture.orderbook_address.to_string().to_lowercase()
            );
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use crate::raindex_client::tests::get_test_yaml;
        use crate::raindex_client::trades::test_helpers::get_sg_trade_json;
        use crate::raindex_client::{ChainIds, RaindexClient};
        use alloy::primitives::{b256, Address, Bytes};
        use httpmock::MockServer;
        use serde_json::json;
        use std::str::FromStr;

        #[tokio::test]
        async fn test_returns_trades() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgTransactionTradesQuery");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [get_sg_trade_json("0x0000000000000000000000000000000000000000")] }
                }));
            });

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

            let tx_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let result = client
                .get_trades_for_transaction(Some(ChainIds(vec![1])), None, tx_hash)
                .await
                .unwrap();

            assert_eq!(result.total_count(), 1);
            assert!(result.summary().is_some());
            let trades = result.trades();
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].id(), Bytes::from_str("0x0123").unwrap());
            assert_eq!(trades[0].chain_id(), 1);
            assert_eq!(
                trades[0].orderbook(),
                Address::from_str("0x1234567890123456789012345678901234567890").unwrap()
            );
        }

        #[tokio::test]
        async fn test_empty_result() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgTransactionTradesQuery");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [] }
                }));
            });

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

            let tx_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let result = client
                .get_trades_for_transaction(Some(ChainIds(vec![1])), None, tx_hash)
                .await
                .unwrap();

            assert_eq!(result.total_count(), 0);
            assert!(result.trades().is_empty());
            assert!(result.summary().is_some());
        }

        #[tokio::test]
        async fn test_with_chain_filter() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg").body_contains("SgTransactionTradesQuery");
                then.status(200).json_body_obj(&json!({
                    "data": { "trades": [get_sg_trade_json("0x0000000000000000000000000000000000000000")] }
                }));
            });

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

            let tx_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let result = client
                .get_trades_for_transaction(Some(ChainIds(vec![137])), None, tx_hash)
                .await
                .unwrap();

            assert!(result.total_count() > 0);
            let trades = result.trades();
            for trade in trades {
                assert_eq!(trade.chain_id(), 137);
            }
        }
    }
}
