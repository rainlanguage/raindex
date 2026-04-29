use super::*;
use alloy::primitives::{Address, Bytes, U256};
use rain_orderbook_app_settings::order::VaultType;
use rain_orderbook_common::raindex_order_builder::order_operations as inner_ops;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct TokenAllowance {
    #[tsify(type = "string")]
    pub token: Address,
    #[tsify(type = "string")]
    pub allowance: U256,
}
impl_wasm_traits!(TokenAllowance);

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct AllowancesResult(pub Vec<TokenAllowance>);
impl_wasm_traits!(AllowancesResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub enum ApprovalCalldataResult {
    NoDeposits,
    Calldatas(Vec<ApprovalCalldata>),
}
impl_wasm_traits!(ApprovalCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct ApprovalCalldata {
    #[tsify(type = "string")]
    pub token: Address,
    #[tsify(type = "string")]
    pub calldata: Bytes,
}
impl_wasm_traits!(ApprovalCalldata);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub enum DepositCalldataResult {
    NoDeposits,
    Calldatas(#[tsify(type = "string[]")] Vec<Bytes>),
}
impl_wasm_traits!(DepositCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AddOrderCalldataResult(#[tsify(type = "string")] pub Bytes);
impl_wasm_traits!(AddOrderCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct DepositAndAddOrderCalldataResult(#[tsify(type = "string")] pub Bytes);
impl_wasm_traits!(DepositAndAddOrderCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct IOVaultIds(
    #[tsify(type = "Map<string, Map<string, string | undefined>>")]
    pub  HashMap<String, HashMap<String, Option<U256>>>,
);
impl_wasm_traits!(IOVaultIds);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WithdrawCalldataResult(#[tsify(type = "string[]")] pub Vec<Bytes>);
impl_wasm_traits!(WithdrawCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct ExtendedApprovalCalldata {
    #[tsify(type = "string")]
    pub token: Address,
    #[tsify(type = "string")]
    pub calldata: Bytes,
    pub symbol: String,
}
impl_wasm_traits!(ExtendedApprovalCalldata);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCall {
    #[tsify(type = "string")]
    pub to: Address,
    #[tsify(type = "string")]
    pub calldata: Bytes,
}
impl_wasm_traits!(ExternalCall);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentTransactionArgs {
    pub approvals: Vec<ExtendedApprovalCalldata>,
    #[tsify(type = "string")]
    pub deployment_calldata: Bytes,
    #[tsify(type = "string")]
    pub orderbook_address: Address,
    pub chain_id: u32,
    pub emit_meta_call: Option<ExternalCall>,
}
impl_wasm_traits!(DeploymentTransactionArgs);

#[wasm_export]
impl RaindexOrderBuilder {
    #[wasm_export(
        js_name = "checkAllowances",
        unchecked_return_type = "AllowancesResult",
        return_description = "Token allowances for the deployment"
    )]
    pub async fn check_allowances(
        &mut self,
        #[wasm_export(param_description = "Owner wallet address")] owner: String,
    ) -> Result<inner_ops::AllowancesResult, RaindexOrderBuilderWasmError> {
        Ok(self.inner.check_allowances(owner).await?)
    }

    #[wasm_export(
        js_name = "generateApprovalCalldatas",
        unchecked_return_type = "ApprovalCalldataResult",
        return_description = "Approval transaction calldatas or NoDeposits"
    )]
    pub async fn generate_approval_calldatas(
        &mut self,
        #[wasm_export(param_description = "Owner wallet address")] owner: String,
    ) -> Result<inner_ops::ApprovalCalldataResult, RaindexOrderBuilderWasmError> {
        Ok(self.inner.generate_approval_calldatas(owner).await?)
    }

    #[wasm_export(
        js_name = "generateDepositCalldatas",
        unchecked_return_type = "DepositCalldataResult",
        return_description = "Deposit transaction calldatas or NoDeposits"
    )]
    pub async fn generate_deposit_calldatas(
        &mut self,
    ) -> Result<inner_ops::DepositCalldataResult, RaindexOrderBuilderWasmError> {
        Ok(self.inner.generate_deposit_calldatas().await?)
    }

    #[wasm_export(
        js_name = "generateAddOrderCalldata",
        unchecked_return_type = "AddOrderCalldataResult",
        return_description = "Add order transaction calldata"
    )]
    pub async fn generate_add_order_calldata(
        &mut self,
    ) -> Result<inner_ops::AddOrderCalldataResult, RaindexOrderBuilderWasmError> {
        Ok(self.inner.generate_add_order_calldata().await?)
    }

    #[wasm_export(
        js_name = "generateDepositAndAddOrderCalldatas",
        unchecked_return_type = "DepositAndAddOrderCalldataResult",
        return_description = "Combined deposit and add order multicall calldata"
    )]
    pub async fn generate_deposit_and_add_order_calldatas(
        &mut self,
    ) -> Result<inner_ops::DepositAndAddOrderCalldataResult, RaindexOrderBuilderWasmError> {
        Ok(self
            .inner
            .generate_deposit_and_add_order_calldatas()
            .await?)
    }

    #[wasm_export(js_name = "setVaultId", unchecked_return_type = "void")]
    pub fn set_vault_id(
        &mut self,
        #[wasm_export(param_description = "Vault type (input or output)")] r#type: VaultType,
        #[wasm_export(param_description = "Token key")] token: String,
        #[wasm_export(param_description = "Optional vault ID as hex string")] vault_id: Option<
            String,
        >,
    ) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.set_vault_id(r#type, token, vault_id)?;
        self.execute_state_update_callback()?;
        Ok(())
    }

    #[wasm_export(
        js_name = "getVaultIds",
        unchecked_return_type = "IOVaultIds",
        return_description = "Map of input/output vault IDs by token"
    )]
    pub fn get_vault_ids(&self) -> Result<inner_ops::IOVaultIds, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_vault_ids()?)
    }

    #[wasm_export(
        js_name = "hasAnyVaultId",
        unchecked_return_type = "boolean",
        return_description = "Whether any vault ID has been set"
    )]
    pub fn has_any_vault_id(&self) -> Result<bool, RaindexOrderBuilderWasmError> {
        Ok(self.inner.has_any_vault_id()?)
    }

    #[wasm_export(js_name = "updateScenarioBindings", unchecked_return_type = "void")]
    pub fn update_scenario_bindings(&mut self) -> Result<(), RaindexOrderBuilderWasmError> {
        self.inner.update_scenario_bindings()?;
        Ok(())
    }

    #[wasm_export(
        js_name = "getDeploymentTransactionArgs",
        unchecked_return_type = "DeploymentTransactionArgs",
        return_description = "Complete deployment transaction arguments"
    )]
    pub async fn get_deployment_transaction_args(
        &mut self,
        #[wasm_export(param_description = "Owner wallet address")] owner: String,
    ) -> Result<inner_ops::DeploymentTransactionArgs, RaindexOrderBuilderWasmError> {
        Ok(self.inner.get_deployment_transaction_args(owner).await?)
    }
}
