use super::functions;
use super::query::{
    FromDbJson, LocalDbQueryError, LocalDbQueryExecutor, SqlStatement, SqlStatementBatch, SqlValue,
};
use async_trait::async_trait;
use rusqlite::{types::ValueRef, Connection};
use serde_json::{json, Map, Value};
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::task::spawn_blocking;

pub struct RusqliteExecutor {
    db_path: PathBuf,
}

fn sqlvalue_to_rusqlite(v: SqlValue) -> rusqlite::types::Value {
    match v {
        SqlValue::Text(t) => rusqlite::types::Value::Text(t),
        SqlValue::I64(i) => rusqlite::types::Value::Integer(i),
        SqlValue::U64(u) => match i64::try_from(u) {
            Ok(i) => rusqlite::types::Value::Integer(i),
            Err(_) => rusqlite::types::Value::Text(u.to_string()),
        },
        SqlValue::Null => rusqlite::types::Value::Null,
    }
}

impl RusqliteExecutor {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
        }
    }

    fn invoke_statement(conn: &Connection, stmt: &SqlStatement) -> Result<(), LocalDbQueryError> {
        if stmt.params().is_empty() {
            conn.execute_batch(stmt.sql())
                .map_err(|e| LocalDbQueryError::database(format!("SQL execution failed: {e}")))?;
        } else {
            let mut prepared = conn.prepare(stmt.sql()).map_err(|e| {
                LocalDbQueryError::database(format!("Failed to prepare query: {e}"))
            })?;
            let bound = stmt.params().iter().cloned().map(sqlvalue_to_rusqlite);
            let params = rusqlite::params_from_iter(bound);
            prepared
                .execute(params)
                .map_err(|e| LocalDbQueryError::database(format!("SQL execution failed: {e}")))?;
        }
        Ok(())
    }
}

fn open_connection(db_path: &Path) -> Result<Connection, LocalDbQueryError> {
    let conn = Connection::open(db_path)
        .map_err(|e| LocalDbQueryError::database(format!("Failed to open database: {e}")))?;
    conn.pragma_update(None, "journal_mode", "wal")
        .map_err(|e| LocalDbQueryError::database(format!("Failed to set WAL journal mode: {e}")))?;
    conn.busy_timeout(Duration::from_millis(500))
        .map_err(|e| LocalDbQueryError::database(format!("Failed to set busy_timeout: {e}")))?;
    functions::register_all(&conn).map_err(|e| {
        LocalDbQueryError::database(format!("Failed to register sqlite functions: {e}"))
    })?;
    Ok(conn)
}

fn join_err(err: tokio::task::JoinError) -> LocalDbQueryError {
    LocalDbQueryError::database(format!("Blocking task failed: {err}"))
}

#[cfg_attr(target_family = "wasm", async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait)]
impl LocalDbQueryExecutor for RusqliteExecutor {
    async fn execute_batch(&self, batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
        if !batch.is_transaction() {
            return Err(LocalDbQueryError::database(
                "SQL statement batch must be wrapped in a transaction",
            ));
        }

        let db_path = self.db_path.clone();
        let batch = batch.clone();
        spawn_blocking(move || {
            let conn = open_connection(&db_path)?;
            for stmt in &batch {
                if let Err(err) = RusqliteExecutor::invoke_statement(&conn, stmt) {
                    let _ = conn.execute_batch("ROLLBACK");
                    return Err(err);
                }
            }
            Ok(())
        })
        .await
        .map_err(join_err)?
    }

    async fn query_text(&self, stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
        let db_path = self.db_path.clone();
        let stmt = stmt.clone();
        spawn_blocking(move || {
            let conn = open_connection(&db_path)?;
            RusqliteExecutor::invoke_statement(&conn, &stmt)?;
            Ok(String::new())
        })
        .await
        .map_err(join_err)?
    }

