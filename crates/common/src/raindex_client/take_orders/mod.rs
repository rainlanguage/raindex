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
    NoopInjector, SelectedTakeOrderLeg, SignedContextInjector,
};
use crate::utils::timing::Timing;
use alloy::primitives::{keccak256, B256};
use alloy::sol_types::SolValue;
use approval::{check_approval_needed, ApprovalCheckParams};
use raindex_bindings::provider::mk_read_provider;
use tracing::{error, info, info_span, warn, Instrument};
use wasm_bindgen_utils::prelude::*;
use wasm_bindgen_utils::wasm_export;

const MAX_INFO_SELECTED_LEG_LOGS: usize = 100;

macro_rules! emit_selected_leg {
    ($level:ident, $leg_index:expr, $leg:expr, $orderbook:expr, $block_number:expr, $message:expr) => {{
        let order = &$leg.candidate.order;
        let input_index = $leg.candidate.input_io_index as usize;
        let output_index = $leg.candidate.output_io_index as usize;
        let input = order.validInputs.get(input_index);
        let output = order.validOutputs.get(output_index);

        $level!(
            orderbook = %$orderbook,
            block_number = $block_number,
            leg_index = $leg_index,
            order_hash = %order_hash_for_leg($leg),
            input_io_index = $leg.candidate.input_io_index,
            output_io_index = $leg.candidate.output_io_index,
            input_token = %input.map(|io| io.token).unwrap_or(alloy::primitives::Address::ZERO),
            output_token = %output.map(|io| io.token).unwrap_or(alloy::primitives::Address::ZERO),
            input_vault_id = %input.map(|io| io.vaultId).unwrap_or_default(),
            output_vault_id = %output.map(|io| io.vaultId).unwrap_or_default(),
            selected_input = %format_float_for_log($leg.input),
            selected_output = %format_float_for_log($leg.output),
            ratio = %format_float_for_log($leg.candidate.ratio),
            event = $message,
            "take-order leg"
        );
    }};
}

fn truncate_error(value: &str) -> String {
    const MAX_LEN: usize = 512;
    if value.chars().count() <= MAX_LEN {
        value.to_string()
    } else {
        format!("{}...", value.chars().take(MAX_LEN).collect::<String>())
    }
}

fn format_float_for_log(value: rain_math_float::Float) -> String {
    value
        .format()
        .unwrap_or_else(|_| "<format_error>".to_string())
}

fn order_hash_for_leg(leg: &SelectedTakeOrderLeg) -> B256 {
    B256::from(keccak256(leg.candidate.order.abi_encode()))
}

fn log_selected_leg(
    leg_index: usize,
    leg: &SelectedTakeOrderLeg,
    orderbook: alloy::primitives::Address,
    block_number: u64,
    message: &'static str,
) {
    emit_selected_leg!(info, leg_index, leg, orderbook, block_number, message);
}

fn warn_selected_leg(
    leg_index: usize,
    leg: &SelectedTakeOrderLeg,
    orderbook: alloy::primitives::Address,
    block_number: u64,
    message: &'static str,
) {
    emit_selected_leg!(warn, leg_index, leg, orderbook, block_number, message);
}

fn log_selected_legs(
    orderbook: alloy::primitives::Address,
    block_number: u64,
    legs: &[SelectedTakeOrderLeg],
    message: &'static str,
) {
    for (leg_index, leg) in legs.iter().take(MAX_INFO_SELECTED_LEG_LOGS).enumerate() {
        log_selected_leg(leg_index, leg, orderbook, block_number, message);
    }

    let omitted = legs.len().saturating_sub(MAX_INFO_SELECTED_LEG_LOGS);
    if omitted > 0 {
        info!(
            orderbook = %orderbook,
            block_number,
            logged_leg_count = MAX_INFO_SELECTED_LEG_LOGS,
            omitted_leg_count = omitted,
            event = message,
            "omitted additional take-order leg logs"
        );
    }
}

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
    ///   const { calldata, effectivePrice, expectedSell, maxSellCap, prices, raindex } = res.value;
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
        self.get_take_orders_calldata_with_injector(request, &NoopInjector)
            .await
    }
}

