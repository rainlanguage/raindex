use super::*;
use alloy::{
    primitives::{Bytes, B256, U256},
    sol_types::SolCall,
};
use rain_math_float::Float;
use rain_metaboard_subgraph::metaboard_client::{
    MetaboardSubgraphClient, MetaboardSubgraphClientError,
};
use rain_metaboard_subgraph::types::metas::BigInt as MetaBigInt;
use rain_metadata::RainMetaDocumentV1Item;
use rain_orderbook_app_settings::{
    order::{OrderIOCfg, VaultType},
    orderbook::OrderbookCfg,
};
use rain_orderbook_bindings::{
    IRaindexV6::deposit4Call, OrderBook::multicallCall, IERC20::approveCall,
};
use rain_orderbook_common::{
    add_order::AddOrderArgs, deposit::DepositArgs, erc20::ERC20, transaction::TransactionArgs,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use url::Url;

pub enum CalldataFunction {
    Allowance,
    Deposit,
    AddOrder,
    DepositAndAddOrder,
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct TokenAllowance {
    #[tsify(type = "string")]
    token: Address,
    #[tsify(type = "string")]
    allowance: U256,
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify)]
pub struct AllowancesResult(Vec<TokenAllowance>);
impl_wasm_traits!(AllowancesResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub enum ApprovalCalldataResult {
    NoDeposits,
    Calldatas(Vec<ApprovalCalldata>),
}
impl_wasm_traits!(ApprovalCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub enum DepositCalldataResult {
    NoDeposits,
    Calldatas(#[tsify(type = "Hex[]")] Vec<Bytes>),
}
impl_wasm_traits!(DepositCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct AddOrderCalldataResult(#[tsify(type = "Hex")] Bytes);
impl_wasm_traits!(AddOrderCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct DepositAndAddOrderCalldataResult(#[tsify(type = "Hex")] Bytes);
impl_wasm_traits!(DepositAndAddOrderCalldataResult);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct IOVaultIds(
    #[tsify(type = "Map<string, Map<string, string | undefined>>")]
    pub  HashMap<String, HashMap<String, Option<U256>>>,
);
impl_wasm_traits!(IOVaultIds);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct WithdrawCalldataResult(#[tsify(type = "Hex[]")] Vec<Bytes>);
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
    approvals: Vec<ExtendedApprovalCalldata>,
    #[tsify(type = "string")]
    deployment_calldata: Bytes,
    #[tsify(type = "string")]
    orderbook_address: Address,
    chain_id: u32,
    #[tsify(type = "ExternalCall | undefined")]
    emit_meta_call: Option<ExternalCall>,
}
impl_wasm_traits!(DeploymentTransactionArgs);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct ApprovalCalldata {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub token: Address,
    #[cfg_attr(target_family = "wasm", tsify(type = "Hex"))]
    pub calldata: Bytes,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ApprovalCalldata);

#[derive(Debug)]
pub struct VaultAndDeposit {
    pub order_io: OrderIOCfg,
    pub deposit_amount: Float,
    pub index: usize,
}

#[wasm_export]
impl DotrainOrderGui {
    fn get_orderbook(&self) -> Result<Arc<OrderbookCfg>, GuiError> {
        let deployment = self.get_current_deployment()?;
        deployment
            .deployment
            .as_ref()
            .order
            .as_ref()
            .orderbook
            .as_ref()
            .ok_or(GuiError::OrderbookNotFound)
            .cloned()
    }

    fn get_transaction_args(&self) -> Result<TransactionArgs, GuiError> {
        let orderbook = self.get_orderbook()?;
        Ok(TransactionArgs {
            orderbook_address: orderbook.address,
            rpcs: orderbook
                .network
                .rpcs
                .clone()
                .into_iter()
                .map(|url| url.to_string())
                .collect(),
            ..Default::default()
        })
    }

    async fn get_deposits_as_map(&self) -> Result<HashMap<Address, Float>, GuiError> {
        let mut map: HashMap<Address, Float> = HashMap::new();
        for d in self.get_deposits()? {
            let token_info = self.get_token_info(d.token.clone()).await?;
            let amount = Float::parse(d.amount)?;
            map.insert(token_info.address, amount);
        }
        Ok(map)
    }

    async fn get_vaults_and_deposits(
        &self,
        deployment: &GuiDeploymentCfg,
    ) -> Result<Vec<VaultAndDeposit>, GuiError> {
        let deposits_map = self.get_deposits_as_map().await?;
        let results = deployment
            .deployment
            .order
            .outputs
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(index, output)| {
                output.token.as_ref().and_then(|token| {
                    deposits_map.get(&token.address).map(|amount| {
                        Ok(VaultAndDeposit {
                            order_io: output.clone(),
                            deposit_amount: *amount,
                            index,
                        })
                    })
                })
            })
            .collect::<Result<Vec<_>, GuiError>>()?;
        Ok(results)
    }

    async fn check_allowance(
        &self,
        deposit_args: &DepositArgs,
        owner: &str,
    ) -> Result<TokenAllowance, GuiError> {
        let allowance = deposit_args
            .read_allowance(Address::from_str(owner)?, self.get_transaction_args()?)
            .await?;
        Ok(TokenAllowance {
            token: deposit_args.token,
            allowance,
        })
    }

    fn prepare_calldata_generation(
        &mut self,
        calldata_function: CalldataFunction,
    ) -> Result<GuiDeploymentCfg, GuiError> {
        let deployment = self.get_current_deployment()?;
        self.check_select_tokens()?;
        match calldata_function {
            CalldataFunction::Deposit => {
                self.populate_vault_ids(&deployment)?;
            }
            CalldataFunction::AddOrder | CalldataFunction::DepositAndAddOrder => {
                self.check_field_values()?;
                self.populate_vault_ids(&deployment)?;
                self.update_bindings(&deployment)?;
            }
            _ => {}
        }
        self.get_current_deployment()
    }

    /// Checks token allowances for all deposits against the orderbook contract.
    ///
    /// Queries the blockchain to determine current  allowances for each output token that
    /// will be deposited. This helps determine which tokens need approval before
    /// the order can be created.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await gui.checkAllowances(walletAddress);
    /// if (result.error) {
    ///   console.error("Allowance check failed:", result.error.readableMsg);
    ///   return;
    /// }
    /// const [allowance1, allowance2, ...] = result.value;
    /// const {
    ///   // token is the token address
    ///   token,
    ///   // allowance is the current allowance for the token
    ///   allowance,
    /// } = allowance1;
    /// ```
    #[wasm_export(
        js_name = "checkAllowances",
        unchecked_return_type = "AllowancesResult",
        return_description = "Current allowances for all deposit tokens"
    )]
    pub async fn check_allowances(
        &mut self,
        #[wasm_export(param_description = "Wallet address to check allowances for")] owner: String,
    ) -> Result<AllowancesResult, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::Allowance)?;

        let vaults_and_deposits = self.get_vaults_and_deposits(&deployment).await?;

        let owner = Address::from_str(&owner)?;

        let mut results = Vec::new();
        for VaultAndDeposit {
            order_io,
            deposit_amount: _,
            index: _,
        } in vaults_and_deposits
        {
            let tx_args = self.get_transaction_args()?;
            let rpcs = tx_args
                .rpcs
                .iter()
                .map(|rpc| Url::parse(rpc))
                .collect::<Result<Vec<_>, _>>()?;

            let token = order_io
                .token
                .as_ref()
                .ok_or(GuiError::SelectTokensNotSet)?
                .address;

            let erc20 = ERC20::new(rpcs, token);
            let allowance = erc20.allowance(owner, tx_args.orderbook_address).await?;

            results.push(TokenAllowance { token, allowance });
        }

        Ok(AllowancesResult(results))
    }

    /// Generates approval calldatas for tokens that need increased allowances.
    ///
    /// Automatically checks current allowances and generates approval calldata
    /// whenever the on-chain allowance differs from the planned deposit amount.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await gui.generateApprovalCalldatas(walletAddress);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // If there are approvals
    /// const [approval1, approval2, ...] = result.value;
    /// const {
    ///   // token is the token address
    ///   token,
    ///   // calldata is the approval calldata
    ///   calldata,
    /// } = approval1;
    /// ```
    #[wasm_export(
        js_name = "generateApprovalCalldatas",
        unchecked_return_type = "ApprovalCalldataResult",
        return_description = "Approval calldatas needed for insufficient allowances"
    )]
    pub async fn generate_approval_calldatas(
        &mut self,
        #[wasm_export(param_description = "Wallet address that will approve the tokens")]
        owner: String,
    ) -> Result<ApprovalCalldataResult, GuiError> {
        let deposits_map = self.get_deposits_as_map().await?;
        if deposits_map.is_empty() {
            return Ok(ApprovalCalldataResult::NoDeposits);
        }

        let mut calldatas = Vec::new();

        for (token_address, deposit_amount) in &deposits_map {
            let tx_args = self.get_transaction_args()?;
            let rpcs = tx_args
                .rpcs
                .iter()
                .map(|rpc| Url::parse(rpc))
                .collect::<Result<Vec<_>, _>>()?;

            let erc20 = ERC20::new(rpcs, *token_address);
            let decimals = erc20.decimals().await?;

            let deposit_args = DepositArgs {
                token: *token_address,
                amount: *deposit_amount,
                decimals,
                vault_id: B256::ZERO,
            };

            let token_allowance = self.check_allowance(&deposit_args, &owner).await?;
            let allowance_float = Float::from_fixed_decimal(token_allowance.allowance, decimals)?;

            if !allowance_float.eq(*deposit_amount)? {
                let calldata = approveCall {
                    spender: tx_args.orderbook_address,
                    amount: deposit_amount.to_fixed_decimal(decimals)?,
                }
                .abi_encode();

                calldatas.push(ApprovalCalldata {
                    token: *token_address,
                    calldata: Bytes::copy_from_slice(&calldata),
                });
            }
        }

        Ok(ApprovalCalldataResult::Calldatas(calldatas))
    }

    fn populate_vault_ids(&mut self, deployment: &GuiDeploymentCfg) -> Result<(), GuiError> {
        self.dotrain_order
            .dotrain_yaml()
            .get_order(&deployment.deployment.order.key)?
            .populate_vault_ids()?;
        Ok(())
    }

    fn update_bindings(&mut self, deployment: &GuiDeploymentCfg) -> Result<(), GuiError> {
        self.dotrain_order
            .dotrain_yaml()
            .get_scenario(&deployment.deployment.scenario.key)?
            .update_bindings(
                self.field_values
                    .keys()
                    .map(|k| Ok((k.clone(), self.get_field_value(k.clone())?.value.clone())))
                    .collect::<Result<HashMap<String, String>, GuiError>>()?,
            )?;
        Ok(())
    }

    async fn prepare_add_order_args(
        &mut self,
        deployment: &GuiDeploymentCfg,
    ) -> Result<AddOrderArgs, GuiError> {
        let dotrain_gui_state_instance_v1 = self.generate_dotrain_gui_state_instance_v1()?;
        let dotrain_gui_state_meta =
            RainMetaDocumentV1Item::try_from(dotrain_gui_state_instance_v1)?;

        let dotrain_for_deployment = self
            .dotrain_order
            .generate_dotrain_for_deployment(&deployment.deployment.key)?;

        let add_order_args = AddOrderArgs::new_from_deployment(
            dotrain_for_deployment,
            deployment.deployment.as_ref().clone(),
            Some(vec![dotrain_gui_state_meta]),
        )
        .await?;

        Ok(add_order_args)
    }

    /// Generates calldata for depositing tokens into orderbook vaults.
    ///
    /// Creates deposit calldatas for all configured deposits, automatically
    /// skipping zero amounts and ensuring vault IDs are properly assigned.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await gui.generateDepositCalldatas();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // If there are deposits
    /// const [depositCalldata1, depositCalldata2, ...] = result.value;
    /// const {
    ///   // calldata is the deposit calldata
    ///   calldata,
    /// } = depositCalldata1;
    /// ```
    #[wasm_export(
        js_name = "generateDepositCalldatas",
        unchecked_return_type = "DepositCalldataResult",
        return_description = "Deposit calldatas to execute or NoDeposits if none configured"
    )]
    pub async fn generate_deposit_calldatas(&mut self) -> Result<DepositCalldataResult, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::Deposit)?;

        let vaults_and_deposits = self.get_vaults_and_deposits(&deployment).await?;
        if vaults_and_deposits.is_empty() {
            return Ok(DepositCalldataResult::NoDeposits);
        }

        let mut calldatas = Vec::new();
        for VaultAndDeposit {
            order_io,
            deposit_amount,
            index,
        } in vaults_and_deposits
        {
            if deposit_amount.eq(Float::parse("0".to_string())?)? {
                continue;
            }

            let token = order_io
                .token
                .as_ref()
                .ok_or(GuiError::SelectTokensNotSet)?;
            let vault_id = order_io
                .vault_id
                .ok_or(GuiError::VaultIdNotFound(index.to_string()))?;

            let decimals = if let Some(decimals) = token.decimals {
                decimals
            } else {
                let tx_args = self.get_transaction_args()?;
                let rpcs = tx_args
                    .rpcs
                    .iter()
                    .map(|rpc| Url::parse(rpc))
                    .collect::<Result<Vec<_>, _>>()?;
                let erc20 = ERC20::new(rpcs, token.address);
                erc20.decimals().await?
            };

            let deposit_args = DepositArgs {
                token: token.address,
                amount: deposit_amount,
                vault_id: vault_id.into(),
                decimals,
            };
            let calldata = deposit4Call::try_from(deposit_args)
                .map_err(rain_orderbook_common::deposit::DepositError::from)?
                .abi_encode();
            calldatas.push(Bytes::copy_from_slice(&calldata));
        }

        Ok(DepositCalldataResult::Calldatas(calldatas))
    }

    /// Generates calldata for adding the order to the orderbook.
    ///
    /// Creates the addOrder calldata with all field values applied to the
    /// Rainlang code and proper vault configurations.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await gui.generateAddOrderCalldata();
    /// if (result.error) {
    ///   console.error("Cannot create order:", result.error.readableMsg);
    ///   // Show user what needs to be fixed
    ///   return;
    /// }
    /// const addOrderCalldata = result.value;
    /// // Do something with the add order calldata
    /// ```
    #[wasm_export(
        js_name = "generateAddOrderCalldata",
        unchecked_return_type = "AddOrderCalldataResult",
        return_description = "Encoded addOrder call ready for execution"
    )]
    pub async fn generate_add_order_calldata(
        &mut self,
    ) -> Result<AddOrderCalldataResult, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::AddOrder)?;

        let add_order_args = self.prepare_add_order_args(&deployment).await?;
        let transaction_args = self.get_transaction_args()?;

        let add_order_call = add_order_args
            .try_into_call(transaction_args.rpcs.clone())
            .await?;

        Ok(AddOrderCalldataResult(Bytes::copy_from_slice(
            &add_order_call.abi_encode(),
        )))
    }

    /// Generates a multicall combining all deposits and add order in one calldata.
    ///
    /// This is the most efficient way to deploy an order, combining all necessary
    /// operations into a single calldata to minimize gas costs and ensure atomicity.
    ///
    /// # Transaction Structure
    ///
    /// The multicall includes:
    /// 1. AddOrder call (always first)
    /// 2. All deposit calls for non-zero amounts
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await gui.generateDepositAndAddOrderCalldatas();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const multicallData = result.value;
    /// // Do something with the multicall data
    /// ```
    #[wasm_export(
        js_name = "generateDepositAndAddOrderCalldatas",
        unchecked_return_type = "DepositAndAddOrderCalldataResult",
        return_description = "Multicall calldata combining deposits and addOrder"
    )]
    pub async fn generate_deposit_and_add_order_calldatas(
        &mut self,
    ) -> Result<DepositAndAddOrderCalldataResult, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::DepositAndAddOrder)?;

        let mut calls = Vec::new();

        let deposit_calldatas = self.generate_deposit_calldatas().await?;

        let deposit_calldatas = match deposit_calldatas {
            DepositCalldataResult::Calldatas(calldatas) => calldatas,
            DepositCalldataResult::NoDeposits => Vec::new(),
        };

        let add_order_args = self.prepare_add_order_args(&deployment).await?;
        let transaction_args = self.get_transaction_args()?;

        let add_order_call = add_order_args
            .try_into_call(transaction_args.rpcs.clone())
            .await?;
        let add_order_calldata = Bytes::copy_from_slice(&add_order_call.abi_encode());

        calls.push(Bytes::copy_from_slice(&add_order_calldata.0));

        for calldata in deposit_calldatas.iter() {
            calls.push(Bytes::copy_from_slice(calldata));
        }

        Ok(DepositAndAddOrderCalldataResult(Bytes::copy_from_slice(
            &multicallCall { data: calls }.abi_encode(),
        )))
    }

    /// Configures vault IDs for order inputs or outputs.
    ///
    /// Sets the vault ID for a specific input or output token. Vault IDs determine
    /// which vaults are used for the input or output tokens in the order.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result1 = gui.setVaultId("input", "token1", "42");
    /// if (result1.error) {
    ///   console.error("Error:", result1.error.readableMsg);
    ///   return;
    /// }
    /// const result2 = gui.setVaultId("output", "token2", "43");
    /// const result3 = gui.setVaultId("output", "token2", undefined);
    /// ```
    #[wasm_export(js_name = "setVaultId", unchecked_return_type = "void")]
    pub fn set_vault_id(
        &mut self,
        #[wasm_export(param_description = "Vault type: 'input' or 'output'")] r#type: VaultType,
        #[wasm_export(param_description = "Token key to identify which token to set vault for")]
        token: String,
        #[wasm_export(
            js_name = "vaultId",
            param_description = "Vault ID number as string. Omit to clear vault ID"
        )]
        vault_id: Option<String>,
    ) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        self.dotrain_order
            .dotrain_yaml()
            .get_order(&deployment.deployment.order.key)?
            .update_vault_id(r#type, token, vault_id)?;

        self.execute_state_update_callback()?;
        Ok(())
    }

    /// Gets all configured vault IDs for inputs and outputs.
    ///
    /// Returns a map with 'input' and 'output' keys, where each value is a map
    /// of token keys to their configured vault IDs (or undefined if not set).
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = gui.getVaultIds();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// // Access input token vault IDs
    /// for (const [tokenKey, vaultId] of result.value.get('input')) {
    ///   console.log(`Input token ${tokenKey} uses vault ${vaultId || 'none'}`);
    /// }
    ///
    /// // Access output token vault IDs
    /// for (const [tokenKey, vaultId] of result.value.get('output')) {
    ///   console.log(`Output token ${tokenKey} uses vault ${vaultId || 'none'}`);
    /// }
    /// ```
    #[wasm_export(
        js_name = "getVaultIds",
        unchecked_return_type = "IOVaultIds",
        return_description = "Map with 'input' and 'output' keys containing token-to-vault-ID maps"
    )]
    pub fn get_vault_ids(&self) -> Result<IOVaultIds, GuiError> {
        let deployment = self.get_current_deployment()?;

        let mut input_map = HashMap::new();
        for input in deployment.deployment.order.inputs.iter() {
            let token_key = input
                .token
                .as_ref()
                .map(|t| t.key.clone())
                .ok_or(GuiError::SelectTokensNotSet)?;
            input_map.insert(token_key, input.vault_id);
        }

        let mut output_map = HashMap::new();
        for output in deployment.deployment.order.outputs.iter() {
            let token_key = output
                .token
                .as_ref()
                .map(|t| t.key.clone())
                .ok_or(GuiError::SelectTokensNotSet)?;
            output_map.insert(token_key, output.vault_id);
        }

        let map = HashMap::from([
            ("input".to_string(), input_map),
            ("output".to_string(), output_map),
        ]);
        Ok(IOVaultIds(map))
    }

    /// Checks if any vault IDs have been configured.
    ///
    /// Quick validation to determine if vault configuration has started.
    /// Useful for UI state management and validation flows.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = gui.hasAnyVaultId();
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const hasVaults = result.value;
    /// // Do something with the has vaults
    /// ```
    #[wasm_export(
        js_name = "hasAnyVaultId",
        unchecked_return_type = "boolean",
        return_description = "True if at least one vault ID is set"
    )]
    pub fn has_any_vault_id(&self) -> Result<bool, GuiError> {
        let map = self.get_vault_ids()?;
        Ok(map
            .0
            .values()
            .any(|token_map| token_map.values().any(|vault_id| vault_id.is_some())))
    }

    #[wasm_export(skip)]
    pub fn update_scenario_bindings(&mut self) -> Result<(), GuiError> {
        let deployment = self.get_current_deployment()?;
        self.update_bindings(&deployment)?;
        Ok(())
    }

    /// Gets transaction data for order deployment including approvals.
    ///
    /// This is the comprehensive function that provides everything needed to deploy
    /// an order: approval calldatas, the main deployment transaction, and metadata.
    /// Use this for full transaction orchestration.
    ///
    /// # Transaction Package
    ///
    /// - `approvals` - Token approval calldatas with symbols for UI
    /// - `deploymentCalldata` - Main order deployment calldata
    /// - `orderbookAddress` - Target contract address
    /// - `chainId` - Network identifier
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const result = await gui.getDeploymentTransactionArgs(walletAddress);
    /// if (result.error) {
    ///   console.error("Error:", result.error.readableMsg);
    ///   return;
    /// }
    ///
    /// const {
    ///   // approvals is an array of extended approval calldatas
    ///   // extended approval calldata includes the token address, calldata, and symbol
    ///   approvals,
    ///   // deploymentCalldata is the multicall calldata for the order
    ///   deploymentCalldata,
    ///   // orderbookAddress is the address of the orderbook
    ///   orderbookAddress,
    ///   // chainId is the chain ID of the network
    ///   chainId,
    /// } = result.value;
    /// ```
    #[wasm_export(
        js_name = "getDeploymentTransactionArgs",
        unchecked_return_type = "DeploymentTransactionArgs",
        return_description = "Complete transaction package including approvals and deployment calldata"
    )]
    pub async fn get_deployment_transaction_args(
        &mut self,
        #[wasm_export(param_description = "Wallet address that will deploy the order")]
        owner: String,
    ) -> Result<DeploymentTransactionArgs, GuiError> {
        let deployment = self.prepare_calldata_generation(CalldataFunction::DepositAndAddOrder)?;

        let mut approvals = Vec::new();
        let approval_calldata = self.generate_approval_calldatas(owner).await?;
        if let ApprovalCalldataResult::Calldatas(calldatas) = approval_calldata {
            let mut output_token_infos = HashMap::new();
            for output in deployment.deployment.order.outputs.clone() {
                if output.token.is_none() {
                    return Err(GuiError::SelectTokensNotSet);
                }
                let token = output.token.as_ref().unwrap();
                let token_info = self.get_token_info(token.key.clone()).await?;
                output_token_infos.insert(token.address, token_info);
            }

            for calldata in calldatas.iter() {
                let token_info = output_token_infos
                    .get(&calldata.token)
                    .ok_or(GuiError::TokenNotFound(calldata.token.to_string()))?;
                approvals.push(ExtendedApprovalCalldata {
                    token: calldata.token,
                    calldata: calldata.calldata.clone(),
                    symbol: token_info.symbol.clone(),
                });
            }
        }

        let deposit_calldata_result = self.generate_deposit_calldatas().await?;
        let deposit_calldatas = match deposit_calldata_result {
            DepositCalldataResult::Calldatas(calldatas) => calldatas,
            DepositCalldataResult::NoDeposits => Vec::new(),
        };

        let add_order_args = self.prepare_add_order_args(&deployment).await?;

        let transaction_args = self.get_transaction_args()?;
        let add_order_call = add_order_args
            .try_into_call(transaction_args.rpcs.clone())
            .await?;

        let mut calls = Vec::new();
        calls.push(Bytes::copy_from_slice(&add_order_call.abi_encode()));
        for calldata in deposit_calldatas.iter() {
            calls.push(Bytes::copy_from_slice(calldata));
        }

        let deployment_calldata =
            Bytes::copy_from_slice(&multicallCall { data: calls }.abi_encode());

        let emit_meta_call = if self.should_emit_meta_call().await? {
            let client = self.get_metaboard_client()?;
            let addresses = client.get_metaboard_addresses(None, None).await?;
            let metaboard_address = addresses
                .first()
                .ok_or_else(|| GuiError::NoAddressInMetaboardSubgraph)?;

            let calldata = add_order_args.try_into_emit_meta_call()?;
            calldata.map(|calldata| ExternalCall {
                to: *metaboard_address,
                calldata: Bytes::copy_from_slice(&calldata.abi_encode()),
            })
        } else {
            None
        };

        Ok(DeploymentTransactionArgs {
            approvals,
            deployment_calldata,
            orderbook_address: deployment
                .deployment
                .order
                .orderbook
                .as_ref()
                .ok_or(GuiError::OrderbookNotFound)?
                .address,
            chain_id: deployment.deployment.order.network.chain_id,
            emit_meta_call,
        })
    }

    fn get_metaboard_client(&self) -> Result<MetaboardSubgraphClient, GuiError> {
        let deployment = self.get_current_deployment()?;
        let orderbook_yaml = self.dotrain_order.orderbook_yaml();
        let metaboard_cfg =
            orderbook_yaml.get_metaboard(&deployment.deployment.order.network.key)?;
        Ok(MetaboardSubgraphClient::new(metaboard_cfg.url.clone()))
    }

    async fn should_emit_meta_call(&self) -> Result<bool, GuiError> {
        let dotrain_gui_state = self.generate_dotrain_gui_state_instance_v1()?;
        let subject = dotrain_gui_state.dotrain_hash();

        let client = self.get_metaboard_client()?;
        match client
            .get_metabytes_by_subject(&MetaBigInt(format!("0x{}", alloy::hex::encode(subject))))
            .await
        {
            Ok(metas) => Ok(metas.is_empty()),
            Err(MetaboardSubgraphClientError::Empty(_)) => Ok(true),
            Err(err) => Err(GuiError::MetaboardSubgraphClientError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::tests::{initialize_gui, initialize_gui_with_select_tokens};
    #[cfg(all(test, not(target_family = "wasm")))]
    use httpmock::{Method::POST, MockServer};
    use rain_metadata::{types::dotrain::source_v1::DotrainSourceV1, RainMetaDocumentV1Item};
    #[cfg(all(test, not(target_family = "wasm")))]
    use serde_json::json;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_generate_deposit_calldatas() {
        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;

        let res = gui.generate_deposit_calldatas().await.unwrap();
        match res {
            DepositCalldataResult::Calldatas(_) => {
                panic!("should not be calldatas");
            }
            DepositCalldataResult::NoDeposits => {}
        }

        gui.set_deposit("token1".to_string(), "1200".to_string())
            .await
            .unwrap();

        let res = gui.generate_deposit_calldatas().await.unwrap();
        match res {
            DepositCalldataResult::Calldatas(calldatas) => {
                assert_eq!(calldatas.len(), 1);
                assert_eq!(calldatas[0].len(), 164);
            }
            DepositCalldataResult::NoDeposits => {
                panic!("should not be no deposits");
            }
        }

        gui.set_deposit("token1".to_string(), "0".to_string())
            .await
            .unwrap();

        let res = gui.generate_deposit_calldatas().await.unwrap();
        match res {
            DepositCalldataResult::Calldatas(calldatas) => {
                assert!(calldatas.is_empty());
            }
            DepositCalldataResult::NoDeposits => {
                panic!("should not be no deposits");
            }
        }
    }

    #[wasm_bindgen_test]
    async fn test_missing_select_tokens() {
        let mut gui = initialize_gui_with_select_tokens().await;

        let err = gui
            .check_allowances(Address::random().to_string())
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        let err = gui.generate_deposit_calldatas().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        let err = gui.generate_add_order_calldata().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );

        let err = gui
            .generate_deposit_and_add_order_calldatas()
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::TokenMustBeSelected("token3".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The token 'token3' must be selected to proceed."
        );
    }

    #[wasm_bindgen_test]
    async fn test_missing_field_values() {
        let mut gui = initialize_gui(None).await;

        let err = gui.generate_add_order_calldata().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The value for field 'Field 2 name' is required but has not been set."
        );

        let err = gui
            .generate_deposit_and_add_order_calldatas()
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            GuiError::FieldValueNotSet("Field 2 name".to_string()).to_string()
        );
        assert_eq!(
            err.to_readable_msg(),
            "The value for field 'Field 2 name' is required but has not been set."
        );
    }

    #[wasm_bindgen_test]
    async fn test_prepare_add_order_args_injects_gui_meta() {
        let mut gui = initialize_gui(None).await;
        gui.set_field_value("binding-1".to_string(), "10".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();

        let deployment = gui.get_current_deployment().unwrap();
        let trimmed_dotrain = gui
            .dotrain_order
            .generate_dotrain_for_deployment(&deployment.deployment.key)
            .unwrap();

        let add_order_args = gui
            .prepare_add_order_args(&deployment)
            .await
            .expect("add order args");

        // additional_meta should carry the GUI state document
        let additional_meta = add_order_args
            .additional_meta
            .as_ref()
            .expect("meta missing");
        assert_eq!(additional_meta.len(), 1);
        assert_eq!(
            additional_meta[0].magic,
            rain_metadata::KnownMagic::DotrainGuiStateV1
        );

        // try_into_emit_meta_call should emit a DotrainSourceV1 for the trimmed deployment dotrain
        let emit_meta_call = add_order_args
            .try_into_emit_meta_call()
            .expect("emit meta call err")
            .expect("emit meta call missing");
        let decoded = RainMetaDocumentV1Item::cbor_decode(emit_meta_call.meta.as_ref()).unwrap();
        assert_eq!(decoded.len(), 1);
        let dotrain_source = DotrainSourceV1::try_from(decoded[0].clone()).unwrap();
        assert_eq!(dotrain_source.0, trimmed_dotrain);

        // Subject must match the dotrain hash
        assert_eq!(
            emit_meta_call.subject,
            DotrainSourceV1(trimmed_dotrain).hash()
        );
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    async fn initialize_gui_with_metaboard_url(url: &str) -> DotrainOrderGui {
        let yaml = crate::gui::tests::get_yaml().replace("https://metaboard.com", url);
        DotrainOrderGui::new_with_deployment(yaml, None, "some-deployment".to_string(), None)
            .await
            .unwrap()
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    #[tokio::test]
    async fn test_should_emit_meta_call_false_when_meta_exists() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "metaV1S": [
                        {
                            "meta": "0x01",
                            "metaHash": "0x00",
                            "sender": "0x00",
                            "id": "0x00",
                            "metaBoard": {
                                "id": "0x00",
                                "metas": [],
                                "address": "0x00"
                            },
                            "subject": "0x00"
                        }
                    ]
                }
            }));
        });

        let mut gui = initialize_gui_with_metaboard_url(&server.url("/")).await;
        gui.set_field_value("binding-1".to_string(), "10".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();

        assert!(!gui.should_emit_meta_call().await.unwrap());
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    #[tokio::test]
    async fn test_should_emit_meta_call_true_when_no_meta() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "metaV1S": []
                }
            }));
        });

        let mut gui = initialize_gui_with_metaboard_url(&server.url("/")).await;
        gui.set_field_value("binding-1".to_string(), "10".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();

        assert!(gui.should_emit_meta_call().await.unwrap());
    }

    #[cfg(all(test, not(target_family = "wasm")))]
    #[tokio::test]
    async fn test_should_emit_meta_call_propagates_errors() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let mut gui = initialize_gui_with_metaboard_url(&server.url("/")).await;
        gui.set_field_value("binding-1".to_string(), "10".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "0".to_string())
            .unwrap();

        let err = gui.should_emit_meta_call().await.unwrap_err();
        assert!(matches!(err, GuiError::MetaboardSubgraphClientError(_)));
    }

    #[wasm_bindgen_test]
    async fn test_get_vault_ids() {
        let gui = initialize_gui(None).await;
        let res = gui.get_vault_ids().unwrap();
        assert_eq!(res.0.len(), 2);
        assert_eq!(res.0["input"]["token1"], Some(U256::from(1)));
        assert_eq!(res.0["output"]["token2"], Some(U256::from(1)));

        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;

        let res = gui.get_vault_ids().unwrap();
        assert_eq!(res.0.len(), 2);
        assert_eq!(res.0["input"]["token1"], None);
        assert_eq!(res.0["output"]["token1"], None);

        gui.set_vault_id(
            VaultType::Input,
            "token1".to_string(),
            Some("999".to_string()),
        )
        .unwrap();
        gui.set_vault_id(
            VaultType::Output,
            "token1".to_string(),
            Some("888".to_string()),
        )
        .unwrap();

        let res = gui.get_vault_ids().unwrap();
        assert_eq!(res.0.len(), 2);
        assert_eq!(res.0["input"]["token1"], Some(U256::from(999)));
        assert_eq!(res.0["output"]["token1"], Some(U256::from(888)));
    }

    #[wasm_bindgen_test]
    async fn test_has_any_vault_id() {
        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;
        assert!(!gui.has_any_vault_id().unwrap());
        gui.set_vault_id(
            VaultType::Input,
            "token1".to_string(),
            Some("1".to_string()),
        )
        .unwrap();
        assert!(gui.has_any_vault_id().unwrap());
    }

    #[wasm_bindgen_test]
    async fn test_update_scenario_bindings() {
        let mut gui = initialize_gui(Some("other-deployment".to_string())).await;

        let deployment = gui.get_current_deployment().unwrap();
        assert!(!deployment
            .deployment
            .scenario
            .bindings
            .contains_key("binding-1"));
        assert!(!deployment
            .deployment
            .scenario
            .bindings
            .contains_key("binding-2"));

        gui.set_field_value("binding-1".to_string(), "100".to_string())
            .unwrap();
        gui.set_field_value("binding-2".to_string(), "200".to_string())
            .unwrap();
        gui.update_scenario_bindings().unwrap();

        let deployment = gui.get_current_deployment().unwrap();
        assert_eq!(deployment.deployment.scenario.bindings["binding-1"], "100");
        assert_eq!(deployment.deployment.scenario.bindings["binding-2"], "200");
    }
}
