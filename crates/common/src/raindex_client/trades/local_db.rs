use super::RaindexTrade;
use crate::local_db::OrderbookIdentifier;
use crate::raindex_client::local_db::query::fetch_owner_trades::fetch_owner_trades;
use crate::raindex_client::local_db::query::fetch_owner_trades_count::fetch_owner_trades_count;
use crate::raindex_client::local_db::query::fetch_trades_by_tx::fetch_trades_by_tx;
use crate::raindex_client::local_db::LocalDb;
use crate::raindex_client::types::{PaginationParams, TimeFilter};
use crate::raindex_client::RaindexError;
use alloy::primitives::{Address, B256};

pub(crate) struct LocalDbTrades<'a> {
    pub(crate) db: &'a LocalDb,
}

impl<'a> LocalDbTrades<'a> {
    pub(crate) fn new(db: &'a LocalDb) -> Self {
        Self { db }
    }

    pub async fn get_by_tx_hash(
        &self,
        ob_id: &OrderbookIdentifier,
        tx_hash: B256,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let local_trades = fetch_trades_by_tx(self.db, ob_id, tx_hash).await?;
        local_trades
            .into_iter()
            .map(|trade| RaindexTrade::try_from_local_db_trade(ob_id.chain_id, trade))
            .collect()
    }

    pub async fn get_by_owner(
        &self,
        ob_id: &OrderbookIdentifier,
        owner: Address,
        pagination: &PaginationParams,
        time_filter: &TimeFilter,
    ) -> Result<Vec<RaindexTrade>, RaindexError> {
        let local_trades =
            fetch_owner_trades(self.db, ob_id, owner, pagination, time_filter).await?;
        local_trades
            .into_iter()
            .map(|trade| RaindexTrade::try_from_local_db_trade(ob_id.chain_id, trade))
            .collect()
    }