    async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
    where
        T: FromDbJson,
    {
        let db_path = self.db_path.clone();
        let stmt = stmt.clone();

        let json_value = spawn_blocking(move || {
            let conn = open_connection(&db_path)?;
            let mut s = conn.prepare(stmt.sql()).map_err(|e| {
                LocalDbQueryError::database(format!("Failed to prepare query: {e}"))
            })?;
            let column_names: Vec<String> = (0..s.column_count())
                .map(|i| {
                    let raw = s.column_name(i).unwrap_or("");
                    let trimmed = raw.trim();
                    if trimmed.is_empty() {
                        format!("column_{}", i)
                    } else {
                        trimmed.to_string()
                    }
                })
                .collect();

            let bound = stmt.params().iter().cloned().map(sqlvalue_to_rusqlite);
            let params = rusqlite::params_from_iter(bound);

            let rows_iter = s
                .query_map(params, |row| {
                    let mut obj = Map::with_capacity(column_names.len());
                    for (i, name) in column_names.iter().enumerate() {
                        let v = match row.get_ref(i)? {
                            ValueRef::Null => Value::Null,
                            ValueRef::Integer(n) => json!(n),
                            ValueRef::Real(f) => json!(f),
                            ValueRef::Text(bytes) => match std::str::from_utf8(bytes) {
                                Ok(s) => json!(s),
                                Err(_) => json!(alloy::hex::encode_prefixed(bytes)),
                            },
                            ValueRef::Blob(bytes) => json!(alloy::hex::encode_prefixed(bytes)),
                        };
                        obj.insert(name.clone(), v);
                    }
                    Ok(Value::Object(obj))
                })
                .map_err(|e| LocalDbQueryError::database(format!("Query failed: {e}")))?;

            let mut out: Vec<Value> = Vec::new();
            for r in rows_iter {
                let v = r.map_err(|e| LocalDbQueryError::database(format!("Row error: {e}")))?;
                out.push(v);
            }

            Ok::<_, LocalDbQueryError>(Value::Array(out))
        })
        .await
        .map_err(join_err)??;

        serde_json::from_value::<T>(json_value)
            .map_err(|e| LocalDbQueryError::deserialization(e.to_string()))
    }

    async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
        let db_path = self.db_path.clone();
        spawn_blocking(move || {
            if db_path.exists() {
                std::fs::remove_file(&db_path).map_err(|e| {
                    LocalDbQueryError::database(format!("Failed to delete database file: {e}"))
                })?;
            }
            let conn = open_connection(&db_path)?;
            drop(conn);
            Ok(())
        })
        .await
        .map_err(join_err)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn execute_batch_runs_all_statements() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("batch.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);

        let mut batch = SqlStatementBatch::new();
        batch.add(SqlStatement::new(
            "CREATE TABLE widgets (name TEXT, qty INTEGER);",
        ));

        let mut param_insert =
            SqlStatement::new("INSERT INTO widgets (name, qty) VALUES (?1, ?2);");
        param_insert.push("widget-a");
        param_insert.push(5i64);
        batch.add(param_insert);

        batch.add(SqlStatement::new(
            "INSERT INTO widgets (name, qty) VALUES ('widget-b', 7);",
        ));

        let batch = batch.ensure_transaction();

        exec.execute_batch(&batch).await.unwrap();

        #[derive(serde::Deserialize)]
        struct CountRow {
            total: i64,
        }
        let count_rows: Vec<CountRow> = exec
            .query_json(&SqlStatement::new("SELECT COUNT(*) AS total FROM widgets;"))
            .await
            .unwrap();
        assert_eq!(count_rows[0].total, 2);

        #[derive(serde::Deserialize)]
        struct WidgetRow {
            qty: i64,
        }
        let widget_rows: Vec<WidgetRow> = exec
            .query_json(&SqlStatement::new(
                "SELECT qty FROM widgets WHERE name = 'widget-a';",
            ))
            .await
            .unwrap();
        assert_eq!(widget_rows[0].qty, 5);
    }

