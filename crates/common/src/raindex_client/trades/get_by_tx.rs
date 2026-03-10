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
