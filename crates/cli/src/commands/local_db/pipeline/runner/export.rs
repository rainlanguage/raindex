use crate::commands::local_db::executor::RusqliteExecutor;
use alloy::primitives::hex::encode_prefixed;
use flate2::write::GzEncoder;
use flate2::Compression;
use rain_orderbook_common::local_db::export::{export_data_only, ExportError};
use rain_orderbook_common::local_db::pipeline::runner::utils::RunnerTarget;
use rain_orderbook_common::local_db::pipeline::SyncOutcome;
use rain_orderbook_common::local_db::query::fetch_target_watermark::{
    fetch_target_watermark_stmt, TargetWatermarkRow,
};
use rain_orderbook_common::local_db::query::LocalDbQueryExecutor;
use rain_orderbook_common::local_db::LocalDbError;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::fs::create_dir_all;

#[derive(Debug)]
pub struct ExportMetadata {
    pub dump_path: PathBuf,
    pub end_block: u64,
    pub end_block_hash: String,
    pub end_block_time_ms: u64,
}

pub(super) async fn export_dump(
    executor: &RusqliteExecutor,
    target: &RunnerTarget,
    outcome: &SyncOutcome,
    out_root: &Path,
) -> Result<Option<ExportMetadata>, LocalDbError> {
    let dump_sql = match export_data_only(executor, &target.inputs.ob_id).await? {
        Some(sql) => sql,
        None => return Ok(None),
    };

    let chain_folder = out_root.join(target.inputs.ob_id.chain_id.to_string());
    create_dir_all(&chain_folder).await?;

    let filename = format!(
        "{}-{}.sql.gz",
        target.inputs.ob_id.chain_id, target.inputs.ob_id.orderbook_address
    );
    let dump_path = chain_folder.join(filename);

    let compressed = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, std::io::Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(dump_sql.as_bytes())?;
        encoder.finish()
    })
    .await??;

    tokio::fs::write(&dump_path, compressed).await?;

    let watermark_stmt = fetch_target_watermark_stmt(&target.inputs.ob_id);
    let rows: Vec<TargetWatermarkRow> = executor.query_json(&watermark_stmt).await?;
    let row = rows.into_iter().next().ok_or_else(|| {
        LocalDbError::from(ExportError::MissingTargetWatermark {
            chain_id: target.inputs.ob_id.chain_id,
            orderbook_address: target.inputs.ob_id.orderbook_address,
        })
    })?;

    Ok(Some(ExportMetadata {
        dump_path,
        end_block: outcome.target_block,
        end_block_hash: encode_prefixed(&row.last_hash),
        end_block_time_ms: row.updated_at,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;
    use flate2::read::GzDecoder;
    use rain_orderbook_common::local_db::{
        pipeline::{engine::SyncInputs, FinalityConfig, SyncConfig, WindowOverrides},
        FetchConfig, OrderbookIdentifier,
    };
    use rusqlite::{params, Connection};
    use std::io::{Cursor, Read};
    use tempfile::TempDir;
    use tokio::fs;
    use url::Url;

    #[tokio::test]
    async fn export_dump_writes_gzip_and_reads_watermark_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("orderbook.sqlite");
        let conn = Connection::open(&db_path).expect("open sqlite db");
        conn.execute_batch(
            rain_orderbook_common::local_db::query::create_tables::CREATE_TABLES_SQL,
        )
        .expect("create tables");

        let chain_id = 42161u32;
        let orderbook_address = address!("0x0000000000000000000000000000000000000abc");
        let orderbook_str = encode_prefixed(orderbook_address);

        conn.execute(
            "INSERT INTO raw_events (chain_id, orderbook_address, transaction_hash, log_index, block_number, block_timestamp, address, topics, data, raw_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
            params![
                chain_id as i64,
                orderbook_str.as_str(),
                "0xtesttx",
                0i64,
                1000i64,
                1_700_000_000i64,
                "event_address",
                "[]",
                "0x00",
                "{}",
            ],
        )
        .expect("insert raw_events row");

        conn.execute(
            "INSERT INTO target_watermarks (chain_id, orderbook_address, last_block, last_hash) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(chain_id, orderbook_address) DO UPDATE SET last_block = excluded.last_block, last_hash = excluded.last_hash, updated_at = 1_700_000_000_000;",
            params![
                chain_id as i64,
                orderbook_str.as_str(),
                1000i64,
                "0xdeadbeef",
            ],
        )
        .expect("insert watermark row");

        drop(conn);

        let executor = RusqliteExecutor::new(&db_path);
        let fetch = FetchConfig::new(1, 1, 1, 1, 0, 0).expect("fetch config");
        let sync_config = SyncConfig {
            deployment_block: 900,
            fetch,
            finality: FinalityConfig { depth: 12 },
            window_overrides: WindowOverrides::default(),
        };

        let ob_id = OrderbookIdentifier {
            chain_id,
            orderbook_address,
        };

        let target = RunnerTarget {
            orderbook_key: "test".to_string(),
            manifest_url: Url::parse("https://example.com/manifest.yaml").unwrap(),
            network_key: "anvil".to_string(),
            inputs: SyncInputs {
                ob_id: ob_id.clone(),
                metadata_rpcs: Vec::new(),
                cfg: sync_config,
                dump_str: None,
                block_number_threshold: 10000,
                manifest_end_block: 1,
            },
        };

        let outcome = SyncOutcome {
            ob_id,
            start_block: 900,
            target_block: 1000,
            fetched_logs: 1,
            decoded_events: 1,
        };

        let metadata = export_dump(&executor, &target, &outcome, temp_dir.path())
            .await
            .expect("export succeeds")
            .expect("dump produced");

        assert_eq!(metadata.end_block, outcome.target_block);
        assert_eq!(metadata.end_block_hash, "0xdeadbeef");
        assert!(
            metadata.end_block_time_ms > 0,
            "expected timestamp to be populated"
        );
        assert_eq!(
            metadata.dump_path.extension().and_then(|ext| ext.to_str()),
            Some("gz")
        );
        let expected_file = format!(
            "{}-{}.sql.gz",
            chain_id, target.inputs.ob_id.orderbook_address
        );
        assert_eq!(
            metadata
                .dump_path
                .file_name()
                .and_then(|name| name.to_str()),
            Some(expected_file.as_str())
        );

        let gz_bytes = fs::read(&metadata.dump_path).await.expect("read dump file");
        let mut decoder = GzDecoder::new(Cursor::new(gz_bytes));
        let mut sql = String::new();
        decoder.read_to_string(&mut sql).expect("decode gzip");

        assert!(
            sql.contains("INSERT INTO \"raw_events\""),
            "export should include raw_events data"
        );
        assert!(sql.starts_with("BEGIN;"));
        assert!(sql.ends_with("COMMIT;\n"));
    }

    #[tokio::test]
    async fn export_dump_returns_none_when_no_rows_are_present() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("orderbook.sqlite");
        let conn = Connection::open(&db_path).expect("open sqlite db");
        conn.execute_batch(
            rain_orderbook_common::local_db::query::create_tables::CREATE_TABLES_SQL,
        )
        .expect("create tables");
        drop(conn);

        let chain_id = 10u32;
        let orderbook_address = address!("0x0000000000000000000000000000000000000fff");
        let executor = RusqliteExecutor::new(&db_path);
        let fetch = FetchConfig::new(1, 1, 1, 1, 0, 0).expect("fetch config");
        let sync_config = SyncConfig {
            deployment_block: 0,
            fetch,
            finality: FinalityConfig { depth: 1 },
            window_overrides: WindowOverrides::default(),
        };

        let ob_id = OrderbookIdentifier {
            chain_id,
            orderbook_address,
        };

        let target = RunnerTarget {
            orderbook_key: "empty".to_string(),
            manifest_url: Url::parse("https://example.com/empty.yaml").unwrap(),
            network_key: "anvil".to_string(),
            inputs: SyncInputs {
                ob_id: ob_id.clone(),
                metadata_rpcs: Vec::new(),
                cfg: sync_config,
                dump_str: None,
                block_number_threshold: 10000,
                manifest_end_block: 1,
            },
        };

        let outcome = SyncOutcome {
            ob_id,
            start_block: 0,
            target_block: 0,
            fetched_logs: 0,
            decoded_events: 0,
        };

        let result = export_dump(&executor, &target, &outcome, temp_dir.path())
            .await
            .expect("export succeeds");

        assert!(
            result.is_none(),
            "expected no export when database is empty"
        );
        let chain_folder = temp_dir.path().join(chain_id.to_string());
        assert!(
            !chain_folder.exists(),
            "no dump directory should be created when nothing is exported"
        );
    }
}