    #[tokio::test]
    async fn execute_and_query_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(
            "CREATE TABLE numbers (n INTEGER); INSERT INTO numbers (n) VALUES (1), (2);",
        ))
        .await
        .unwrap();

        #[derive(serde::Deserialize)]
        struct CountRow {
            c: i64,
        }
        let rows: Vec<CountRow> = exec
            .query_json(&SqlStatement::new("SELECT COUNT(*) AS c FROM numbers;"))
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].c, 2);
    }

    #[tokio::test]
    async fn detects_required_tables() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("schema.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(
            "CREATE TABLE foo (id INTEGER); CREATE TABLE bar (id INTEGER);",
        ))
        .await
        .unwrap();

        #[derive(serde::Deserialize)]
        struct TableNameRow {
            name: String,
        }
        const TABLE_QUERY: &str =
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';";
        let rows: Vec<TableNameRow> = exec
            .query_json(&SqlStatement::new(TABLE_QUERY))
            .await
            .unwrap();
        let existing: std::collections::HashSet<String> = rows
            .into_iter()
            .map(|row| row.name.to_ascii_lowercase())
            .collect();

        let has = |req: &[&str]| {
            req.iter()
                .all(|t| existing.contains(&t.to_ascii_lowercase()))
        };
        assert!(has(&["foo", "bar"]));
        assert!(!has(&["foo", "baz"]));
    }

    #[tokio::test]
    async fn query_json_multi_column_rows() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("multi.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(
            r#"
                CREATE TABLE people (
                    id INTEGER,
                    name TEXT,
                    note BLOB
                );
                INSERT INTO people (id, name, note) VALUES
                    (1, 'Alice', X'000102'),
                    (2, 'Bob',   X'ff'),
                    (3, 'Carol', NULL),
                    (4, 'Дора',  X'c0'),
                    (5, 'Eve',   X'');
                "#,
        ))
        .await
        .unwrap();

        #[derive(serde::Deserialize, Debug, PartialEq, Eq)]
        struct PersonRow {
            id: i64,
            name: String,
            note: Option<String>,
        }

        let rows: Vec<PersonRow> = exec
            .query_json(&SqlStatement::new(
                "SELECT id, name, note FROM people ORDER BY id ASC;",
            ))
            .await
            .unwrap();

        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows,
            vec![
                PersonRow {
                    id: 1,
                    name: "Alice".to_string(),
                    note: Some("0x000102".to_string()),
                },
                PersonRow {
                    id: 2,
                    name: "Bob".to_string(),
                    note: Some("0xff".to_string()),
                },
                PersonRow {
                    id: 3,
                    name: "Carol".to_string(),
                    note: None,
                },
                PersonRow {
                    id: 4,
                    name: "Дора".to_string(),
                    note: Some("0xc0".to_string()),
                },
                PersonRow {
                    id: 5,
                    name: "Eve".to_string(),
                    note: Some("0x".to_string()),
                },
            ]
        );
    }

    #[tokio::test]
    async fn binds_u64_params_as_integer_and_text() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("u64.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new(
            "CREATE TABLE uvals (label TEXT PRIMARY KEY, val);",
        ))
        .await
        .unwrap();

        let mut insert_small = SqlStatement::new("INSERT INTO uvals (label, val) VALUES (?1, ?2);");
        insert_small.push("small");
        insert_small.push(123u64);

        let large_value = (i64::MAX as u64) + 1;
        let mut insert_large = SqlStatement::new("INSERT INTO uvals (label, val) VALUES (?1, ?2);");
        insert_large.push("large");
        insert_large.push(large_value);

        let batch = SqlStatementBatch::from(vec![insert_small, insert_large]).ensure_transaction();
        exec.execute_batch(&batch).await.unwrap();

        #[derive(serde::Deserialize, Debug)]
        struct U64Row {
            label: String,
            ty: String,
            val: Value,
        }

        let rows: Vec<U64Row> = exec
            .query_json(&SqlStatement::new(
                "SELECT label, typeof(val) AS ty, val FROM uvals ORDER BY label ASC;",
            ))
            .await
            .unwrap();

        assert_eq!(rows.len(), 2);

        let small = rows.iter().find(|row| row.label == "small").unwrap();
        assert_eq!(small.ty, "integer");
        assert_eq!(small.val, json!(123));

        let large = rows.iter().find(|row| row.label == "large").unwrap();
        assert_eq!(large.ty, "text");
        assert_eq!(large.val, json!(large_value.to_string()));
    }

    #[tokio::test]
    async fn execute_batch_rolls_back_on_error_non_parameterized() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("rollback-non-param.db");
        let db_path_str = db_path.to_string_lossy();
        let exec = RusqliteExecutor::new(&*db_path_str);

        let mut setup_batch = SqlStatementBatch::new();
        setup_batch.add(SqlStatement::new(
            "CREATE TABLE rollback_non_param (id INTEGER PRIMARY KEY, value TEXT NOT NULL);",
        ));
        let setup_batch = setup_batch.ensure_transaction();
        exec.execute_batch(&setup_batch).await.unwrap();

        let mut batch = SqlStatementBatch::new();
        batch.add(SqlStatement::new(
            "INSERT INTO rollback_non_param (id, value) VALUES (1, 'first');",
        ));
        batch.add(SqlStatement::new(
            "INSERT INTO rollback_non_param (id, value) VALUES (1, 'duplicate');",
        ));
        let batch = batch.ensure_transaction();

        let err = exec.execute_batch(&batch).await.unwrap_err();
        assert!(matches!(err, LocalDbQueryError::Database { .. }));

        let conn = Connection::open(&*db_path_str).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM rollback_non_param;", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn execute_batch_rolls_back_on_error_parameterized() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("rollback-param.db");
        let db_path_str = db_path.to_string_lossy();
        let exec = RusqliteExecutor::new(&*db_path_str);

        let mut setup_batch = SqlStatementBatch::new();
        setup_batch.add(SqlStatement::new(
            "CREATE TABLE rollback_param (id INTEGER PRIMARY KEY, value TEXT NOT NULL);",
        ));
        let setup_batch = setup_batch.ensure_transaction();
        exec.execute_batch(&setup_batch).await.unwrap();

        let mut insert_ok =
            SqlStatement::new("INSERT INTO rollback_param (id, value) VALUES (?1, ?2);");
        insert_ok.push(1i64);
        insert_ok.push("first");

        let mut insert_fail =
            SqlStatement::new("INSERT INTO rollback_param (id, value) VALUES (?1, ?2);");
        insert_fail.push(1i64);
        insert_fail.push("duplicate");

        let mut batch = SqlStatementBatch::new();
        batch.add(insert_ok);
        batch.add(insert_fail);
        let batch = batch.ensure_transaction();

        let err = exec.execute_batch(&batch).await.unwrap_err();
        assert!(matches!(err, LocalDbQueryError::Database { .. }));

        let conn = Connection::open(&*db_path_str).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM rollback_param;", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn execute_batch_requires_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("batch.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);

        let batch = SqlStatementBatch::from(vec![SqlStatement::new("SELECT 1")]);
        let err = exec.execute_batch(&batch).await.unwrap_err();

        assert!(matches!(err, LocalDbQueryError::Database { .. }));
    }

    #[tokio::test]
    async fn wipe_and_recreate_removes_and_recreates_db() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("wipe.db");
        let db_path_str = db_path.to_string_lossy();

        let exec = RusqliteExecutor::new(&*db_path_str);
        exec.query_text(&SqlStatement::new("CREATE TABLE test_table (id INTEGER);"))
            .await
            .unwrap();

        assert!(db_path.exists());

        exec.wipe_and_recreate().await.unwrap();

        assert!(db_path.exists());

        #[derive(serde::Deserialize)]
        struct TableNameRow {
            #[allow(dead_code)]
            name: String,
        }
        let rows: Vec<TableNameRow> = exec
            .query_json(&SqlStatement::new(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%';",
            ))
            .await
            .unwrap();
        assert!(rows.is_empty());
    }
}
