use super::*;
use crate::local_db::query::{
    FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch, SqlValue,
};
use async_trait::async_trait;
use futures::lock::Mutex;
use js_sys::{Array, BigInt};
use std::rc::Rc;
use wasm_bindgen_utils::prelude::wasm_bindgen_futures::JsFuture;
use wasm_bindgen_utils::result::WasmEncodedResult;

#[derive(Clone)]
pub struct JsCallbackExecutor {
    query_callback: js_sys::Function,
    wipe_callback: Option<js_sys::Function>,
    serialize: Rc<Mutex<()>>,
}

impl JsCallbackExecutor {
    pub fn new(query_callback: js_sys::Function, wipe_callback: Option<js_sys::Function>) -> Self {
        Self {
            query_callback,
            wipe_callback,
            serialize: Rc::new(Mutex::new(())),
        }
    }

    pub fn from_ref(query_callback: &js_sys::Function) -> Self {
        Self::new(query_callback.clone(), None)
    }

    fn function(&self) -> &js_sys::Function {
        &self.query_callback
    }

    async fn invoke_statement_unlocked(
        &self,
        stmt: &SqlStatement,
    ) -> Result<String, LocalDbQueryError> {
        // If there are no parameters, pass `undefined` to the JS callback
        // instead of an empty array to match the SDK's expected semantics.
        let js_params_val = if stmt.params.is_empty() {
            JsValue::UNDEFINED
        } else {
            let array = Array::new();
            for param in stmt.params() {
                let js_param = match param {
                    SqlValue::Text(text) => JsValue::from_str(text),
                    SqlValue::I64(value) => JsValue::from(BigInt::from(*value)),
                    SqlValue::U64(value) => JsValue::from(BigInt::from(*value)),
                    SqlValue::Null => JsValue::NULL,
                };
                array.push(&js_param);
            }
            JsValue::from(array)
        };

        let result = self
            .function()
            .call2(
                &JsValue::NULL,
                &JsValue::from_str(&stmt.sql),
                &js_params_val,
            )
            .map_err(|e| {
                LocalDbQueryError::database(format!(
                    "JavaScript callback invocation failed: {:?}",
                    e
                ))
            })?;

        let promise = js_sys::Promise::resolve(&result);
        let future = JsFuture::from(promise);
        let js_result = future.await.map_err(|e| {
            LocalDbQueryError::database(format!("Promise resolution failed: {:?}", e))
        })?;

        let wasm_result: WasmEncodedResult<String> = serde_wasm_bindgen::from_value(js_result)
            .map_err(|_| LocalDbQueryError::invalid_response())?;

        match wasm_result {
            WasmEncodedResult::Success { value, .. } => Ok(value),
            WasmEncodedResult::Err { error, .. } => {
                Err(LocalDbQueryError::database(error.readable_msg))
            }
        }
    }

    async fn invoke_statement(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
        let _guard = self.serialize.lock().await;
        self.invoke_statement_unlocked(stmt).await
    }
}

// SAFETY: WASM builds run on a single thread; the wrapped JavaScript callback is only invoked on
// that thread, so sharing the executor across async tasks is safe.
unsafe impl Sync for JsCallbackExecutor {}

#[cfg(target_family = "wasm")]
#[async_trait(?Send)]
impl LocalDbQueryExecutor for JsCallbackExecutor {
    async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
        let _guard = self.serialize.lock().await;
        if !batch.is_transaction() {
            return Err(LocalDbQueryError::database(
                "SQL statement batch must be wrapped in a transaction",
            ));
        }

