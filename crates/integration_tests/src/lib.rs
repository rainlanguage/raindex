#[cfg(test)]
mod tests {
    use alloy::primitives::B256;
    use alloy::serde::WithOtherFields;
    use alloy::sol_types::SolCall;
    use alloy::{
        network::TransactionBuilder,
        primitives::{utils::parse_ether, U256},
        rpc::types::TransactionRequest,
    };
    use rain_math_float::Float;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use rain_orderbook_test_fixtures::{LocalEvm, Orderbook::QuoteV2};

    #[tokio::test]
    async fn test_post_task_set() {
        let local_evm = LocalEvm::new_with_tokens(2).await;

        let orderbook = &local_evm.orderbook;

        let token1_holder = local_evm.signer_wallets[0].default_signer().address();

        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    some-key:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
rainlangs:
    some-key:
        address: {rainlang_address}
tokens:
    eth:
        network: some-key
        address: {token2}
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: some-key
        address: {token1}
        decimals: 18
        label: Dai
        symbol: DAI
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: eth
        outputs:
            - token: dai
              vault-id: 0x01
scenarios:
    some-key:
        rainlang: some-key
        bindings:
            key1: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key1 !Test binding
#calculate-io
using-words-from {orderbook_subparser}
amount price: get("amount") 52;
#handle-add-order
:set("amount" 100);
#handle-io
:;
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            orderbook_subparser = local_evm.orderbook_subparser.address(),
            rainlang_address = local_evm.rainlang,
            token1 = token1.address(),
            token2 = token2.address(),
            spec_version = SpecVersion::current()
        );

        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment, None)
            .await
            .unwrap()
            .try_into_call(vec![local_evm.url()])
            .await
            .unwrap()
            .abi_encode();

        let order = local_evm
            .add_order_and_deposit(
                &calldata,
                token1_holder,
                *token1.address(),
                parse_ether("1000").unwrap(),
                18,
                B256::from(U256::from(1)),
            )
            .await
            .0
            .order;

        let quote = local_evm
            .call_contract(orderbook.quote2(QuoteV2 {
                order,
                inputIOIndex: U256::from(0),
                outputIOIndex: U256::from(0),
                signedContext: vec![],
            }))
            .await
            .unwrap()
            .unwrap();

        let amount = Float::from_raw(quote._1);
        let price = Float::from_raw(quote._2);

        let expected_amount = Float::parse("100".to_string()).unwrap();
        let expected_price = Float::parse("52".to_string()).unwrap();

        assert!(amount.eq(expected_amount).unwrap());
        assert!(price.eq(expected_price).unwrap());
    }

    #[tokio::test]
    async fn test_post_task_revert() {
        let local_evm = LocalEvm::new().await;
        let orderbook = &local_evm.orderbook;

        let dotrain = format!(
            r#"
version: {spec_version}
networks:
    some-key:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
rainlangs:
    some-key:
        address: {rainlang_address}
tokens:
    eth:
        network: some-key
        address: 0xabc0000000000000000000000000000000000003
        decimals: 18
        label: Ethereum
        symbol: ETH
    dai:
        network: some-key
        address: 0xabc0000000000000000000000000000000000004
        decimals: 18
        label: Dai
        symbol: DAI
orderbook:
    some-key:
        address: {orderbook}
orders:
    some-key:
        inputs:
            - token: eth
            - token: dai
        outputs:
            - token: dai
scenarios:
    some-key:
        rainlang: some-key
        bindings:
            key1: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key1 !Test binding
#calculate-io
amount price: get("amount") 52;
#handle-io
:;
#handle-add-order
:ensure(0 "should fail");
"#,
            rpc_url = local_evm.url(),
            orderbook = orderbook.address(),
            rainlang_address = local_evm.rainlang,
            spec_version = SpecVersion::current(),
        );

        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment, None)
            .await
            .unwrap()
            .try_into_call(vec![local_evm.url()])
            .await
            .unwrap()
            .abi_encode();
        let tx = TransactionRequest::default()
            .with_input(calldata)
            .with_to(*orderbook.address());

        let res = local_evm
            .send_transaction(WithOtherFields::new(tx))
            .await
            .expect_err("Transaction should have reverted");

        assert!(res.to_string().contains("should fail"));
    }
}
