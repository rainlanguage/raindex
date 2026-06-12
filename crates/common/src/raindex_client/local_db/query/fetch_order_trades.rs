use crate::local_db::query::fetch_order_trades::{
    build_fetch_order_trades_batch_stmt, build_fetch_order_trades_stmt, LocalDbOrderTrade,
};
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::RaindexIdentifier;
use alloy::primitives::B256;

pub async fn fetch_order_trades<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    raindex_id: &RaindexIdentifier,
    order_hash: B256,
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<Vec<LocalDbOrderTrade>, LocalDbQueryError> {
    let stmt =
        build_fetch_order_trades_stmt(raindex_id, order_hash, start_timestamp, end_timestamp)?;
    exec.query_json(&stmt).await
}

/// Batched variant of [`fetch_order_trades`]: fetches trades for many order
/// hashes in a single `WHERE order_hash IN (...)` query, avoiding the N+1
/// pattern (and per-query connection overhead) of looping the single-hash
/// fetcher. Returns an empty vec for an empty input without touching the DB.
pub async fn fetch_order_trades_batch<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    raindex_id: &RaindexIdentifier,
    order_hashes: &[B256],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<Vec<LocalDbOrderTrade>, LocalDbQueryError> {
    if order_hashes.is_empty() {
        return Ok(Vec::new());
    }
    let stmt = build_fetch_order_trades_batch_stmt(
        raindex_id,
        order_hashes,
        start_timestamp,
        end_timestamp,
    )?;
    exec.query_json(&stmt).await
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use alloy::primitives::{b256, Address};
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        let chain_id = 111;
        let raindex = Address::from([0x77; 20]);
        let order_hash =
            b256!("0x000000000000000000000000000000000000000000000000000000000000abcd");
        let start = Some(100);
        let end = Some(200);

        let expected_stmt = build_fetch_order_trades_stmt(
            &RaindexIdentifier::new(chain_id, raindex),
            order_hash.clone(),
            start,
            end,
        )
        .unwrap();

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);

        let res = super::fetch_order_trades(
            &exec,
            &RaindexIdentifier::new(chain_id, raindex),
            order_hash,
            start,
            end,
        )
        .await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured.0, expected_stmt.sql);
    }

    #[wasm_bindgen_test]
    async fn batch_wrapper_uses_builder_sql_exactly() {
        let chain_id = 111;
        let raindex = Address::from([0x77; 20]);
        let hash_a = b256!("0x000000000000000000000000000000000000000000000000000000000000abcd");
        let hash_b = b256!("0x000000000000000000000000000000000000000000000000000000000000ef01");
        let start = Some(100);
        let end = Some(200);

        let expected_stmt = build_fetch_order_trades_batch_stmt(
            &RaindexIdentifier::new(chain_id, raindex),
            &[hash_a, hash_b],
            start,
            end,
        )
        .unwrap();

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);

        let res = super::fetch_order_trades_batch(
            &exec,
            &RaindexIdentifier::new(chain_id, raindex),
            &[hash_a, hash_b],
            start,
            end,
        )
        .await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured.0, expected_stmt.sql);
    }

    #[wasm_bindgen_test]
    async fn batch_wrapper_short_circuits_empty_input() {
        let chain_id = 111;
        let raindex = Address::from([0x77; 20]);

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);

        let res = super::fetch_order_trades_batch(
            &exec,
            &RaindexIdentifier::new(chain_id, raindex),
            &[],
            None,
            None,
        )
        .await
        .unwrap();

        // No DB call is made and an empty result is returned.
        assert!(res.is_empty());
        assert_eq!(store.borrow().0, String::new());
    }
}
