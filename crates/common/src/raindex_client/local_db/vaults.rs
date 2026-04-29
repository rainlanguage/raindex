use super::super::RaindexError;
use super::{
    query::fetch_all_tokens::fetch_all_tokens,
    query::fetch_vault_balance_changes::fetch_vault_balance_changes,
    query::fetch_vaults::fetch_vaults, LocalDb, RaindexIdentifier,
};
use crate::raindex_client::ClientRef;
use crate::{
    local_db::query::{
        fetch_all_tokens::FetchAllTokensArgs, fetch_vaults::FetchVaultsArgs,
        fetch_vaults::LocalDbVault,
    },
    raindex_client::vaults::{
        GetVaultsFilters, RaindexVault, RaindexVaultBalanceChange, RaindexVaultToken,
        VaultBalanceChangeFilter, VaultsDataSource,
    },
};
use alloy::primitives::Bytes;
#[cfg(target_family = "wasm")]
use alloy::primitives::{Address, U256};
use async_trait::async_trait;
#[cfg(target_family = "wasm")]
use std::str::FromStr;

pub struct LocalDbVaults<'a> {
    pub(crate) db: &'a LocalDb,
    pub(crate) client: ClientRef,
}

impl<'a> LocalDbVaults<'a> {
    pub(crate) fn new(db: &'a LocalDb, client: ClientRef) -> Self {
        Self { db, client }
    }

    fn convert_local_db_vaults(
        &self,
        local_vaults: Vec<LocalDbVault>,
        vault_type: Option<crate::raindex_client::vaults::RaindexVaultType>,
    ) -> Result<Vec<RaindexVault>, RaindexError> {
        let raindex_client = ClientRef::clone(&self.client);
        local_vaults
            .into_iter()
            .map(|local_vault| {
                RaindexVault::try_from_local_db(
                    ClientRef::clone(&raindex_client),
                    local_vault,
                    vault_type.clone(),
                )
            })
            .collect()
    }
}

