use crate::transaction::{read_call, TransactionArgsError};
use alloy::primitives::{Address, U256};
use raindex_bindings::IERC20::allowanceCall;

/// Reads an ERC20 allowance.
///
/// An allowance read needs only the token being queried, the `owner` whose
/// allowance is read, the `spender` it is read for, and the RPCs to read from.
/// It deliberately takes no deposit context (amount, vault id, decimals): those
/// are unrelated to reading an allowance, so approval/allowance callers don't
/// have to construct them.
pub async fn read_allowance(
    rpcs: &[String],
    token: Address,
    owner: Address,
    spender: Address,
) -> Result<U256, TransactionArgsError> {
    read_call(rpcs, token, allowanceCall { owner, spender }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::hex::encode_prefixed;
    use alloy::sol_types::SolCall;
    use httpmock::MockServer;
    use serde_json::json;
    use std::str::FromStr;

    // The whole point of #1979's decoupling: reading an allowance needs ONLY the
    // token, owner, spender and RPCs - there is no deposit context (amount, vault
    // id, decimals) in this function's signature, so an approval/allowance caller
    // can't even see, let alone construct, `DepositArgs`.
    //
    // The RPC mock answers ONLY a well-formed `allowance(owner, spender)`
    // `eth_call` against the given token, returning a distinct value. If
    // `read_allowance` queried the wrong token/owner/spender, or somehow smuggled
    // deposit fields into the read, the mock wouldn't match and the call would
    // error - so a green assertion proves the deposit-free read targets exactly
    // the right ERC20 allowance.
    #[tokio::test]
    async fn test_read_allowance_uses_token_owner_spender_only() {
        let token = Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap();
        let owner = Address::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let spender = Address::from_str("0x0000000000000000000000000000000000000002").unwrap();

        let expected_calldata = encode_prefixed(allowanceCall { owner, spender }.abi_encode());
        // ERC20 allowance(address,address) selector.
        assert!(expected_calldata.starts_with("0xdd62ed3e"));

        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.path("/rpc")
                // The token contract being read.
                .body_contains(&encode_prefixed(token.as_slice())[2..])
                // allowance(owner, spender) calldata.
                .body_contains(&expected_calldata);
            then.status(200).json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x000000000000000000000000000000000000000000000000000000000000002a",
            }));
        });

        let rpcs = vec![server.url("/rpc")];
        // Note: no amount, no vault id, no decimals - no `DepositArgs` anywhere.
        let allowance = read_allowance(&rpcs, token, owner, spender).await.unwrap();
        assert_eq!(allowance, U256::from(42));
    }

    // A second, distinct value rules out a refactor that hard-codes/short-circuits
    // a constant rather than actually reading the on-chain allowance.
    #[tokio::test]
    async fn test_read_allowance_returns_actual_value() {
        let token = Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap();
        let owner = Address::from_str("0x0000000000000000000000000000000000000003").unwrap();
        let spender = Address::from_str("0x0000000000000000000000000000000000000004").unwrap();

        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.path("/rpc").body_contains(encode_prefixed(
                allowanceCall { owner, spender }.abi_encode(),
            ));
            then.status(200).json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            }));
        });

        let rpcs = vec![server.url("/rpc")];
        let allowance = read_allowance(&rpcs, token, owner, spender).await.unwrap();
        assert_eq!(allowance, U256::MAX);
    }
}
