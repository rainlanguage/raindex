use crate::local_db::pipeline::{FinalityConfig, SyncConfig, WindowPipeline};
use crate::local_db::query::fetch_target_watermark::{
    fetch_target_watermark_stmt, TargetWatermarkRow,
};
use crate::local_db::query::LocalDbQueryExecutor;
use crate::local_db::{LocalDbError, OrderbookIdentifier};
use async_trait::async_trait;

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultWindowPipeline;

impl DefaultWindowPipeline {
    pub const fn new() -> Self {
        Self
    }

    fn safe_head(latest_block: u64, deployment_block: u64, finality: &FinalityConfig) -> u64 {
        let depth = finality.depth as u64;
        let head = latest_block.saturating_sub(depth);
        head.max(deployment_block)
    }

    fn base_start(last_synced_block: u64, deployment_block: u64) -> Result<u64, LocalDbError> {
        if last_synced_block == 0 {
            Ok(deployment_block)
        } else {
            last_synced_block
                .checked_add(1)
                .ok_or(LocalDbError::LastSyncedBlockOverflow { last_synced_block })
        }
    }
}

#[async_trait(?Send)]
impl WindowPipeline for DefaultWindowPipeline {
    async fn compute<DB>(
        &self,
        db: &DB,
        ob_id: &OrderbookIdentifier,
        cfg: &SyncConfig,
        latest_block: u64,
    ) -> Result<(u64, u64), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        // 1) Read watermark
        let last_synced_block = {
            let rows: Vec<TargetWatermarkRow> = db
                .query_json(&fetch_target_watermark_stmt(ob_id))
                .await
                .map_err(LocalDbError::from)?;
            rows.first().map(|r| r.last_block).unwrap_or(0)
        };

        // 2) Compute safe head with finality clamp
        let safe_head = Self::safe_head(latest_block, cfg.deployment_block, &cfg.finality);

        // 3) Determine base start
        let mut start = Self::base_start(last_synced_block, cfg.deployment_block)?;

        // 4) Apply start override and bump if needed
        if let Some(override_start) = cfg.window_overrides.start_block {
            start = override_start;
            if last_synced_block > 0 && start <= last_synced_block {
                start = last_synced_block
                    .checked_add(1)
                    .ok_or(LocalDbError::LastSyncedBlockOverflow { last_synced_block })?;
            }
            if last_synced_block == 0 && start < cfg.deployment_block {
                start = cfg.deployment_block;
            }
        }

        // 5) Apply end override then clamp to safe head
        let requested_target = cfg.window_overrides.end_block.unwrap_or(safe_head);
        let target = requested_target.min(safe_head);

        Ok((start, target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::pipeline::{FinalityConfig, SyncConfig, WindowOverrides};
    use crate::local_db::query::{LocalDbQueryError, SqlStatement, SqlStatementBatch};
    use alloy::primitives::{Address, Bytes};
    use async_trait::async_trait;
    use std::str::FromStr;

    #[derive(Default)]
    struct MockDb {
        last_synced: u64,
        fail: bool,
    }

    #[cfg_attr(target_family = "wasm", async_trait(?Send))]
    #[cfg_attr(not(target_family = "wasm"), async_trait)]
    impl LocalDbQueryExecutor for MockDb {
        async fn execute_batch(&self, _batch: &SqlStatementBatch) -> Result<(), LocalDbQueryError> {
            Ok(())
        }

        async fn query_json<T>(&self, stmt: &SqlStatement) -> Result<T, LocalDbQueryError>
        where
            T: crate::local_db::query::FromDbJson,
        {
            if self.fail {
                return Err(LocalDbQueryError::database("boom"));
            }
            let expected_sql =
                crate::local_db::query::fetch_target_watermark::FETCH_TARGET_WATERMARK_SQL;
            if stmt.sql() == expected_sql {
                let params = stmt.params();
                let chain_id = match params.first() {
                    Some(crate::local_db::query::SqlValue::I64(v)) => *v as u32,
                    Some(crate::local_db::query::SqlValue::U64(v)) => *v as u32,
                    _ => 0,
                };
                let orderbook_address = match params.get(1) {
                    Some(crate::local_db::query::SqlValue::Text(v)) => v.clone(),
                    _ => format!("0x{:040x}", 0u128),
                };
                let body = if self.last_synced == 0 {
                    "[]".to_string()
                } else {
                    serde_json::to_string(&vec![TargetWatermarkRow {
                        chain_id,
                        orderbook_address: Address::from_str(&orderbook_address).unwrap(),
                        last_block: self.last_synced,
                        last_hash: Bytes::from_str("0xbeef").unwrap(),
                        updated_at: 1,
                    }])
                    .unwrap()
                };
                return serde_json::from_str(&body)
                    .map_err(|e| LocalDbQueryError::deserialization(e.to_string()));
            }
            Err(LocalDbQueryError::database("unexpected query"))
        }

        async fn query_text(&self, _stmt: &SqlStatement) -> Result<String, LocalDbQueryError> {
            Ok(String::new())
        }

        async fn wipe_and_recreate(&self) -> Result<(), LocalDbQueryError> {
            Err(LocalDbQueryError::not_implemented("wipe_and_recreate"))
        }
    }

    fn sample_ob_id() -> OrderbookIdentifier {
        OrderbookIdentifier {
            chain_id: 1,
            orderbook_address: Address::from([0u8; 20]),
        }
    }

    fn cfg(deployment_block: u64, depth: u32, start: Option<u64>, end: Option<u64>) -> SyncConfig {
        SyncConfig {
            deployment_block,
            fetch: Default::default(),
            finality: FinalityConfig { depth },
            window_overrides: WindowOverrides {
                start_block: start,
                end_block: end,
            },
        }
    }

    #[tokio::test]
    async fn base_start_from_deploy_when_empty() {
        let db = MockDb {
            last_synced: 0,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        let (start, target) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, None, None), 200)
            .await
            .unwrap();
        assert_eq!(start, 100);
        assert_eq!(target, 200);
    }

    #[tokio::test]
    async fn base_start_is_last_plus_one() {
        let db = MockDb {
            last_synced: 150,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        let (start, _) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, None, None), 200)
            .await
            .unwrap();
        assert_eq!(start, 151);
    }

