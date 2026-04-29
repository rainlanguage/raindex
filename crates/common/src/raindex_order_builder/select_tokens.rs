use super::*;
use crate::raindex_client::vaults::AccountBalance;
use futures::StreamExt;
use rain_math_float::Float;
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg, network::NetworkCfg, order::OrderCfg,
    order_builder::OrderBuilderSelectTokensCfg, token::TokenCfg, yaml::YamlParsableHash,
};
use std::str::FromStr;

const MAX_CONCURRENT_FETCHES: usize = 5;

impl RaindexOrderBuilder {
    pub fn get_select_tokens(
        &self,
    ) -> Result<Vec<OrderBuilderSelectTokensCfg>, RaindexOrderBuilderError> {
        let select_tokens = OrderBuilderCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        Ok(select_tokens.unwrap_or(vec![]))
    }

    pub fn is_select_token_set(&self, key: String) -> Result<bool, RaindexOrderBuilderError> {
        Ok(self.dotrain_order.orderbook_yaml().get_token(&key).is_ok())
    }

    pub fn check_select_tokens(&self) -> Result<(), RaindexOrderBuilderError> {
        let select_tokens = OrderBuilderCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;

        if let Some(select_tokens) = select_tokens {
            for select_token in select_tokens {
                if self
                    .dotrain_order
                    .orderbook_yaml()
                    .get_token(&select_token.key)
                    .is_err()
                {
                    return Err(RaindexOrderBuilderError::TokenMustBeSelected(
                        select_token.key.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub async fn set_select_token(
        &mut self,
        key: String,
        address: String,
    ) -> Result<(), RaindexOrderBuilderError> {
        let select_tokens = OrderBuilderCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(RaindexOrderBuilderError::SelectTokensNotSet)?;
        if !select_tokens.iter().any(|token| token.key == key) {
            return Err(RaindexOrderBuilderError::TokenNotFound(key.clone()));
        }

        if TokenCfg::parse_from_yaml(self.dotrain_order.dotrain_yaml().documents, &key, None)
            .is_ok()
        {
            TokenCfg::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)?;
        }

        let address = Address::from_str(&address)?;

        let order_key = DeploymentCfg::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        let network_key =
            OrderCfg::parse_network_key(self.dotrain_order.dotrain_yaml().documents, &order_key)?;
        let rpcs =
            NetworkCfg::parse_rpcs(self.dotrain_order.dotrain_yaml().documents, &network_key)?;

        let erc20 = ERC20::new(rpcs, address);
        let token_info = erc20.token_info(None).await?;

        TokenCfg::add_record_to_yaml(
            self.dotrain_order.orderbook_yaml().documents,
            &key,
            &network_key,
            &address.to_string(),
            Some(&token_info.decimals.to_string()),
            Some(&token_info.name),
            Some(&token_info.symbol),
        )?;

        Ok(())
    }

    pub fn unset_select_token(&mut self, key: String) -> Result<(), RaindexOrderBuilderError> {
        let select_tokens = OrderBuilderCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(RaindexOrderBuilderError::SelectTokensNotSet)?;
        if !select_tokens.iter().any(|token| token.key == key) {
            return Err(RaindexOrderBuilderError::TokenNotFound(key.clone()));
        }

        TokenCfg::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)?;

        Ok(())
    }

    pub fn are_all_tokens_selected(&self) -> Result<bool, RaindexOrderBuilderError> {
        let select_tokens = self.get_select_tokens()?;
        for token in select_tokens {
            if !self.is_select_token_set(token.key)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn get_all_tokens(
        &self,
        search: Option<String>,
    ) -> Result<Vec<ExtendedTokenInfo>, RaindexOrderBuilderError> {
        let order_key = DeploymentCfg::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        let network_key =
            OrderCfg::parse_network_key(self.dotrain_order.dotrain_yaml().documents, &order_key)?;
        let tokens = self.dotrain_order.orderbook_yaml().get_tokens()?;

        let mut fetch_futures = Vec::new();

        for (_, token) in tokens
            .into_iter()
            .filter(|(_, token)| token.network.key == network_key)
        {
            fetch_futures.push(async move { ExtendedTokenInfo::from_token_cfg(&token).await });
        }

        let mut results: Vec<ExtendedTokenInfo> = futures::stream::iter(fetch_futures)
            .buffer_unordered(MAX_CONCURRENT_FETCHES)
            .filter_map(|res| async { res.ok() })
            .collect()
            .await;
        results.sort_by(|a, b| {
            a.address
                .to_string()
                .to_lowercase()
                .cmp(&b.address.to_string().to_lowercase())
        });
        results.dedup_by(|a, b| {
            a.address.to_string().to_lowercase() == b.address.to_string().to_lowercase()
        });

        if let Some(search_term) = search {
            if !search_term.is_empty() {
                let search_lower = search_term.to_lowercase();
                results.retain(|token| {
                    token.name.to_lowercase().contains(&search_lower)
                        || token.symbol.to_lowercase().contains(&search_lower)
                        || token
                            .address
                            .to_string()
                            .to_lowercase()
                            .contains(&search_lower)
                });
            }
        }

        Ok(results)
    }

    pub async fn get_account_balance(
        &self,
        token_address: String,
        owner: String,
    ) -> Result<AccountBalance, RaindexOrderBuilderError> {
        let order_key = DeploymentCfg::parse_order_key(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        let network_key =
            OrderCfg::parse_network_key(self.dotrain_order.dotrain_yaml().documents, &order_key)?;
        let network = self
            .dotrain_order
            .orderbook_yaml()
            .get_network(&network_key)?;

        let erc20 = ERC20::new(network.rpcs, Address::from_str(&token_address)?);
        let decimals = erc20.decimals().await?;
        let balance = erc20
            .get_account_balance(Address::from_str(&owner)?)
            .await?;
        let float_balance = Float::from_fixed_decimal(balance, decimals)?;

        Ok(AccountBalance::new(float_balance, float_balance.format()?))
    }
}

#[cfg(test)]
impl RaindexOrderBuilder {
    pub fn add_record_to_yaml(
        &self,
        key: String,
        network_key: String,
        address: String,
        decimals: String,
        label: String,
        symbol: String,
    ) {
        TokenCfg::add_record_to_yaml(
            self.dotrain_order.orderbook_yaml().documents,
            &key,
            &network_key,
            &address,
            Some(&decimals),
            Some(&label),
            Some(&symbol),
        )
        .unwrap();
    }

    pub fn remove_record_from_yaml(&self, key: String) {
        TokenCfg::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::raindex_order_builder::tests::{
        initialize_builder, initialize_builder_with_select_tokens,
    };
    use crate::raindex_order_builder::RaindexOrderBuilderError;

    #[tokio::test]
    async fn test_get_select_tokens() {
        let builder = initialize_builder_with_select_tokens().await;
        let select_tokens = builder.get_select_tokens().unwrap();
        assert_eq!(select_tokens.len(), 2);
        assert_eq!(select_tokens[0].key, "token3");
        assert_eq!(select_tokens[1].key, "token4");

        let builder = initialize_builder(None).await;
        let select_tokens = builder.get_select_tokens().unwrap();
        assert_eq!(select_tokens.len(), 0);
    }

    #[tokio::test]
    async fn test_is_select_token_set() {
        let builder = initialize_builder_with_select_tokens().await;
        let is_select_token_set = builder.is_select_token_set("token3".to_string()).unwrap();
        assert!(!is_select_token_set);

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let is_select_token_set = builder.is_select_token_set("token3".to_string()).unwrap();
        assert!(is_select_token_set);
    }

    #[tokio::test]
    async fn test_check_select_tokens() {
        let builder = initialize_builder_with_select_tokens().await;

        let err = builder.check_select_tokens().unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let err = builder.check_select_tokens().unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::TokenMustBeSelected("token4".to_string()).to_string()
        );

        builder.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "18".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );

        assert!(builder.check_select_tokens().is_ok());
    }

    #[tokio::test]
    async fn test_remove_select_token() {
        let mut builder = initialize_builder_with_select_tokens().await;
        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let err = builder
            .unset_select_token("token5".to_string())
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::TokenNotFound("token5".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token5' could not be found in the YAML configuration."
        );

        assert!(builder.unset_select_token("token3".to_string()).is_ok());
        assert!(!builder.is_select_token_set("token3".to_string()).unwrap());

        let mut builder = initialize_builder(None).await;
        let err = builder
            .unset_select_token("token3".to_string())
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            RaindexOrderBuilderError::SelectTokensNotSet.to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "No tokens have been configured for selection. Please check your YAML configuration."
        );
    }

    #[tokio::test]
    async fn test_are_all_tokens_selected() {
        let builder = initialize_builder_with_select_tokens().await;

        let are_all_tokens_selected = builder.are_all_tokens_selected().unwrap();
        assert!(!are_all_tokens_selected);

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let are_all_tokens_selected = builder.are_all_tokens_selected().unwrap();
        assert!(!are_all_tokens_selected);

        builder.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "18".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );

        let are_all_tokens_selected = builder.are_all_tokens_selected().unwrap();
        assert!(are_all_tokens_selected);
    }

    #[tokio::test]
    async fn test_get_all_tokens() {
        let builder = initialize_builder_with_select_tokens().await;

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );
        builder.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "6".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );
        builder.add_record_to_yaml(
            "token-other".to_string(),
            "other-network".to_string(),
            "0x0000000000000000000000000000000000000003".to_string(),
            "8".to_string(),
            "Token Other".to_string(),
            "TO".to_string(),
        );

        let tokens = builder.get_all_tokens(None).await.unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(
            tokens[0].address.to_string(),
            "0x0000000000000000000000000000000000000001"
        );
        assert_eq!(tokens[0].decimals, 18);
        assert_eq!(tokens[0].name, "Token 3");
        assert_eq!(tokens[0].symbol, "T3");
        assert_eq!(
            tokens[1].address.to_string(),
            "0x0000000000000000000000000000000000000002"
        );
        assert_eq!(tokens[1].decimals, 6);
        assert_eq!(tokens[1].name, "Token 4");
        assert_eq!(tokens[1].symbol, "T4");
    }

    #[tokio::test]
    async fn test_get_all_tokens_search_by_name() {
        let builder = initialize_builder_with_select_tokens().await;

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );
        builder.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "6".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );

        let tokens = builder
            .get_all_tokens(Some("Token 3".to_string()))
            .await
            .unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].name, "Token 3");
    }

    #[tokio::test]
    async fn test_get_all_tokens_search_empty_string() {
        let builder = initialize_builder_with_select_tokens().await;

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );

        let tokens = builder.get_all_tokens(Some("".to_string())).await.unwrap();
        assert_eq!(tokens.len(), 3);
    }

    #[tokio::test]
    async fn test_get_all_tokens_search_by_symbol() {
        let builder = initialize_builder_with_select_tokens().await;

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );
        builder.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "6".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );

        let tokens = builder
            .get_all_tokens(Some("T4".to_string()))
            .await
            .unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].symbol, "T4");
    }

    #[tokio::test]
    async fn test_get_all_tokens_search_by_address() {
        let builder = initialize_builder_with_select_tokens().await;

        builder.add_record_to_yaml(
            "token3".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "18".to_string(),
            "Token 3".to_string(),
            "T3".to_string(),
        );
        builder.add_record_to_yaml(
            "token4".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "6".to_string(),
            "Token 4".to_string(),
            "T4".to_string(),
        );

        let tokens = builder
            .get_all_tokens(Some(
                "0x0000000000000000000000000000000000000002".to_string(),
            ))
            .await
            .unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].address.to_string(),
            "0x0000000000000000000000000000000000000002"
        );
    }

    #[tokio::test]
    async fn test_get_all_tokens_search_partial_match() {
        let builder = initialize_builder_with_select_tokens().await;

        builder.add_record_to_yaml(
            "usdc".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000001".to_string(),
            "6".to_string(),
            "USD Coin".to_string(),
            "USDC".to_string(),
        );
        builder.add_record_to_yaml(
            "usdt".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000002".to_string(),
            "6".to_string(),
            "Tether USD".to_string(),
            "USDT".to_string(),
        );
        builder.add_record_to_yaml(
            "eth".to_string(),
            "some-network".to_string(),
            "0x0000000000000000000000000000000000000003".to_string(),
            "18".to_string(),
            "Ethereum".to_string(),
            "ETH".to_string(),
        );

        let tokens = builder
            .get_all_tokens(Some("USD".to_string()))
            .await
            .unwrap();
        assert_eq!(tokens.len(), 2);

        for token in &tokens {
            assert!(
                token.name.contains("USD") || token.symbol.contains("USD"),
                "Token {} should contain 'USD' in name or symbol",
                token.symbol
            );
        }

        let tokens = builder
            .get_all_tokens(Some("000000000000000000000000000000000000000".to_string()))
            .await
            .unwrap();
        assert_eq!(tokens.len(), 3);
    }

    #[cfg(not(target_family = "wasm"))]
    mod mockserver_tests {
        use super::*;
        use crate::raindex_order_builder::RaindexOrderBuilder;
        use alloy::primitives::Address;
        use httpmock::MockServer;
        use rain_orderbook_app_settings::spec_version::SpecVersion;
        use serde_json::json;
        use std::str::FromStr;

        const TEST_YAML_TEMPLATE: &str = r#"
builder:
  name: Fixed limit
  description: Fixed limit order
  short-description: Buy WETH with USDC on Base.
  deployments:
    some-deployment:
      name: Select token deployment
      description: Select token deployment description
      deposits:
        - token: token3
          min: 0
          presets:
            - "0"
      fields:
        - binding: binding-1
          name: Field 1 name
          description: Field 1 description
          presets:
            - name: Preset 1
              value: "0"
        - binding: binding-2
          name: Field 2 name
          description: Field 2 description
          min: 100
          presets:
            - value: "0"
      select-tokens:
        - key: token3
          name: Token 3
          description: Token 3 description
    normal-deployment:
      name: Normal deployment
      description: Normal deployment description
      deposits:
        - token: token3
      fields:
        - binding: binding-1
          name: Field 1 name
          default: 10
subgraphs:
  some-sg: https://www.some-sg.com
metaboards:
  some-network: https://metaboard.com
rainlangs:
  some-deployer:
    network: some-network
    address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
orderbooks:
  some-orderbook:
    address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
    network: some-network
    subgraph: some-sg
    deployment-block: 12345
scenarios:
  some-scenario:
    rainlang: some-deployer
    bindings:
      test-binding: 5
    scenarios:
      sub-scenario:
        bindings:
          another-binding: 300
orders:
  some-order:
    rainlang: some-deployer
    inputs:
      - token: token3
    outputs:
      - token: token3
deployments:
  some-deployment:
    scenario: some-scenario
    order: some-order
  normal-deployment:
    scenario: some-scenario
    order: some-order
---
#test-binding !
#another-binding !
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#;

        #[tokio::test]
        async fn test_set_select_token() {
            let server = MockServer::start_async().await;
            let yaml = format!(
                r#"
version: {spec_version}
networks:
  some-network:
    rpcs:
      - {rpc_url}
    chain-id: 123
    network-id: 123
    currency: ETH
{yaml}
"#,
                spec_version = SpecVersion::current(),
                yaml = TEST_YAML_TEMPLATE,
                rpc_url = server.url("/rpc")
            );

            server.mock(|when, then| {
                when.method("POST").path("/rpc").body_contains("252dba42");
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e2031000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000",
                }));
            });

            let mut builder = RaindexOrderBuilder::new_with_deployment(
                yaml.to_string(),
                None,
                "some-deployment".to_string(),
            )
            .await
            .unwrap();

            let deployment = builder.get_current_deployment().unwrap();
            assert_eq!(deployment.deployment.order.inputs[0].token, None);
            assert_eq!(deployment.deployment.order.outputs[0].token, None);

            builder
                .set_select_token(
                    "token3".to_string(),
                    "0x0000000000000000000000000000000000000001".to_string(),
                )
                .await
                .unwrap();
            assert!(builder.is_select_token_set("token3".to_string()).unwrap());

            let deployment = builder.get_current_deployment().unwrap();
            let token = deployment.deployment.order.inputs[0]
                .token
                .as_ref()
                .unwrap();
            assert_eq!(
                token.address,
                Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
            );
            assert_eq!(token.decimals, Some(6));
            assert_eq!(token.label, Some("Token 1".to_string()));
            assert_eq!(token.symbol, Some("T1".to_string()));

            let err = builder
                .set_select_token(
                    "token4".to_string(),
                    "0x0000000000000000000000000000000000000002".to_string(),
                )
                .await
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                RaindexOrderBuilderError::TokenNotFound("token4".to_string()).to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "The token 'token4' could not be found in the YAML configuration."
            );

            let mut builder = RaindexOrderBuilder::new_with_deployment(
                yaml.to_string(),
                None,
                "normal-deployment".to_string(),
            )
            .await
            .unwrap();

            let err = builder
                .set_select_token(
                    "token3".to_string(),
                    "0x0000000000000000000000000000000000000002".to_string(),
                )
                .await
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                RaindexOrderBuilderError::SelectTokensNotSet.to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "No tokens have been configured for selection. Please check your YAML configuration."
            );
        }

        #[tokio::test]
        async fn test_get_account_balance() {
            let server = MockServer::start_async().await;
            let yaml = format!(
                r#"
version: {spec_version}
networks:
  some-network:
    rpcs:
      - {rpc_url}
    chain-id: 123
    network-id: 123
    currency: ETH
{yaml}
"#,
                spec_version = SpecVersion::current(),
                yaml = TEST_YAML_TEMPLATE,
                rpc_url = server.url("/rpc")
            );

            server.mock(|when, then| {
                when.method("POST").path("/rpc").body_contains("0x313ce567");
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x0000000000000000000000000000000000000000000000000000000000000012",
                }));
            });
            server.mock(|when, then| {
                when.method("POST").path("/rpc").body_contains("0x70a08231");
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x00000000000000000000000000000000000000000000000000000000000003e8",
                }));
            });

            let builder = RaindexOrderBuilder::new_with_deployment(
                yaml.to_string(),
                None,
                "some-deployment".to_string(),
            )
            .await
            .unwrap();

            let balance = builder
                .get_account_balance(
                    "0x0000000000000000000000000000000000000001".to_string(),
                    "0x0000000000000000000000000000000000000002".to_string(),
                )
                .await
                .unwrap();

            assert_eq!(balance.formatted_balance(), "1e-15");
        }
    }
}
