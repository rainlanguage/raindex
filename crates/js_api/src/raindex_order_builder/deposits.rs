use super::*;
use alloy::primitives::Address;
use rain_orderbook_common::raindex_order_builder::deposits as inner_deposits;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct TokenDeposit {
    pub token: String,
    pub amount: String,
    #[tsify(type = "string")]
    pub address: Address,
}
impl_wasm_traits!(TokenDeposit);

impl From<inner_deposits::TokenDeposit> for TokenDeposit {
    fn from(td: inner_deposits::TokenDeposit) -> Self {
        Self {
            token: td.token,
            amount: td.amount,
            address: td.address,
        }
    }
}

#[wasm_export]
impl RaindexOrderBuilder {
    #[wasm_export(
        js_name = "getDeposits",
        unchecked_return_type = "TokenDeposit[]",
        return_description = "Array of configured deposits with token info and amounts"
    )]
    pub fn get_deposits(&self) -> Result<Vec<TokenDeposit>, RaindexOrderBuilderWasmError> {
        Ok(self
            .inner
            .get_deposits()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    #[wasm_export(js_name = "setDeposit", unchecked_return_type = "void")]
    pub async fn set_deposit(
        &mut self,
        #[wasm_export(param_description = "Token key from the YAML configuration")] token: String,
        #[wasm_export(param_description = "Deposit amount as a decimal string")] amount: String,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.set_deposit(token, amount).await?;
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(js_name = "unsetDeposit", unchecked_return_type = "void")]
    pub fn unset_deposit(
        &mut self,
        #[wasm_export(param_description = "Token key to remove deposit for")] token: String,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.unset_deposit(token)?;
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(
        js_name = "getDepositPresets",
        unchecked_return_type = "string[]",
        return_description = "Array of preset deposit amounts"
    )]
    pub fn get_deposit_presets(
        &self,
        #[wasm_export(param_description = "Token key to get presets for")] key: String,
    ) -> Result<Vec<String>, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_deposit_presets(key)?)
    }

    #[wasm_export(
        js_name = "getMissingDeposits",
        unchecked_return_type = "string[]",
        return_description = "Array of token keys that need deposits"
    )]
    pub fn get_missing_deposits(&self) -> Result<Vec<String>, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_missing_deposits()?)
    }

    #[wasm_export(
        js_name = "hasAnyDeposit",
        unchecked_return_type = "boolean",
        return_description = "Whether any deposit has been set"
    )]
    pub fn has_any_deposit(&self) -> Result<bool, RaindexOrderBuilderWasmError> {
        Ok(self.inner.has_any_deposit()?)
    }
}
