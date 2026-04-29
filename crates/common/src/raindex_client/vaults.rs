use super::ClientRef;
use super::*;
use crate::local_db::query::fetch_order_vaults_volume::LocalDbVaultVolume;
use crate::local_db::query::fetch_vaults::LocalDbVault;
use crate::local_db::{
    query::fetch_vault_balance_changes::LocalDbVaultBalanceChange, OrderbookIdentifier,
};
use crate::raindex_client::local_db::vaults::LocalDbVaults;
use crate::raindex_client::QuerySource;
use crate::types::VaultBalanceChangeKind;
use crate::{
    deposit::DepositArgs,
    erc20::ERC20,
    raindex_client::{
        orders::RaindexOrderAsIO, transactions::RaindexTransaction, vaults_list::RaindexVaultsList,
    },
    transaction::TransactionArgs,
    withdraw::WithdrawArgs,
};
use alloy::sol_types::SolCall;
use alloy::{
    hex,
    primitives::{Address, Bytes, B256, U256},
};
use async_trait::async_trait;
use rain_math_float::Float;
use rain_orderbook_bindings::{IRaindexV6::deposit4Call, IERC20::approveCall};
use rain_orderbook_subgraph_client::{
    performance::vol::{VaultVolume, VolumeDetails},
    types::{
        common::{
            SgBigInt, SgBytes, SgErc20, SgOrderAsIO, SgOrderbook, SgTradeVaultBalanceChange,
            SgVault, SgVaultBalanceChangeType, SgVaultBalanceChangeUnwrapped,
            SgVaultsListFilterArgs,
        },
        Id,
    },
    MultiOrderbookSubgraphClient, OrderbookSubgraphClient, OrderbookSubgraphClientError,
    SgPaginationArgs,
};
use std::str::FromStr;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::BigInt;

const DEFAULT_PAGE_SIZE: u16 = 100;

pub(crate) struct SubgraphVaults<'a> {
    client: &'a RaindexClient,
}
impl<'a> SubgraphVaults<'a> {
    pub(crate) fn new(client: &'a RaindexClient) -> Self {
        Self { client }
    }
}

#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
pub(crate) trait VaultsDataSource {
    async fn list(
        &self,
        chain_ids: Option<Vec<u32>>,
        filters: &GetVaultsFilters,
        page: Option<u16>,
    ) -> Result<Vec<RaindexVault>, RaindexError>;

    async fn get_by_id(
        &self,
        ob_id: &OrderbookIdentifier,
        vault_id: &Bytes,
    ) -> Result<Option<RaindexVault>, RaindexError>;

    async fn balance_changes_list(
        &self,
        vault: &RaindexVault,
        page: Option<u16>,
        filter_types: Option<&[VaultBalanceChangeFilter]>,
    ) -> Result<Vec<RaindexVaultBalanceChange>, RaindexError>;

    async fn tokens_list(
        &self,
        chain_ids: Option<Vec<u32>>,
    ) -> Result<Vec<RaindexVaultToken>, RaindexError>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum RaindexVaultType {
    Input,
    Output,
    InputOutput,
}
impl_wasm_traits!(RaindexVaultType);

/// Represents a vault with balance and token information within a given orderbook.
///
/// A vault is a fundamental component that holds tokens and participates in order execution.
/// Each vault has a unique identifier, current balance, associated token metadata, and
/// belongs to a specific orderbook contract on the blockchain.
///
/// Vaults can serve different roles in relation to orders - they can provide tokens (input),
/// receive tokens (output), or both (input/output), depending on the trading algorithm.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVault {
    raindex_client: ClientRef,
    chain_id: u32,
    vault_type: Option<RaindexVaultType>,
    id: Bytes,
    owner: Address,
    vault_id: U256,
    balance: Float,
    formatted_balance: String,
    token: RaindexVaultToken,
    orderbook: Address,
    orders_as_inputs: Vec<RaindexOrderAsIO>,
    orders_as_outputs: Vec<RaindexOrderAsIO>,
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVault {
    fn u256_to_bigint(value: U256) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&value.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }

