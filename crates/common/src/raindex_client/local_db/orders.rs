use super::super::orders::{
    GetOrdersFilters, OrdersDataSource, RaindexOrder, RaindexOrdersListResult,
};
use super::super::trades::RaindexTrade;
use super::super::RaindexError;
use super::query::fetch_order_trades::fetch_order_trades;
use super::query::fetch_order_trades_count::fetch_order_trades_count;
use super::query::fetch_orders_count::fetch_orders_count;
use super::LocalDb;
use crate::local_db::query::fetch_orders::LocalDbOrder;
use crate::local_db::query::fetch_vaults::LocalDbVault;
use crate::local_db::query::LocalDbQueryError;
use crate::local_db::{query::fetch_orders::FetchOrdersArgs, OrderbookIdentifier};
use crate::raindex_client::local_db::query::fetch_orders::fetch_orders;
use crate::raindex_client::ClientRef;
use alloy::primitives::{Address, B256};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::from_str;

pub struct LocalDbOrders<'a> {
    pub(crate) db: &'a LocalDb,
    pub(crate) client: ClientRef,
}

fn convert_local_db_order(
    client: ClientRef,
    local_db_order: LocalDbOrder,
) -> Result<RaindexOrder, RaindexError> {
    let inputs = parse_io_vaults("inputs", &local_db_order.inputs)?;
    let outputs = parse_io_vaults("outputs", &local_db_order.outputs)?;
    RaindexOrder::from_local_db_order(client, local_db_order, inputs, outputs)
}

impl<'a> LocalDbOrders<'a> {
    pub(crate) fn new(db: &'a LocalDb, client: ClientRef) -> Self {
        Self { db, client }
    }

    async fn fetch_orders_by_tx_hash(
        &self,
        chain_id: u32,
        orderbook: Address,
        tx_hash: B256,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        let fetch_args = FetchOrdersArgs {
            chain_ids: vec![chain_id],
            orderbook_addresses: vec![orderbook],
            tx_hash: Some(tx_hash),
            ..FetchOrdersArgs::default()
        };

        let local_db_orders = fetch_orders(self.db, fetch_args).await?;
        let client = ClientRef::clone(&self.client);

        local_db_orders
            .into_iter()
            .map(|order| convert_local_db_order(ClientRef::clone(&client), order))
            .collect()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalDbOrderIo {
    #[serde(rename = "ioIndex")]
    io_index: usize,
    vault: LocalDbVault,
}

pub(crate) fn parse_io_vaults(
    label: &str,
    payload: &Option<String>,
) -> Result<Vec<LocalDbVault>, RaindexError> {
    let Some(raw) = payload else {
        return Ok(Vec::new());
    };

    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parsed: Vec<Option<LocalDbOrderIo>> = from_str(raw).map_err(|err| {
        RaindexError::from(LocalDbQueryError::deserialization(format!(
            "Failed to decode {label} payload: {err}"
        )))
    })?;

    let mut ios: Vec<LocalDbOrderIo> = parsed.into_iter().flatten().collect();
    ios.sort_by_key(|io| io.io_index);

    Ok(ios.into_iter().map(|io| io.vault).collect())
}

#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
impl OrdersDataSource for LocalDbOrders<'_> {
    async fn list(
        &self,
        chain_ids: Option<Vec<u32>>,
        filters: &GetOrdersFilters,
        page: Option<u16>,
        page_size: Option<u16>,
    ) -> Result<RaindexOrdersListResult, RaindexError> {
        let mut fetch_args = FetchOrdersArgs::from(filters.clone());
        if let Some(ids) = chain_ids {
            if !ids.is_empty() {
                fetch_args.chain_ids = ids;
            }
        }

        let total_count = if page.is_some() {
            let count_args = FetchOrdersArgs {
                page: None,
                page_size: None,
                ..fetch_args.clone()
            };
            fetch_orders_count(self.db, count_args).await?
        } else {
            0
        };

        fetch_args.page = page;
        fetch_args.page_size = page_size;

        let local_db_orders = fetch_orders(self.db, fetch_args).await?;
        let client = ClientRef::clone(&self.client);

        let orders: Vec<RaindexOrder> = local_db_orders
            .into_iter()
            .map(|order| convert_local_db_order(ClientRef::clone(&client), order))
            .collect::<Result<_, _>>()?;

        Ok(RaindexOrdersListResult {
            orders,
            total_count,
        })
    }

    async fn get_by_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
    ) -> Result<Option<RaindexOrder>, RaindexError> {
        let fetch_args = FetchOrdersArgs {
            chain_ids: vec![ob_id.chain_id],
            orderbook_addresses: vec![ob_id.orderbook_address],
            order_hash: Some(*order_hash),
            ..FetchOrdersArgs::default()
        };

        let local_db_orders = fetch_orders(self.db, fetch_args).await?;
        let client = ClientRef::clone(&self.client);

        local_db_orders
            .into_iter()
            .next()
            .map(|order| convert_local_db_order(client, order))
            .transpose()
    }

