use crate::raindex_client::order_quotes::{get_order_quotes_batch, RaindexOrderQuote};
use crate::raindex_client::orders::RaindexOrder;
use crate::raindex_client::RaindexError;
use alloy::primitives::Address;
use rain_math_float::Float;
use rain_orderbook_bindings::IRaindexV6::{OrderV4, SignedContextV1};
#[cfg(target_family = "wasm")]
use std::str::FromStr;

fn indices_in_bounds(order: &OrderV4, input_index: u32, output_index: u32) -> bool {
    (input_index as usize) < order.validInputs.len()
        && (output_index as usize) < order.validOutputs.len()
}

fn matches_direction(
    order: &OrderV4,
    input_index: u32,
    output_index: u32,
    input_token: Address,
    output_token: Address,
) -> bool {
    let order_input_token = order.validInputs[input_index as usize].token;
    let order_output_token = order.validOutputs[output_index as usize].token;
    order_input_token == input_token && order_output_token == output_token
}

fn has_capacity(
    data: &crate::raindex_client::order_quotes::RaindexOrderQuoteValue,
) -> Result<bool, RaindexError> {
    Ok(data.max_output.gt(Float::zero()?)?)
}

#[derive(Clone, Debug)]
pub struct TakeOrderCandidate {
    pub orderbook: Address,
    pub order: OrderV4,
    pub input_io_index: u32,
    pub output_io_index: u32,
    pub max_output: Float,
    pub ratio: Float,
    /// Signed context data fetched from the order's oracle endpoint (if any).
    pub signed_context: Vec<SignedContextV1>,
}

fn get_orderbook_address(order: &RaindexOrder) -> Result<Address, RaindexError> {
    #[cfg(target_family = "wasm")]
    {
        Ok(Address::from_str(&order.orderbook())?)
    }
    #[cfg(not(target_family = "wasm"))]
    {
        Ok(order.orderbook())
    }
}

pub async fn build_take_order_candidates_for_pair(
    orders: &[RaindexOrder],
    input_token: Address,
    output_token: Address,
    block_number: Option<u64>,
    chunk_size: Option<u32>,
) -> Result<Vec<TakeOrderCandidate>, RaindexError> {
    let all_quotes = get_order_quotes_batch(orders, block_number, chunk_size).await?;

    let mut all_candidates = vec![];
    for (order, quotes) in orders.iter().zip(all_quotes) {
        let order_v4: OrderV4 = order.try_into()?;
        let orderbook = get_orderbook_address(order)?;
        let oracle_url = order.oracle_url();

        for quote in &quotes {
            let signed_context = match &oracle_url {
                Some(url) => {
                    fetch_oracle_for_pair(
                        url,
                        &order_v4,
                        quote.pair.input_index,
                        quote.pair.output_index,
                        // Counterparty is unknown at candidate-building time (the taker
                        // address isn't known until the transaction is submitted).
                        // The oracle server ignores this field — it only uses the order's
                        // input/output tokens to determine the price pair.
                        Address::ZERO,
                    )
                    .await?
                }
                None => vec![],
            };

            if let Some(candidate) = try_build_candidate(
                orderbook,
                &order_v4,
                quote,
                input_token,
                output_token,
                signed_context,
            )? {
                all_candidates.push(candidate);
            }
        }
    }

    Ok(all_candidates)
}

/// Fetch signed context from an order's oracle endpoint for a specific IO pair.
async fn fetch_oracle_for_pair(
    oracle_url: &str,
    order: &OrderV4,
    input_io_index: u32,
    output_io_index: u32,
    counterparty: Address,
) -> Result<Vec<SignedContextV1>, RaindexError> {
    let body =
        crate::oracle::encode_oracle_body(order, input_io_index, output_io_index, counterparty);
    match crate::oracle::fetch_signed_context(oracle_url, body).await {
        Ok(ctx) => Ok(vec![ctx]),
        Err(e) => Err(RaindexError::OracleFetchError(format!(
            "Oracle fetch failed for pair ({}, {}): {}",
            input_io_index, output_io_index, e
        ))),
    }
}