impl RaindexClient {
    /// Non-wasm variant of [`Self::get_take_orders_calldata`] that accepts a
    /// caller-supplied [`SignedContextInjector`]. The injector contributes
    /// additional `SignedContextV1` entries appended after any oracle-fetched
    /// contexts for each candidate (composition order: `[oracle..., injected...]`).
    pub async fn get_take_orders_calldata_with_injector(
        &self,
        request: TakeOrdersRequest,
        injector: &dyn SignedContextInjector,
    ) -> Result<TakeOrdersCalldataResult, RaindexError> {
        let span = info_span!(
            "get_take_orders_calldata",
            chain_id = request.chain_id,
            taker = %request.taker,
            sell_token = %request.sell_token,
            buy_token = %request.buy_token,
            mode = ?request.mode,
            amount = %request.amount,
            price_cap = %request.price_cap,
        );
        let result = async move {
            let started_at = Timing::now();
            let req = {
            let step = info_span!("take_orders.request_parsing");
            async {
                let parsed = request::parse_request(&request)?;
                info!(
                    chain_id = request.chain_id,
                    taker = %parsed.taker,
                    sell_token = %parsed.sell_token,
                    buy_token = %parsed.buy_token,
                    mode = ?parsed.mode.mode,
                    amount = %parsed.mode.amount.format().unwrap_or_else(|_| "<format_error>".to_string()),
                    price_cap = %parsed.price_cap.format().unwrap_or_else(|_| "<format_error>".to_string()),
                    "parsed take-orders request"
                );
                Ok::<_, RaindexError>(parsed)
            }
            .instrument(step)
            .await?
        };

        let orders = self
            .fetch_orders_for_pair(request.chain_id, req.sell_token, req.buy_token)
            .instrument(info_span!("take_orders.fetching_orders"))
            .await?;
        let orders_count = orders.len();

        let rpc_urls = {
            let _span = info_span!("take_orders.resolving_rpc_urls").entered();
            let urls = self.get_rpc_urls_for_chain(request.chain_id)?;
            info!(chain_id = request.chain_id, rpc_url_count = urls.len(), "resolved RPC URLs");
            urls
        };
        let rpc_client = RpcClient::new_with_urls(rpc_urls.clone())?;
        let block_number = rpc_client
            .get_latest_block_number()
            .instrument(info_span!("take_orders.fetching_latest_block"))
            .await?;
        info!(chain_id = request.chain_id, block_number, "fetched latest block number");

        let candidates = selection::build_candidates_for_chain(
            &orders,
            req.sell_token,
            req.buy_token,
            Some(block_number),
            None,
            req.taker,
            injector,
        )
        .instrument(info_span!("take_orders.building_candidates"))
        .await?;
        let candidates_count = candidates.len();

        let (best_raindex, best_sim) = {
            let _span = info_span!("take_orders.selecting_best_raindex").entered();
            selection::select_best_raindex_simulation(candidates.clone(), req.mode, req.price_cap)?
        };
        let selected_legs_count = best_sim.legs.len();

        let mut built = {
            let _span = info_span!("take_orders.building_config", raindex = %best_raindex).entered();
            build_take_orders_config_from_simulation(best_sim.clone(), req.mode, req.price_cap)?
                .ok_or(RaindexError::NoLiquidity)?
        };
        info!(
            raindex = %best_raindex,
            legs_count = built.sim.legs.len(),
            orders_count = built.config.orders.len(),
            "built take-orders config"
        );
        log_selected_legs(
            best_raindex,
            block_number,
            &built.sim.legs,
            "selected take-orders candidate leg",
        );

        let approval_params = ApprovalCheckParams {
            rpc_urls: rpc_urls.clone(),
            sell_token: req.sell_token,
            taker: req.taker,
            raindex: best_raindex,
            mode: req.mode,
            price_cap: req.price_cap,
        };

        if let Some(approval_result) = check_approval_needed(&approval_params)
            .instrument(info_span!("take_orders.checking_approval", raindex = %best_raindex))
            .await?
        {
            info!(
                result_type = "approval_required",
                orders_count,
                candidates_count,
                selected_raindex = %best_raindex,
                selected_legs_count,
                block_number,
                preflight_iterations = 0usize,
                removed_orders_count = 0usize,
                duration_ms = started_at.elapsed_ms(),
                "take-orders calldata requires approval"
            );
            return Ok(approval_result);
        }

        let provider =
            mk_read_provider(&rpc_urls).map_err(|e| RaindexError::PreflightError(e.to_string()))?;

        let mut removed_orders_count = 0usize;
        let max_iterations = built.config.orders.len();
        for iteration in 0..max_iterations {
            let iteration_started_at = Timing::now();
            let sim_result = simulate_take_orders(
                &provider,
                best_raindex,
                req.taker,
                &built.config,
                Some(block_number),
            )
            .instrument(info_span!(
                "take_orders.preflight_iteration",
                raindex = %best_raindex,
                taker = %req.taker,
                block_number,
                orders_count = built.config.orders.len(),
                iteration
            ))
            .await;

            match sim_result {
                Ok(()) => {
                    log_selected_legs(
                        best_raindex,
                        block_number,
                        &built.sim.legs,
                        "final take-orders transaction leg",
                    );
                    info!(
                        result_type = "take_orders",
                        orders_count,
                        candidates_count,
                        selected_raindex = %best_raindex,
                        selected_legs_count = built.sim.legs.len(),
                        block_number,
                        preflight_iterations = iteration + 1,
                        removed_orders_count,
                        duration_ms = started_at.elapsed_ms(),
                        "take-orders calldata ready"
                    );
                    return result::build_calldata_result(
                        best_raindex,
                        built,
                        req.mode,
                        req.price_cap,
                    );
                }
                Err(sim_error) => {
                    warn!(
                        raindex = %best_raindex,
                        taker = %req.taker,
                        block_number,
                        orders_count = built.config.orders.len(),
                        iteration,
                        error = %truncate_error(&sim_error),
                        duration_ms = iteration_started_at.elapsed_ms(),
                        "preflight simulation failed"
                    );
                    if let Some(failing_idx) = find_failing_order_index(
                        &provider,
                        best_raindex,
                        req.taker,
                        &built.config,
                        Some(block_number),
                    )
                    .instrument(info_span!(
                        "take_orders.find_failing_order",
                        raindex = %best_raindex,
                        taker = %req.taker,
                        block_number,
                        order_count = built.config.orders.len(),
                        iteration
                    ))
                    .await
                    {
                        if built.config.orders.len() <= 1 {
                            error!(
                                raindex = %best_raindex,
                                iteration,
                                failing_order_index = failing_idx,
                                error = %truncate_error(&sim_error),
                                "all orders failed preflight simulation"
                            );
                            return Err(RaindexError::PreflightError(format!(
                                "All orders failed simulation. Last error: {}",
                                sim_error
                            )));
                        }

                        if let Some(failing_leg) = built.sim.legs.get(failing_idx) {
                            warn_selected_leg(
                                failing_idx,
                                failing_leg,
                                best_raindex,
                                block_number,
                                "removed failing preflight leg",
                            );
                        }

                        built.config.orders.remove(failing_idx);
                        built.sim.legs.remove(failing_idx);
                        removed_orders_count += 1;
                        warn!(
                            raindex = %best_raindex,
                            iteration,
                            failing_order_index = failing_idx,
                            orders_remaining = built.config.orders.len(),
                            removed_orders_count,
                            "removed failing order after preflight"
                        );
                    } else {
                        error!(
                            raindex = %best_raindex,
                            iteration,
                            error = %truncate_error(&sim_error),
                            "preflight failed without identifiable order"
                        );
                        return Err(RaindexError::PreflightError(format!(
                            "Simulation failed but could not identify failing order: {}",
                            sim_error
                        )));
                    }
                }
            }
        }

        error!(
            raindex = %best_raindex,
            max_iterations,
            removed_orders_count,
            "exceeded maximum preflight iterations"
        );
            Err(RaindexError::PreflightError(
                "Exceeded maximum preflight iterations".to_string(),
            ))
        }
        .instrument(span)
        .await;
        result
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