        for stmt in batch {
            if let Err(err) = self.invoke_statement_unlocked(stmt).await {
                let rollback_stmt = SqlStatement::new("ROLLBACK");
                let _ = self.invoke_statement_unlocked(&rollback_stmt).await;
                return Err(err);
            }
        }
        Ok(())
    }

    async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
        self.invoke_statement(stmt).await
    }

    async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson,
    {
        let value = self.query_text(stmt).await?;
        serde_json::from_str(&value)
            .map_err(|err| LocalDbQueryError::deserialization(err.to_string()))
    }

    async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
        let _guard = self.serialize.lock().await;
        let wipe_callback = self.wipe_callback.as_ref().ok_or_else(|| {
            LocalDbQueryError::database("wipe_and_recreate callback not configured")
        })?;

        let result = wipe_callback.call0(&JsValue::NULL).map_err(|e| {
            LocalDbQueryError::database(format!(
                "JavaScript wipe callback invocation failed: {:?}",
                e
            ))
        })?;

        let promise = js_sys::Promise::resolve(&result);
        let future = JsFuture::from(promise);
        let js_result = future.await.map_err(|e| {
            LocalDbQueryError::database(format!("Wipe promise resolution failed: {:?}", e))
        })?;

        let wasm_result: WasmEncodedResult<()> = serde_wasm_bindgen::from_value(js_result)
            .map_err(|_| LocalDbQueryError::invalid_response())?;

        match wasm_result {
            WasmEncodedResult::Success { .. } => Ok(()),
            WasmEncodedResult::Err { error, .. } => {
                Err(LocalDbQueryError::database(error.readable_msg))
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    #[cfg(target_family = "wasm")]
    use super::*;

    #[cfg(target_family = "wasm")]
    pub use wasm_tests::create_sql_capturing_callback;

    #[cfg(target_family = "wasm")]
    mod wasm_tests {
        use super::*;
        use js_sys::Function;
        use serde::{Deserialize, Serialize};
        use wasm_bindgen::JsCast;
        use wasm_bindgen_test::*;
        use wasm_bindgen_utils::prelude::serde_wasm_bindgen;
        use wasm_bindgen_utils::prelude::JsValue;
        use wasm_bindgen_utils::result::{WasmEncodedError, WasmEncodedResult};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TestData {
            id: u32,
            name: String,
        }

        pub fn create_success_callback(response: &str) -> Function {
            let success = WasmEncodedResult::Success::<String> {
                value: response.to_string(),
                error: None,
            };
            let js_value = serde_wasm_bindgen::to_value(&success).unwrap();
            Function::new_no_args(&format!(
                "return {}",
                js_sys::JSON::stringify(&js_value)
                    .unwrap()
                    .as_string()
                    .unwrap()
            ))
        }

        pub fn create_sql_capturing_callback(
            response: &str,
            store: std::rc::Rc<std::cell::RefCell<(String, JsValue)>>,
        ) -> Function {
            use wasm_bindgen::prelude::Closure;

            let response = response.to_string();
            let store_clone = store.clone();
            let closure = Closure::wrap(Box::new(
                move |sql: String, params: JsValue| -> wasm_bindgen::JsValue {
                    *store_clone.borrow_mut() = (sql, params);
                    let result = WasmEncodedResult::Success::<String> {
                        value: response.clone(),
                        error: None,
                    };
                    serde_wasm_bindgen::to_value(&result).unwrap()
                },
            )
                as Box<dyn FnMut(String, JsValue) -> wasm_bindgen::JsValue>);

            let func: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();
            func
        }

        #[wasm_bindgen_test]
        async fn test_query_json_success_case() {
            let test_data = vec![
                TestData {
                    id: 1,
                    name: "Alice".to_string(),
                },
                TestData {
                    id: 2,
                    name: "Bob".to_string(),
                },
            ];
            let json_data = serde_json::to_string(&test_data).unwrap();
            let callback = create_success_callback(&json_data);
            let exec = JsCallbackExecutor::from_ref(&callback);

            let result: Result<Vec<TestData>, LocalDbQueryError> = exec
                .query_json(&SqlStatement::new("SELECT * FROM users"))
                .await;
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(data[0].name, "Alice");
            assert_eq!(data[1].name, "Bob");
        }

        #[wasm_bindgen_test]
        async fn test_query_text_success() {
            let callback = create_success_callback("text-result");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let val = exec
                .query_text(&SqlStatement::new("SELECT 1"))
                .await
                .unwrap();
            assert_eq!(val, "text-result");
        }

        #[wasm_bindgen_test]
        async fn passes_undefined_params_when_empty() {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::JsValue;

            let store = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let callback = create_sql_capturing_callback("OK", store.clone());
            let exec = JsCallbackExecutor::from_ref(&callback);

            let _ = exec
                .query_text(&SqlStatement::new("SELECT 42"))
                .await
                .unwrap();

            let (_, captured_params) = store.borrow().clone();
            assert!(captured_params.is_undefined());
        }

        #[wasm_bindgen_test]
        async fn passes_array_params_when_non_empty() {
            use js_sys::Array;
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::JsValue;

            let store = Rc::new(RefCell::new((String::new(), JsValue::UNDEFINED)));
            let callback = create_sql_capturing_callback("OK", store.clone());
            let exec = JsCallbackExecutor::from_ref(&callback);

            // Build a statement with parameters
            let mut stmt = SqlStatement::new("SELECT ?1, ?2");
            let _ = stmt.push(123i64);
            let _ = stmt.push("abc");

            let _ = exec.query_text(&stmt).await.unwrap();

            let (_, captured_params) = store.borrow().clone();

            // Ensure non-empty params are passed as a JavaScript Array
            assert!(Array::is_array(&captured_params));

            // Decode and assert expected contents and length
            let decoded = Array::from(&captured_params);
            assert_eq!(decoded.length(), 2);

            let first = decoded.get(0).dyn_into::<BigInt>().unwrap();
            assert_eq!(first.to_string(10).unwrap().as_string().unwrap(), "123");

            let second = decoded.get(1);
            assert_eq!(
                second.as_string().unwrap(),
                "abc",
                "expected text param in position 2"
            );
        }

        #[wasm_bindgen_test]
        async fn execute_batch_invokes_all_statements_in_order() {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::prelude::Closure;

            let calls: Rc<RefCell<Vec<(String, JsValue)>>> = Rc::new(RefCell::new(Vec::new()));
            let calls_clone = calls.clone();
            let closure = Closure::wrap(Box::new(move |sql: String, params: JsValue| -> JsValue {
                calls_clone.borrow_mut().push((sql, params.clone()));
                let result = WasmEncodedResult::Success::<String> {
                    value: String::new(),
                    error: None,
                };
                serde_wasm_bindgen::to_value(&result).unwrap()
            })
                as Box<dyn FnMut(String, JsValue) -> JsValue>);

            let callback: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let exec = JsCallbackExecutor::from_ref(&callback);

            let mut batch = SqlStatementBatch::new();
            batch.add(SqlStatement::new("CREATE TABLE example (val INTEGER)"));
            let mut insert = SqlStatement::new("INSERT INTO example (val) VALUES (?1)");
            insert.push(42i64);
            batch.add(insert);
            batch.add(SqlStatement::new("DELETE FROM example WHERE val = 0"));

            let batch = batch.ensure_transaction();

            exec.execute_batch(&batch).await.unwrap();

            let calls = calls.borrow();
            assert_eq!(calls.len(), 5);

            assert_eq!(calls[0].0, "BEGIN TRANSACTION");
            assert!(calls[0].1.is_undefined());

            assert_eq!(calls[1].0, "CREATE TABLE example (val INTEGER)");
            assert!(calls[1].1.is_undefined());

            assert_eq!(calls[2].0, "INSERT INTO example (val) VALUES (?1)");
            let params_value = calls[2].1.clone();

            assert_eq!(calls[3].0, "DELETE FROM example WHERE val = 0");
            assert!(calls[3].1.is_undefined());

            assert_eq!(calls[4].0, "COMMIT");
            assert!(calls[4].1.is_undefined());
            drop(calls);

            assert!(Array::is_array(&params_value));
            let decoded = Array::from(&params_value);
            assert_eq!(decoded.length(), 1);
            let first = decoded.get(0).dyn_into::<BigInt>().unwrap();
            assert_eq!(first.to_string(10).unwrap().as_string().unwrap(), "42");
        }

        #[wasm_bindgen_test]
        async fn execute_batch_rolls_back_on_failure_without_params() {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::prelude::Closure;
            use wasm_bindgen_utils::prelude::JsValue;

            let calls: Rc<RefCell<Vec<(String, JsValue)>>> = Rc::new(RefCell::new(Vec::new()));
            let calls_clone = calls.clone();
            let closure = Closure::wrap(Box::new(move |sql: String, params: JsValue| -> JsValue {
                calls_clone.borrow_mut().push((sql.clone(), params));
                let result: WasmEncodedResult<String> =
                    if sql == "INSERT INTO rollback_test (value) VALUES ('fail')" {
                        WasmEncodedResult::Err {
                            value: None,
                            error: WasmEncodedError {
                                msg: "boom".to_string(),
                                readable_msg: "boom".to_string(),
                            },
                        }
                    } else {
                        WasmEncodedResult::Success {
                            value: String::new(),
                            error: None,
                        }
                    };
                serde_wasm_bindgen::to_value(&result).unwrap()
            })
                as Box<dyn FnMut(String, JsValue) -> JsValue>);
            let callback: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let exec = JsCallbackExecutor::from_ref(&callback);

            let mut batch = SqlStatementBatch::new();
            batch.add(SqlStatement::new("CREATE TABLE rollback_test (value TEXT)"));
            batch.add(SqlStatement::new(
                "INSERT INTO rollback_test (value) VALUES ('ok')",
            ));
            batch.add(SqlStatement::new(
                "INSERT INTO rollback_test (value) VALUES ('fail')",
            ));
            let batch = batch.ensure_transaction();

            let err = exec.execute_batch(&batch).await.unwrap_err();
            assert!(matches!(err, LocalDbQueryError::Database { .. }));

            let calls = calls.borrow();
            assert_eq!(calls.len(), 5);
            assert_eq!(calls[0].0, "BEGIN TRANSACTION");
            assert_eq!(calls[1].0, "CREATE TABLE rollback_test (value TEXT)");
            assert_eq!(
                calls[2].0,
                "INSERT INTO rollback_test (value) VALUES ('ok')"
            );
            assert_eq!(
                calls[3].0,
                "INSERT INTO rollback_test (value) VALUES ('fail')"
            );
            assert_eq!(calls[4].0, "ROLLBACK");
            assert!(!calls.iter().any(|(sql, _)| sql == "COMMIT"));
        }

        #[wasm_bindgen_test]
        async fn execute_batch_rolls_back_on_failure_with_params() {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::prelude::Closure;
            use wasm_bindgen_utils::prelude::JsValue;

            let calls: Rc<RefCell<Vec<(String, JsValue)>>> = Rc::new(RefCell::new(Vec::new()));
            let calls_clone = calls.clone();
            let seen_insert = Rc::new(RefCell::new(false));
            let seen_insert_clone = seen_insert.clone();
            let closure = Closure::wrap(Box::new(move |sql: String, params: JsValue| -> JsValue {
                calls_clone.borrow_mut().push((sql.clone(), params.clone()));
                let result: WasmEncodedResult<String> =
                    if sql == "INSERT INTO rollback_param (id, value) VALUES (?1, ?2)" {
                        let mut seen = seen_insert_clone.borrow_mut();
                        if !*seen {
                            *seen = true;
                            WasmEncodedResult::Success {
                                value: String::new(),
                                error: None,
                            }
                        } else {
                            WasmEncodedResult::Err {
                                value: None,
                                error: WasmEncodedError {
                                    msg: "boom".to_string(),
                                    readable_msg: "boom".to_string(),
                                },
                            }
                        }
                    } else {
                        WasmEncodedResult::Success {
                            value: String::new(),
                            error: None,
                        }
                    };
                serde_wasm_bindgen::to_value(&result).unwrap()
            })
                as Box<dyn FnMut(String, JsValue) -> JsValue>);
            let callback: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let exec = JsCallbackExecutor::from_ref(&callback);

            let mut batch = SqlStatementBatch::new();
            batch.add(SqlStatement::new(
                "CREATE TABLE rollback_param (id INTEGER PRIMARY KEY, value TEXT)",
            ));

            let mut insert_ok =
                SqlStatement::new("INSERT INTO rollback_param (id, value) VALUES (?1, ?2)");
            insert_ok.push(1i64);
            insert_ok.push("ok");
            batch.add(insert_ok);

            let mut insert_fail =
                SqlStatement::new("INSERT INTO rollback_param (id, value) VALUES (?1, ?2)");
            insert_fail.push(2i64);
            insert_fail.push("fail");
            batch.add(insert_fail);

            let batch = batch.ensure_transaction();

            let err = exec.execute_batch(&batch).await.unwrap_err();
            assert!(matches!(err, LocalDbQueryError::Database { .. }));

            let calls = calls.borrow();
            assert_eq!(calls.len(), 5);
            assert_eq!(calls[0].0, "BEGIN TRANSACTION");
            assert_eq!(
                calls[1].0,
                "CREATE TABLE rollback_param (id INTEGER PRIMARY KEY, value TEXT)"
            );
            assert_eq!(
                calls[2].0,
                "INSERT INTO rollback_param (id, value) VALUES (?1, ?2)"
            );
            assert!(Array::is_array(&calls[2].1));
            let params_ok = Array::from(&calls[2].1);
            assert_eq!(params_ok.length(), 2);
            let first = params_ok.get(0).dyn_into::<BigInt>().unwrap();
            assert_eq!(first.to_string(10).unwrap().as_string().unwrap(), "1");
            let second = params_ok.get(1);
            assert_eq!(second.as_string().unwrap(), "ok");
            assert_eq!(
                calls[3].0,
                "INSERT INTO rollback_param (id, value) VALUES (?1, ?2)"
            );
            assert!(Array::is_array(&calls[3].1));
            let params_fail = Array::from(&calls[3].1);
            assert_eq!(params_fail.length(), 2);
            let first = params_fail.get(0).dyn_into::<BigInt>().unwrap();
            assert_eq!(first.to_string(10).unwrap().as_string().unwrap(), "2");
            let second = params_fail.get(1);
            assert_eq!(second.as_string().unwrap(), "fail");
            assert_eq!(calls[4].0, "ROLLBACK");
            assert!(!calls.iter().any(|(sql, _)| sql == "COMMIT"));

            assert!(*seen_insert.borrow());
        }

        #[wasm_bindgen_test]
        async fn execute_batch_rejects_non_transactions() {
            let callback = create_success_callback("");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let batch = SqlStatementBatch::from(vec![SqlStatement::new("SELECT 1")]);

            let err = exec.execute_batch(&batch).await.unwrap_err();
            assert!(matches!(err, LocalDbQueryError::Database { .. }));
        }

        #[wasm_bindgen_test]
        async fn test_callback_throws() {
            // callback that throws synchronously
            let callback = Function::new_with_args("sql, params", "throw new Error('boom')");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let err = exec
                .query_text(&SqlStatement::new("SELECT 1"))
                .await
                .err()
                .unwrap();
            match err {
                LocalDbQueryError::Database { .. } => {}
                other => panic!("unexpected error variant: {:?}", other),
            }
        }

        #[wasm_bindgen_test]
        async fn test_promise_rejects() {
            // callback returns a rejected Promise
            let callback =
                Function::new_with_args("sql, params", "return Promise.reject('rejected')");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let err = exec
                .query_text(&SqlStatement::new("SELECT 1"))
                .await
                .err()
                .unwrap();
            match err {
                LocalDbQueryError::Database { .. } => {}
                other => panic!("unexpected error variant: {:?}", other),
            }
        }

        #[wasm_bindgen_test]
        async fn test_invalid_wrapper_yields_invalid_response() {
            // returns a plain string instead of WasmEncodedResult
            let callback = Function::new_with_args("sql, params", "return 'not-a-wrapper'");
            let exec = JsCallbackExecutor::from_ref(&callback);
            let res: Result<Vec<TestData>, LocalDbQueryError> =
                exec.query_json(&SqlStatement::new("SELECT 1")).await;
            assert!(matches!(res, Err(LocalDbQueryError::InvalidResponse)));
        }

        #[wasm_bindgen_test]
        async fn test_deserialization_error() {
            // Success wrapper but invalid JSON payload
            use wasm_bindgen_utils::result::WasmEncodedResult;
            let store =
                std::rc::Rc::new(std::cell::RefCell::new((String::new(), JsValue::UNDEFINED)));
            let store_clone = store.clone();
            let closure = wasm_bindgen::prelude::Closure::wrap(Box::new(
                move |sql: String, params: JsValue| -> JsValue {
                    *store_clone.borrow_mut() = (sql, params);
                    let result: WasmEncodedResult<String> = WasmEncodedResult::Success {
                        value: "not-json".to_string(),
                        error: None,
                    };
                    serde_wasm_bindgen::to_value(&result).unwrap()
                },
            )
                as Box<dyn FnMut(String, JsValue) -> JsValue>);
            let callback: Function = closure.as_ref().clone().unchecked_into();
            closure.forget();

            let exec = JsCallbackExecutor::from_ref(&callback);
            let res: Result<Vec<TestData>, LocalDbQueryError> =
                exec.query_json(&SqlStatement::new("SELECT 1")).await;
            assert!(matches!(
                res,
                Err(LocalDbQueryError::Deserialization { .. })
            ));
        }

        #[wasm_bindgen_test]
        async fn wipe_and_recreate_returns_error_when_no_callback() {
            let callback = create_success_callback("[]");
            let exec = JsCallbackExecutor::from_ref(&callback);

            let result = exec.wipe_and_recreate().await;
            assert!(matches!(result, Err(LocalDbQueryError::Database { .. })));
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("wipe_and_recreate callback not configured"));
        }

        #[wasm_bindgen_test]
        async fn wipe_and_recreate_calls_wipe_callback() {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::prelude::Closure;

            let wipe_called = Rc::new(RefCell::new(false));
            let wipe_called_clone = wipe_called.clone();
            let wipe_closure = Closure::wrap(Box::new(move || -> JsValue {
                *wipe_called_clone.borrow_mut() = true;
                let result = WasmEncodedResult::Success::<()> {
                    value: (),
                    error: None,
                };
                serde_wasm_bindgen::to_value(&result).unwrap()
            }) as Box<dyn FnMut() -> JsValue>);
            let wipe_callback: Function = wipe_closure.as_ref().clone().unchecked_into();
            wipe_closure.forget();

            let callback = create_success_callback("[]");
            let exec = JsCallbackExecutor::new(callback, Some(wipe_callback));

            exec.wipe_and_recreate().await.unwrap();

            assert!(
                *wipe_called.borrow(),
                "wipe callback should have been called"
            );
        }

        #[wasm_bindgen_test]
        async fn wipe_and_recreate_propagates_callback_error() {
            let wipe_callback = Function::new_no_args(&{
                let error = WasmEncodedResult::Err::<()> {
                    value: None,
                    error: WasmEncodedError {
                        msg: "wipe failed".to_string(),
                        readable_msg: "wipe failed readable".to_string(),
                    },
                };
                let js_value = serde_wasm_bindgen::to_value(&error).unwrap();
                format!(
                    "return {}",
                    js_sys::JSON::stringify(&js_value)
                        .unwrap()
                        .as_string()
                        .unwrap()
                )
            });

            let callback = create_success_callback("[]");
            let exec = JsCallbackExecutor::new(callback, Some(wipe_callback));

            let result = exec.wipe_and_recreate().await;
            assert!(matches!(result, Err(LocalDbQueryError::Database { .. })));
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("wipe failed readable"));
        }
    }
}
