use super::*;

#[wasm_export]
impl RaindexOrderBuilder {
    #[wasm_export(
        js_name = "getSelectTokens",
        unchecked_return_type = "OrderBuilderSelectTokensCfg[]",
        return_description = "Array of token selection configurations"
    )]
    pub fn get_select_tokens(
        &self,
    ) -> Result<
        Vec<rain_orderbook_app_settings::order_builder::OrderBuilderSelectTokensCfg>,
        RaindexOrderBuilderWasmError,
    > {
        Ok(self.inner.get_select_tokens()?)
    }

    #[wasm_export(
        js_name = "isSelectTokenSet",
        unchecked_return_type = "boolean",
        return_description = "Whether the token has been selected"
    )]
    pub fn is_select_token_set(
        &self,
        #[wasm_export(param_description = "Token key to check")] key: String,
    ) -> Result<bool, RaindexOrderBuilderWasmError> {
        Ok(self.inner.is_select_token_set(key)?)
    }

    #[wasm_export(js_name = "checkSelectTokens", unchecked_return_type = "void")]
    pub fn check_select_tokens(&self) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.check_select_tokens()?;
        Ok(())
    }

    #[wasm_export(js_name = "setSelectToken", unchecked_return_type = "void")]
    pub async fn set_select_token(
        &mut self,
        #[wasm_export(param_description = "Token key from select-tokens configuration")]
        key: String,
        #[wasm_export(param_description = "Token contract address as hex string")] address: String,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.set_select_token(key, address).await?;
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "unsetSelectToken", unchecked_return_type = "void")]
    pub fn unset_select_token(
        &mut self,
        #[wasm_export(param_description = "Token key to deselect")] key: String,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.unset_select_token(key)?;
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(
        js_name = "areAllTokensSelected",
        unchecked_return_type = "boolean",
        return_description = "Whether all required tokens have been selected"
    )]
    pub fn are_all_tokens_selected(&self) -> Result<bool, RaindexOrderBuilderWasmError> {
        Ok(self.inner.are_all_tokens_selected()?)
    }

    #[wasm_export(
        js_name = "getAllTokens",
        unchecked_return_type = "ExtendedTokenInfo[]",
        return_description = "Array of token information for the deployment's network"
    )]
    pub async fn get_all_tokens(
        &self,
        #[wasm_export(param_description = "Optional search string to filter tokens")]
        search: Option<String>,
    ) -> Result<Vec<ExtendedTokenInfo>, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_all_tokens(search).await?)
    }

    #[wasm_export(
        js_name = "getAccountBalance",
        unchecked_return_type = "AccountBalance",
        return_description = "Token balance for the specified account",
        preserve_js_class
    )]
    pub async fn get_account_balance(
        &self,
        #[wasm_export(param_description = "Token contract address")] token_address: String,
        #[wasm_export(param_description = "Account address to check balance for")] owner: String,
    ) -> Result<
        rain_orderbook_common::raindex_client::vaults::AccountBalance,
        RaindexOrderBuilderWasmError,
    > {
        Ok(self.inner.get_account_balance(token_address, owner).await?)
    }
}