fn try_build_candidate(
    orderbook: Address,
    order: &OrderV4,
    quote: &RaindexOrderQuote,
    input_token: Address,
    output_token: Address,
    signed_context: Vec<SignedContextV1>,
) -> Result<Option<TakeOrderCandidate>, RaindexError> {
    let data = match (quote.success, &quote.data) {
        (true, Some(d)) => d,
        _ => return Ok(None),
    };

    let input_io_index = quote.pair.input_index;
    let output_io_index = quote.pair.output_index;

    if !indices_in_bounds(order, input_io_index, output_io_index) {
        return Ok(None);
    }

    if !matches_direction(
        order,
        input_io_index,
        output_io_index,
        input_token,
        output_token,
    ) {
        return Ok(None);
    }

    if !has_capacity(data)? {
        return Ok(None);
    }

    Ok(Some(TakeOrderCandidate {
        orderbook,
        order: order.clone(),
        input_io_index,
        output_io_index,
        max_output: data.max_output,
        ratio: data.ratio,
        signed_context,
    }))
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;
    use crate::test_helpers::orders::make_basic_order;
    use crate::test_helpers::quotes::{make_quote, make_quote_value};
    use alloy::primitives::Address;
    use rain_math_float::Float;

    #[test]
    fn test_indices_in_bounds_valid() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(indices_in_bounds(&order, 0, 0));
    }

    #[test]
    fn test_indices_in_bounds_input_out_of_bounds() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!indices_in_bounds(&order, 99, 0));
    }

    #[test]
    fn test_indices_in_bounds_output_out_of_bounds() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!indices_in_bounds(&order, 0, 99));
    }

    #[test]
    fn test_indices_in_bounds_both_out_of_bounds() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!indices_in_bounds(&order, 99, 99));
    }

    #[test]
    fn test_matches_direction_correct() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(matches_direction(&order, 0, 0, token_a, token_b));
    }

    #[test]
    fn test_matches_direction_wrong_input() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let token_c = Address::from([6u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!matches_direction(&order, 0, 0, token_c, token_b));
    }

    #[test]
    fn test_matches_direction_wrong_output() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let token_c = Address::from([6u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!matches_direction(&order, 0, 0, token_a, token_c));
    }

    #[test]
    fn test_matches_direction_reversed() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let order = make_basic_order(token_a, token_b);
        assert!(!matches_direction(&order, 0, 0, token_b, token_a));
    }

    fn make_quote_value_for_capacity_test(
        max_output: Float,
    ) -> crate::raindex_client::order_quotes::RaindexOrderQuoteValue {
        let zero = Float::zero().unwrap();
        crate::raindex_client::order_quotes::RaindexOrderQuoteValue {
            max_output,
            formatted_max_output: "0".to_string(),
            max_input: zero,
            formatted_max_input: "0".to_string(),
            ratio: zero,
            formatted_ratio: "0".to_string(),
            inverse_ratio: zero,
            formatted_inverse_ratio: "0".to_string(),
        }
    }

    #[test]
    fn test_has_capacity_positive() {
        let positive = Float::parse("100".to_string()).unwrap();
        let data = make_quote_value_for_capacity_test(positive);
        assert!(has_capacity(&data).unwrap());
    }

    #[test]
    fn test_has_capacity_zero() {
        let zero = Float::zero().unwrap();
        let data = make_quote_value_for_capacity_test(zero);
        assert!(!has_capacity(&data).unwrap());
    }

    #[test]
    fn test_has_capacity_negative() {
        let negative = Float::parse("-1".to_string()).unwrap();
        let data = make_quote_value_for_capacity_test(negative);
        assert!(!has_capacity(&data).unwrap());
    }

    #[test]
    fn test_try_build_candidate_wrong_direction() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let f1 = Float::parse("1".to_string()).unwrap();
        let quote = make_quote(0, 0, Some(make_quote_value(f1, f1, f1)), true);

        let result =
            try_build_candidate(orderbook, &order, &quote, token_b, token_a, vec![]).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_try_build_candidate_zero_capacity() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let zero = Float::zero().unwrap();
        let f1 = Float::parse("1".to_string()).unwrap();
        let quote = make_quote(0, 0, Some(make_quote_value(zero, zero, f1)), true);

        let result =
            try_build_candidate(orderbook, &order, &quote, token_a, token_b, vec![]).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_try_build_candidate_success() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let f1 = Float::parse("1".to_string()).unwrap();
        let f2 = Float::parse("2".to_string()).unwrap();
        let quote = make_quote(0, 0, Some(make_quote_value(f2, f1, f1)), true);

        let result =
            try_build_candidate(orderbook, &order, &quote, token_a, token_b, vec![]).unwrap();

        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.orderbook, orderbook);
        assert_eq!(candidate.input_io_index, 0);
        assert_eq!(candidate.output_io_index, 0);
        assert!(candidate.max_output.eq(f2).unwrap());
    }

    #[test]
    fn test_try_build_candidate_failed_quote() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let quote = make_quote(0, 0, None, false);

        let result = try_build_candidate(orderbook, &order, &quote, token_a, token_b, vec![]);

        assert!(
            result.is_ok(),
            "Failed quote must not cause an error, got: {:?}",
            result.unwrap_err()
        );
        assert!(
            result.unwrap().is_none(),
            "Failed quote must not produce a candidate"
        );
    }

    #[test]
    fn test_try_build_candidate_out_of_bounds_indices() {
        let token_a = Address::from([4u8; 20]);
        let token_b = Address::from([5u8; 20]);
        let orderbook = Address::from([0xAAu8; 20]);

        let order = make_basic_order(token_a, token_b);
        let f1 = Float::parse("1".to_string()).unwrap();

        let quote_bad_input_index = make_quote(99, 0, Some(make_quote_value(f1, f1, f1)), true);
        let result = try_build_candidate(
            orderbook,
            &order,
            &quote_bad_input_index,
            token_a,
            token_b,
            vec![],
        );
        assert!(
            result.is_ok(),
            "Out-of-bounds input index must not cause an error"
        );
        assert!(
            result.unwrap().is_none(),
            "Out-of-bounds input index must not produce a candidate"
        );

        let quote_bad_output_index = make_quote(0, 99, Some(make_quote_value(f1, f1, f1)), true);
        let result = try_build_candidate(
            orderbook,
            &order,
            &quote_bad_output_index,
            token_a,
            token_b,
            vec![],
        );
        assert!(
            result.is_ok(),
            "Out-of-bounds output index must not cause an error"
        );
        assert!(
            result.unwrap().is_none(),
            "Out-of-bounds output index must not produce a candidate"
        );
    }
}
