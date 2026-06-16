use crate::local_db::query::fetch_order_trades::LocalDbOrderTrade;
use crate::local_db::query::fetch_owner_trades::build_fetch_owner_trades_stmt;
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};
use crate::local_db::OrderbookIdentifier;
use crate::raindex_client::types::{PaginationParams, TimeFilter};
use alloy::primitives::Address;

pub async fn fetch_owner_trades<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
    ob_id: &OrderbookIdentifier,
    owner: Address,
    pagination: &PaginationParams,
    time_filter: &TimeFilter,
) -> Result<Vec<LocalDbOrderTrade>, LocalDbQueryError> {
    let stmt = build_fetch_owner_trades_stmt(ob_id, owner, pagination, time_filter)?;
    exec.query_json(&stmt).await
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use alloy::primitives::Address;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::*;

    #[wasm_bindgen_test]
    async fn wrapper_uses_builder_sql_exactly() {
        use crate::raindex_client::types::{PaginationParams, TimeFilter};

        let chain_id = 111;
        let orderbook = Address::from([0x77; 20]);
        let owner = Address::from([0x88; 20]);

        let pagination = PaginationParams {
            page: Some(2),
            ..Default::default()
        };

        let expected_stmt = build_fetch_owner_trades_stmt(
            &OrderbookIdentifier::new(chain_id, orderbook),
            owner,
            &pagination,
            &TimeFilter::default(),
        )
        .unwrap();

        let store = Rc::new(RefCell::new((
            String::new(),
            wasm_bindgen::JsValue::UNDEFINED,
        )));
        let callback = create_sql_capturing_callback("[]", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);

        let res = super::fetch_owner_trades(
            &exec,
            &OrderbookIdentifier::new(chain_id, orderbook),
            owner,
            &pagination,
            &TimeFilter::default(),
        )
        .await;
        assert!(res.is_ok());

        let captured = store.borrow().clone();
        assert_eq!(captured.0, expected_stmt.sql);
    }
}