    #[tokio::test]
    async fn start_override_below_last_bumps() {
        let db = MockDb {
            last_synced: 150,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        let (start, _) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, Some(100), None), 200)
            .await
            .unwrap();
        assert_eq!(start, 151);
    }

    #[tokio::test]
    async fn start_override_before_deploy_when_empty_clamped_to_deploy() {
        let db = MockDb {
            last_synced: 0,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        let (start, _) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, Some(50), None), 200)
            .await
            .unwrap();
        assert_eq!(start, 100);
    }

    #[tokio::test]
    async fn target_is_finality_clamped() {
        let db = MockDb {
            last_synced: 0,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        // latest=1000, depth=20 => safe_head= max(100, 980)=980
        let (_, target) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 20, None, None), 1000)
            .await
            .unwrap();
        assert_eq!(target, 980);
    }

    #[tokio::test]
    async fn end_override_is_clamped_to_safe_head() {
        let db = MockDb {
            last_synced: 0,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        // safe_head = 980
        let (_, target) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 20, None, Some(2000)), 1000)
            .await
            .unwrap();
        assert_eq!(target, 980);
    }

    #[tokio::test]
    async fn db_error_bubbles() {
        let db = MockDb {
            last_synced: 0,
            fail: true,
        };
        let pipe = DefaultWindowPipeline::new();
        let err = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, None, None), 200)
            .await
            .unwrap_err();
        match err {
            LocalDbError::LocalDbQueryError(..) => {}
            other => panic!("expected LocalDbError::LocalDbQueryError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn start_override_equals_last_bumps() {
        let db = MockDb {
            last_synced: 150,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        let (start, _) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, Some(150), None), 200)
            .await
            .unwrap();
        assert_eq!(start, 151);
    }

    #[tokio::test]
    async fn end_override_below_safe_head_is_respected() {
        let db = MockDb {
            last_synced: 0,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        // safe_head = 980; end_override = 500 -> target = 500
        let (_, target) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 20, None, Some(500)), 1000)
            .await
            .unwrap();
        assert_eq!(target, 500);
    }

    #[tokio::test]
    async fn safe_head_clamped_to_deploy_when_depth_gt_latest() {
        let db = MockDb {
            last_synced: 0,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        // latest=100, depth=200 => latest-depth saturates to 0; max(deploy=90, 0) = 90
        let (_, target) = pipe
            .compute(&db, &sample_ob_id(), &cfg(90, 200, None, None), 100)
            .await
            .unwrap();
        assert_eq!(target, 90);
    }

    #[tokio::test]
    async fn overflow_on_last_synced_max() {
        let db = MockDb {
            last_synced: u64::MAX,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        let err = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, None, None), 200)
            .await
            .unwrap_err();
        match err {
            LocalDbError::LastSyncedBlockOverflow { last_synced_block } => {
                assert_eq!(last_synced_block, u64::MAX);
            }
            other => panic!("expected LocalDbError::LastSyncedBlockOverflow, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn no_op_window_allowed_when_start_gt_target() {
        let db = MockDb {
            last_synced: 200,
            fail: false,
        };
        let pipe = DefaultWindowPipeline::new();
        // base start = 201; end_override = 150; depth=0 so safe_head is high and does not clamp
        let (start, target) = pipe
            .compute(&db, &sample_ob_id(), &cfg(100, 0, None, Some(150)), 1000)
            .await
            .unwrap();
        assert_eq!(start, 201);
        assert_eq!(target, 150);
        assert!(start > target, "documented no-op window condition");
    }
}