    #[wasm_bindgen(getter = chainId)]
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    #[wasm_bindgen(getter = vaultType)]
    pub fn vault_type(&self) -> Option<RaindexVaultType> {
        self.vault_type.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Hex")]
    pub fn id(&self) -> String {
        self.id.to_string()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn owner(&self) -> String {
        self.owner.to_string()
    }
    pub(crate) fn vault_id_hex(&self) -> String {
        hex::encode_prefixed(self.vault_id.to_be_bytes::<32>())
    }
    #[wasm_bindgen(getter = vaultId)]
    pub fn vault_id(&self) -> Result<BigInt, RaindexError> {
        Self::u256_to_bigint(self.vault_id)
    }
    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> Float {
        self.balance
    }
    #[wasm_bindgen(getter = formattedBalance)]
    pub fn formatted_balance(&self) -> String {
        self.formatted_balance.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
    #[wasm_bindgen(getter = ordersAsInput)]
    pub fn orders_as_inputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_inputs.clone()
    }
    #[wasm_bindgen(getter = ordersAsOutput)]
    pub fn orders_as_outputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_outputs.clone()
    }
}

#[cfg(not(target_family = "wasm"))]
impl RaindexVault {
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    pub fn vault_type(&self) -> Option<RaindexVaultType> {
        self.vault_type.clone()
    }
    pub fn id(&self) -> Bytes {
        self.id.clone()
    }
    pub fn owner(&self) -> Address {
        self.owner
    }
    pub fn vault_id(&self) -> U256 {
        self.vault_id
    }
    pub fn balance(&self) -> Float {
        self.balance
    }
    pub fn formatted_balance(&self) -> String {
        self.formatted_balance.clone()
    }
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
    pub fn orders_as_inputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_inputs.clone()
    }
    pub fn orders_as_outputs(&self) -> Vec<RaindexOrderAsIO> {
        self.orders_as_outputs.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct AccountBalance {
    balance: Float,
    formatted_balance: String,
}
impl AccountBalance {
    pub fn new(balance: Float, formatted_balance: String) -> Self {
        Self {
            balance,
            formatted_balance,
        }
    }
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl AccountBalance {
    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> Float {
        self.balance
    }
    #[wasm_bindgen(getter = formattedBalance)]
    pub fn formatted_balance(&self) -> String {
        self.formatted_balance.clone()
    }
}
#[cfg(not(target_family = "wasm"))]
impl AccountBalance {
    pub fn balance(&self) -> Float {
        self.balance
    }
    pub fn formatted_balance(&self) -> String {
        self.formatted_balance.clone()
    }
}

/// Token metadata associated with a vault in the Raindex system.
///
/// Contains comprehensive information about the ERC20 token held within a vault,
/// including contract address, human-readable identifiers, and decimal precision.
/// Some fields may be optional as they depend on the token's implementation and
/// whether the metadata has been successfully retrieved from the blockchain.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVaultToken {
    chain_id: u32,
    id: String,
    address: Address,
    name: Option<String>,
    symbol: Option<String>,
    decimals: u8,
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVaultToken {
    #[wasm_bindgen(getter = chainId)]
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn address(&self) -> String {
        self.address.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn decimals(&self) -> u8 {
        self.decimals
    }
}

#[cfg(not(target_family = "wasm"))]
impl RaindexVaultToken {
    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }
    pub fn id(&self) -> String {
        self.id.clone()
    }
    pub fn address(&self) -> Address {
        self.address
    }
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
    pub fn symbol(&self) -> Option<String> {
        self.symbol.clone()
    }
    pub fn decimals(&self) -> u8 {
        self.decimals
    }
}

#[wasm_export]
impl RaindexVault {
    #[wasm_export(skip)]
    pub fn get_orderbook_client(&self) -> Result<OrderbookSubgraphClient, RaindexError> {
        self.raindex_client.get_orderbook_client(self.orderbook)
    }

    /// Fetches balance change history for a vault
    ///
    /// Retrieves chronological list of deposits, withdrawals, and trades affecting
    /// a vault's balance. Optionally filter by balance change type.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// // Fetch all balance changes
    /// const result = await vault.getBalanceChanges();
    /// if (result.error) {
    ///   console.error("Error fetching history:", result.error.readableMsg);
    ///   return;
    /// }
    /// const changes = result.value;
    ///
    /// // Fetch only deposits and withdrawals
    /// const filteredResult = await vault.getBalanceChanges(1, ["deposit", "withdrawal"]);
    /// ```
    #[wasm_export(
        js_name = "getBalanceChanges",
        return_description = "Array of balance change events",
        unchecked_return_type = "RaindexVaultBalanceChange[]",
        preserve_js_class
    )]
    pub async fn get_balance_changes(
        &self,
        #[wasm_export(param_description = "Optional page number (default to 1)")] page: Option<u16>,
        #[wasm_export(
            param_description = "Optional filter types array (deposit, withdrawal, takeOrder, clear, clearBounty)"
        )]
        filter_types: Option<Vec<VaultBalanceChangeFilter>>,
    ) -> Result<Vec<RaindexVaultBalanceChange>, RaindexError> {
        match self.raindex_client.query_source(self.chain_id) {
            QuerySource::LocalDb(local_db) => {
                let local_source =
                    LocalDbVaults::new(&local_db, ClientRef::clone(&self.raindex_client));
                local_source
                    .balance_changes_list(self, page, filter_types.as_deref())
                    .await
            }
            QuerySource::Subgraph => {
                let subgraph_source = SubgraphVaults::new(&self.raindex_client);
                subgraph_source
                    .balance_changes_list(self, page, filter_types.as_deref())
                    .await
            }
        }
    }

    fn validate_amount(&self, amount: &Float) -> Result<(), RaindexError> {
        let zero_float = Float::parse("0".to_string())?;
        if amount.is_zero()? {
            return Err(RaindexError::ZeroAmount);
        }
        if amount.lt(zero_float)? {
            return Err(RaindexError::NegativeAmount);
        }
        Ok(())
    }

    /// Generates transaction calldata for depositing tokens into a vault
    ///
    /// Creates the contract calldata needed to deposit a specified amount of tokens
    /// into a vault.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getDepositCalldata(vault, "10.5");
    /// if (result.error) {
    ///   console.error("Cannot generate deposit:", result.error.readableMsg);
    ///   return;
    /// }
    /// const calldata = result.value;
    /// // Do something with the calldata
    /// ```
    #[wasm_export(
        js_name = "getDepositCalldata",
        return_description = "Encoded transaction calldata as hex string",
        unchecked_return_type = "Hex"
    )]
    pub async fn get_deposit_calldata(
        &self,
        #[wasm_export(param_description = "Amount to deposit in Float value")] amount: &Float,
    ) -> Result<Bytes, RaindexError> {
        self.validate_amount(amount)?;
        let (deposit_args, _) = self.get_deposit_and_transaction_args(amount).await?;
        let call = deposit4Call::try_from(deposit_args)?;
        Ok(Bytes::copy_from_slice(&call.abi_encode()))
    }

    /// Generates transaction calldata for withdrawing tokens from a vault
    ///
    /// Creates the contract calldata needed to withdraw a specified amount of tokens
    /// from a vault.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getWithdrawCalldata("55.2");
    /// if (result.error) {
    ///   console.error("Cannot generate withdrawal:", result.error.readableMsg);
    ///   return;
    /// }
    /// const calldata = result.value;
    /// // Do something with the calldata
    /// ```
    #[wasm_export(
        js_name = "getWithdrawCalldata",
        return_description = "Encoded transaction calldata as hex string",
        unchecked_return_type = "Hex"
    )]
    pub async fn get_withdraw_calldata(
        &self,
        #[wasm_export(param_description = "Amount to withdraw in Float value")] amount: &Float,
    ) -> Result<Bytes, RaindexError> {
        self.validate_amount(amount)?;
        Ok(Bytes::copy_from_slice(
            &WithdrawArgs {
                token: self.token.address,
                vault_id: B256::from(self.vault_id),
                target_amount: *amount,
            }
            .get_withdraw_calldata()
            .await?,
        ))
    }

    async fn get_deposit_and_transaction_args(
        &self,
        amount: &Float,
    ) -> Result<(DepositArgs, TransactionArgs), RaindexError> {
        let rpcs = self.raindex_client.get_rpc_urls_for_chain(self.chain_id)?;

        let deposit_args = DepositArgs {
            token: self.token.address,
            vault_id: B256::from(self.vault_id),
            amount: *amount,
            decimals: self.token.decimals,
        };

        let transaction_args = TransactionArgs {
            orderbook_address: self.orderbook,
            rpcs: rpcs.iter().map(|rpc| rpc.to_string()).collect(),
            ..Default::default()
        };

        Ok((deposit_args, transaction_args))
    }

    /// Generates ERC20 approval calldata for vault deposits
    ///
    /// Creates the contract calldata needed to approve the orderbook contract to spend
    /// tokens for a vault deposit, but only if additional approval is needed.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getApprovalCalldata("20.75");
    /// if (result.error) {
    ///   console.error("Approval error:", result.error.readableMsg);
    ///   return;
    /// }
    /// const calldata = result.value;
    /// // Do something with the calldata
    /// ```
    #[wasm_export(
        js_name = "getApprovalCalldata",
        return_description = "Encoded approval calldata as hex string",
        unchecked_return_type = "Hex"
    )]
    pub async fn get_approval_calldata(
        &self,
        #[wasm_export(param_description = "Amount requiring approval in Float value")]
        amount: &Float,
    ) -> Result<Bytes, RaindexError> {
        self.validate_amount(amount)?;

        let (deposit_args, transaction_args) =
            self.get_deposit_and_transaction_args(amount).await?;

        let allowance = deposit_args
            .read_allowance(self.owner, transaction_args.clone())
            .await?;
        let allowance_float = Float::from_fixed_decimal(allowance, self.token.decimals)?;

        if allowance_float.gte(*amount)? {
            return Err(RaindexError::ExistingAllowance);
        }

        let calldata = approveCall {
            spender: transaction_args.orderbook_address,
            amount: amount.to_fixed_decimal(self.token.decimals)?,
        }
        .abi_encode();

        Ok(Bytes::copy_from_slice(&calldata))
    }

    /// Gets the current ERC20 allowance for a vault
    ///
    /// Determines how much the orderbook contract is currently approved to spend
    /// on behalf of the vault owner.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getAllowance();
    /// if (result.error) {
    ///   console.error("Cannot check allowance:", result.error.readableMsg);
    ///   return;
    /// }
    /// const allowance = result.value;
    /// // Do something with the allowance
    /// ```
    #[wasm_export(
        js_name = "getAllowance",
        return_description = "Current allowance amount in token's smallest unit (e.g., \"1000000000000000000\" for 1 token with 18 decimals)"
    )]
    pub async fn get_allowance(&self) -> Result<RaindexVaultAllowance, RaindexError> {
        let (deposit_args, transaction_args) = self
            .get_deposit_and_transaction_args(&Float::parse("0".to_string())?)
            .await?;
        let allowance = deposit_args
            .read_allowance(self.owner, transaction_args.clone())
            .await?;
        Ok(RaindexVaultAllowance(allowance))
    }

    /// Fetches the balance of the owner for this vault
    ///
    /// Retrieves the current balance of the vault owner.
    /// The returned balance is an object containing both raw and formatted values.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await vault.getOwnerBalance();
    /// if (result.error) {
    ///  console.error("Error fetching balance:", result.error.readableMsg);
    /// return;
    /// }
    /// const accountBalance = result.value;
    /// console.log("Raw balance:", accountBalance.balance);
    /// console.log("Formatted balance:", accountBalance.formattedBalance);
    /// ```
    #[wasm_export(
        js_name = "getOwnerBalance",
        return_description = "Owner balance in both raw and human-readable format",
        unchecked_return_type = "AccountBalance",
        preserve_js_class
    )]
    pub async fn get_owner_balance_wasm_binding(&self) -> Result<AccountBalance, RaindexError> {
        let balance = self.get_owner_balance(self.owner).await?;
        let decimals = self.token.decimals;
        let float_balance = Float::from_fixed_decimal(balance, decimals)?;
        let account_balance = AccountBalance {
            balance: float_balance,
            formatted_balance: float_balance.format()?,
        };
        Ok(account_balance)
    }
}
impl RaindexVault {
    pub async fn get_owner_balance(&self, owner: Address) -> Result<U256, RaindexError> {
        let rpcs = self.raindex_client.get_rpc_urls_for_chain(self.chain_id)?;
        let erc20 = ERC20::new(rpcs, self.token.address);
        Ok(erc20.get_account_balance(owner).await?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum RaindexVaultBalanceChangeType {
    Deposit,
    Withdrawal,
    TakeOrder,
    Clear,
    ClearBounty,
    Unknown,
}
impl_wasm_traits!(RaindexVaultBalanceChangeType);

impl From<VaultBalanceChangeKind> for RaindexVaultBalanceChangeType {
    fn from(kind: VaultBalanceChangeKind) -> Self {
        match kind {
            VaultBalanceChangeKind::Deposit => Self::Deposit,
            VaultBalanceChangeKind::Withdrawal => Self::Withdrawal,
            VaultBalanceChangeKind::TakeOrder => Self::TakeOrder,
            VaultBalanceChangeKind::Clear => Self::Clear,
            VaultBalanceChangeKind::ClearBounty => Self::ClearBounty,
            VaultBalanceChangeKind::Unknown => Self::Unknown,
        }
    }
}

impl TryFrom<String> for RaindexVaultBalanceChangeType {
    type Error = RaindexError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let kind = VaultBalanceChangeKind::from_local_db_change_type(&value);
        if matches!(kind, VaultBalanceChangeKind::Unknown) {
            let kind_from_sg = VaultBalanceChangeKind::from_subgraph_typename(&value);
            if matches!(kind_from_sg, VaultBalanceChangeKind::Unknown) && value != "Unknown" {
                return Err(RaindexError::InvalidVaultBalanceChangeType(value));
            }
            return Ok(kind_from_sg.into());
        }
        Ok(kind.into())
    }
}

impl RaindexVaultBalanceChangeType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Deposit => "Deposit",
            Self::Withdrawal => "Withdrawal",
            Self::TakeOrder => "Take order",
            Self::Clear => "Clear",
            Self::ClearBounty => "Clear Bounty",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum VaultBalanceChangeFilter {
    Deposit,
    Withdrawal,
    TakeOrder,
    Clear,
    ClearBounty,
}
impl_wasm_traits!(VaultBalanceChangeFilter);

impl VaultBalanceChangeFilter {
    pub fn to_kind(&self) -> VaultBalanceChangeKind {
        match self {
            Self::Deposit => VaultBalanceChangeKind::Deposit,
            Self::Withdrawal => VaultBalanceChangeKind::Withdrawal,
            Self::TakeOrder => VaultBalanceChangeKind::TakeOrder,
            Self::Clear => VaultBalanceChangeKind::Clear,
            Self::ClearBounty => VaultBalanceChangeKind::ClearBounty,
        }
    }

    pub fn to_local_db_types(&self) -> &'static [&'static str] {
        self.to_kind().to_local_db_change_types()
    }

    pub fn to_raindex_type(&self) -> RaindexVaultBalanceChangeType {
        match self {
            Self::Deposit => RaindexVaultBalanceChangeType::Deposit,
            Self::Withdrawal => RaindexVaultBalanceChangeType::Withdrawal,
            Self::TakeOrder => RaindexVaultBalanceChangeType::TakeOrder,
            Self::Clear => RaindexVaultBalanceChangeType::Clear,
            Self::ClearBounty => RaindexVaultBalanceChangeType::ClearBounty,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVaultBalanceChange {
    r#type: RaindexVaultBalanceChangeType,
    vault_id: U256,
    token: RaindexVaultToken,
    amount: Float,
    formatted_amount: String,
    new_balance: Float,
    formatted_new_balance: String,
    old_balance: Float,
    formatted_old_balance: String,
    timestamp: U256,
    transaction: RaindexTransaction,
    orderbook: Address,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVaultBalanceChange {
    #[wasm_bindgen(getter = type)]
    pub fn type_getter(&self) -> RaindexVaultBalanceChangeType {
        self.r#type.clone()
    }
    #[wasm_bindgen(getter = typeDisplayName)]
    pub fn type_display_name(&self) -> String {
        self.r#type.display_name().to_string()
    }
    #[wasm_bindgen(getter = vaultId)]
    pub fn vault_id(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.vault_id.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> Float {
        self.amount
    }
    #[wasm_bindgen(getter = formattedAmount)]
    pub fn formatted_amount(&self) -> String {
        self.formatted_amount.clone()
    }
    #[wasm_bindgen(getter = newBalance)]
    pub fn new_balance(&self) -> Float {
        self.new_balance
    }
    #[wasm_bindgen(getter = formattedNewBalance)]
    pub fn formatted_new_balance(&self) -> String {
        self.formatted_new_balance.clone()
    }
    #[wasm_bindgen(getter = oldBalance)]
    pub fn old_balance(&self) -> Float {
        self.old_balance
    }
    #[wasm_bindgen(getter = formattedOldBalance)]
    pub fn formatted_old_balance(&self) -> String {
        self.formatted_old_balance.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.timestamp.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn transaction(&self) -> RaindexTransaction {
        self.transaction.clone()
    }
    #[wasm_bindgen(getter, unchecked_return_type = "Address")]
    pub fn orderbook(&self) -> String {
        self.orderbook.to_string()
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexVaultBalanceChange {
    pub fn r#type(&self) -> RaindexVaultBalanceChangeType {
        self.r#type.clone()
    }
    pub fn type_display_name(&self) -> &'static str {
        self.r#type.display_name()
    }
    pub fn vault_id(&self) -> U256 {
        self.vault_id
    }
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    pub fn amount(&self) -> Float {
        self.amount
    }
    pub fn formatted_amount(&self) -> String {
        self.formatted_amount.clone()
    }
    pub fn new_balance(&self) -> Float {
        self.new_balance
    }
    pub fn formatted_new_balance(&self) -> String {
        self.formatted_new_balance.clone()
    }
    pub fn old_balance(&self) -> Float {
        self.old_balance
    }
    pub fn formatted_old_balance(&self) -> String {
        self.formatted_old_balance.clone()
    }
    pub fn timestamp(&self) -> U256 {
        self.timestamp
    }
    pub fn transaction(&self) -> RaindexTransaction {
        self.transaction.clone()
    }
    pub fn orderbook(&self) -> Address {
        self.orderbook
    }
}

#[derive(Clone)]
pub(crate) struct LocalTradeTokenInfo {
    pub address: Address,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Clone)]
pub(crate) struct LocalTradeBalanceInfo {
    pub delta: String,
    pub running_balance: Option<String>,
    pub trade_kind: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct RaindexVaultAllowance(#[tsify(type = "string")] U256);
impl_wasm_traits!(RaindexVaultAllowance);

impl RaindexVaultBalanceChange {
    pub fn try_from_sg_balance_change(
        chain_id: u32,
        balance_change: SgVaultBalanceChangeUnwrapped,
    ) -> Result<Self, RaindexError> {
        let token = RaindexVaultToken::try_from_sg_erc20(chain_id, balance_change.vault.token)?;

        let amount = Float::from_hex(&balance_change.amount.0)?;
        let new_balance = Float::from_hex(&balance_change.new_vault_balance.0)?;
        let old_balance = Float::from_hex(&balance_change.old_vault_balance.0)?;

        let formatted_amount = amount.format()?;
        let formatted_new_balance = new_balance.format()?;
        let formatted_old_balance = old_balance.format()?;

        Ok(Self {
            r#type: balance_change.__typename.try_into()?,
            vault_id: U256::from_str(&balance_change.vault.vault_id.0)?,
            token,
            amount,
            formatted_amount,
            new_balance,
            formatted_new_balance,
            old_balance,
            formatted_old_balance,
            timestamp: U256::from_str(&balance_change.timestamp.0)?,
            transaction: RaindexTransaction::try_from(balance_change.transaction)?,
            orderbook: Address::from_str(&balance_change.orderbook.id.0)?,
        })
    }
}

impl RaindexVaultBalanceChange {
    pub fn try_from_sg_trade_balance_change(
        chain_id: u32,
        balance_change: SgTradeVaultBalanceChange,
    ) -> Result<Self, RaindexError> {
        let token = RaindexVaultToken::try_from_sg_erc20(chain_id, balance_change.vault.token)?;

        let amount = Float::from_hex(&balance_change.amount.0)?;
        let new_balance = Float::from_hex(&balance_change.new_vault_balance.0)?;
        let old_balance = Float::from_hex(&balance_change.old_vault_balance.0)?;

        let formatted_amount = amount.format()?;
        let formatted_new_balance = new_balance.format()?;
        let formatted_old_balance = old_balance.format()?;

        let change_type: RaindexVaultBalanceChangeType =
            VaultBalanceChangeKind::from_subgraph_typename(
                &balance_change.trade.trade_event.__typename,
            )
            .into();

        Ok(Self {
            r#type: change_type,
            vault_id: U256::from_str(&balance_change.vault.vault_id.0)?,
            token,
            amount,
            formatted_amount,
            new_balance,
            formatted_new_balance,
            old_balance,
            formatted_old_balance,
            timestamp: U256::from_str(&balance_change.timestamp.0)?,
            transaction: RaindexTransaction::try_from(balance_change.transaction)?,
            orderbook: Address::from_str(&balance_change.orderbook.id.0)?,
        })
    }
}

impl RaindexVaultBalanceChange {
    pub fn try_from_sg_balance_change_type(
        chain_id: u32,
        balance_change: SgVaultBalanceChangeType,
    ) -> Result<Self, RaindexError> {
        match balance_change {
            SgVaultBalanceChangeType::Deposit(deposit) => {
                let token = RaindexVaultToken::try_from_sg_erc20(chain_id, deposit.vault.token)?;
                let amount = Float::from_hex(&deposit.amount.0)?;
                let new_balance = Float::from_hex(&deposit.new_vault_balance.0)?;
                let old_balance = Float::from_hex(&deposit.old_vault_balance.0)?;

                Ok(Self {
                    r#type: RaindexVaultBalanceChangeType::Deposit,
                    vault_id: U256::from_str(&deposit.vault.vault_id.0)?,
                    token,
                    amount,
                    formatted_amount: amount.format()?,
                    new_balance,
                    formatted_new_balance: new_balance.format()?,
                    old_balance,
                    formatted_old_balance: old_balance.format()?,
                    timestamp: U256::from_str(&deposit.timestamp.0)?,
                    transaction: RaindexTransaction::try_from(deposit.transaction)?,
                    orderbook: Address::from_str(&deposit.orderbook.id.0)?,
                })
            }
            SgVaultBalanceChangeType::Withdrawal(withdrawal) => {
                let token = RaindexVaultToken::try_from_sg_erc20(chain_id, withdrawal.vault.token)?;
                let amount = Float::from_hex(&withdrawal.amount.0)?;
                let new_balance = Float::from_hex(&withdrawal.new_vault_balance.0)?;
                let old_balance = Float::from_hex(&withdrawal.old_vault_balance.0)?;

                Ok(Self {
                    r#type: RaindexVaultBalanceChangeType::Withdrawal,
                    vault_id: U256::from_str(&withdrawal.vault.vault_id.0)?,
                    token,
                    amount,
                    formatted_amount: amount.format()?,
                    new_balance,
                    formatted_new_balance: new_balance.format()?,
                    old_balance,
                    formatted_old_balance: old_balance.format()?,
                    timestamp: U256::from_str(&withdrawal.timestamp.0)?,
                    transaction: RaindexTransaction::try_from(withdrawal.transaction)?,
                    orderbook: Address::from_str(&withdrawal.orderbook.id.0)?,
                })
            }
            SgVaultBalanceChangeType::TradeVaultBalanceChange(trade_change) => {
                Self::try_from_sg_trade_balance_change(chain_id, trade_change)
            }
            SgVaultBalanceChangeType::ClearBounty(bounty) => {
                let token = RaindexVaultToken::try_from_sg_erc20(chain_id, bounty.vault.token)?;
                let amount = Float::from_hex(&bounty.amount.0)?;
                let new_balance = Float::from_hex(&bounty.new_vault_balance.0)?;
                let old_balance = Float::from_hex(&bounty.old_vault_balance.0)?;

                Ok(Self {
                    r#type: RaindexVaultBalanceChangeType::ClearBounty,
                    vault_id: U256::from_str(&bounty.vault.vault_id.0)?,
                    token,
                    amount,
                    formatted_amount: amount.format()?,
                    new_balance,
                    formatted_new_balance: new_balance.format()?,
                    old_balance,
                    formatted_old_balance: old_balance.format()?,
                    timestamp: U256::from_str(&bounty.timestamp.0)?,
                    transaction: RaindexTransaction::try_from(bounty.transaction)?,
                    orderbook: Address::from_str(&bounty.orderbook.id.0)?,
                })
            }
            SgVaultBalanceChangeType::Unknown => Err(RaindexError::InvalidVaultBalanceChangeType(
                "Unknown".to_string(),
            )),
        }
    }
}

impl RaindexVaultBalanceChange {
    pub fn try_from_local_db(
        vault: &RaindexVault,
        change: LocalDbVaultBalanceChange,
    ) -> Result<Self, RaindexError> {
        let amount = Float::from_hex(&change.delta)?;
        let new_balance = Float::from_hex(&change.running_balance)?;
        let old_balance = (new_balance - amount)?;

        let formatted_amount = amount.format()?;
        let formatted_new_balance = new_balance.format()?;
        let formatted_old_balance = old_balance.format()?;

        let transaction = RaindexTransaction::from_local_parts(
            change.transaction_hash,
            change.owner,
            change.block_number,
            change.block_timestamp,
        )?;

        let change_type = RaindexVaultBalanceChangeType::try_from(change.change_type)?;

        Ok(Self {
            r#type: change_type,
            vault_id: vault.vault_id,
            token: vault.token.clone(),
            amount,
            formatted_amount,
            new_balance,
            formatted_new_balance,
            old_balance,
            formatted_old_balance,
            timestamp: U256::from(change.block_timestamp),
            transaction,
            orderbook: vault.orderbook,
        })
    }

    pub(crate) fn try_from_local_trade_side(
        chain_id: u32,
        orderbook: Address,
        transaction: &RaindexTransaction,
        vault_id: U256,
        token: LocalTradeTokenInfo,
        balance: LocalTradeBalanceInfo,
        block_timestamp: u64,
    ) -> Result<Self, RaindexError> {
        let amount = Float::from_hex(&balance.delta)?;
        let new_balance = match balance.running_balance.as_ref() {
            Some(balance) => Float::from_hex(balance)?,
            None => amount,
        };
        let old_balance = (new_balance - amount)?;

        let formatted_amount = amount.format()?;
        let formatted_new_balance = new_balance.format()?;
        let formatted_old_balance = old_balance.format()?;

        let LocalTradeTokenInfo {
            address,
            name,
            symbol,
            decimals,
        } = token;
        let decimals = decimals.unwrap_or(18);
        let token = RaindexVaultToken {
            chain_id,
            id: hex::encode_prefixed(address),
            address,
            name,
            symbol,
            decimals,
        };

        let change_type: RaindexVaultBalanceChangeType =
            VaultBalanceChangeKind::from_local_db_trade_kind(&balance.trade_kind).into();

        Ok(Self {
            r#type: change_type,
            vault_id,
            token,
            amount,
            formatted_amount,
            new_balance,
            formatted_new_balance,
            old_balance,
            formatted_old_balance,
            timestamp: U256::from(block_timestamp),
            transaction: transaction.clone(),
            orderbook,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVaultVolume {
    id: U256,
    token: RaindexVaultToken,
    details: RaindexVaultVolumeDetails,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVaultVolume {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Result<BigInt, RaindexError> {
        BigInt::from_str(&self.id.to_string())
            .map_err(|e| RaindexError::JsError(e.to_string().into()))
    }
    #[wasm_bindgen(getter)]
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn details(&self) -> RaindexVaultVolumeDetails {
        self.details.clone()
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexVaultVolume {
    pub fn id(&self) -> U256 {
        self.id
    }
    pub fn token(&self) -> RaindexVaultToken {
        self.token.clone()
    }
    pub fn details(&self) -> RaindexVaultVolumeDetails {
        self.details.clone()
    }
}
impl RaindexVaultVolume {
    pub fn try_from_vault_volume(
        chain_id: u32,
        vault_volume: VaultVolume,
    ) -> Result<Self, RaindexError> {
        let token = RaindexVaultToken::try_from_sg_erc20(chain_id, vault_volume.token)?;
        let details = RaindexVaultVolumeDetails::from_volume_details(vault_volume.vol_details)?;
        Ok(Self {
            id: U256::from_str(&vault_volume.id)?,
            token,
            details,
        })
    }

    pub fn try_from_local_db_vault_volume(
        chain_id: u32,
        volume: LocalDbVaultVolume,
    ) -> Result<Self, RaindexError> {
        let decimals = volume
            .token_decimals
            .ok_or(RaindexError::MissingErc20Decimals(volume.token.to_string()))?;

        let token = RaindexVaultToken {
            chain_id,
            id: volume.token.to_string(),
            address: volume.token,
            name: volume.token_name,
            symbol: volume.token_symbol,
            decimals,
        };

        let total_in = Float::from_hex(&volume.total_in)?;
        let total_out = Float::from_hex(&volume.total_out)?;
        let total_vol = (total_in + total_out)?;
        let net_vol = (total_in - total_out)?;

        let details = RaindexVaultVolumeDetails {
            total_in,
            formatted_total_in: total_in.format()?,
            total_out,
            formatted_total_out: total_out.format()?,
            total_vol,
            formatted_total_vol: total_vol.format()?,
            net_vol,
            formatted_net_vol: net_vol.format()?,
        };

        Ok(Self {
            id: volume.vault_id,
            token,
            details,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct RaindexVaultVolumeDetails {
    total_in: Float,
    formatted_total_in: String,
    total_out: Float,
    formatted_total_out: String,
    total_vol: Float,
    formatted_total_vol: String,
    net_vol: Float,
    formatted_net_vol: String,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexVaultVolumeDetails {
    #[wasm_bindgen(getter = totalIn)]
    pub fn total_in(&self) -> Float {
        self.total_in
    }
    #[wasm_bindgen(getter = formattedTotalIn)]
    pub fn formatted_total_in(&self) -> String {
        self.formatted_total_in.clone()
    }
    #[wasm_bindgen(getter = totalOut)]
    pub fn total_out(&self) -> Float {
        self.total_out
    }
    #[wasm_bindgen(getter = formattedTotalOut)]
    pub fn formatted_total_out(&self) -> String {
        self.formatted_total_out.clone()
    }
    #[wasm_bindgen(getter = totalVol)]
    pub fn total_vol(&self) -> Float {
        self.total_vol
    }
    #[wasm_bindgen(getter = formattedTotalVol)]
    pub fn formatted_total_vol(&self) -> String {
        self.formatted_total_vol.clone()
    }
    #[wasm_bindgen(getter = netVol)]
    pub fn net_vol(&self) -> Float {
        self.net_vol
    }
    #[wasm_bindgen(getter = formattedNetVol)]
    pub fn formatted_net_vol(&self) -> String {
        self.formatted_net_vol.clone()
    }
}
#[cfg(not(target_family = "wasm"))]
impl RaindexVaultVolumeDetails {
    pub fn total_in(&self) -> Float {
        self.total_in
    }
    pub fn formatted_total_in(&self) -> String {
        self.formatted_total_in.clone()
    }
    pub fn total_out(&self) -> Float {
        self.total_out
    }
    pub fn formatted_total_out(&self) -> String {
        self.formatted_total_out.clone()
    }
    pub fn total_vol(&self) -> Float {
        self.total_vol
    }
    pub fn formatted_total_vol(&self) -> String {
        self.formatted_total_vol.clone()
    }
    pub fn net_vol(&self) -> Float {
        self.net_vol
    }
    pub fn formatted_net_vol(&self) -> String {
        self.formatted_net_vol.clone()
    }
}
impl RaindexVaultVolumeDetails {
    pub fn from_volume_details(volume_details: VolumeDetails) -> Result<Self, RaindexError> {
        Ok(Self {
            total_in: volume_details.total_in,
            formatted_total_in: volume_details.total_in.format()?,
            total_out: volume_details.total_out,
            formatted_total_out: volume_details.total_out.format()?,
            total_vol: volume_details.total_vol,
            formatted_total_vol: volume_details.total_vol.format()?,
            net_vol: volume_details.net_vol,
            formatted_net_vol: volume_details.net_vol.format()?,
        })
    }
}

#[wasm_export]
impl RaindexClient {
    /// Fetches vault data from multiple subgraphs across different networks
    ///
    /// Queries multiple subgraphs simultaneously to retrieve vault information
    /// across different networks.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getVaults(
    ///   {
    ///     owners: ["0x1234567890abcdef1234567890abcdef12345678"],
    ///     hide_zero_balance: true
    ///   },
    /// );
    /// if (result.error) {
    ///   console.error("Error fetching vaults:", result.error.readableMsg);
    ///   return;
    /// }
    /// const vaults = result.value;
    /// // Do something with the vaults
    /// ```
    #[wasm_export(
        js_name = "getVaults",
        return_description = "Array of raindex vault instances",
        preserve_js_class
    )]
    pub async fn get_vaults(
        &self,
        #[wasm_export(
            js_name = "chainIds",
            param_description = "Specific networks to query (optional)"
        )]
        chain_ids: Option<ChainIds>,
        #[wasm_export(
            param_description = "Optional filtering options including owners and hide_zero_balance"
        )]
        filters: Option<GetVaultsFilters>,
        #[wasm_export(param_description = "Optional page number (defaults to 1)")] page: Option<
            u16,
        >,
    ) -> Result<RaindexVaultsList, RaindexError> {
        let filters = filters.unwrap_or_default();
        let page_number = page.unwrap_or(1);
        let ids = chain_ids.map(|ChainIds(ids)| ids);

        let (local_db, local_ids, sg_ids) = self.classify_chains(ids)?;

        let mut all_vaults = Vec::new();

        if let Some(db) = local_db {
            let local_source = LocalDbVaults::new(&db, ClientRef::new(self.clone()));
            let vaults = local_source
                .list(Some(local_ids), &filters, Some(page_number))
                .await?;
            all_vaults.extend(vaults);
        }

        if !sg_ids.is_empty() {
            let subgraph_source = SubgraphVaults::new(self);
            let vaults = subgraph_source
                .list(Some(sg_ids), &filters, Some(page_number))
                .await?;
            all_vaults.extend(vaults);
        }

        Ok(RaindexVaultsList::new(all_vaults))
    }

    /// Fetches detailed information for a specific vault
    ///
    /// Retrieves complete vault information including token details, balance, etc.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getVault(
    ///   137, // Polygon network
    ///   "0x1234567890abcdef1234567890abcdef12345678"
    /// );
    /// if (result.error) {
    ///   console.error("Vault not found:", result.error.readableMsg);
    ///   return;
    /// }
    /// const vault = result.value;
    /// // Do something with the vault
    /// ```
    #[wasm_export(
        js_name = "getVault",
        return_description = "Complete vault information",
        unchecked_return_type = "RaindexVault",
        preserve_js_class
    )]
    pub async fn get_vault_wasm_binding(
        &self,
        #[wasm_export(
            js_name = "chainId",
            param_description = "Chain ID of the network the vault is on"
        )]
        chain_id: u32,
        #[wasm_export(
            js_name = "orderbookAddress",
            param_description = "Orderbook contract address",
            unchecked_param_type = "Address"
        )]
        orderbook_address: String,
        #[wasm_export(
            js_name = "vaultId",
            param_description = "Unique vault identifier",
            unchecked_param_type = "Hex"
        )]
        vault_id: String,
    ) -> Result<RaindexVault, RaindexError> {
        let orderbook_address = Address::from_str(&orderbook_address)?;
        let vault_id = Bytes::from_str(&vault_id)?;
        self.get_vault(
            &OrderbookIdentifier::new(chain_id, orderbook_address),
            vault_id,
        )
        .await
    }

    /// Fetches all unique tokens that exist in vaults.
    ///
    /// Retrieves all unique ERC20 tokens that have associated vaults by querying
    /// all vaults and extracting their token information, removing duplicates.
    ///
    /// ## Examples
    ///
    /// ```javascript
    /// const result = await client.getAllVaultTokens();
    /// if (result.error) {
    ///   console.error("Error fetching tokens:", result.error.readableMsg);
    ///   return;
    /// }
    /// const tokens = result.value;
    /// console.log(`Found ${tokens.length} unique tokens`);
    /// console.log(`Token ${tokens[0].name} in ${tokens[0].chainId}`);
    /// ```
    #[wasm_export(
        js_name = "getAllVaultTokens",
        return_description = "Array of raindex vault token instances",
        unchecked_return_type = "RaindexVaultToken[]"
    )]
    pub async fn get_all_vault_tokens(
        &self,
        #[wasm_export(
            js_name = "chainIds",
            param_description = "Specific networks to query (optional)"
        )]
        chain_ids: Option<ChainIds>,
    ) -> Result<Vec<RaindexVaultToken>, RaindexError> {
        let ids = chain_ids.map(|ChainIds(ids)| ids);

        let (local_db, local_ids, sg_ids) = self.classify_chains(ids)?;

        let mut tokens: Vec<RaindexVaultToken> = Vec::new();

        if let Some(db) = local_db {
            let local_source = LocalDbVaults::new(&db, ClientRef::new(self.clone()));
            let local_tokens = local_source.tokens_list(Some(local_ids)).await?;
            tokens.extend(local_tokens);
        }

        if !sg_ids.is_empty() {
            let subgraph_source = SubgraphVaults::new(self);
            let sg_tokens = subgraph_source.tokens_list(Some(sg_ids)).await?;
            tokens.extend(sg_tokens);
        }

        Ok(tokens)
    }
}
impl RaindexClient {
    pub async fn get_vault(
        &self,
        ob_id: &OrderbookIdentifier,
        vault_id: Bytes,
    ) -> Result<RaindexVault, RaindexError> {
        let orderbook_cfg = self.get_orderbook_by_address(ob_id.orderbook_address)?;
        if orderbook_cfg.network.chain_id != ob_id.chain_id {
            return Err(RaindexError::OrderbookNotFound(
                ob_id.orderbook_address.to_string(),
                ob_id.chain_id,
            ));
        }

        match self.query_source(ob_id.chain_id) {
            QuerySource::LocalDb(local_db) => {
                let local_source = LocalDbVaults::new(&local_db, ClientRef::new(self.clone()));
                local_source
                    .get_by_id(ob_id, &vault_id)
                    .await?
                    .ok_or_else(|| {
                        RaindexError::VaultNotFound(
                            ob_id.orderbook_address.to_string(),
                            ob_id.chain_id,
                            vault_id.to_string(),
                        )
                    })
            }
            QuerySource::Subgraph => SubgraphVaults::new(self)
                .get_by_id(ob_id, &vault_id)
                .await?
                .ok_or_else(|| {
                    RaindexError::VaultNotFound(
                        ob_id.orderbook_address.to_string(),
                        ob_id.chain_id,
                        vault_id.to_string(),
                    )
                }),
        }
    }
}

#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
impl VaultsDataSource for SubgraphVaults<'_> {
    async fn list(
        &self,
        chain_ids: Option<Vec<u32>>,
        filters: &GetVaultsFilters,
        page: Option<u16>,
    ) -> Result<Vec<RaindexVault>, RaindexError> {
        let raindex_client = ClientRef::new(self.client.clone());
        let multi_subgraph_args = self.client.get_multi_subgraph_args(chain_ids)?;
        let client = MultiOrderbookSubgraphClient::new(
            multi_subgraph_args.values().flatten().cloned().collect(),
        );

        let vaults = client
            .vaults_list(
                filters.clone().try_into()?,
                SgPaginationArgs {
                    page: page.unwrap_or(1),
                    page_size: DEFAULT_PAGE_SIZE,
                },
            )
            .await?;

        let vaults = vaults
            .iter()
            .map(|vault| {
                let chain_id = multi_subgraph_args
                    .iter()
                    .find(|(_, args)| args.iter().any(|arg| arg.name == vault.subgraph_name))
                    .map(|(chain_id, _)| *chain_id)
                    .ok_or_else(|| {
                        RaindexError::SubgraphNotFound(
                            vault.subgraph_name.clone(),
                            vault.vault.vault_id.0.clone(),
                        )
                    })?;
                let vault = RaindexVault::try_from_sg_vault(
                    raindex_client.clone(),
                    chain_id,
                    vault.vault.clone(),
                    None,
                )?;
                Ok(vault)
            })
            .collect::<Result<Vec<RaindexVault>, RaindexError>>()?;

        Ok(vaults)
    }

    async fn get_by_id(
        &self,
        ob_id: &OrderbookIdentifier,
        vault_id: &Bytes,
    ) -> Result<Option<RaindexVault>, RaindexError> {
        let raindex_client = ClientRef::new(self.client.clone());
        let client = self.client.get_orderbook_client(ob_id.orderbook_address)?;
        let vault = match client.vault_detail(Id::new(vault_id.to_string())).await {
            Ok(vault) => vault,
            Err(OrderbookSubgraphClientError::Empty) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let vault = RaindexVault::try_from_sg_vault(raindex_client, ob_id.chain_id, vault, None)?;
        Ok(Some(vault))
    }

    async fn balance_changes_list(
        &self,
        vault: &RaindexVault,
        page: Option<u16>,
        filter_types: Option<&[VaultBalanceChangeFilter]>,
    ) -> Result<Vec<RaindexVaultBalanceChange>, RaindexError> {
        let client = self.client.get_orderbook_client(vault.orderbook)?;

        let filter_typenames: Option<Vec<&str>> = filter_types.map(|filters| {
            filters
                .iter()
                .flat_map(|f| f.to_kind().to_subgraph_typenames())
                .copied()
                .collect()
        });

        let balance_changes = client
            .vault_balance_changes_list(
                Id::new(vault.id.to_string()),
                SgPaginationArgs {
                    page: page.unwrap_or(1),
                    page_size: 1000,
                },
                filter_typenames.as_deref(),
            )
            .await?;

        balance_changes
            .into_iter()
            .map(|balance_change| {
                RaindexVaultBalanceChange::try_from_sg_balance_change_type(
                    vault.chain_id,
                    balance_change,
                )
            })
            .collect()
    }

    async fn tokens_list(
        &self,
        chain_ids: Option<Vec<u32>>,
    ) -> Result<Vec<RaindexVaultToken>, RaindexError> {
        let multi_subgraph_args = self.client.get_multi_subgraph_args(chain_ids)?;
        let client = MultiOrderbookSubgraphClient::new(
            multi_subgraph_args.values().flatten().cloned().collect(),
        );

        let token_list = client.tokens_list().await?;
        token_list
            .iter()
            .map(|v| {
                let chain_id = multi_subgraph_args
                    .iter()
                    .find(|(_, args)| args.iter().any(|arg| arg.name == v.subgraph_name))
                    .map(|(chain_id, _)| *chain_id)
                    .ok_or(RaindexError::SubgraphNotFound(
                        v.subgraph_name.clone(),
                        v.token.address.0.clone(),
                    ))?;
                RaindexVaultToken::try_from_sg_erc20(chain_id, v.token.clone())
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Tsify, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetVaultsFilters {
    #[tsify(type = "Address[]")]
    pub owners: Vec<Address>,
    pub hide_zero_balance: bool,
    #[tsify(optional, type = "Address[]")]
    pub tokens: Option<Vec<Address>>,
    #[tsify(optional, type = "Address[]")]
    pub orderbook_addresses: Option<Vec<Address>>,
    #[serde(default)]
    pub only_active_orders: bool,
}
impl_wasm_traits!(GetVaultsFilters);

impl TryFrom<GetVaultsFilters> for SgVaultsListFilterArgs {
    type Error = RaindexError;
    fn try_from(filters: GetVaultsFilters) -> Result<Self, Self::Error> {
        Ok(Self {
            owners: filters
                .owners
                .into_iter()
                .map(|owner| SgBytes(owner.to_string()))
                .collect(),
            hide_zero_balance: filters.hide_zero_balance,
            tokens: filters
                .tokens
                .map(|tokens| {
                    tokens
                        .into_iter()
                        .map(|token| token.to_string().to_lowercase())
                        .collect()
                })
                .unwrap_or_default(),
            orderbooks: filters
                .orderbook_addresses
                .map(|addrs| {
                    addrs
                        .into_iter()
                        .map(|addr| addr.to_string().to_lowercase())
                        .collect()
                })
                .unwrap_or_default(),
            only_active_orders: filters.only_active_orders,
        })
    }
}

impl RaindexVault {
    pub fn try_from_sg_vault(
        raindex_client: ClientRef,
        chain_id: u32,
        vault: SgVault,
        vault_type: Option<RaindexVaultType>,
    ) -> Result<Self, RaindexError> {
        let token = RaindexVaultToken::try_from_sg_erc20(chain_id, vault.token)?;

        let balance = Float::from_hex(&vault.balance.0)?;
        let formatted_balance = balance.format()?;

        Ok(Self {
            raindex_client,
            chain_id,
            vault_type,
            id: Bytes::from_str(&vault.id.0)?,
            owner: Address::from_str(&vault.owner.0)?,
            vault_id: U256::from_str(&vault.vault_id.0)?,
            balance,
            formatted_balance,
            token,
            orderbook: Address::from_str(&vault.orderbook.id.0)?,
            orders_as_inputs: vault
                .orders_as_input
                .iter()
                .map(|order| RaindexOrderAsIO::try_from(order.clone()))
                .collect::<Result<Vec<RaindexOrderAsIO>, RaindexError>>()?,
            orders_as_outputs: vault
                .orders_as_output
                .iter()
                .map(|order| RaindexOrderAsIO::try_from(order.clone()))
                .collect::<Result<Vec<RaindexOrderAsIO>, RaindexError>>()?,
        })
    }

    pub fn with_vault_type(&self, vault_type: RaindexVaultType) -> Self {
        Self {
            raindex_client: ClientRef::clone(&self.raindex_client),
            chain_id: self.chain_id,
            vault_type: Some(vault_type),
            id: self.id.clone(),
            owner: self.owner,
            vault_id: self.vault_id,
            balance: self.balance,
            formatted_balance: self.formatted_balance.clone(),
            token: self.token.clone(),
            orderbook: self.orderbook,
            orders_as_inputs: self.orders_as_inputs.clone(),
            orders_as_outputs: self.orders_as_outputs.clone(),
        }
    }

    pub fn into_sg_vault(self) -> Result<SgVault, RaindexError> {
        Ok(SgVault {
            id: SgBytes(self.id.to_string()),
            vault_id: SgBytes(self.vault_id.to_string()),
            balance: SgBytes(self.balance.as_hex()),
            owner: SgBytes(self.owner.to_string()),
            token: self.token.try_into()?,
            orderbook: SgOrderbook {
                id: SgBytes(self.orderbook.to_string()),
            },
            orders_as_input: self
                .orders_as_inputs
                .into_iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<SgOrderAsIO>, RaindexError>>()?,
            orders_as_output: self
                .orders_as_outputs
                .into_iter()
                .map(|v| v.try_into())
                .collect::<Result<Vec<SgOrderAsIO>, RaindexError>>()?,
            balance_changes: vec![],
        })
    }

    pub fn try_from_local_db(
        raindex_client: ClientRef,
        vault: LocalDbVault,
        vault_type: Option<RaindexVaultType>,
    ) -> Result<Self, RaindexError> {
        let balance = Float::from_hex(&vault.balance)?;
        let formatted_balance = balance.format()?;

        let mut id = Vec::from(vault.orderbook_address.as_slice());
        id.extend_from_slice(vault.owner.as_slice());
        id.extend_from_slice(vault.token.as_slice());
        id.extend_from_slice(&vault.vault_id.to_le_bytes::<32>());

        Ok(Self {
            raindex_client,
            chain_id: vault.chain_id,
            vault_type,
            id: Bytes::from(id),
            owner: vault.owner,
            vault_id: vault.vault_id,
            balance,
            formatted_balance,
            token: RaindexVaultToken {
                chain_id: vault.chain_id,
                id: vault.token.to_string(),
                address: vault.token,
                name: Some(vault.token_name),
                symbol: Some(vault.token_symbol),
                decimals: vault.token_decimals,
            },
            orderbook: vault.orderbook_address,
            orders_as_inputs: RaindexOrderAsIO::try_from_local_db_orders_csv(
                "inputOrders",
                &vault.input_orders,
            )?,
            orders_as_outputs: RaindexOrderAsIO::try_from_local_db_orders_csv(
                "outputOrders",
                &vault.output_orders,
            )?,
        })
    }
}

impl RaindexVaultToken {
    fn try_from_sg_erc20(chain_id: u32, erc20: SgErc20) -> Result<Self, RaindexError> {
        let address = Address::from_str(&erc20.address.0)?;
        let decimals = erc20
            .decimals
            .ok_or(RaindexError::MissingErc20Decimals(address.to_string()))?
            .0
            .parse::<u8>()?;
        Ok(Self {
            chain_id,
            id: erc20.id.0,
            address,
            name: erc20.name,
            symbol: erc20.symbol,
            decimals,
        })
    }

    pub(crate) fn from_local_db_token(
        token: crate::local_db::query::fetch_all_tokens::LocalDbToken,
    ) -> Self {
        Self {
            chain_id: token.chain_id,
            id: token.token_address.to_string(),
            address: token.token_address,
            name: Some(token.name),
            symbol: Some(token.symbol),
            decimals: token.decimals,
        }
    }
}
impl TryFrom<RaindexVaultToken> for SgErc20 {
    type Error = RaindexError;
    fn try_from(token: RaindexVaultToken) -> Result<Self, Self::Error> {
        Ok(Self {
            id: SgBytes(token.id),
            address: SgBytes(token.address.to_string()),
            name: token.name,
            symbol: token.symbol,
            decimals: Some(SgBigInt(token.decimals.to_string())),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::local_db::query::fetch_vault_balance_changes::LocalDbVaultBalanceChange;
        use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
        use crate::raindex_client::tests::{
            get_local_db_test_yaml, new_test_client_with_db_callback,
        };
        use alloy::primitives::{address, b256, Address, Bytes};
        use rain_math_float::Float;
        use serde_json;
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::str::FromStr;
        use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::WasmEncodedResult;
        use LocalDbVault;

        fn make_local_db_vaults_callback(vaults: Vec<LocalDbVault>) -> js_sys::Function {
            let json = serde_json::to_string(&vaults).unwrap();
            let result = WasmEncodedResult::Success::<String> {
                value: json,
                error: None,
            };
            let payload = js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&result).unwrap())
                .unwrap()
                .as_string()
                .unwrap();

            let callback = Closure::wrap(Box::new(move |_sql: String| -> JsValue {
                js_sys::JSON::parse(&payload).unwrap()
            }) as Box<dyn Fn(String) -> JsValue>);

            callback.into_js_value().dyn_into().unwrap()
        }

        fn make_local_db_vaults_with_balance_changes_callback(
            vaults: Vec<LocalDbVault>,
            balance_changes: Vec<LocalDbVaultBalanceChange>,
        ) -> js_sys::Function {
            let vaults_json = serde_json::to_string(&vaults).unwrap();
            let vaults_result = WasmEncodedResult::Success::<String> {
                value: vaults_json,
                error: None,
            };
            let vaults_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&vaults_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let balance_json = serde_json::to_string(&balance_changes).unwrap();
            let balance_result = WasmEncodedResult::Success::<String> {
                value: balance_json,
                error: None,
            };
            let balance_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&balance_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let callback = Closure::wrap(Box::new(move |sql: String| -> JsValue {
                if sql.contains("runningBalance") {
                    js_sys::JSON::parse(&balance_payload).unwrap()
                } else {
                    js_sys::JSON::parse(&vaults_payload).unwrap()
                }
            }) as Box<dyn Fn(String) -> JsValue>);

            callback.into_js_value().dyn_into().unwrap()
        }

        fn make_local_vault(
            vault_id: &str,
            token: &str,
            owner: &str,
            balance: Float,
        ) -> LocalDbVault {
            LocalDbVault {
                chain_id: 42161,
                vault_id: U256::from_str(vault_id).unwrap(),
                token: Address::from_str(token).unwrap(),
                owner: Address::from_str(owner).unwrap(),
                orderbook_address: address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB"),
                token_name: "Token".to_string(),
                token_symbol: "TKN".to_string(),
                token_decimals: 18,
                balance: balance.as_hex(),
                input_orders: None,
                output_orders: None,
            }
        }

        #[wasm_bindgen_test]
        async fn test_get_vaults_local_db_path() {
            let owner = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let token = "0x00000000000000000000000000000000000000aa";
            let vault =
                make_local_vault("0x01", token, owner, Float::parse("1".to_string()).unwrap());

            let callback = make_local_db_vaults_callback(vec![vault]);

            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let vaults = client
                .get_vaults(Some(ChainIds(vec![42161])), None, None)
                .await
                .expect("local db vaults should load");

            let items = vaults.items();
            assert_eq!(items.len(), 1);
            let result_vault = &items[0];
            assert_eq!(result_vault.chain_id(), 42161);
            assert_eq!(result_vault.owner().to_lowercase(), owner.to_string());
            assert_eq!(
                result_vault.orderbook().to_lowercase(),
                "0x2f209e5b67a33b8fe96e28f24628df6da301c8eb".to_string()
            );
            assert_eq!(result_vault.formatted_balance(), "1".to_string());
            let token_meta = result_vault.token();
            assert_eq!(token_meta.address().to_lowercase(), token.to_string());
        }

        #[wasm_bindgen_test]
        async fn test_get_vault_local_db_path() {
            let owner = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let token = "0x00000000000000000000000000000000000000aa";
            let local_vault =
                make_local_vault("0x02", token, owner, Float::parse("5".to_string()).unwrap());

            let callback = make_local_db_vaults_callback(vec![local_vault.clone()]);

            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let rc_client = Rc::new(client.clone());
            let derived_vault =
                RaindexVault::try_from_local_db(Rc::clone(&rc_client), local_vault, None)
                    .expect("local vault should convert");

            let vault_id_hex = derived_vault.id();
            let vault_id_bytes = Bytes::from_str(&vault_id_hex).expect("valid vault id");

            let orderbook =
                Address::from_str("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB").unwrap();
            let retrieved = client
                .get_vault(&OrderbookIdentifier::new(42161, orderbook), vault_id_bytes)
                .await
                .expect("local vault retrieval should succeed");

            assert_eq!(retrieved.chain_id(), 42161);
            assert_eq!(retrieved.owner().to_lowercase(), owner.to_string());
            assert_eq!(retrieved.formatted_balance(), "5".to_string());
            assert_eq!(
                retrieved.token().address().to_lowercase(),
                token.to_string()
            );
            assert_eq!(retrieved.id(), vault_id_hex);
        }

        #[wasm_bindgen_test]
        async fn test_get_balance_changes_local_db_path() {
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let token = address!("0x00000000000000000000000000000000000000aa");
            let owner_str = owner.to_string();
            let token_str = token.to_string();
            let local_vault = make_local_vault(
                "0x02",
                &token_str,
                &owner_str,
                Float::parse("5".to_string()).unwrap(),
            );

            let amount = Float::parse("1".to_string()).unwrap();
            let running_balance = Float::parse("5".to_string()).unwrap();

            let balance_change = LocalDbVaultBalanceChange {
                transaction_hash: b256!(
                    "0x00000000000000000000000000000000000000000000000000000000deadbeef"
                ),
                log_index: 1,
                block_number: 1234,
                block_timestamp: 5678,
                owner,
                change_type: "DEPOSIT".to_string(),
                token,
                vault_id: local_vault.vault_id.clone(),
                delta: amount.as_hex(),
                running_balance: running_balance.as_hex(),
            };

            let callback = make_local_db_vaults_with_balance_changes_callback(
                vec![local_vault.clone()],
                vec![balance_change],
            );

            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let rc_client = Rc::new(client.clone());
            let derived_vault =
                RaindexVault::try_from_local_db(Rc::clone(&rc_client), local_vault, None)
                    .expect("local vault should convert");

            let vault_id_bytes = Bytes::from_str(&derived_vault.id()).expect("valid vault id");

            let orderbook =
                Address::from_str("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB").unwrap();
            let vault = client
                .get_vault(&OrderbookIdentifier::new(42161, orderbook), vault_id_bytes)
                .await
                .expect("local vault retrieval should succeed");

            let changes = vault
                .get_balance_changes(None, None)
                .await
                .expect("balance changes should load from local db");

            assert_eq!(changes.len(), 1);
            let change = &changes[0];
            assert_eq!(change.type_getter(), RaindexVaultBalanceChangeType::Deposit);
            assert_eq!(change.formatted_amount(), "1");
            assert_eq!(change.formatted_new_balance(), "5");
            assert_eq!(change.formatted_old_balance(), "4");
            assert_eq!(
                change.transaction().id(),
                "0x00000000000000000000000000000000000000000000000000000000deadbeef"
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_vaults_local_db_filters() {
            use wasm_bindgen::JsCast;
            use wasm_bindgen_utils::prelude::JsValue;
            let owner_kept = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let token_kept = "0x00000000000000000000000000000000000000aa";

            let keep_vault = make_local_vault(
                "0x01",
                token_kept,
                owner_kept,
                Float::parse("2".to_string()).unwrap(),
            );
            let captured_sql = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let json = serde_json::to_string(&vec![keep_vault]).unwrap();
            let callback = create_sql_capturing_callback(&json, captured_sql.clone());

            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let filters = GetVaultsFilters {
                owners: vec![Address::from_str(owner_kept).unwrap()],
                hide_zero_balance: true,
                tokens: Some(vec![Address::from_str(token_kept).unwrap()]),
                orderbook_addresses: None,
                only_active_orders: false,
            };

            let vaults = client
                .get_vaults(Some(ChainIds(vec![42161])), Some(filters), None)
                .await
                .expect("filtered vaults should load");

            let items = vaults.items();
            assert_eq!(items.len(), 1);
            let vault = &items[0];
            assert_eq!(vault.owner().to_lowercase(), owner_kept.to_string());
            let token_meta = vault.token();
            assert_eq!(token_meta.address().to_lowercase(), token_kept.to_string());
            assert_eq!(vault.formatted_balance(), "2".to_string());

            let sql = captured_sql.borrow();
            // SQL should contain parameterized IN-clauses and hide-zero filter body
            assert!(sql.0.contains("o.owner IN ("));
            assert!(sql.0.contains("o.token IN ("));
            assert!(sql.0.contains("AND NOT FLOAT_IS_ZERO("));

            // Params should include chain id, owner and token values bound in order
            let params_js = sql.1.clone();
            assert!(
                js_sys::Array::is_array(&params_js),
                "expected array params from callback"
            );
            let params_array = js_sys::Array::from(&params_js);
            assert!(
                params_array.length() >= 3,
                "expected at least three params (chain id, owner, token)"
            );

            // Expect first param to be chain id (U64 encoded as BigInt)
            let chain_id = params_array.get(0);
            let chain_id_bigint = chain_id
                .dyn_into::<js_sys::BigInt>()
                .expect("chain id should be BigInt");
            let chain_id_str = chain_id_bigint.to_string(10).unwrap().as_string().unwrap();
            assert_eq!(chain_id_str, "42161");

            // Expect owner and token to be present among text params
            let mut has_owner = false;
            let mut has_token = false;
            for value in params_array.iter() {
                if let Some(text) = value.as_string() {
                    if text == owner_kept {
                        has_owner = true;
                    }
                    if text == token_kept {
                        has_token = true;
                    }
                }
            }
            assert!(has_owner, "owner missing in params");
            assert!(has_token, "token missing in params");
        }
    }

    #[cfg(not(target_family = "wasm"))]
    mod non_wasm {
        use super::*;
        use crate::raindex_client::tests::get_test_yaml;
        use crate::raindex_client::tests::CHAIN_ID_1_ORDERBOOK_ADDRESS;
        use alloy::hex::encode_prefixed;
        use alloy::primitives::{address, b256};
        use alloy::sol_types::SolCall;
        use httpmock::MockServer;
        use rain_orderbook_bindings::IERC20Metadata::decimalsCall;
        use rain_orderbook_bindings::{
            IRaindexV6::{deposit4Call, withdraw4Call},
            IERC20::approveCall,
        };
        use rain_orderbook_subgraph_client::utils::float::*;
        use serde_json::{json, Value};
        use std::sync::Arc;
        use LocalDbVault;

        #[test]
        fn test_try_from_local_trade_side_with_running_balance() {
            let chain_id = 42161;
            let orderbook = address!("0x0000000000000000000000000000000000000001");
            let transaction = RaindexTransaction::from_local_parts(
                b256!("0x00000000000000000000000000000000000000000000000000000000deadbeef"),
                address!("0x0000000000000000000000000000000000000002"),
                123,
                456,
            )
            .unwrap();

            let amount = Float::parse("1".to_string()).unwrap();
            let amount_hex = amount.as_hex();
            let new_balance = Float::parse("5".to_string()).unwrap();
            let new_balance_hex = new_balance.as_hex();
            let expected_old_balance = Float::parse("4".to_string()).unwrap();

            let change = RaindexVaultBalanceChange::try_from_local_trade_side(
                chain_id,
                orderbook,
                &transaction,
                U256::from(16),
                LocalTradeTokenInfo {
                    address: address!("0x0000000000000000000000000000000000000003"),
                    name: Some("Token In".to_string()),
                    symbol: Some("TIN".to_string()),
                    decimals: Some(6),
                },
                LocalTradeBalanceInfo {
                    delta: amount_hex.clone(),
                    running_balance: Some(new_balance_hex.clone()),
                    trade_kind: "take".to_string(),
                },
                789,
            )
            .unwrap();

            assert_eq!(change.r#type(), RaindexVaultBalanceChangeType::TakeOrder);
            assert_eq!(change.vault_id(), U256::from_str("0x10").unwrap());
            assert!(change.amount().eq(amount).unwrap());
            assert!(change.new_balance().eq(new_balance).unwrap());
            assert_eq!(change.formatted_amount(), amount.format().unwrap());
            assert_eq!(
                change.formatted_new_balance(),
                new_balance.format().unwrap()
            );
            assert_eq!(
                change.formatted_old_balance(),
                expected_old_balance.format().unwrap()
            );
            assert_eq!(change.timestamp(), U256::from(789));
            assert_eq!(change.orderbook(), orderbook);
            assert_eq!(change.transaction().id(), transaction.id());

            let token = change.token();
            assert_eq!(token.chain_id(), chain_id);
            assert_eq!(
                token.address(),
                Address::from_str("0x0000000000000000000000000000000000000003").unwrap()
            );
            assert_eq!(token.decimals(), 6);
            assert_eq!(token.name(), Some("Token In".to_string()));
            assert_eq!(token.symbol(), Some("TIN".to_string()));
            assert_eq!(
                token.id(),
                "0x0000000000000000000000000000000000000003".to_string()
            );
        }

        #[test]
        fn test_try_from_local_trade_side_defaults() {
            let chain_id = 1;
            let orderbook = address!("0x0000000000000000000000000000000000000004");
            let transaction = RaindexTransaction::from_local_parts(
                b256!("0x00000000000000000000000000000000000000000000000000000000feedface"),
                address!("0x0000000000000000000000000000000000000005"),
                111,
                222,
            )
            .unwrap();

            let amount = Float::parse("2".to_string()).unwrap();
            let amount_hex = amount.as_hex();
            let zero = Float::parse("0".to_string()).unwrap();

            let change = RaindexVaultBalanceChange::try_from_local_trade_side(
                chain_id,
                orderbook,
                &transaction,
                U256::from(2),
                LocalTradeTokenInfo {
                    address: address!("0x0000000000000000000000000000000000000006"),
                    name: None,
                    symbol: None,
                    decimals: None,
                },
                LocalTradeBalanceInfo {
                    delta: amount_hex.clone(),
                    running_balance: None,
                    trade_kind: "clear".to_string(),
                },
                333,
            )
            .unwrap();

            assert_eq!(change.r#type(), RaindexVaultBalanceChangeType::Clear);

            assert!(change.amount().eq(amount).unwrap());
            assert!(change.new_balance().eq(amount).unwrap());
            assert!(change.old_balance().eq(zero).unwrap());
            assert_eq!(change.formatted_amount(), amount.format().unwrap());
            assert_eq!(change.formatted_new_balance(), amount.format().unwrap());
            assert_eq!(change.formatted_old_balance(), zero.format().unwrap());

            let token = change.token();
            assert_eq!(token.decimals(), 18);
            assert!(token.name().is_none());
            assert!(token.symbol().is_none());
            assert_eq!(
                token.address(),
                Address::from_str("0x0000000000000000000000000000000000000006").unwrap()
            );
            assert_eq!(
                token.id(),
                "0x0000000000000000000000000000000000000006".to_string()
            );
        }

        #[tokio::test]
        async fn test_try_from_local_db_maps_token_metadata() {
            // Build a minimal client; it won't be used in mapping
            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    "http://sg1",
                    "http://sg2",
                    "http://rpc1",
                    "http://rpc2",
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let local_vault = LocalDbVault {
                chain_id: 1,
                vault_id: U256::from(1),
                token: address!("0x0000000000000000000000000000000000000000"),
                owner: address!("0x0000000000000000000000000000000000000000"),
                orderbook_address: Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                token_name: "Test Token".to_string(),
                token_symbol: "TST".to_string(),
                token_decimals: 6,
                balance: Float::parse("0".to_string()).unwrap().as_hex(),
                input_orders: None,
                output_orders: None,
            };

            let rv = RaindexVault::try_from_local_db(
                Arc::new(raindex_client),
                local_vault,
                Some(RaindexVaultType::Input),
            )
            .unwrap();

            assert_eq!(rv.token.name(), Some("Test Token".to_string()));
            assert_eq!(rv.token.symbol(), Some("TST".to_string()));
            assert_eq!(rv.token.decimals(), 6);
        }

        fn get_vault1_json() -> Value {
            json!({
              "id": "0x0123",
              "owner": "0x0000000000000000000000000000000000000000",
              "vaultId": "0x0123",
              "balance": F1,
              "token": {
                "id": "token1",
                "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                "name": "Token 1",
                "symbol": "TKN1",
                "decimals": "18"
              },
              "orderbook": {
                "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
              },
              "ordersAsOutput": [],
              "ordersAsInput": [],
              "balanceChanges": []
            })
        }

        fn get_vault2_json() -> Value {
            json!({
                "id": "0x0234",
                "owner": "0x0000000000000000000000000000000000000000",
                "vaultId": "0x0234",
                "balance": F2,
                "token": {
                    "id": "token2",
                    "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                    "name": "Token 2",
                    "symbol": "TKN2",
                    "decimals": "18"
                },
                "orderbook": {
                    "id": "0x0000000000000000000000000000000000000000"
                },
                "ordersAsOutput": [],
                "ordersAsInput": [],
                "balanceChanges": []
            })
        }

        #[tokio::test]
        async fn test_get_vaults() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault1_json()]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault2_json()]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let result = raindex_client
                .get_vaults(None, None, None)
                .await
                .unwrap()
                .items();
            assert_eq!(result.len(), 2);

            let vault1 = result[0].clone();
            assert_eq!(vault1.chain_id, 1);
            assert_eq!(vault1.id, Bytes::from_str("0x0123").unwrap());
            assert_eq!(
                vault1.owner,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(vault1.vault_id, U256::from_str("0x0123").unwrap());
            assert!(vault1.balance.eq(F1).unwrap());
            assert_eq!(vault1.formatted_balance, "1");
            assert_eq!(vault1.token.id, "token1");
            assert_eq!(
                vault1.orderbook,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap()
            );

            let vault2 = result[1].clone();
            assert_eq!(vault2.chain_id, 137);
            assert_eq!(vault2.id, Bytes::from_str("0x0234").unwrap());
            assert_eq!(
                vault2.owner,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(vault2.vault_id, U256::from_str("0x0234").unwrap());
            assert!(vault2.balance.eq(F2).unwrap());
            assert_eq!(vault2.formatted_balance, "2");
            assert_eq!(vault2.token.id, "token2");
            assert_eq!(
                vault2.orderbook,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_vault() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x10").unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(vault.chain_id, 1);
            assert_eq!(vault.id, Bytes::from_str("0x0123").unwrap());
            assert_eq!(
                vault.owner,
                Address::from_str("0x0000000000000000000000000000000000000000").unwrap()
            );
            assert_eq!(vault.vault_id, U256::from_str("0x0123").unwrap());

            assert!(
                vault.balance.eq(F1).unwrap(),
                "unexpected balance: {}",
                vault.balance.format().unwrap()
            );
            assert_eq!(vault.formatted_balance, "1");

            assert_eq!(vault.token.id, "token1");
            assert_eq!(
                vault.orderbook,
                Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_vault_missing_decimals() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": json!({
                            "id": "0x0123",
                            "owner": "0x0000000000000000000000000000000000000000",
                            "vaultId": "0x10",
                            "balance": "69862789",
                            "token": {
                                "id": "token1",
                                "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                "name": "Token 1",
                                "symbol": "TKN1",
                                "decimals": null // Missing decimals
                            },
                            "orderbook": {
                                "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                            },
                            "ordersAsOutput": [],
                            "ordersAsInput": [],
                            "balanceChanges": []
                        })
                    }
                }));
            });
            // 6 decimals token info
            sg_server.mock(|when, then| {
                when.method("POST")
                    .path("/rpc1")
                    .body_contains("0x313ce567");
                then.body(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": "0x0000000000000000000000000000000000000000000000000000000000000006"
                    })
                    .to_string(),
                );
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let err = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap_err();
            assert!(matches!(
                err,
                RaindexError::MissingErc20Decimals(token)
                if token == "0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d"
            ));
        }

        #[tokio::test]
        async fn test_get_vault_balance_changes() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1").body_contains("SgVaultDetailQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("SgVaultBalanceChangesListQuery")
                    .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": [
                            {
                                "id": "0xdeposit001",
                                "__typename": "Deposit",
                                "amount": F5,
                                "newVaultBalance": F5,
                                "oldVaultBalance": F0,
                                "vault": {
                                    "id": "0x166aeed725f0f3ef9fe62f2a9054035756d55e5560b17afa1ae439e9cd362902",
                                    "vaultId": "1",
                                    "token": {
                                        "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "name": "Wrapped Flare",
                                        "symbol": "WFLR",
                                        "decimals": "18"
                                    }
                                },
                                "timestamp": "1734054063",
                                "transaction": {
                                    "id": "0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22",
                                    "from": "0x7177b9d00bb5dbcaaf069cc63190902763783b09",
                                    "blockNumber": "34407047",
                                    "timestamp": "1734054063"
                                },
                                "orderbook": {
                                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                                }
                            }
                        ]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("SgVaultBalanceChangesListQuery")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": []
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    // not used
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let result = vault.get_balance_changes(None, None).await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].r#type, RaindexVaultBalanceChangeType::Deposit);
            assert_eq!(result[0].vault_id, U256::from_str("1").unwrap());
            assert_eq!(
                result[0].token.id,
                "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d"
            );
            assert_eq!(
                result[0].token.address,
                Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
            );
            assert_eq!(result[0].token.name, Some("Wrapped Flare".to_string()));
            assert_eq!(result[0].token.symbol, Some("WFLR".to_string()));
            assert_eq!(result[0].token.decimals, 18);
            assert!(result[0].amount.eq(F5).unwrap());
            assert_eq!(result[0].formatted_amount, "5");
            assert!(result[0].new_balance.eq(F5).unwrap());
            assert_eq!(result[0].formatted_new_balance, "5");
            assert!(result[0].old_balance.eq(F0).unwrap());
            assert_eq!(result[0].formatted_old_balance, "0");
            assert_eq!(result[0].timestamp, U256::from_str("1734054063").unwrap());
            assert_eq!(
                result[0].transaction.id(),
                b256!("0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22")
            );
            assert_eq!(
                result[0].transaction.from(),
                Address::from_str("0x7177b9d00bB5dbcaaF069CC63190902763783b09").unwrap()
            );
            assert_eq!(result[0].transaction.block_number(), U256::from(34407047));
            assert_eq!(result[0].transaction.timestamp(), U256::from(1734054063));
            assert_eq!(
                result[0].orderbook,
                Address::from_str("0xcee8cd002f151a536394e564b84076c41bbbcd4d").unwrap()
            );
        }

        #[tokio::test]
        async fn test_formatted_balance_with_different_decimals() {
            let vault_6_decimals_json = json!({
                "id": "0x0456",
                "owner": "0x0000000000000000000000000000000000000000",
                "vaultId": "0x30",
                "balance": F1_5,
                "token": {
                    "id": "token_usdc",
                    "address": "0xa0b86a33e6c3a0e4e8c7b6c6b0c2f6a3b7e8d9e0",
                    "name": "USD Coin",
                    "symbol": "USDC",
                    "decimals": "6"
                },
                "orderbook": {
                    "id": CHAIN_ID_1_ORDERBOOK_ADDRESS
                },
                "ordersAsOutput": [],
                "ordersAsInput": [],
                "balanceChanges": []
            });

            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": vault_6_decimals_json
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0456").unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(vault.formatted_balance, "1.5");
            assert!(vault.balance.eq(F1_5).unwrap());
        }

        #[tokio::test]
        async fn test_formatted_balance_change_with_negative_amount() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1").body_contains("SgVaultDetailQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("SgVaultBalanceChangesListQuery")
                    .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": [
                            {
                                "id": "0xwithdrawal001",
                                "__typename": "Withdrawal",
                                "amount": NEG2,
                                "newVaultBalance": F3,
                                "oldVaultBalance": F5,
                                "vault": {
                                    "id": "0x166aeed725f0f3ef9fe62f2a9054035756d55e5560b17afa1ae439e9cd362902",
                                    "vaultId": "1",
                                    "token": {
                                        "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "name": "Wrapped Ether",
                                        "symbol": "WETH",
                                        "decimals": "18"
                                    }
                                },
                                "timestamp": "1734054063",
                                "transaction": {
                                    "id": "0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22",
                                    "from": "0x7177b9d00bb5dbcaaf069cc63190902763783b09",
                                    "blockNumber": "34407047",
                                    "timestamp": "1734054063"
                                },
                                "orderbook": {
                                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                                }
                            }
                        ]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("SgVaultBalanceChangesListQuery")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": []
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let result = vault.get_balance_changes(None, None).await.unwrap();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].r#type, RaindexVaultBalanceChangeType::Withdrawal);