    async fn get_added_by_tx_hash(
        &self,
        chain_id: u32,
        orderbook: Address,
        tx_hash: B256,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        self.fetch_orders_by_tx_hash(chain_id, orderbook, tx_hash)
            .await
    }

    async fn get_removed_by_tx_hash(
        &self,
        chain_id: u32,
        orderbook: Address,
        tx_hash: B256,
    ) -> Result<Vec<RaindexOrder>, RaindexError> {
        self.fetch_orders_by_tx_hash(chain_id, orderbook, tx_hash)
            .await
    }

    async fn trades_list(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
        _page: Option<u16>,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let local_trades =
            fetch_order_trades(self.db, ob_id, *order_hash, start_timestamp, end_timestamp).await?;

        local_trades
            .into_iter()
            .map(|trade| RaindexTrade::try_from_local_db_trade(ob_id.chain_id, trade))
            .collect()
    }

    async fn trades_count(
        &self,
        ob_id: &OrderbookIdentifier,
        order_hash: &B256,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<u64, RaindexError> {
        Ok(
            fetch_order_trades_count(self.db, ob_id, *order_hash, start_timestamp, end_timestamp)
                .await?,
        )
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use std::str::FromStr;

        use super::*;
        use crate::local_db::query::{fetch_orders::LocalDbOrder, fetch_vaults::LocalDbVault};
        use crate::raindex_client::tests::{
            get_local_db_test_yaml, new_test_client_with_db_callback,
        };
        use crate::raindex_client::ChainIds;
        use alloy::primitives::{address, b256, bytes, Bytes, U256};
        use serde_json::{self, json};
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::*;

        fn make_local_db_callback(orders: Vec<LocalDbOrder>) -> js_sys::Function {
            let order_count = orders.len();
            let orders_json = serde_json::to_string(&orders).unwrap();
            let orders_result = WasmEncodedResult::Success::<String> {
                value: orders_json,
                error: None,
            };
            let orders_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&orders_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let count_json = format!("[{{\"orders_count\":{}}}]", order_count);
            let count_result = WasmEncodedResult::Success::<String> {
                value: count_json,
                error: None,
            };
            let count_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&count_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let empty_result = WasmEncodedResult::Success::<String> {
                value: "[]".to_string(),
                error: None,
            };
            let empty_payload =
                js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&empty_result).unwrap())
                    .unwrap()
                    .as_string()
                    .unwrap();

            let callback =
                Closure::wrap(Box::new(move |sql: String, _params: JsValue| -> JsValue {
                    if sql.contains("orders_count") {
                        return js_sys::JSON::parse(&count_payload).unwrap();
                    }
                    if sql.contains("FROM order_events") && sql.contains("json_group_array") {
                        return js_sys::JSON::parse(&orders_payload).unwrap();
                    }

                    js_sys::JSON::parse(&empty_payload).unwrap()
                })
                    as Box<dyn Fn(String, JsValue) -> JsValue>);

            callback.into_js_value().dyn_into().unwrap()
        }

