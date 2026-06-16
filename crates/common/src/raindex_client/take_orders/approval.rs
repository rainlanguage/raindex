use crate::erc20::ERC20;
use crate::raindex_client::RaindexError;
use crate::take_orders::{check_taker_allowance, ParsedTakeOrdersMode};
use alloy::primitives::Address;
use rain_math_float::Float;
use std::ops::Mul;
use url::Url;

use super::result::build_approval_result;
use super::TakeOrdersCalldataResult;

pub struct ApprovalCheckParams {
    pub rpc_urls: Vec<Url>,
    pub sell_token: Address,
    pub taker: Address,
    pub orderbook: Address,
    pub mode: ParsedTakeOrdersMode,
    pub price_cap: Float,
}

pub async fn check_approval_needed(
    params: &ApprovalCheckParams,
) -> Result<Option<TakeOrdersCalldataResult>, RaindexError> {
    let max_sell_cap = calculate_max_sell_cap(params.mode, params.price_cap)?;

    let erc20 = ERC20::new(params.rpc_urls.clone(), params.sell_token);
    let decimals = erc20.decimals().await?;
    let required_u256 = max_sell_cap.to_fixed_decimal_lossy(decimals)?.0;

    let allowance_result =
        check_taker_allowance(&erc20, params.taker, params.orderbook, required_u256)
            .await
            .map_err(|e| RaindexError::PreflightError(e.to_string()))?;

    if allowance_result.needs_approval {
        Ok(Some(build_approval_result(
            params.sell_token,
            params.orderbook,
            max_sell_cap,
            decimals,
        )?))
    } else {
        Ok(None)
    }
}

