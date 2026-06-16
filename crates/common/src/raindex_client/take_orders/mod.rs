pub(crate) mod approval;
mod request;
pub(crate) mod result;
mod selection;
pub mod single;

#[cfg(all(test, not(target_family = "wasm")))]
mod e2e_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod single_tests;

pub use request::TakeOrdersRequest;
pub use result::{ApprovalInfo, TakeOrderEstimate, TakeOrdersCalldataResult, TakeOrdersInfo};
pub use single::{build_candidate_from_quote, estimate_take_order, execute_single_take};

use super::{RaindexClient, RaindexError};
use crate::rpc_client::RpcClient;
use crate::take_orders::{
    build_take_orders_config_from_simulation, find_failing_order_index, simulate_take_orders,
};
use approval::{check_approval_needed, ApprovalCheckParams};
use rain_orderbook_bindings::provider::mk_read_provider;
use wasm_bindgen_utils::prelude::*;
use wasm_bindgen_utils::wasm_export;

#[wasm_export]
impl RaindexClient {
    /// Generates calldata for `IRaindexV6.takeOrders4` using a mode + price-cap policy.
    ///
    /// This method includes preflight simulation to validate the transaction will succeed
    /// and automatically removes failing orders from the config.
    ///
    /// The request object contains:
    /// - `taker`: Address of the account that will execute the takeOrders transaction
    /// - `chainId`: Chain ID of the target network
    /// - `sellToken`: Token address the taker will GIVE
    /// - `buyToken`: Token address the taker will RECEIVE
    /// - `mode`: One of `buyExact`, `buyUpTo`, `spendExact`, or `spendUpTo`
    /// - `amount`: Target amount (output tokens for buy modes, input tokens for spend modes)
    /// - `priceCap`: human-readable decimal string for max sell per 1 buy
    ///
    /// Returns calldata plus pricing info:
    /// - `calldata`: ABI-encoded bytes for `takeOrders4`.
    /// - `effectivePrice`: expected blended sell per 1 buy from the simulation.
    /// - `prices`: per-leg ratios, best→worst.
    /// - `expectedSell`: simulated sell at current quotes.
    /// - `maxSellCap`: `amount * priceCap` for buy modes, `amount` for spend modes (worst-case on-chain spend cap).
    ///
    /// ## Example (JS)
    /// ```javascript
    /// const res = await client.getTakeOrdersCalldata({
    ///   chainId: 137,
    ///   taker: "0xTAKER...",
    ///   sellToken: "0xSELL...",
    ///   buyToken: "0xBUY...",
    ///   mode: "buyUpTo",
    ///   amount: "10",
    ///   priceCap: "1.2",
    /// });
    /// if (res.error) {
    ///   console.error(res.error.readableMsg);
    /// } else {
    ///   const { calldata, effectivePrice, expectedSell, maxSellCap, prices, orderbook } = res.value;
    /// }
    /// ```
    #[wasm_export(
        js_name = "getTakeOrdersCalldata",
        return_description = "Encoded takeOrders4 calldata and price information",
        unchecked_return_type = "TakeOrdersCalldataResult",
        preserve_js_class
    )]
    pub async fn get_take_orders_calldata(
        &self,
        #[wasm_export(
            js_name = "request",
            param_description = "Take orders request parameters"
        )]
        request: TakeOrdersRequest,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let req = request::parse_request(&request)?;

        let orders = self
            .fetch_orders_for_pair(request.chain_id, req.sell_token, req.buy_token)
            .await?;

        let rpc_urls = self.get_rpc_urls_for_chain(request.chain_id)?;
        let rpc_client = RpcClient::new_with_urls(rpc_urls.clone())?;
        let block_number = rpc_client.get_latest_block_number().await?;

        let candidates = selection::build_candidates_for_chain(
            &orders,
            req.sell_token,
            req.buy_token,
            Some(block_number),
            None,
        )
        .await?;

        let (best_orderbook, best_sim) = selection::select_best_orderbook_simulation(
            candidates.clone(),
            req.mode,
            req.price_cap,
        )?;

        let mut built =
            build_take_orders_config_from_simulation(best_sim.clone(), req.mode, req.price_cap)?
                .ok_or(RaindexError::NoLiquidity)?;

        let approval_params = ApprovalCheckParams {
            rpc_urls: rpc_urls.clone(),
            sell_token: req.sell_token,
            taker: req.taker,
            orderbook: best_orderbook,
            mode: req.mode,
            price_cap: req.price_cap,
        };

        if let Some(approval_result) = check_approval_needed(&approval_params).await? {
            return Ok(approval_result);
        }

        let provider =
            mk_read_provider(&rpc_urls).map_err(|e| RaindexError::PreflightError(e.to_string()))?;

        for _ in 0..built.config.orders.len() {
            let sim_result = simulate_take_orders(
                &provider,
                best_orderbook,
                req.taker,
                &built.config,
                Some(block_number),
            )
            .await;

            match sim_result {
                Ok(()) => {
                    return result::build_calldata_result(
                        best_orderbook,
                        built,
                        req.mode,
                        req.price_cap,
                    );
                }
                Err(sim_error) => {
                    if let Some(failing_idx) = find_failing_order_index(
                        &provider,
                        best_orderbook,
                        req.taker,
                        &built.config,
                        Some(block_number),
                    )
                    .await
                    {
                        if built.config.orders.len() <= 1 {
                            return Err(RaindexError::PreflightError(format!(
                                "All orders failed simulation. Last error: {}",
                                sim_error
                            )));
                        }

                        built.config.orders.remove(failing_idx);
                        built.sim.legs.remove(failing_idx);
                    } else {
                        return Err(RaindexError::PreflightError(format!(
                            "Simulation failed but could not identify failing order: {}",
                            sim_error
                        )));
                    }
                }
            }
        }

        Err(RaindexError::PreflightError(
            "Exceeded maximum preflight iterations".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::take_orders::TakeOrdersMode;
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::{from_js_value, to_js_value};

        #[wasm_bindgen_test]
        fn test_take_orders_mode_serialization() {
            let buy_up_to = TakeOrdersMode::BuyUpTo;
            let buy_exact = TakeOrdersMode::BuyExact;
            let spend_up_to = TakeOrdersMode::SpendUpTo;
            let spend_exact = TakeOrdersMode::SpendExact;

            let buy_up_to_js = to_js_value(&buy_up_to).unwrap();
            let buy_exact_js = to_js_value(&buy_exact).unwrap();
            let spend_up_to_js = to_js_value(&spend_up_to).unwrap();
            let spend_exact_js = to_js_value(&spend_exact).unwrap();

            let buy_up_to_back: TakeOrdersMode = from_js_value(buy_up_to_js).unwrap();
            let buy_exact_back: TakeOrdersMode = from_js_value(buy_exact_js).unwrap();
            let spend_up_to_back: TakeOrdersMode = from_js_value(spend_up_to_js).unwrap();
            let spend_exact_back: TakeOrdersMode = from_js_value(spend_exact_js).unwrap();

            assert_eq!(buy_up_to_back, TakeOrdersMode::BuyUpTo);
            assert_eq!(buy_exact_back, TakeOrdersMode::BuyExact);
            assert_eq!(spend_up_to_back, TakeOrdersMode::SpendUpTo);
            assert_eq!(spend_exact_back, TakeOrdersMode::SpendExact);
        }
    }
}
