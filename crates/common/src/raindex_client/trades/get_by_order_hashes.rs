use super::get_all::{normalize_trade_tokens, sg_trade_matches_token_filter, GetTradesTokenFilter};
use super::*;
use crate::local_db::query::fetch_trades::{FetchTradesArgs, FetchTradesTokensFilter};
use crate::raindex_client::local_db::query::fetch_trades::fetch_trades;
use crate::utils::timing::Timing;
use rain_orderbook_subgraph_client::types::common::{
    SgBigInt, SgBytes, SgTradeEventFilter, SgTradeOrderFilter, SgTradesListQueryFilters,
    SgTradesTokensFilterArgs,
};
use rain_orderbook_subgraph_client::MultiOrderbookSubgraphClient;
use std::collections::{HashMap, HashSet};
use tsify::Tsify;
use wasm_bindgen_utils::{impl_wasm_traits, wasm_export};

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct OrderHashes(#[tsify(type = "Hex[]")] pub Vec<B256>);
impl_wasm_traits!(OrderHashes);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GetTradesByOrderHashesFilters {
    #[tsify(optional, type = "Address[]")]
    pub owners: Vec<Address>,
    #[tsify(optional, type = "Address[]")]
    pub takers: Vec<Address>,
    #[tsify(optional)]
    pub tokens: Option<GetTradesTokenFilter>,
    #[tsify(optional, type = "Address[]")]
    pub orderbook_addresses: Option<Vec<Address>>,
    #[tsify(optional)]
    pub time_filter: Option<TimeFilter>,
}
impl_wasm_traits!(GetTradesByOrderHashesFilters);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexOrderHashTrades {
    order_hash: B256,
    trades: Vec<RaindexTrade>,
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexOrderHashTrades {
    #[wasm_bindgen(getter = orderHash, unchecked_return_type = "Hex")]
    pub fn order_hash(&self) -> String {
        self.order_hash.to_string()
    }

    #[wasm_bindgen(getter, unchecked_return_type = "RaindexTrade[]")]
    pub fn trades(&self) -> Vec<RaindexTrade> {
        self.trades.clone()
    }
}

#[cfg(not(target_family = "wasm"))]
impl RaindexOrderHashTrades {
    pub fn order_hash(&self) -> B256 {
        self.order_hash
    }

    pub fn trades(&self) -> &[RaindexTrade] {
        &self.trades
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexTradesByOrderHashResult {
    trades_by_order_hash: Vec<RaindexOrderHashTrades>,
    total_count: u64,
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexTradesByOrderHashResult {
    #[wasm_bindgen(getter = tradesByOrderHash, unchecked_return_type = "RaindexOrderHashTrades[]")]
    pub fn trades_by_order_hash(&self) -> Vec<RaindexOrderHashTrades> {
        self.trades_by_order_hash.clone()
    }

    #[wasm_bindgen(getter = totalCount)]
    pub fn total_count(&self) -> u64 {
        self.total_count
    }
}

#[cfg(not(target_family = "wasm"))]
impl RaindexTradesByOrderHashResult {
    pub fn trades_by_order_hash(&self) -> &[RaindexOrderHashTrades] {
        &self.trades_by_order_hash
    }

    pub fn total_count(&self) -> u64 {
        self.total_count
    }
}

fn unique_order_hashes_preserving_order(order_hashes: Vec<B256>) -> Vec<B256> {
    let mut seen = HashSet::new();
    let mut unique = Vec::with_capacity(order_hashes.len());
    for order_hash in order_hashes {
        if seen.insert(order_hash) {
            unique.push(order_hash);
        }
    }
    unique
}

fn build_sg_filters(
    filters: GetTradesByOrderHashesFilters,
    order_hashes: &[B256],
) -> SgTradesListQueryFilters {
    let time_filter = filters.time_filter.unwrap_or_default();
    let mut sg_filters = SgTradesListQueryFilters {
        timestamp_gte: Some(
            time_filter
                .start
                .map_or(SgBigInt("0".to_string()), |v| SgBigInt(v.to_string())),
        ),
        timestamp_lte: Some(
            time_filter
                .end
                .map_or(SgBigInt(u64::MAX.to_string()), |v| SgBigInt(v.to_string())),
        ),
        orderbook_in: filters
            .orderbook_addresses
            .unwrap_or_default()
            .into_iter()
            .map(|address| address.to_string().to_lowercase())
            .collect(),
        ..Default::default()
    };

    sg_filters.order_ = Some(SgTradeOrderFilter {
        owner_in: filters
            .owners
            .into_iter()
            .map(|owner| SgBytes(owner.to_string()))
            .collect(),
        order_hash: None,
        order_hash_in: order_hashes
            .iter()
            .map(|hash| SgBytes(hash.to_string()))
            .collect(),
    });

    if !filters.takers.is_empty() {
        sg_filters.trade_event_ = Some(SgTradeEventFilter {
            sender_in: filters
                .takers
                .into_iter()
                .map(|taker| SgBytes(taker.to_string()))
                .collect(),
        });
    }

    sg_filters
}

fn group_trades_by_order_hash(
    order_hashes: Vec<B256>,
    mut trades: Vec<RaindexTrade>,
) -> RaindexTradesByOrderHashResult {
    trades.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));
    let total_count = trades.len() as u64;
    let mut grouped: HashMap<B256, Vec<RaindexTrade>> = HashMap::new();
    for trade in trades {
        grouped.entry(trade.order_hash).or_default().push(trade);
    }

    let trades_by_order_hash = order_hashes
        .into_iter()
        .map(|order_hash| RaindexOrderHashTrades {
            order_hash,
            trades: grouped.remove(&order_hash).unwrap_or_default(),
        })
        .collect();

    RaindexTradesByOrderHashResult {
        trades_by_order_hash,
        total_count,
    }
}

#[wasm_export]
impl RaindexClient {
    #[wasm_export(
        js_name = "getTradesByOrderHashes",
        return_description = "Trades grouped by requested order hash",
        unchecked_return_type = "RaindexTradesByOrderHashResult",
        preserve_js_class
    )]
    pub async fn get_trades_by_order_hashes(
        &self,
        #[wasm_export(
            js_name = "chainIds",
            param_description = "Specific blockchain networks to query (optional, queries all networks if not specified)"
        )]
        chain_ids: Option<ChainIds>,
        #[wasm_export(
            js_name = "orderHashes",
            param_description = "Order hashes to query in a single batch",
            unchecked_param_type = "Hex[]"
        )]
        order_hashes: OrderHashes,
        #[wasm_export(
            param_description = "Filtering criteria including owners, takers, token addresses, orderbooks, and time range"
        )]
        filters: Option<GetTradesByOrderHashesFilters>,
    ) -> Result<RaindexTradesByOrderHashResult, RaindexError> {
        let started = Timing::now();
        let requested_order_hashes_count = order_hashes.0.len();
        let order_hashes = unique_order_hashes_preserving_order(order_hashes.0);
        if order_hashes.is_empty() {
            return Ok(RaindexTradesByOrderHashResult {
                trades_by_order_hash: Vec::new(),
                total_count: 0,
            });
        }

        let filters = filters.unwrap_or_default();
        let input_tokens_count = filters
            .tokens
            .as_ref()
            .and_then(|tokens| tokens.inputs.as_ref())
            .map_or(0, Vec::len);
        let output_tokens_count = filters
            .tokens
            .as_ref()
            .and_then(|tokens| tokens.outputs.as_ref())
            .map_or(0, Vec::len);
        let takers_count = filters.takers.len();
        let owners_count = filters.owners.len();
        let orderbooks_count = filters.orderbook_addresses.as_ref().map_or(0, Vec::len);
        let ids = chain_ids.map(|ChainIds(ids)| ids);
        let requested_chain_ids_count = ids.as_ref().map_or(0, Vec::len);
        let (local_db, local_ids, sg_ids) = self.classify_chains(ids)?;
        let local_chain_ids_count = local_ids.len();
        let subgraph_chain_ids_count = sg_ids.len();

        let mut all_trades = Vec::new();
        let mut local_rows = 0usize;
        let mut subgraph_rows_before_token_filter = 0usize;
        let mut subgraph_rows_after_token_filter = 0usize;

        if let Some(db) = local_db {
            let local_tokens = filters
                .tokens
                .clone()
                .map(|tokens| FetchTradesTokensFilter {
                    inputs: tokens.inputs.unwrap_or_default(),
                    outputs: tokens.outputs.unwrap_or_default(),
                })
                .unwrap_or_default();
            let local_started = Timing::now();
            let local_trades = fetch_trades(
                &db,
                FetchTradesArgs {
                    chain_ids: local_ids,
                    orderbook_addresses: filters.orderbook_addresses.clone().unwrap_or_default(),
                    owners: filters.owners.clone(),
                    takers: filters.takers.clone(),
                    order_hash: None,
                    order_hashes: order_hashes.clone(),
                    tokens: local_tokens,
                    time_filter: filters.time_filter.clone().unwrap_or_default(),
                    pagination: PaginationParams::default(),
                },
            )
            .await?;
            local_rows = local_trades.len();
            all_trades.extend(
                local_trades
                    .into_iter()
                    .map(RaindexTrade::try_from_local_db_trade)
                    .collect::<Result<Vec<_>, _>>()?,
            );
            tracing::debug!(
                order_hashes_count = order_hashes.len(),
                rows = local_rows,
                duration_ms = local_started.elapsed_ms(),
                "getTradesByOrderHashes local DB completed"
            );
        }

        if !sg_ids.is_empty() {
            let multi_subgraph_args = self.get_multi_subgraph_args(Some(sg_ids))?;
            let sg_filters = build_sg_filters(filters.clone(), &order_hashes);
            let sg_token_filter = filters
                .tokens
                .clone()
                .and_then(Option::<SgTradesTokensFilterArgs>::from)
                .map(normalize_trade_tokens);
            let name_to_chain_id: HashMap<&str, u32> = multi_subgraph_args
                .iter()
                .flat_map(|(chain_id, args)| args.iter().map(|arg| (arg.name.as_str(), *chain_id)))
                .collect();
            let client = MultiOrderbookSubgraphClient::new(
                multi_subgraph_args.values().flatten().cloned().collect(),
            );
            let subgraph_started = Timing::now();
            let sg_trades = client.trades_list_all(sg_filters).await?;
            subgraph_rows_before_token_filter = sg_trades.len();
            let sg_trades: Vec<_> = if let Some(tokens) = sg_token_filter {
                sg_trades
                    .into_iter()
                    .filter(|trade| sg_trade_matches_token_filter(&trade.trade, &tokens))
                    .collect()
            } else {
                sg_trades
            };
            subgraph_rows_after_token_filter = sg_trades.len();
            for trade_with_name in sg_trades {
                let chain_id = name_to_chain_id
                    .get(trade_with_name.subgraph_name.as_str())
                    .copied()
                    .ok_or(RaindexError::SubgraphNotFound(
                        trade_with_name.subgraph_name.clone(),
                        trade_with_name.trade.id.0.clone(),
                    ))?;
                all_trades.push(RaindexTrade::try_from_sg_trade(
                    chain_id,
                    trade_with_name.trade,
                )?);
            }
            tracing::debug!(
                order_hashes_count = order_hashes.len(),
                rows_before_token_filter = subgraph_rows_before_token_filter,
                rows_after_token_filter = subgraph_rows_after_token_filter,
                duration_ms = subgraph_started.elapsed_ms(),
                "getTradesByOrderHashes subgraph completed"
            );
        }

        let result = group_trades_by_order_hash(order_hashes, all_trades);
        tracing::debug!(
            requested_chain_ids_count,
            local_chain_ids_count,
            subgraph_chain_ids_count,
            requested_order_hashes_count,
            unique_order_hashes_count = result.trades_by_order_hash.len(),
            owners_count,
            takers_count,
            orderbooks_count,
            input_tokens_count,
            output_tokens_count,
            local_rows,
            subgraph_rows_before_token_filter,
            subgraph_rows_after_token_filter,
            total_count = result.total_count,
            duration_ms = started.elapsed_ms(),
            "getTradesByOrderHashes completed"
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::fetch_order_trades::LocalDbOrderTrade;
    use alloy::primitives::{b256, Bytes};

    fn local_trade(order_hash: B256, block_timestamp: u64, trade_id: u8) -> LocalDbOrderTrade {
        LocalDbOrderTrade {
            chain_id: 1,
            trade_kind: "take".to_string(),
            orderbook: Address::ZERO,
            order_hash,
            order_owner: Address::ZERO,
            order_nonce: "0".to_string(),
            transaction_hash: B256::ZERO,
            log_index: 1,
            block_number: 1,
            block_timestamp,
            transaction_sender: Address::ZERO,
            input_vault_id: U256::from(1),
            input_token: Address::ZERO,
            input_token_name: None,
            input_token_symbol: None,
            input_token_decimals: Some(18),
            input_delta: "0x0000000000000000000000000000000000000000000000000000000000000001"
                .to_string(),
            input_running_balance: Some(
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            ),
            output_vault_id: U256::from(2),
            output_token: Address::ZERO,
            output_token_name: None,
            output_token_symbol: None,
            output_token_decimals: Some(18),
            output_delta: "0x00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffe"
                .to_string(),
            output_running_balance: Some(
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            ),
            trade_id: Bytes::from(vec![trade_id]).to_string(),
        }
    }

    #[test]
    fn unique_order_hashes_preserves_first_seen_order() {
        let hash_a = b256!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let hash_b = b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");

        assert_eq!(
            unique_order_hashes_preserving_order(vec![hash_b, hash_a, hash_b]),
            vec![hash_b, hash_a]
        );
    }

    #[test]
    fn groups_requested_hashes_and_preserves_empty_buckets() {
        let hash_a = b256!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let hash_b = b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        let trade = RaindexTrade::try_from_local_db_trade(local_trade(hash_b, 1, 1)).unwrap();

        let result = group_trades_by_order_hash(vec![hash_a, hash_b], vec![trade]);

        assert_eq!(result.total_count(), 1);
        assert_eq!(result.trades_by_order_hash[0].order_hash, hash_a);
        assert!(result.trades_by_order_hash[0].trades.is_empty());
        assert_eq!(result.trades_by_order_hash[1].order_hash, hash_b);
        assert_eq!(result.trades_by_order_hash[1].trades.len(), 1);
    }

    #[test]
    fn sorts_grouped_trades_by_timestamp_desc() {
        let hash = b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");

        let result = group_trades_by_order_hash(
            vec![hash],
            vec![
                RaindexTrade::try_from_local_db_trade(local_trade(hash, 1, 1)).unwrap(),
                RaindexTrade::try_from_local_db_trade(local_trade(hash, 2, 2)).unwrap(),
            ],
        );

        assert_eq!(
            result.trades_by_order_hash[0].trades[0].timestamp,
            U256::from(2)
        );
    }

    #[test]
    fn builds_subgraph_order_hash_in_filter() {
        let hash_a = b256!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        let hash_b = b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        let filters = build_sg_filters(GetTradesByOrderHashesFilters::default(), &[hash_a, hash_b]);
        let order_filter = filters.order_.unwrap();

        assert_eq!(order_filter.order_hash, None);
        assert_eq!(
            order_filter.order_hash_in,
            vec![SgBytes(hash_a.to_string()), SgBytes(hash_b.to_string())]
        );
    }
}