        #[wasm_bindgen_test]
        async fn test_get_orders_local_db_callback_path() {
            let order_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let order_hash_str = order_hash.to_string();
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let order_bytes =
                bytes!("0x00000000000000000000000000000000000000000000000000000000000000ff");
            let order_bytes_str = order_bytes.to_string();
            let transaction_hash =
                b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            let meta = Bytes::from_str("0x1234").unwrap();
            let meta_str = meta.to_string();
            let input_vault_id = U256::from_str("0x0a").unwrap();
            let output_vault_id = U256::from_str("0x0b").unwrap();
            let input_token = address!("0x00000000000000000000000000000000000000aa");
            let output_token = address!("0x00000000000000000000000000000000000000bb");
            let orderbook_address = address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB");

            let input_vault = LocalDbVault {
                chain_id: 1,
                vault_id: input_vault_id,
                token: input_token,
                owner,
                orderbook_address,
                token_name: "Token A".to_string(),
                token_symbol: "TKNA".to_string(),
                token_decimals: 18,
                balance: "0x000000000000000000000000000000000000000000000000000000000000000a"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let output_vault = LocalDbVault {
                chain_id: 1,
                vault_id: output_vault_id,
                token: output_token,
                owner,
                orderbook_address,
                token_name: "Token B".to_string(),
                token_symbol: "TKNB".to_string(),
                token_decimals: 6,
                balance: "0x0000000000000000000000000000000000000000000000000000000000000005"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let inputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": input_vault.chain_id,
                    "vaultId": input_vault.vault_id,
                    "token": input_vault.token,
                    "owner": input_vault.owner,
                    "orderbookAddress": input_vault.orderbook_address,
                    "tokenName": input_vault.token_name,
                    "tokenSymbol": input_vault.token_symbol,
                    "tokenDecimals": input_vault.token_decimals,
                    "balance": input_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let outputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": output_vault.chain_id,
                    "vaultId": output_vault.vault_id,
                    "token": output_vault.token,
                    "owner": output_vault.owner,
                    "orderbookAddress": output_vault.orderbook_address,
                    "tokenName": output_vault.token_name,
                    "tokenSymbol": output_vault.token_symbol,
                    "tokenDecimals": output_vault.token_decimals,
                    "balance": output_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let local_order = LocalDbOrder {
                chain_id: 42161,
                order_hash: order_hash.clone(),
                owner,
                block_timestamp: 123456,
                block_number: 654321,
                orderbook_address,
                order_bytes: order_bytes,
                transaction_hash: transaction_hash.clone(),
                inputs: Some(inputs_json.to_string()),
                outputs: Some(outputs_json.to_string()),
                trade_count: 7,
                active: true,
                meta: Some(meta.clone()),
            };

            let callback = make_local_db_callback(vec![local_order.clone()]);

            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let result = client
                .get_orders(Some(ChainIds(vec![42161])), None, None, None)
                .await
                .expect("local db query should succeed");

            assert_eq!(result.orders().len(), 1);
            assert_eq!(result.total_count(), 1);

            let order = &result.orders()[0];
            assert_eq!(order.chain_id(), 42161);
            assert_eq!(order.order_hash(), order_hash_str);
            assert_eq!(order.order_bytes(), order_bytes_str);
            assert_eq!(
                order.owner().to_lowercase(),
                owner.to_string().to_lowercase()
            );
            assert!(order.active());
            assert_eq!(order.trades_count(), local_order.trade_count as u16);
            assert_eq!(order.meta(), Some(meta_str));
            assert_eq!(order.orderbook(), orderbook_address.to_string());
            let tx = order
                .transaction()
                .expect("transaction should be populated");
            assert_eq!(
                tx.id().to_lowercase(),
                transaction_hash.to_string().to_lowercase()
            );
            assert_eq!(tx.from().to_lowercase(), owner.to_string().to_lowercase());
            assert_eq!(
                tx.block_number()
                    .unwrap()
                    .to_string(10)
                    .unwrap()
                    .as_string()
                    .unwrap(),
                U256::from(local_order.block_number).to_string(),
            );
            assert_eq!(
                tx.timestamp()
                    .unwrap()
                    .to_string(10)
                    .unwrap()
                    .as_string()
                    .unwrap(),
                U256::from(local_order.block_timestamp).to_string()
            );

            let timestamp = order.timestamp_added().unwrap();
            let timestamp_str = timestamp
                .to_string(10)
                .expect("timestamp to_string should succeed")
                .as_string()
                .expect("timestamp string conversion should succeed");
            assert_eq!(timestamp_str, local_order.block_timestamp.to_string());

            let input_vaults = order.inputs_list().items();
            assert_eq!(input_vaults.len(), 1);
            assert_eq!(
                input_vaults[0].token().symbol(),
                Some(input_vault.token_symbol.clone())
            );
            assert_eq!(
                input_vaults[0].orderbook(),
                input_vault.orderbook_address.to_string()
            );

            let output_vaults = order.outputs_list().items();
            assert_eq!(output_vaults.len(), 1);
            assert_eq!(
                output_vaults[0].token().symbol(),
                Some(output_vault.token_symbol.clone())
            );
            assert_eq!(
                output_vaults[0].orderbook(),
                output_vault.orderbook_address.to_string()
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_order_by_hash_local_db_path() {
            let order_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let transaction_hash =
                b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            let order_bytes =
                bytes!("0x00000000000000000000000000000000000000000000000000000000000000ff");
            let meta = Bytes::from_str("0x1234").unwrap();
            let input_vault_id = U256::from_str("0x0a").unwrap();
            let output_vault_id = U256::from_str("0x0b").unwrap();
            let input_token = address!("0x00000000000000000000000000000000000000aa");
            let output_token = address!("0x00000000000000000000000000000000000000bb");
            let orderbook_address = address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB");

            let input_vault = LocalDbVault {
                chain_id: 1,
                vault_id: input_vault_id,
                token: input_token,
                owner,
                orderbook_address,
                token_name: "Token A".to_string(),
                token_symbol: "TKNA".to_string(),
                token_decimals: 18,
                balance: "0x000000000000000000000000000000000000000000000000000000000000000a"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let output_vault = LocalDbVault {
                chain_id: 1,
                vault_id: output_vault_id,
                token: output_token,
                owner,
                orderbook_address,
                token_name: "Token B".to_string(),
                token_symbol: "TKNB".to_string(),
                token_decimals: 6,
                balance: "0x0000000000000000000000000000000000000000000000000000000000000005"
                    .to_string(),
                input_orders: None,
                output_orders: None,
            };

            let inputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": input_vault.chain_id,
                    "vaultId": input_vault.vault_id,
                    "token": input_vault.token,
                    "owner": input_vault.owner,
                    "orderbookAddress": input_vault.orderbook_address,
                    "tokenName": input_vault.token_name,
                    "tokenSymbol": input_vault.token_symbol,
                    "tokenDecimals": input_vault.token_decimals,
                    "balance": input_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let outputs_json = json!([{
                "ioIndex": 0,
                "vault": {
                    "chainId": output_vault.chain_id,
                    "vaultId": output_vault.vault_id,
                    "token": output_vault.token,
                    "owner": output_vault.owner,
                    "orderbookAddress": output_vault.orderbook_address,
                    "tokenName": output_vault.token_name,
                    "tokenSymbol": output_vault.token_symbol,
                    "tokenDecimals": output_vault.token_decimals,
                    "balance": output_vault.balance,
                    "inputOrders": serde_json::Value::Null,
                    "outputOrders": serde_json::Value::Null
                }
            }]);

            let local_order = LocalDbOrder {
                chain_id: 42161,
                order_hash,
                owner,
                block_timestamp: 123456,
                block_number: 654321,
                orderbook_address,
                order_bytes: order_bytes.clone(),
                transaction_hash: transaction_hash.clone(),
                inputs: Some(inputs_json.to_string()),
                outputs: Some(outputs_json.to_string()),
                trade_count: 3,
                active: true,
                meta: Some(meta.clone()),
            };

            let callback = make_local_db_callback(vec![local_order.clone()]);

            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161],
            );

            let order = client
                .get_order_by_hash(
                    &OrderbookIdentifier::new(42161, orderbook_address),
                    order_hash.clone(),
                )
                .await
                .expect("local db order fetch should succeed");

            assert_eq!(order.chain_id(), 42161);
            assert_eq!(order.order_hash(), order_hash.to_string());
            assert_eq!(order.order_bytes(), order_bytes.to_string());
            assert_eq!(
                order.owner().to_lowercase(),
                owner.to_string().to_lowercase()
            );
            assert!(order.active());
            assert_eq!(order.trades_count(), local_order.trade_count as u16);
            assert_eq!(order.meta(), Some(meta.to_string()));
            assert_eq!(order.orderbook(), orderbook_address.to_string());
        }

        #[wasm_bindgen_test]
        async fn test_get_added_by_tx_hash_local_db_path() {
            let order_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000abc");
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let transaction_hash =
                b256!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            let order_bytes =
                bytes!("0x00000000000000000000000000000000000000000000000000000000000000ff");
            let orderbook_address = address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB");

            let local_order = LocalDbOrder {
                chain_id: 137,
                order_hash,
                owner,
                block_timestamp: 123456,
                block_number: 654321,
                orderbook_address,
                order_bytes: order_bytes.clone(),
                transaction_hash,
                inputs: None,
                outputs: None,
                trade_count: 0,
                active: true,
                meta: None,
            };

            let callback = make_local_db_callback(vec![local_order.clone()]);
            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161, 137],
            );

            let orders = client
                .get_add_orders_for_transaction_wasm_binding(
                    137,
                    orderbook_address.to_string(),
                    transaction_hash.to_string(),
                    Some(1),
                    Some(1),
                )
                .await
                .expect("local db query should succeed");

            assert_eq!(orders.len(), 1);
            let order = &orders[0];
            assert_eq!(order.chain_id(), 137);
            assert_eq!(order.order_hash(), order_hash.to_string());
            assert_eq!(order.order_bytes(), order_bytes.to_string());
            let tx = order.transaction().expect("transaction should be set");
            assert_eq!(
                tx.id().to_lowercase(),
                transaction_hash.to_string().to_lowercase()
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_removed_by_tx_hash_local_db_path() {
            let order_hash =
                b256!("0x0000000000000000000000000000000000000000000000000000000000000def");
            let owner = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let transaction_hash =
                b256!("0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc");
            let order_bytes =
                bytes!("0x00000000000000000000000000000000000000000000000000000000000000ee");
            let orderbook_address = address!("0x2f209e5b67A33B8fE96E28f24628dF6Da301c8eB");

            let local_order = LocalDbOrder {
                chain_id: 137,
                order_hash,
                owner,
                block_timestamp: 789012,
                block_number: 210987,
                orderbook_address,
                order_bytes: order_bytes.clone(),
                transaction_hash,
                inputs: None,
                outputs: None,
                trade_count: 5,
                active: false,
                meta: None,
            };

            let callback = make_local_db_callback(vec![local_order.clone()]);
            let client = new_test_client_with_db_callback(
                vec![get_local_db_test_yaml()],
                callback,
                vec![42161, 137],
            );

            let orders = client
                .get_remove_orders_for_transaction_wasm_binding(
                    137,
                    orderbook_address.to_string(),
                    transaction_hash.to_string(),
                    Some(1),
                    Some(1),
                )
                .await
                .expect("local db query should succeed");

            assert_eq!(orders.len(), 1);
            let order = &orders[0];
            assert_eq!(order.chain_id(), 137);
            assert_eq!(order.order_hash(), order_hash.to_string());
            assert_eq!(order.order_bytes(), order_bytes.to_string());
            assert!(!order.active());
            let tx = order.transaction().expect("transaction should be set");
            assert_eq!(
                tx.id().to_lowercase(),
                transaction_hash.to_string().to_lowercase()
            );
        }
    }
}