fn calculate_max_sell_cap(
    mode: ParsedTakeOrdersMode,
    price_cap: Float,
) -> Result<Float, RaindexError> {
    if mode.is_buy_mode() {
        Ok(mode.target_amount().mul(price_cap)?)
    } else {
        Ok(mode.target_amount())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::take_orders::TakeOrdersMode;

    fn make_mode(mode: TakeOrdersMode, amount: &str) -> ParsedTakeOrdersMode {
        ParsedTakeOrdersMode {
            mode,
            amount: Float::parse(amount.to_string()).unwrap(),
        }
    }

    #[test]
    fn test_calculate_max_sell_cap_buy_mode() {
        let price_cap = Float::parse("2.5".to_string()).unwrap();
        let mode = make_mode(TakeOrdersMode::BuyUpTo, "100");

        let result = calculate_max_sell_cap(mode, price_cap).unwrap();
        let expected = Float::parse("250".to_string()).unwrap();

        assert!(
            result.eq(expected).unwrap(),
            "max_sell_cap should be amount * price_cap = 100 * 2.5 = 250, got: {:?}",
            result.format()
        );
    }

    #[test]
    fn test_calculate_max_sell_cap_buy_exact_mode() {
        let price_cap = Float::parse("3".to_string()).unwrap();
        let mode = make_mode(TakeOrdersMode::BuyExact, "50");

        let result = calculate_max_sell_cap(mode, price_cap).unwrap();
        let expected = Float::parse("150".to_string()).unwrap();

        assert!(
            result.eq(expected).unwrap(),
            "max_sell_cap should be amount * price_cap = 50 * 3 = 150, got: {:?}",
            result.format()
        );
    }

    #[test]
    fn test_calculate_max_sell_cap_spend_mode() {
        let price_cap = Float::parse("10".to_string()).unwrap();
        let mode = make_mode(TakeOrdersMode::SpendUpTo, "200");

        let result = calculate_max_sell_cap(mode, price_cap).unwrap();
        let expected = Float::parse("200".to_string()).unwrap();

        assert!(
            result.eq(expected).unwrap(),
            "max_sell_cap in spend mode should equal amount = 200, got: {:?}",
            result.format()
        );
    }

    #[test]
    fn test_calculate_max_sell_cap_spend_exact_mode() {
        let price_cap = Float::parse("5".to_string()).unwrap();
        let mode = make_mode(TakeOrdersMode::SpendExact, "75");

        let result = calculate_max_sell_cap(mode, price_cap).unwrap();
        let expected = Float::parse("75".to_string()).unwrap();

        assert!(
            result.eq(expected).unwrap(),
            "max_sell_cap in spend exact mode should equal amount = 75, got: {:?}",
            result.format()
        );
    }

    #[test]
    fn test_calculate_max_sell_cap_high_price_cap() {
        let price_cap = Float::parse("1000000".to_string()).unwrap();
        let mode = make_mode(TakeOrdersMode::BuyUpTo, "100");

        let result = calculate_max_sell_cap(mode, price_cap).unwrap();
        let expected = Float::parse("100000000".to_string()).unwrap();

        assert!(
            result.eq(expected).unwrap(),
            "max_sell_cap should be 100 * 1000000 = 100000000, got: {:?}",
            result.format()
        );
    }

    #[test]
    fn test_calculate_max_sell_cap_fractional_price_cap() {
        let price_cap = Float::parse("0.5".to_string()).unwrap();
        let mode = make_mode(TakeOrdersMode::BuyUpTo, "100");

        let result = calculate_max_sell_cap(mode, price_cap).unwrap();
        let expected = Float::parse("50".to_string()).unwrap();

        assert!(
            result.eq(expected).unwrap(),
            "max_sell_cap should be 100 * 0.5 = 50, got: {:?}",
            result.format()
        );
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod local_evm_tests {
    use super::*;
    use crate::take_orders::TakeOrdersMode;
    use alloy::primitives::U256;
    use rain_orderbook_test_fixtures::LocalEvm;
    use url::Url;

    fn make_mode(mode: TakeOrdersMode, amount: &str) -> ParsedTakeOrdersMode {
        ParsedTakeOrdersMode {
            mode,
            amount: Float::parse(amount.to_string()).unwrap(),
        }
    }

    #[tokio::test]
    async fn test_check_approval_needed_insufficient_allowance_returns_approval() {
        let mut local_evm = LocalEvm::new().await;
        let owner = local_evm.signer_wallets[0].default_signer().address();
        let taker = local_evm.signer_wallets[1].default_signer().address();
        let token = local_evm
            .deploy_new_token("TestToken", "TT", 18, U256::MAX, owner)
            .await;
        let orderbook = *local_evm.orderbook.address();

        token
            .transfer(
                taker,
                U256::from(1000u64) * U256::from(10).pow(U256::from(18)),
            )
            .from(owner)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();

        let rpc_url = Url::parse(&local_evm.url().to_string()).unwrap();

        let params = ApprovalCheckParams {
            rpc_urls: vec![rpc_url],
            sell_token: *token.address(),
            taker,
            orderbook,
            mode: make_mode(TakeOrdersMode::SpendUpTo, "100"),
            price_cap: Float::parse("2".to_string()).unwrap(),
        };

        let result = check_approval_needed(&params).await.unwrap();

        assert!(
            result.is_some(),
            "Should return approval result when allowance is insufficient"
        );
        let approval_result = result.unwrap();
        assert!(
            approval_result.is_needs_approval(),
            "Expected NeedsApproval variant"
        );
        let approval_info = approval_result.approval_info().unwrap();

        assert_eq!(
            approval_info.token(),
            *token.address(),
            "token should match"
        );
        assert_eq!(
            approval_info.spender(),
            orderbook,
            "spender should be orderbook"
        );
        assert!(
            !approval_info.calldata().is_empty(),
            "calldata should not be empty"
        );
    }

    #[tokio::test]
    async fn test_check_approval_needed_sufficient_allowance_returns_none() {
        let mut local_evm = LocalEvm::new().await;
        let owner = local_evm.signer_wallets[0].default_signer().address();
        let taker = local_evm.signer_wallets[1].default_signer().address();
        let token = local_evm
            .deploy_new_token("TestToken", "TT", 18, U256::MAX, owner)
            .await;
        let orderbook = *local_evm.orderbook.address();

        token
            .transfer(
                taker,
                U256::from(1000u64) * U256::from(10).pow(U256::from(18)),
            )
            .from(owner)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();

        token
            .approve(orderbook, U256::MAX)
            .from(taker)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();

        let rpc_url = Url::parse(&local_evm.url().to_string()).unwrap();

        let params = ApprovalCheckParams {
            rpc_urls: vec![rpc_url],
            sell_token: *token.address(),
            taker,
            orderbook,
            mode: make_mode(TakeOrdersMode::SpendUpTo, "100"),
            price_cap: Float::parse("2".to_string()).unwrap(),
        };

        let result = check_approval_needed(&params).await.unwrap();

        assert!(
            result.is_none(),
            "Should return None when allowance is sufficient"
        );
    }

    #[tokio::test]
    async fn test_check_approval_needed_excess_decimal_precision() {
        let mut local_evm = LocalEvm::new().await;
        let owner = local_evm.signer_wallets[0].default_signer().address();
        let taker = local_evm.signer_wallets[1].default_signer().address();
        let token = local_evm
            .deploy_new_token("USDC", "USDC", 6, U256::MAX, owner)
            .await;
        let orderbook = *local_evm.orderbook.address();

        token
            .transfer(
                taker,
                U256::from(1000u64) * U256::from(10).pow(U256::from(6)),
            )
            .from(owner)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();

        let rpc_url = Url::parse(&local_evm.url().to_string()).unwrap();

        let params = ApprovalCheckParams {
            rpc_urls: vec![rpc_url],
            sell_token: *token.address(),
            taker,
            orderbook,
            mode: make_mode(TakeOrdersMode::SpendUpTo, "1.57126799999999998"),
            price_cap: Float::parse("1".to_string()).unwrap(),
        };

        let result = check_approval_needed(&params).await;
        assert!(
            result.is_ok(),
            "Should not error on excess decimal precision: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_check_approval_needed_buy_mode_uses_max_sell_cap() {
        let mut local_evm = LocalEvm::new().await;
        let owner = local_evm.signer_wallets[0].default_signer().address();
        let taker = local_evm.signer_wallets[1].default_signer().address();
        let token = local_evm
            .deploy_new_token("TestToken", "TT", 18, U256::MAX, owner)
            .await;
        let orderbook = *local_evm.orderbook.address();

        token
            .transfer(
                taker,
                U256::from(1000u64) * U256::from(10).pow(U256::from(18)),
            )
            .from(owner)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();

        token
            .approve(
                orderbook,
                U256::from(150u64) * U256::from(10).pow(U256::from(18)),
            )
            .from(taker)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap();

        let rpc_url = Url::parse(&local_evm.url().to_string()).unwrap();

        let params = ApprovalCheckParams {
            rpc_urls: vec![rpc_url],
            sell_token: *token.address(),
            taker,
            orderbook,
            mode: make_mode(TakeOrdersMode::BuyUpTo, "100"),
            price_cap: Float::parse("2".to_string()).unwrap(),
        };

        let result = check_approval_needed(&params).await.unwrap();

        assert!(
            result.is_some(),
            "Should return approval result when allowance (150) < max_sell_cap (200)"
        );
    }
}
