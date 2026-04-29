use crate::local_db::query::clear_tables::clear_tables_stmt;
use crate::local_db::query::{LocalDbQueryError, LocalDbQueryExecutor};

pub async fn clear_tables<E: LocalDbQueryExecutor + ?Sized>(
    exec: &E,
) -> Result<(), LocalDbQueryError> {
    let stmt = clear_tables_stmt();
    exec.query_text(&stmt).await.map(|_| ())
}

#[cfg(target_family = "wasm")]
mod wasm {
    use super::*;
    use crate::local_db::LocalDbError;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use wasm_bindgen_utils::{prelude::*, wasm_export};

    #[wasm_export(js_name = "clearTables", unchecked_return_type = "void")]
    pub async fn clear_tables_wasm(
        #[wasm_export(param_description = "JavaScript function to execute database queries")]
        db_callback: js_sys::Function,
    ) -> Result<(), LocalDbError> {
        let exec = JsCallbackExecutor::from_ref(&db_callback);
        clear_tables(&exec).await.map_err(LocalDbError::from)
    }
}

#[cfg(all(test, target_family = "wasm"))]
mod wasm_tests {
    use super::*;
    use crate::raindex_client::local_db::executor::tests::create_sql_capturing_callback;
    use crate::raindex_client::local_db::executor::JsCallbackExecutor;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;
    use wasm_bindgen_utils::prelude::JsValue;

    #[wasm_bindgen_test]
    async fn wrapper_uses_raw_sql_exactly() {
        let expected_sql = clear_tables_stmt();
        let store = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
        let callback = create_sql_capturing_callback("OK", store.clone());
        let exec = JsCallbackExecutor::from_ref(&callback);
        let res = super::clear_tables(&exec).await;
        assert!(res.is_ok());
        assert_eq!(store.borrow().clone().0, expected_sql.sql);
    }
}