    pub async fn count_by_owner(
        &self,
        ob_id: &OrderbookIdentifier,
        owner: Address,
        time_filter: &TimeFilter,
    ) -> Result<u64, RaindexError> {
        Ok(fetch_owner_trades_count(self.db, ob_id, owner, time_filter).await?)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use crate::raindex_client::local_db::executor::JsCallbackExecutor;
        use crate::raindex_client::local_db::LocalDb;
        use alloy::primitives::{address, b256};
        use serde_json::json;
        use wasm_bindgen_test::wasm_bindgen_test;
        use wasm_bindgen_utils::prelude::*;

        fn create_mock_callback(response_json: &str) -> js_sys::Function {
            let json_str = response_json.to_string();
            let result = WasmEncodedResult::Success::<String> {
                value: json_str,
                error: None,
            };
            let payload = js_sys::JSON::stringify(&serde_wasm_bindgen::to_value(&result).unwrap())
                .unwrap()
                .as_string()
                .unwrap();

            let closure =
                Closure::wrap(Box::new(move |_sql: String, _params: JsValue| -> JsValue {
                    js_sys::JSON::parse(&payload).unwrap()
                })
                    as Box<dyn Fn(String, JsValue) -> JsValue>);

            closure.into_js_value().dyn_into().unwrap()
        }

        #[wasm_bindgen_test]
        async fn test_get_by_tx_hash_returns_trades_when_found() {
            let tx_hash =
                b256!("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
            let orderbook = address!("0x2222222222222222222222222222222222222222");
            let order_hash =
                b256!("0x1111111111111111111111111111111111111111111111111111111111111111");
            let input_token = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let output_token = address!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            let sender = address!("0x3333333333333333333333333333333333333333");
            let owner = address!("0x4444444444444444444444444444444444444444");

            let trade_json = json!([{
                "trade_kind": "take",
                "orderbook": orderbook.to_string(),
                "order_hash": order_hash.to_string(),
                "order_owner": owner.to_string(),
                "order_nonce": "1",
                "transaction_hash": tx_hash.to_string(),
                "log_index": 5,
                "block_number": 12345,
                "block_timestamp": 1700000000u64,
                "transaction_sender": sender.to_string(),
                "input_vault_id": "0x01",
                "input_token": input_token.to_string(),
                "input_token_name": "Token A",
                "input_token_symbol": "TKNA",
                "input_token_decimals": 18,
                "input_delta": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "input_running_balance": "0x0000000000000000000000000000000000000000000000000000000000000003",
                "output_vault_id": "0x02",
                "output_token": output_token.to_string(),
                "output_token_name": "Token B",
                "output_token_symbol": "TKNB",
                "output_token_decimals": 6,
                "output_delta": "0x00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
                "output_running_balance": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "trade_id": format!(
                    "0x{}{:016x}",
                    tx_hash.to_string().trim_start_matches("0x"),
                    5u64
                )
            }]);

            let callback = create_mock_callback(&trade_json.to_string());
            let exec = JsCallbackExecutor::from_ref(&callback);
            let local_db = LocalDb::new(exec);

            let trades = LocalDbTrades::new(&local_db);
            let ob_id = OrderbookIdentifier::new(42161, orderbook);

            let result = trades.get_by_tx_hash(&ob_id, tx_hash).await;

            assert!(result.is_ok());
            let trades = result.unwrap();
            assert_eq!(trades.len(), 1);

            let trade = &trades[0];
            assert_eq!(trade.transaction().id(), tx_hash.to_string());
            assert_eq!(trade.orderbook(), orderbook.to_string());
            assert_eq!(
                trade
                    .timestamp()
                    .unwrap()
                    .to_string(10)
                    .unwrap()
                    .as_string()
                    .unwrap(),
                "1700000000"
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_by_tx_hash_returns_empty_when_not_found() {
            let tx_hash =
                b256!("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
            let orderbook = address!("0x2222222222222222222222222222222222222222");

            let callback = create_mock_callback("[]");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let local_db = LocalDb::new(exec);

            let trades = LocalDbTrades::new(&local_db);
            let ob_id = OrderbookIdentifier::new(42161, orderbook);

            let result = trades.get_by_tx_hash(&ob_id, tx_hash).await;

            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[wasm_bindgen_test]
        async fn test_get_by_owner_returns_trades_when_found() {
            let tx_hash =
                b256!("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
            let orderbook = address!("0x2222222222222222222222222222222222222222");
            let order_hash =
                b256!("0x1111111111111111111111111111111111111111111111111111111111111111");
            let input_token = address!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            let output_token = address!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
            let sender = address!("0x3333333333333333333333333333333333333333");
            let owner = address!("0x4444444444444444444444444444444444444444");

            let trade_json = json!([{
                "trade_kind": "take",
                "orderbook": orderbook.to_string(),
                "order_hash": order_hash.to_string(),
                "order_owner": owner.to_string(),
                "order_nonce": "1",
                "transaction_hash": tx_hash.to_string(),
                "log_index": 5,
                "block_number": 12345,
                "block_timestamp": 1700000000u64,
                "transaction_sender": sender.to_string(),
                "input_vault_id": "0x01",
                "input_token": input_token.to_string(),
                "input_token_name": "Token A",
                "input_token_symbol": "TKNA",
                "input_token_decimals": 18,
                "input_delta": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "input_running_balance": "0x0000000000000000000000000000000000000000000000000000000000000003",
                "output_vault_id": "0x02",
                "output_token": output_token.to_string(),
                "output_token_name": "Token B",
                "output_token_symbol": "TKNB",
                "output_token_decimals": 6,
                "output_delta": "0x00000000fffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
                "output_running_balance": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "trade_id": format!(
                    "0x{}{:016x}",
                    tx_hash.to_string().trim_start_matches("0x"),
                    5u64
                )
            }]);

            let callback = create_mock_callback(&trade_json.to_string());
            let exec = JsCallbackExecutor::from_ref(&callback);
            let local_db = LocalDb::new(exec);

            let trades = LocalDbTrades::new(&local_db);
            let ob_id = OrderbookIdentifier::new(42161, orderbook);

            let result = trades
                .get_by_owner(
                    &ob_id,
                    owner,
                    &PaginationParams::default(),
                    &TimeFilter::default(),
                )
                .await;

            assert!(result.is_ok());
            let trades = result.unwrap();
            assert_eq!(trades.len(), 1);

            let trade = &trades[0];
            assert_eq!(trade.orderbook(), orderbook.to_string());
            assert_eq!(
                trade
                    .timestamp()
                    .unwrap()
                    .to_string(10)
                    .unwrap()
                    .as_string()
                    .unwrap(),
                "1700000000"
            );
        }

        #[wasm_bindgen_test]
        async fn test_get_by_owner_returns_empty_when_not_found() {
            let orderbook = address!("0x2222222222222222222222222222222222222222");
            let owner = address!("0x4444444444444444444444444444444444444444");

            let callback = create_mock_callback("[]");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let local_db = LocalDb::new(exec);

            let trades = LocalDbTrades::new(&local_db);
            let ob_id = OrderbookIdentifier::new(42161, orderbook);

            let result = trades
                .get_by_owner(
                    &ob_id,
                    owner,
                    &PaginationParams::default(),
                    &TimeFilter::default(),
                )
                .await;

            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[wasm_bindgen_test]
        async fn test_count_by_owner_returns_count() {
            let orderbook = address!("0x2222222222222222222222222222222222222222");
            let owner = address!("0x4444444444444444444444444444444444444444");

            let callback = create_mock_callback("[{\"trade_count\":7}]");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let local_db = LocalDb::new(exec);

            let trades = LocalDbTrades::new(&local_db);
            let ob_id = OrderbookIdentifier::new(42161, orderbook);

            let result = trades
                .count_by_owner(&ob_id, owner, &TimeFilter::default())
                .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 7);
        }
    }
}