            assert!(result[0].amount.eq(NEG2).unwrap());
            assert_eq!(result[0].formatted_amount, "-2");

            assert!(result[0].old_balance.eq(F5).unwrap());
            assert_eq!(result[0].formatted_old_balance, "5");

            assert!(result[0].new_balance.eq(F3).unwrap());
            assert_eq!(result[0].formatted_new_balance, "3");
        }

        #[tokio::test]
        async fn test_missing_decimals_formatted_balance() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1").body_contains("SgVaultDetailQuery");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("SgVaultBalanceChangesListQuery")
                    .body_contains("\"skip\":0");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": [
                            {
                                "id": "0xwithdrawal002",
                                "__typename": "Withdrawal",
                                "amount": "-25354",
                                "newVaultBalance": "3378982",
                                "oldVaultBalance": "50008796",
                                "vault": {
                                    "id": "0x166aeed725f0f3ef9fe62f2a9054035756d55e5560b17afa1ae439e9cd362902",
                                    "vaultId": "1",
                                    "token": {
                                        "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                        "name": "Wrapped Ether",
                                        "symbol": "WETH",
                                        "decimals": null
                                    }
                                },
                                "timestamp": "1734054063",
                                "transaction": {
                                    "id": "0x85857b5c6d0b277f9e971b6b45cab98720f90b8f24d65df020776d675b71fc22",
                                    "from": "0x7177b9d00bb5dbcaaf069cc63190902763783b09",
                                    "blockNumber": "34407047",
                                    "timestamp": "1734054063"
                                },
                                "orderbook": {
                                    "id": "0xcee8cd002f151a536394e564b84076c41bbbcd4d"
                                }
                            }
                        ]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("SgVaultBalanceChangesListQuery")
                    .body_contains("\"skip\":200");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaultBalanceChanges": []
                    }
                }));
            });
            // 6 decimals token info
            sg_server.mock(|when, then| {
                when.method("POST")
                    .path("/rpc1")
                    .body_contains("0x313ce567");
                then.body(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": "0x0000000000000000000000000000000000000000000000000000000000000006"
                    })
                    .to_string(),
                );
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let err = vault.get_balance_changes(None, None).await.unwrap_err();
            assert!(matches!(
                err,
                RaindexError::MissingErc20Decimals(token)
                if token == "0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d"
            ));
        }

        #[tokio::test]
        async fn test_get_vault_deposit_calldata() {
            let server = MockServer::start_async().await;
            server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            server.mock(|when, then| {
                when.path("/rpc1");
                then.status(200).json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": encode_prefixed(decimalsCall::abi_encode_returns(&18))
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &server.url("/sg1"),
                    &server.url("/sg2"),
                    &server.url("/rpc1"),
                    // not used
                    &server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let result = vault
                .get_deposit_calldata(&Float::parse("500".to_string()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                result,
                Bytes::copy_from_slice(
                    &deposit4Call {
                        token: Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d")
                            .unwrap(),
                        vaultId: B256::from(U256::from_str("0x0123").unwrap()),
                        depositAmount: Float::parse("500".to_string()).unwrap().get_inner(),
                        tasks: vec![],
                    }
                    .abi_encode()
                )
            );

            let err = vault
                .get_deposit_calldata(&Float::parse("0".to_string()).unwrap())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ZeroAmount.to_string());
        }

        #[tokio::test]
        async fn test_get_vault_withdraw_calldata() {
            let server = MockServer::start_async().await;
            server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            server.mock(|when, then| {
                when.path("/rpc1");
                then.status(200).json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": encode_prefixed(decimalsCall::abi_encode_returns(&18))
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &server.url("/sg1"),
                    &server.url("/sg2"),
                    &server.url("/rpc1"),
                    // not used
                    &server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let amount: Float = Float::parse("0.0000000000000005".to_string()).unwrap();
            let result = vault.get_withdraw_calldata(&amount).await.unwrap();
            assert_eq!(
                result,
                Bytes::copy_from_slice(
                    &withdraw4Call {
                        token: Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d")
                            .unwrap(),
                        vaultId: B256::from(U256::from_str("0x0123").unwrap()),
                        targetAmount: amount.get_inner(),
                        tasks: vec![],
                    }
                    .abi_encode()
                )
            );

            let err = vault
                .get_withdraw_calldata(&Float::parse("0".to_string()).unwrap())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ZeroAmount.to_string());
        }

        #[tokio::test]
        async fn test_get_vault_approval_calldata() {
            let rpc_server = MockServer::start_async().await;
            rpc_server.mock(|when, then| {
                when.path("/rpc1");
                then.status(200).json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x0000000000000000000000000000000000000000000000056BC75E2D63100000",
                }));
            });

            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &rpc_server.url("/rpc1"),
                    &rpc_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let result = vault
                .get_approval_calldata(&Float::parse("600".to_string()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                result,
                Bytes::copy_from_slice(
                    &approveCall {
                        spender: Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                        amount: U256::from(600000000000000000000u128),
                    }
                    .abi_encode(),
                )
            );

            let err = vault
                .get_approval_calldata(&Float::parse("0".to_string()).unwrap())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ZeroAmount.to_string());

            let err = vault
                .get_approval_calldata(&Float::parse("90".to_string()).unwrap())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ExistingAllowance.to_string());

            let err = vault
                .get_approval_calldata(&Float::parse("100".to_string()).unwrap())
                .await
                .unwrap_err();
            assert_eq!(err.to_string(), RaindexError::ExistingAllowance.to_string());
        }

        #[tokio::test]
        async fn test_check_vault_allowance() {
            let rpc_server = MockServer::start_async().await;
            rpc_server.mock(|when, then| {
                when.path("/rpc1");
                then.status(200).json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x0000000000000000000000000000000000000000000000000000000000000001",
                }));
            });

            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &rpc_server.url("/rpc1"),
                    &rpc_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();
            let result = vault.get_allowance().await.unwrap();
            assert_eq!(result.0, U256::from(1));
        }

        #[tokio::test]
        async fn test_get_vaults_with_token_filter() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("\"token_in\":[\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\"]");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault1_json()]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2")
                    .body_contains("\"token_in\":[\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\"]");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": []
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let filters = GetVaultsFilters {
                owners: vec![],
                hide_zero_balance: false,
                tokens: Some(vec![Address::from_str(
                    "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                )
                .unwrap()]),
                orderbook_addresses: None,
                only_active_orders: false,
            };

            let result = raindex_client
                .get_vaults(None, Some(filters), None)
                .await
                .unwrap()
                .items();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].id, Bytes::from_str("0x0123").unwrap());
            assert_eq!(
                result[0].token.address,
                Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap()
            );
        }

        #[tokio::test]
        async fn test_get_vaults_with_multiple_token_filters() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1")
                    .body_contains("\"token_in\":[\"0x1d80c49bbbcd1c0911346656b529df9e5c2f783d\",\"0x12e605bc104e93b45e1ad99f9e555f659051c2bb\"]");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": [get_vault1_json(), get_vault2_json()]
                    }
                }));
            });
            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vaults": []
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();

            let filters = GetVaultsFilters {
                owners: vec![],
                hide_zero_balance: false,
                tokens: Some(vec![
                    Address::from_str("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d").unwrap(),
                    Address::from_str("0x12e605bc104e93b45e1ad99f9e555f659051c2bb").unwrap(),
                ]),
                orderbook_addresses: None,
                only_active_orders: false,
            };

            let result = raindex_client
                .get_vaults(None, Some(filters), None)
                .await
                .unwrap()
                .items();

            assert_eq!(result.len(), 2);
        }

        #[tokio::test]
        async fn test_get_all_vault_tokens_without_filter() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "erc20S": [
                            {
                                "id": "token1",
                                "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                "name": "Token 1",
                                "symbol": "TKN1",
                                "decimals": "18"
                            }
                        ]
                    }
                }));
            });

            sg_server.mock(|when, then| {
                when.path("/sg2");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "erc20S": [
                            {
                                "id": "token2",
                                "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783f",
                                "name": "Token 2",
                                "symbol": "TKN2",
                                "decimals": "18"
                            }
                        ]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();

            // Test with specific chain filter (only chain 1)
            let result = raindex_client.get_all_vault_tokens(None).await.unwrap();

            assert_eq!(result.len(), 2);
        }

        #[tokio::test]
        async fn test_get_all_vault_tokens_with_chain_filter() {
            let sg_server = MockServer::start_async().await;
            sg_server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "erc20S": [
                            {
                                "id": "token1",
                                "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                "name": "Token 1",
                                "symbol": "TKN1",
                                "decimals": "18"
                            }
                        ]
                    }
                }));
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &sg_server.url("/sg1"),
                    &sg_server.url("/sg2"),
                    &sg_server.url("/rpc1"),
                    &sg_server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();

            // Test with specific chain filter (only chain 1)
            let result = raindex_client
                .get_all_vault_tokens(Some(ChainIds(vec![1])))
                .await
                .unwrap();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].id(), "token1");
            assert_eq!(result[0].chain_id(), 1);
        }

        #[tokio::test]
        async fn test_get_account_balance_from_vault() {
            let server = MockServer::start_async().await;
            server.mock(|when, then| {
                when.path("/sg1");
                then.status(200).json_body_obj(&json!({
                    "data": {
                        "vault": get_vault1_json()
                    }
                }));
            });
            server.mock(|when, then| {
                when.method("POST")
                    .path("/rpc1")
                    .body_contains("0x70a08231");
                then.body(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": "0x00000000000000000000000000000000000000000000000000000000000003e8"
                    })
                    .to_string(),
                );
            });

            let raindex_client = RaindexClient::new(
                vec![get_test_yaml(
                    &server.url("/sg1"),
                    &server.url("/sg2"),
                    &server.url("/rpc1"),
                    &server.url("/rpc2"),
                )],
                None,
                None,
            )
            .await
            .unwrap();
            let vault = raindex_client
                .get_vault(
                    &OrderbookIdentifier::new(
                        1,
                        Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                    ),
                    Bytes::from_str("0x0123").unwrap(),
                )
                .await
                .unwrap();

            let balance = vault.get_owner_balance(Address::random()).await.unwrap();
            assert_eq!(balance, U256::from(1000));
        }

        #[test]
        fn get_vaults_filters_to_sg_filter_args_maps_orderbook_addresses() {
            use rain_orderbook_subgraph_client::types::common::SgVaultsListFilterArgs;

            let filters = GetVaultsFilters {
                owners: vec![],
                hide_zero_balance: false,
                tokens: None,
                orderbook_addresses: Some(vec![
                    address!("0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
                    address!("0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"),
                ]),
                only_active_orders: false,
            };

            let sg_filter_args: SgVaultsListFilterArgs = filters.try_into().unwrap();

            assert_eq!(sg_filter_args.orderbooks.len(), 2);
            assert_eq!(
                sg_filter_args.orderbooks[0],
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            );
            assert_eq!(
                sg_filter_args.orderbooks[1],
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            );
        }

        #[test]
        fn get_vaults_filters_to_sg_filter_args_empty_orderbook_addresses() {
            use rain_orderbook_subgraph_client::types::common::SgVaultsListFilterArgs;

            let filters = GetVaultsFilters {
                owners: vec![],
                hide_zero_balance: false,
                tokens: None,
                orderbook_addresses: None,
                only_active_orders: false,
            };

            let sg_filter_args: SgVaultsListFilterArgs = filters.try_into().unwrap();

            assert!(sg_filter_args.orderbooks.is_empty());
        }

        #[test]
        fn get_vaults_filters_to_sg_filter_args_lowercases_mixed_case_addresses() {
            use rain_orderbook_subgraph_client::types::common::SgVaultsListFilterArgs;

            let filters = GetVaultsFilters {
                owners: vec![],
                hide_zero_balance: false,
                tokens: None,
                orderbook_addresses: Some(vec![address!(
                    "0xDeaDbEEfDeaDbEEfDeaDbEEfDeaDbEEfDeaDbEEf"
                )]),
                only_active_orders: false,
            };

            let sg_filter_args: SgVaultsListFilterArgs = filters.try_into().unwrap();

            assert_eq!(sg_filter_args.orderbooks.len(), 1);
            assert_eq!(
                sg_filter_args.orderbooks[0],
                "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
            );
        }
    }
}