#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
impl VaultsDataSource for LocalDbVaults<'_> {
    async fn list(
        &self,
        chain_ids: Option<Vec<u32>>,
        filters: &GetVaultsFilters,
        _page: Option<u16>,
    ) -> Result<Vec<RaindexVault>, RaindexError> {
        let mut fetch_args = FetchVaultsArgs::from_filters(filters.clone());
        if let Some(ids) = chain_ids {
            if !ids.is_empty() {
                fetch_args.chain_ids = ids;
            }
        }

        let local_vaults = fetch_vaults(self.db, fetch_args).await?;
        self.convert_local_db_vaults(local_vaults, None)
    }

    async fn get_by_id(
        &self,
        raindex_id: &RaindexIdentifier,
        vault_id: &Bytes,
    ) -> Result<Option<RaindexVault>, RaindexError> {
        let fetch_args = FetchVaultsArgs {
            chain_ids: vec![raindex_id.chain_id],
            raindex_addresses: vec![raindex_id.raindex_address],
            hide_zero_balance: false,
            ..FetchVaultsArgs::default()
        };

        let local_vaults = fetch_vaults(self.db, fetch_args).await?;
        let requested_id = vault_id.to_string().to_lowercase();

        let vault = self
            .convert_local_db_vaults(local_vaults, None)?
            .into_iter()
            .find(|vault| vault.id().to_string().to_lowercase() == requested_id);

        Ok(vault)
    }

    async fn balance_changes_list(
        &self,
        vault: &RaindexVault,
        _page: Option<u16>,
        filter_types: Option<&[VaultBalanceChangeFilter]>,
    ) -> Result<Vec<RaindexVaultBalanceChange>, RaindexError> {
        #[cfg(target_family = "wasm")]
        let raindex_address = Address::from_str(&vault.raindex())?;
        #[cfg(not(target_family = "wasm"))]
        let raindex_address = vault.raindex();

        #[cfg(target_family = "wasm")]
        let vault_id = U256::from_str(&vault.vault_id_hex())?;
        #[cfg(not(target_family = "wasm"))]
        let vault_id = vault.vault_id();

        #[cfg(target_family = "wasm")]
        let token_address = Address::from_str(&vault.token().address())?;
        #[cfg(not(target_family = "wasm"))]
        let token_address = vault.token().address();

        #[cfg(target_family = "wasm")]
        let owner_address = Address::from_str(&vault.owner())?;
        #[cfg(not(target_family = "wasm"))]
        let owner_address = vault.owner();

        let raindex_id = crate::local_db::RaindexIdentifier::new(vault.chain_id(), raindex_address);
        let local_changes = fetch_vault_balance_changes(
            self.db,
            &raindex_id,
            vault_id,
            token_address,
            owner_address,
            filter_types,
        )
        .await?;

        local_changes
            .into_iter()
            .map(|change| RaindexVaultBalanceChange::try_from_local_db(vault, change))
            .collect()
    }

    async fn tokens_list(
        &self,
        chain_ids: Option<Vec<u32>>,
    ) -> Result<Vec<RaindexVaultToken>, RaindexError> {
        let mut fetch_args = FetchAllTokensArgs::default();
        if let Some(ids) = chain_ids {
            if !ids.is_empty() {
                fetch_args.chain_ids = ids;
            }
        }

        let local_tokens = fetch_all_tokens(self.db, fetch_args).await?;

        Ok(local_tokens
            .into_iter()
            .map(RaindexVaultToken::from_local_db_token)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::local_db::query::fetch_vaults::LocalDbVault;
        use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
        use crate::raindex_client::local_db::LocalDb;
        use crate::raindex_client::tests::get_local_db_test_yaml;
        use crate::raindex_client::RaindexClient;
        use alloy::primitives::{address, Address, Bytes, U256};
        use rain_math_float::Float;
        use serde_json;
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::str::FromStr;
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::*;

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
                raindex_address: address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB"),
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
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));
            let vaults = data_source
                .list(Some(vec![42161]), &GetVaultsFilters::default(), None)
                .await
                .expect("local db vaults should load");

            assert_eq!(vaults.len(), 1);
            let result_vault = &vaults[0];
            assert_eq!(result_vault.chain_id(), 42161);
            assert_eq!(result_vault.owner().to_lowercase(), owner.to_string());
            assert_eq!(
                result_vault.raindex().to_lowercase(),
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
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let rc_client = Rc::new(client.clone());
            let derived_vault =
                RaindexVault::try_from_local_db(Rc::clone(&rc_client), local_vault, None)
                    .expect("local vault should convert");

            let vault_id_hex = derived_vault.id();
            let vault_id_bytes = Bytes::from_str(&vault_id_hex).expect("valid vault id");

            let raindex_addr =
                Address::from_str("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB").unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));
            let retrieved = data_source
                .get_by_id(
                    &RaindexIdentifier::new(42161, raindex_addr),
                    &vault_id_bytes,
                )
                .await
                .expect("local vault retrieval should succeed")
                .expect("vault should be found");

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
        async fn test_list_with_none_chain_ids_queries_all_vaults() {
            let owner = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let token = "0x00000000000000000000000000000000000000aa";
            let vault =
                make_local_vault("0x01", token, owner, Float::parse("1".to_string()).unwrap());

            let captured_sql = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let json = serde_json::to_string(&vec![vault]).unwrap();
            let callback = create_sql_capturing_callback(&json, captured_sql.clone());
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));

            let vaults = data_source
                .list(None, &GetVaultsFilters::default(), None)
                .await
                .expect("should query without chain_ids");

            assert_eq!(vaults.len(), 1);

            let sql = captured_sql.borrow();
            assert!(
                !sql.0.contains("o.chain_id IN"),
                "should not filter by chain_id when None: {}",
                sql.0
            );
        }

        #[wasm_bindgen_test]
        async fn test_list_with_empty_chain_ids_queries_all_vaults() {
            let owner = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let token = "0x00000000000000000000000000000000000000aa";
            let vault =
                make_local_vault("0x01", token, owner, Float::parse("1".to_string()).unwrap());

            let captured_sql = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let json = serde_json::to_string(&vec![vault]).unwrap();
            let callback = create_sql_capturing_callback(&json, captured_sql.clone());
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));

            let vaults = data_source
                .list(Some(vec![]), &GetVaultsFilters::default(), None)
                .await
                .expect("should query with empty chain_ids");

            assert_eq!(vaults.len(), 1);

            let sql = captured_sql.borrow();
            assert!(
                !sql.0.contains("o.chain_id IN"),
                "should not filter by chain_id when empty: {}",
                sql.0
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_vaults_local_db_filters() {
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
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));

            let filters = GetVaultsFilters {
                owners: vec![Address::from_str(owner_kept).unwrap()],
                hide_zero_balance: true,
                tokens: Some(vec![Address::from_str(token_kept).unwrap()]),
                raindex_addresses: None,
                only_active_orders: false,
            };

            let vaults = data_source
                .list(Some(vec![42161]), &filters, None)
                .await
                .expect("filtered vaults should load");

            assert_eq!(vaults.len(), 1);
            let vault = &vaults[0];
            assert_eq!(vault.owner().to_lowercase(), owner_kept.to_string());
            let token_meta = vault.token();
            assert_eq!(token_meta.address().to_lowercase(), token_kept.to_string());
            assert_eq!(vault.formatted_balance(), "2".to_string());

            let sql = captured_sql.borrow();
            assert!(sql.0.contains("o.owner IN ("));
            assert!(sql.0.contains("o.token IN ("));
            assert!(sql.0.contains("AND NOT FLOAT_IS_ZERO("));

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

            let chain_id = params_array.get(0);
            let chain_id_bigint = chain_id
                .dyn_into::<js_sys::BigInt>()
                .expect("chain id should be BigInt");
            let chain_id_str = chain_id_bigint.to_string(10).unwrap().as_string().unwrap();
            assert_eq!(chain_id_str, "42161");

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

        #[wasm_bindgen_test]
        async fn test_tokens_list_with_none_chain_ids_returns_all() {
            use crate::local_db::query::fetch_all_tokens::LocalDbToken;

            let token = LocalDbToken {
                chain_id: 42161,
                raindex_address: address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB"),
                token_address: address!("0x00000000000000000000000000000000000000aa"),
                name: "Test Token".to_string(),
                symbol: "TST".to_string(),
                decimals: 18,
            };

            let captured_sql = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let json = serde_json::to_string(&vec![token]).unwrap();
            let callback = create_sql_capturing_callback(&json, captured_sql.clone());
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));

            let tokens = data_source
                .tokens_list(None)
                .await
                .expect("should query tokens without chain_ids");

            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].symbol(), Some("TST".to_string()));

            let sql = captured_sql.borrow();
            assert!(
                !sql.0.contains("chain_id IN"),
                "should not filter by chain_id when None: {}",
                sql.0
            );
        }

        #[wasm_bindgen_test]
        async fn test_tokens_list_with_empty_chain_ids_returns_all() {
            use crate::local_db::query::fetch_all_tokens::LocalDbToken;

            let token = LocalDbToken {
                chain_id: 42161,
                raindex_address: address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB"),
                token_address: address!("0x00000000000000000000000000000000000000aa"),
                name: "Test Token".to_string(),
                symbol: "TST".to_string(),
                decimals: 18,
            };

            let captured_sql = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let json = serde_json::to_string(&vec![token]).unwrap();
            let callback = create_sql_capturing_callback(&json, captured_sql.clone());
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));

            let tokens = data_source
                .tokens_list(Some(vec![]))
                .await
                .expect("should query tokens with empty chain_ids");

            assert_eq!(tokens.len(), 1);

            let sql = captured_sql.borrow();
            assert!(
                !sql.0.contains("chain_id IN"),
                "should not filter by chain_id when empty: {}",
                sql.0
            );
        }

        #[wasm_bindgen_test]
        async fn test_tokens_list_with_chain_ids_filters() {
            use crate::local_db::query::fetch_all_tokens::LocalDbToken;

            let token = LocalDbToken {
                chain_id: 42161,
                raindex_address: address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB"),
                token_address: address!("0x00000000000000000000000000000000000000aa"),
                name: "Test Token".to_string(),
                symbol: "TST".to_string(),
                decimals: 18,
            };

            let captured_sql = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let json = serde_json::to_string(&vec![token]).unwrap();
            let callback = create_sql_capturing_callback(&json, captured_sql.clone());
            let local_db = LocalDb::from_js_callback(callback, None);

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));

            let tokens = data_source
                .tokens_list(Some(vec![42161]))
                .await
                .expect("should query tokens with chain_ids filter");

            assert_eq!(tokens.len(), 1);

            let sql = captured_sql.borrow();
            assert!(
                sql.0.contains("chain_id IN"),
                "should filter by chain_id: {}",
                sql.0
            );
        }

        #[wasm_bindgen_test]
        async fn test_balance_changes_list_returns_changes() {
            use crate::local_db::query::fetch_vault_balance_changes::LocalDbVaultBalanceChange;
            use alloy::primitives::B256;

            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let token = address!("0x00000000000000000000000000000000000000aa");
            let raindex_addr = address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB");
            let vault_id = U256::from(1);

            let balance_change = LocalDbVaultBalanceChange {
                transaction_hash: B256::ZERO,
                log_index: 0,
                block_number: 12345,
                block_timestamp: 1700000000,
                owner,
                change_type: "Deposit".to_string(),
                token,
                vault_id,
                delta: Float::parse("100".to_string()).unwrap().as_hex(),
                running_balance: Float::parse("100".to_string()).unwrap().as_hex(),
            };

            let local_vault = LocalDbVault {
                chain_id: 42161,
                vault_id,
                token,
                owner,
                raindex_address: raindex_addr,
                token_name: "Token".to_string(),
                token_symbol: "TKN".to_string(),
                token_decimals: 18,
                balance: Float::parse("100".to_string()).unwrap().as_hex(),
                input_orders: None,
                output_orders: None,
            };

            let client = RaindexClient::new(vec![get_local_db_test_yaml()], None, None, None, None)
                .await
                .unwrap();
            let rc_client = Rc::new(client.clone());
            let raindex_vault =
                RaindexVault::try_from_local_db(Rc::clone(&rc_client), local_vault, None)
                    .expect("should convert vault");

            let json = serde_json::to_string(&vec![balance_change]).unwrap();
            let captured_sql = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let callback = create_sql_capturing_callback(&json, captured_sql.clone());
            let local_db = LocalDb::from_js_callback(callback, None);

            let data_source = LocalDbVaults::new(&local_db, Rc::new(client));

            let changes = data_source
                .balance_changes_list(&raindex_vault, None, None)
                .await
                .expect("should get balance changes");

            assert_eq!(changes.len(), 1);
            assert_eq!(changes[0].formatted_amount(), "100");
        }
    }
}
