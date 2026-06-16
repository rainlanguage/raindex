use crate::raindex_client::orders::RaindexOrder;
use crate::raindex_client::RaindexError;
use crate::take_orders::{
    build_take_order_candidates_for_pair, simulate_buy_over_candidates,
    simulate_spend_over_candidates, ParsedTakeOrdersMode, SimulationResult, TakeOrderCandidate,
};
use crate::utils::float::cmp_float;
use alloy::primitives::Address;
use rain_math_float::Float;
use std::collections::HashMap;

pub(crate) async fn build_candidates_for_chain(
    orders: &[RaindexOrder],
    sell_token: Address,
    buy_token: Address,
    block_number: Option<u64>,
    chunk_size: Option<u32>,
) -> Result<Vec<TakeOrderCandidate>, RaindexError> {
    let candidates = build_take_order_candidates_for_pair(
        orders,
        sell_token,
        buy_token,
        block_number,
        chunk_size,
    )
    .await?;
    if candidates.is_empty() {
        return Err(RaindexError::NoLiquidity);
    }
    Ok(candidates)
}

pub(crate) fn worst_price(sim: &SimulationResult) -> Result<Option<Float>, RaindexError> {
    let mut max: Option<Float> = None;
    for leg in &sim.legs {
        let ratio = leg.candidate.ratio;
        match &max {
            None => max = Some(ratio),
            Some(current_max) => {
                if cmp_float(&ratio, current_max)? == std::cmp::Ordering::Greater {
                    max = Some(ratio);
                }
            }
        }
    }
    Ok(max)
}

