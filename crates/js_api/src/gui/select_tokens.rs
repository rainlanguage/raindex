use super::*;
use futures::StreamExt;
use rain_math_float::Float;
use rain_orderbook_app_settings::{
    deployment::DeploymentCfg, gui::GuiSelectTokensCfg, network::NetworkCfg, order::OrderCfg,
    token::TokenCfg, yaml::YamlParsableHash,
};
use rain_orderbook_common::raindex_client::vaults::AccountBalance;
use std::str::FromStr;

const MAX_CONCURRENT_FETCHES: usize = 5;

#[wasm_export]
impl DotrainOrderGui {
    /// Gets tokens that need user selection for the current deployment.
    ///
    /// Returns tokens defined in the select-tokens section that require user input
    /// to specify contract addresses. This enables generic orders that work
    /// with user-chosen tokens.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getSelectTokens();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const selectableTokens = result.value;
    /// // Do something with the selectable tokens
    /// ```
    #[wasm_export(
        js_name = "getSelectTokens",
        unchecked_return_type = "GuiSelectTokensCfg[]",
        return_description = "Array of selectable token configurations"
    )]
    pub fn get_select_tokens(&self) -> Result<Vec<GuiSelectTokensCfg>, GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?;
        Ok(select_tokens.unwrap_or(vec![]))
    }

    /// Checks if a selectable token has been configured with an address.
    ///
    /// Use this to determine if a user has provided an address for a select-token,
    /// enabling progressive UI updates and validation.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.isSelectTokenSet("stable-token");
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const isSelectTokenSet = result.value;
    /// // Do something
    /// ```
    #[wasm_export(
        js_name = "isSelectTokenSet",
        unchecked_return_type = "boolean",
        return_description = "True if token address has been set"
    )]
    pub fn is_select_token_set(
        &self,
        #[wasm_export(param_description = "Token key from select-tokens configuration")]
        key: String,
    ) -> Result<bool, GuiError> {
        Ok(self.dotrain_order.orderbook_yaml().get_token(&key).is_ok())
    }

    /// Validates that all required tokens have been selected.
    ///
    /// Checks if all tokens in the select-tokens configuration have been given
    /// addresses. Use this before generating transactions to ensure completeness.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.checkSelectTokens();
    /// if (result.error) {
    ///   console.error("Selection incomplete:", result.error.readableMsg);
    ///   return;
    /// // Do something
    /// ```
    #[wasm_export(
        js_name = "checkSelectTokens",
        unchecked_return_type = "void",
        return_description = "All required tokens are configured"
    )]
    pub fn check_select_tokens(&self) -> Result<(), GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
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
                    return Err(GuiError::TokenMustBeSelected(select_token.key.clone()));
                }
            }
        }

        Ok(())
    }

    /// Sets a custom token address to be used in the order.
    ///
    /// Takes a token address provided by the user and queries the blockchain to get
    /// the token's name, symbol, and decimals. This information is then cached for efficient access.
    ///
    /// # Network Usage
    ///
    /// The function uses the deployment's network configuration to determine the
    /// RPC endpoint for querying token information.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // User selects token
    /// const result = await gui.setSelectToken(
    ///   "stable-token",
    ///   "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    /// );
    ///
    /// if (result.error) {
    ///   console.error("Selection failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// // Do something with the token
    /// ```
    #[wasm_export(js_name = "setSelectToken", unchecked_return_type = "void")]
    pub async fn set_select_token(
        &mut self,
        #[wasm_export(param_description = "Token key from select-tokens configuration")]
        key: String,
        #[wasm_export(param_description = "Token contract address provided by user")]
        address: String,
    ) -> Result<(), GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.iter().any(|token| token.key == key) {
            return Err(GuiError::TokenNotFound(key.clone()));
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

        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Removes a previously selected token configuration.
    ///
    /// Clears the address and cached information for a select-token, returning it
    /// to an unselected state.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Remove token selection
    /// const result = gui.unsetSelectToken("stable-token");
    /// if (result.error) {
    ///   console.error("Remove failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// ```
    #[wasm_export(js_name = "unsetSelectToken", unchecked_return_type = "void")]
    pub fn unset_select_token(
        &mut self,
        #[wasm_export(param_description = "Token key to clear")] key: String,
    ) -> Result<(), GuiError> {
        let select_tokens = GuiCfg::parse_select_tokens(
            self.dotrain_order.dotrain_yaml().documents,
            &self.selected_deployment,
        )?
        .ok_or(GuiError::SelectTokensNotSet)?;
        if !select_tokens.iter().any(|token| token.key == key) {
            return Err(GuiError::TokenNotFound(key.clone()));
        }

        TokenCfg::remove_record_from_yaml(self.dotrain_order.orderbook_yaml().documents, &key)?;

        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Checks if all required tokens have been selected and configured.
    ///
    /// Validates that every token in the select-tokens configuration has been
    /// given an address. Use this for overall validation and progress tracking.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.areAllTokensSelected();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const allSelected = result.value;
    /// // Do something
    /// ```
    #[wasm_export(
        js_name = "areAllTokensSelected",
        unchecked_return_type = "boolean",
        return_description = "True if all tokens are configured"
    )]
    pub fn are_all_tokens_selected(&self) -> Result<bool, GuiError> {
        let select_tokens = self.get_select_tokens()?;
        for token in select_tokens {
            if !self.is_select_token_set(token.key)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Gets all tokens configured for the selected deployment's network.
    ///
    /// Retrieves token information from the YAML configuration, using cached
    /// metadata when available or fetching from blockchain via ERC20 contracts.
    /// Results are filtered by the search term (matching name or address) and
    /// sorted by address and deduplicated.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Get all tokens
    /// const result = await gui.getAllTokens();
    ///
    /// // Search for specific tokens
    /// const usdcResult = await gui.getAllTokens("USDC");
    /// const addressResult = await gui.getAllTokens("0x1234...");
    /// ```
    #[wasm_export(
        js_name = "getAllTokens",
        unchecked_return_type = "ExtendedTokenInfo[]",
        return_description = "Array of token information for the current network"
    )]
    pub async fn get_all_tokens(
        &self,
        #[wasm_export(
            param_description = "Optional search term to filter tokens by name, symbol, or address"
        )]
        search: Option<String>,
    ) -> Result<Vec<ExtendedTokenInfo>, GuiError> {
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

    /// Gets the balance of a specific token for a given owner address
    ///
    /// Retrieves the ERC20 token balance by connecting to the current deployment's network RPC
    /// and querying the token contract.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await gui.getAccountBalance("0x123...", "0xabc...");
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// console.log("Raw balance:", result.value.balance);
    /// console.log("Formatted balance:", result.value.formattedBalance);
    /// ```
    #[wasm_export(
        js_name = "getAccountBalance",
        unchecked_return_type = "AccountBalance",
        return_description = "Owner balance in both raw and human-readable format",
        preserve_js_class
    )]
    pub async fn get_account_balance(
        &self,
        #[wasm_export(js_name = "tokenAddress", param_description = "Token contract address")]
        token_address: String,
        #[wasm_export(
            param_description = "Owner address to check balance for",
            unchecked_param_type = "Hex"
        )]
        owner: String,
    ) -> Result<AccountBalance, GuiError> {
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
impl DotrainOrderGui {
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
    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use crate::gui::{
            tests::{initialize_gui, initialize_gui_with_select_tokens},
            GuiError,
        };
        use wasm_bindgen_test::wasm_bindgen_test;

        #[wasm_bindgen_test]
        async fn test_get_select_tokens() {
            let gui = initialize_gui_with_select_tokens().await;
            let select_tokens = gui.get_select_tokens().unwrap();
            assert_eq!(select_tokens.len(), 2);
            assert_eq!(select_tokens[0].key, "token3");
            assert_eq!(select_tokens[1].key, "token4");

            let gui = initialize_gui(None).await;
            let select_tokens = gui.get_select_tokens().unwrap();
            assert_eq!(select_tokens.len(), 0);
        }

        #[wasm_bindgen_test]
        async fn test_is_select_token_set() {
            let gui = initialize_gui_with_select_tokens().await;
            let is_select_token_set = gui.is_select_token_set("token3".to_string()).unwrap();
            assert!(!is_select_token_set);

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );

            let is_select_token_set = gui.is_select_token_set("token3".to_string()).unwrap();
            assert!(is_select_token_set);
        }

        #[wasm_bindgen_test]
        async fn test_check_select_tokens() {
            let gui = initialize_gui_with_select_tokens().await;

            let err = gui.check_select_tokens().unwrap_err();
            assert_eq!(
                err.to_string(),
                GuiError::TokenMustBeSelected("token3".to_string()).to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "The token 'token3' must be selected to proceed."
            );

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );

            let err = gui.check_select_tokens().unwrap_err();
            assert_eq!(
                err.to_string(),
                GuiError::TokenMustBeSelected("token4".to_string()).to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "The token 'token4' must be selected to proceed."
            );

            gui.add_record_to_yaml(
                "token4".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "18".to_string(),
                "Token 4".to_string(),
                "T4".to_string(),
            );

            assert!(gui.check_select_tokens().is_ok());
        }

        #[wasm_bindgen_test]
        async fn test_set_select_token() {
            let mut gui = initialize_gui_with_select_tokens().await;
            let err = gui
                .set_select_token(
                    "token5".to_string(),
                    "0x0000000000000000000000000000000000000001".to_string(),
                )
                .await
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                GuiError::TokenNotFound("token5".to_string()).to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "The token 'token5' could not be found in the YAML configuration."
            );
        }

        #[wasm_bindgen_test]
        async fn test_remove_select_token() {
            let mut gui = initialize_gui_with_select_tokens().await;
            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );

            let err = gui.unset_select_token("token5".to_string()).unwrap_err();
            assert_eq!(
                err.to_string(),
                GuiError::TokenNotFound("token5".to_string()).to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "The token 'token5' could not be found in the YAML configuration."
            );

            assert!(gui.unset_select_token("token3".to_string()).is_ok());
            assert!(!gui.is_select_token_set("token3".to_string()).unwrap());

            let mut gui = initialize_gui(None).await;
            let err = gui.unset_select_token("token3".to_string()).unwrap_err();
            assert_eq!(err.to_string(), GuiError::SelectTokensNotSet.to_string());
            assert_eq!(
                err.to_readable_msg(),
                "No tokens have been configured for selection. Please check your YAML configuration."
            );
        }

        #[wasm_bindgen_test]
        async fn test_are_all_tokens_selected() {
            let gui = initialize_gui_with_select_tokens().await;

            let are_all_tokens_selected = gui.are_all_tokens_selected().unwrap();
            assert!(!are_all_tokens_selected);

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );

            let are_all_tokens_selected = gui.are_all_tokens_selected().unwrap();
            assert!(!are_all_tokens_selected);

            gui.add_record_to_yaml(
                "token4".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "18".to_string(),
                "Token 4".to_string(),
                "T4".to_string(),
            );

            let are_all_tokens_selected = gui.are_all_tokens_selected().unwrap();
            assert!(are_all_tokens_selected);
        }

        #[wasm_bindgen_test]
        async fn test_get_all_tokens() {
            let gui = initialize_gui_with_select_tokens().await;

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );
            gui.add_record_to_yaml(
                "token4".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "6".to_string(),
                "Token 4".to_string(),
                "T4".to_string(),
            );
            gui.add_record_to_yaml(
                "token-other".to_string(),
                "other-network".to_string(),
                "0x0000000000000000000000000000000000000003".to_string(),
                "8".to_string(),
                "Token Other".to_string(),
                "TO".to_string(),
            );

            let tokens = gui.get_all_tokens(None).await.unwrap();
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

        #[wasm_bindgen_test]
        async fn test_get_all_tokens_search_by_name() {
            let gui = initialize_gui_with_select_tokens().await;

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );
            gui.add_record_to_yaml(
                "token4".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "6".to_string(),
                "Token 4".to_string(),
                "T4".to_string(),
            );

            let tokens = gui
                .get_all_tokens(Some("Token 3".to_string()))
                .await
                .unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].name, "Token 3");
        }

        #[wasm_bindgen_test]
        async fn test_get_all_tokens_search_by_symbol() {
            let gui = initialize_gui_with_select_tokens().await;

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );
            gui.add_record_to_yaml(
                "token4".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "6".to_string(),
                "Token 4".to_string(),
                "T4".to_string(),
            );

            let tokens = gui.get_all_tokens(Some("T4".to_string())).await.unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].symbol, "T4");
        }

        #[wasm_bindgen_test]
        async fn test_get_all_tokens_search_by_address() {
            let gui = initialize_gui_with_select_tokens().await;

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );
            gui.add_record_to_yaml(
                "token4".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "6".to_string(),
                "Token 4".to_string(),
                "T4".to_string(),
            );

            let tokens = gui
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

        #[wasm_bindgen_test]
        async fn test_get_all_tokens_search_partial_match() {
            let gui = initialize_gui_with_select_tokens().await;

            gui.add_record_to_yaml(
                "usdc".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "6".to_string(),
                "USD Coin".to_string(),
                "USDC".to_string(),
            );
            gui.add_record_to_yaml(
                "usdt".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "6".to_string(),
                "Tether USD".to_string(),
                "USDT".to_string(),
            );
            gui.add_record_to_yaml(
                "eth".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000003".to_string(),
                "18".to_string(),
                "Ethereum".to_string(),
                "ETH".to_string(),
            );

            let tokens = gui.get_all_tokens(Some("USD".to_string())).await.unwrap();
            assert_eq!(tokens.len(), 2);

            for token in &tokens {
                assert!(
                    token.name.contains("USD") || token.symbol.contains("USD"),
                    "Token {} should contain 'USD' in name or symbol",
                    token.symbol
                );
            }

            let tokens = gui
                .get_all_tokens(Some("000000000000000000000000000000000000000".to_string()))
                .await
                .unwrap();
            assert_eq!(tokens.len(), 3);
        }

        #[wasm_bindgen_test]
        async fn test_get_all_tokens_search_empty_string() {
            let gui = initialize_gui_with_select_tokens().await;

            gui.add_record_to_yaml(
                "token3".to_string(),
                "some-network".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
                "18".to_string(),
                "Token 3".to_string(),
                "T3".to_string(),
            );

            let tokens = gui.get_all_tokens(Some("".to_string())).await.unwrap();
            assert_eq!(tokens.len(), 3);
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm_tests {
        use crate::gui::{DotrainOrderGui, GuiError};
        use alloy::primitives::Address;
        use httpmock::MockServer;
        use rain_orderbook_app_settings::spec_version::SpecVersion;
        use serde_json::json;
        use std::str::FromStr;

        const TEST_YAML_TEMPLATE: &str = r#"
version: {spec_version}
gui:
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
networks:
  some-network:
    rpcs:
      - {rpc_url}
    chain-id: 123
    network-id: 123
    currency: ETH
subgraphs:
  some-sg: https://www.some-sg.com
metaboards:
  some-network: https://metaboard.com
rainlangs:
  some-rainlang:
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
    rainlang: some-rainlang
    bindings:
      test-binding: 5
    scenarios:
      sub-scenario:
        bindings:
          another-binding: 300
orders:
  some-order:
    rainlang: some-rainlang
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
            let yaml = TEST_YAML_TEMPLATE
                .replace("{rpc_url}", &server.url("/rpc"))
                .replace("{spec_version}", &SpecVersion::current().to_string());

            server.mock(|when, then| {
                when.method("POST").path("/rpc").body_contains("252dba42");
                then.json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000007546f6b656e2031000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000025431000000000000000000000000000000000000000000000000000000000000",
                }));
            });

            let mut gui = DotrainOrderGui::new_with_deployment(
                yaml.to_string(),
                None,
                "some-deployment".to_string(),
                None,
            )
            .await
            .unwrap();

            let deployment = gui.get_current_deployment().unwrap();
            assert_eq!(deployment.deployment.order.inputs[0].token, None);
            assert_eq!(deployment.deployment.order.outputs[0].token, None);

            gui.set_select_token(
                "token3".to_string(),
                "0x0000000000000000000000000000000000000001".to_string(),
            )
            .await
            .unwrap();
            assert!(gui.is_select_token_set("token3".to_string()).unwrap());

            let deployment = gui.get_current_deployment().unwrap();
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

            let err = gui
                .set_select_token(
                    "token4".to_string(),
                    "0x0000000000000000000000000000000000000002".to_string(),
                )
                .await
                .unwrap_err();
            assert_eq!(
                err.to_string(),
                GuiError::TokenNotFound("token4".to_string()).to_string()
            );
            assert_eq!(
                err.to_readable_msg(),
                "The token 'token4' could not be found in the YAML configuration."
            );

            let mut gui = DotrainOrderGui::new_with_deployment(
                yaml.to_string(),
                None,
                "normal-deployment".to_string(),
                None,
            )
            .await
            .unwrap();

            let err = gui
                .set_select_token(
                    "token3".to_string(),
                    "0x0000000000000000000000000000000000000002".to_string(),
                )
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), GuiError::SelectTokensNotSet.to_string());
            assert_eq!(
            err.to_readable_msg(),
            "No tokens have been configured for selection. Please check your YAML configuration."
        );
        }

        #[tokio::test]
        async fn test_get_account_balance() {
            let server = MockServer::start_async().await;
            let yaml = TEST_YAML_TEMPLATE
                .replace("{rpc_url}", &server.url("/rpc"))
                .replace("{spec_version}", &SpecVersion::current().to_string());

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

            let gui = DotrainOrderGui::new_with_deployment(
                yaml.to_string(),
                None,
                "some-deployment".to_string(),
                None,
            )
            .await
            .unwrap();

            let balance = gui
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