pub(crate) fn select_best_orderbook_simulation(
    candidates: Vec<TakeOrderCandidate>,
    mode: ParsedTakeOrdersMode,
    price_cap: Float,
) -> Result<(Address, SimulationResult), RaindexError> {
    let mut orderbook_candidates: HashMap<Address, Vec<TakeOrderCandidate>> = HashMap::new();
    for candidate in candidates {
        orderbook_candidates
            .entry(candidate.orderbook)
            .or_default()
            .push(candidate);
    }

    let target = mode.target_amount();
    let is_buy_mode = mode.is_buy_mode();

    let mut best_result: Option<(Address, SimulationResult)> = None;

    for (orderbook, candidates) in orderbook_candidates {
        let sim = if is_buy_mode {
            simulate_buy_over_candidates(candidates, target, price_cap)?
        } else {
            simulate_spend_over_candidates(candidates, target, price_cap)?
        };

        if sim.legs.is_empty() {
            continue;
        }

        let achieved = sim.total_output;

        let is_better = match &best_result {
            None => true,
            Some((best_addr, best_sim)) => {
                let best_achieved = best_sim.total_output;

                if achieved.gt(best_achieved)? {
                    true
                } else if achieved.eq(best_achieved)? {
                    let sim_worst = worst_price(&sim)?;
                    let best_worst = worst_price(best_sim)?;
                    match (sim_worst, best_worst) {
                        (Some(sw), Some(bw)) => match cmp_float(&sw, &bw)? {
                            std::cmp::Ordering::Less => true,
                            std::cmp::Ordering::Equal => orderbook < *best_addr,
                            std::cmp::Ordering::Greater => false,
                        },
                        _ => orderbook < *best_addr,
                    }
                } else {
                    false
                }
            }
        };

        if is_better {
            best_result = Some((orderbook, sim));
        }
    }

    best_result.ok_or(RaindexError::NoLiquidity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::candidates::make_candidate;

    fn high_price_cap() -> Float {
        Float::parse("1000000".to_string()).unwrap()
    }

    fn buy_up_to(amount: Float) -> ParsedTakeOrdersMode {
        ParsedTakeOrdersMode {
            mode: crate::take_orders::TakeOrdersMode::BuyUpTo,
            amount,
        }
    }

    fn spend_up_to(amount: Float) -> ParsedTakeOrdersMode {
        ParsedTakeOrdersMode {
            mode: crate::take_orders::TakeOrdersMode::SpendUpTo,
            amount,
        }
    }

    #[test]
    fn test_select_best_orderbook_single_orderbook() {
        let ob1 = Address::from([0x11u8; 20]);
        let max_output = Float::parse("10".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob1, max_output, ratio);
        let candidates = vec![candidate];
        let buy_target = Float::parse("10".to_string()).unwrap();

        let result =
            select_best_orderbook_simulation(candidates, buy_up_to(buy_target), high_price_cap());

        assert!(result.is_ok());
        let (addr, sim) = result.unwrap();
        assert_eq!(addr, ob1);
        assert!(!sim.legs.is_empty());
        assert!(sim.total_output.gt(Float::zero().unwrap()).unwrap());
        assert!(sim.total_input.gt(Float::zero().unwrap()).unwrap());
    }

    #[test]
    fn test_select_best_orderbook_multiple_books_picks_best() {
        let ob1 = Address::from([0x11u8; 20]);
        let ob2 = Address::from([0x22u8; 20]);

        let ob1_max_output = Float::parse("5".to_string()).unwrap();
        let ob1_ratio = Float::parse("1".to_string()).unwrap();
        let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

        let ob2_max_output = Float::parse("8".to_string()).unwrap();
        let ob2_ratio = Float::parse("1".to_string()).unwrap();
        let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

        let candidates = vec![ob1_candidate, ob2_candidate];
        let buy_target = Float::parse("100".to_string()).unwrap();

        let result =
            select_best_orderbook_simulation(candidates, buy_up_to(buy_target), high_price_cap());

        assert!(result.is_ok());
        let (winner, sim) = result.unwrap();
        assert_eq!(winner, ob2);
        let expected_output = Float::parse("8".to_string()).unwrap();
        assert!(sim.total_output.eq(expected_output).unwrap());
    }

    #[test]
    fn test_select_best_orderbook_skips_empty_sims() {
        let ob1 = Address::from([0x11u8; 20]);
        let ob2 = Address::from([0x22u8; 20]);

        let ob1_max_output = Float::parse("10".to_string()).unwrap();
        let ob1_ratio = Float::parse("2".to_string()).unwrap();
        let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

        let ob2_max_output = Float::parse("5".to_string()).unwrap();
        let ob2_ratio = Float::parse("1".to_string()).unwrap();
        let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

        let candidates = vec![ob1_candidate, ob2_candidate];
        let buy_target = Float::zero().unwrap();

        let result =
            select_best_orderbook_simulation(candidates, buy_up_to(buy_target), high_price_cap());

        assert!(matches!(result, Err(RaindexError::NoLiquidity)));
    }

    #[test]
    fn test_select_best_orderbook_all_empty_returns_no_liquidity() {
        let ob1 = Address::from([0x11u8; 20]);
        let ob2 = Address::from([0x22u8; 20]);

        let ob1_max_output = Float::parse("10".to_string()).unwrap();
        let ob1_ratio = Float::parse("2".to_string()).unwrap();
        let ob1_candidate = make_candidate(ob1, ob1_max_output, ob1_ratio);

        let ob2_max_output = Float::parse("5".to_string()).unwrap();
        let ob2_ratio = Float::parse("1".to_string()).unwrap();
        let ob2_candidate = make_candidate(ob2, ob2_max_output, ob2_ratio);

        let candidates = vec![ob1_candidate, ob2_candidate];
        let buy_target = Float::zero().unwrap();

        let result =
            select_best_orderbook_simulation(candidates, buy_up_to(buy_target), high_price_cap());

        assert!(result.is_err());
        assert!(matches!(result, Err(RaindexError::NoLiquidity)));
    }

    #[test]
    fn test_select_best_orderbook_price_cap_filters_expensive() {
        let ob_expensive = Address::from([0x11u8; 20]);
        let ob_cheap = Address::from([0x22u8; 20]);

        let expensive_max_output = Float::parse("100".to_string()).unwrap();
        let expensive_ratio = Float::parse("5".to_string()).unwrap();
        let expensive_candidate =
            make_candidate(ob_expensive, expensive_max_output, expensive_ratio);

        let cheap_max_output = Float::parse("50".to_string()).unwrap();
        let cheap_ratio = Float::parse("1".to_string()).unwrap();
        let cheap_candidate = make_candidate(ob_cheap, cheap_max_output, cheap_ratio);

        let candidates = vec![expensive_candidate, cheap_candidate];
        let buy_target = Float::parse("100".to_string()).unwrap();
        let price_cap = Float::parse("2".to_string()).unwrap();

        let result = select_best_orderbook_simulation(candidates, buy_up_to(buy_target), price_cap);

        assert!(result.is_ok());
        let (winner, sim) = result.unwrap();
        assert_eq!(
            winner, ob_cheap,
            "Should pick the cheap orderbook since expensive is filtered by price cap"
        );
        let expected_output = Float::parse("50".to_string()).unwrap();
        assert!(sim.total_output.eq(expected_output).unwrap());
    }

    #[test]
    fn test_select_best_orderbook_tiebreak_identical_totals_prefers_lower_address() {
        let ob_higher = Address::from([0x22u8; 20]);
        let ob_lower = Address::from([0x11u8; 20]);

        let max_output = Float::parse("10".to_string()).unwrap();
        let ratio = Float::parse("1".to_string()).unwrap();

        let higher_candidate = make_candidate(ob_higher, max_output, ratio);
        let lower_candidate = make_candidate(ob_lower, max_output, ratio);

        let buy_target = Float::parse("10".to_string()).unwrap();

        for _ in 0..20 {
            let candidates = vec![higher_candidate.clone(), lower_candidate.clone()];
            let result = select_best_orderbook_simulation(
                candidates,
                buy_up_to(buy_target),
                high_price_cap(),
            );
            assert!(result.is_ok());
            let (winner, sim) = result.unwrap();

            assert_eq!(
                winner, ob_lower,
                "Tie-break rule: when total_output amounts and worst prices are equal, \
                 prefer the lower orderbook address (0x{:x} < 0x{:x})",
                ob_lower, ob_higher
            );
            assert_eq!(sim.legs.len(), 1);
            assert_eq!(sim.legs[0].candidate.orderbook, ob_lower);
        }
    }

    #[test]
    fn test_select_best_orderbook_tiebreak_identical_totals_prefers_lower_worst_price() {
        let ob_better_price = Address::from([0x22u8; 20]);
        let ob_worse_price = Address::from([0x11u8; 20]);

        let max_output = Float::parse("10".to_string()).unwrap();
        let better_ratio = Float::parse("0.9".to_string()).unwrap();
        let worse_ratio = Float::parse("1.1".to_string()).unwrap();

        let better_candidate = make_candidate(ob_better_price, max_output, better_ratio);
        let worse_candidate = make_candidate(ob_worse_price, max_output, worse_ratio);

        let buy_target = Float::parse("10".to_string()).unwrap();

        for _ in 0..20 {
            let candidates = vec![worse_candidate.clone(), better_candidate.clone()];
            let result = select_best_orderbook_simulation(
                candidates,
                buy_up_to(buy_target),
                high_price_cap(),
            );
            assert!(result.is_ok());
            let (winner, sim) = result.unwrap();

            assert_eq!(
                winner, ob_better_price,
                "Tie-break rule: when total_output amounts are equal, \
                 prefer the orderbook with the lower worst price (ratio 0.9 < 1.1)"
            );
            assert_eq!(sim.legs.len(), 1);
            assert_eq!(sim.legs[0].candidate.orderbook, ob_better_price);
        }
    }

    #[test]
    fn test_select_best_orderbook_spend_mode() {
        let ob1 = Address::from([0x11u8; 20]);
        let max_output = Float::parse("100".to_string()).unwrap();
        let ratio = Float::parse("2".to_string()).unwrap();
        let candidate = make_candidate(ob1, max_output, ratio);
        let candidates = vec![candidate];
        let spend_budget = Float::parse("100".to_string()).unwrap();

        let result = select_best_orderbook_simulation(
            candidates,
            spend_up_to(spend_budget),
            high_price_cap(),
        );

        assert!(result.is_ok());
        let (addr, sim) = result.unwrap();
        assert_eq!(addr, ob1);
        assert!(!sim.legs.is_empty());
        assert!(sim.total_input.eq(spend_budget).unwrap());
        let expected_output = Float::parse("50".to_string()).unwrap();
        assert!(sim.total_output.eq(expected_output).unwrap());
    }

    #[test]
    fn test_select_best_orderbook_spend_mode_prefers_higher_output() {
        let ob_bad_rate = Address::from([0x11u8; 20]);
        let ob_good_rate = Address::from([0x22u8; 20]);

        let bad_rate_max_output = Float::parse("50".to_string()).unwrap();
        let bad_rate_ratio = Float::parse("2".to_string()).unwrap();
        let bad_rate_candidate = make_candidate(ob_bad_rate, bad_rate_max_output, bad_rate_ratio);

        let good_rate_max_output = Float::parse("90".to_string()).unwrap();
        let good_rate_ratio = Float::parse("1".to_string()).unwrap();
        let good_rate_candidate =
            make_candidate(ob_good_rate, good_rate_max_output, good_rate_ratio);

        let candidates = vec![bad_rate_candidate, good_rate_candidate];
        let spend_budget = Float::parse("100".to_string()).unwrap();

        let result = select_best_orderbook_simulation(
            candidates,
            spend_up_to(spend_budget),
            high_price_cap(),
        );

        assert!(result.is_ok());
        let (winner, sim) = result.unwrap();
        assert_eq!(
            winner, ob_good_rate,
            "Should pick orderbook with higher output (90) over one that can absorb more input but yields less output (50)"
        );
        let expected_output = Float::parse("90".to_string()).unwrap();
        assert!(sim.total_output.eq(expected_output).unwrap());
    }
}
