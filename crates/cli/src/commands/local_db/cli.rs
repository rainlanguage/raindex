use super::pipeline::runner::ProducerRunReport;
use crate::commands::local_db::pipeline::runner::ProducerRunner;
use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::local_db::pipeline::runner::TargetFailure;
use std::io::{self, Write};
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone, Parser)]
#[command(about = "Run the producer pipeline across all orderbooks in settings.yaml")]
pub struct RunPipeline {
    #[clap(
        long,
        help = "Full YAML document that configures the local DB pipeline",
        value_name = "YAML"
    )]
    pub settings_yaml: String,

    #[clap(
        long,
        help = "HyperRPC API token used to fetch logs during the run",
        value_name = "TOKEN"
    )]
    pub api_token: String,

    #[clap(
        long,
        help = "Output directory where per-orderbook SQLite databases and dumps are written",
        value_name = "PATH",
        default_value = "./local-db"
    )]
    pub out_root: PathBuf,

    #[clap(
        long,
        help = "Base URL for published dumps (e.g., https://example.com/releases)",
        value_name = "URL",
        value_parser = clap::value_parser!(Url)
    )]
    pub release_base_url: Url,

    #[clap(
        long,
        help = "Emit per-stage status updates from the pipeline as it runs"
    )]
    pub debug_status: bool,
}

impl RunPipeline {
    pub async fn execute(self) -> Result<()> {
        println!("Starting producer pipeline run");

        let RunPipeline {
            settings_yaml,
            api_token,
            out_root,
            release_base_url,
            debug_status,
        } = self;

        std::fs::create_dir_all(&out_root)?;

        let runner = ProducerRunner::new(
            settings_yaml,
            out_root,
            release_base_url,
            api_token,
            debug_status.into(),
        )?;
        let report = runner.run().await?;

        render_report(&report);

        if report.failures().is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "one or more producer jobs failed ({} failure(s))",
                report.failures().len()
            ))
        }
    }
}

fn render_report(report: &ProducerRunReport) {
    let mut stdout = io::stdout();
    let _ = render_report_to(report, &mut stdout);
}

fn render_report_to<W: Write>(report: &ProducerRunReport, writer: &mut W) -> io::Result<()> {
    if report.successes().is_empty() {
        writeln!(writer, "No producer jobs completed successfully.")?;
    } else {
        writeln!(
            writer,
            "Producer pipeline completed {} successful job(s):",
            report.successes().len()
        )?;
        for outcome in report.successes() {
            let ob_id = &outcome.outcome.ob_id;
            match report.export_for(ob_id) {
                Some(export) => {
                    writeln!(
                        writer,
                        "- chain {} orderbook {:#x}: start {} → target {} | logs {} | events {} | dump {} (end block {}, hash {}, time {})",
                        ob_id.chain_id,
                        ob_id.orderbook_address,
                        outcome.outcome.start_block,
                        outcome.outcome.target_block,
                        outcome.outcome.fetched_logs,
                        outcome.outcome.decoded_events,
                        export.dump_path.display(),
                        export.end_block,
                        export.end_block_hash,
                        export.end_block_time_ms,
                    )?;
                }
                None => {
                    writeln!(
                        writer,
                        "- chain {} orderbook {:#x}: start {} → target {} | logs {} | events {} | dump <none>",
                        ob_id.chain_id,
                        ob_id.orderbook_address,
                        outcome.outcome.start_block,
                        outcome.outcome.target_block,
                        outcome.outcome.fetched_logs,
                        outcome.outcome.decoded_events,
                    )?;
                }
            }
        }
    }

    if report.failures().is_empty() {
        writeln!(writer, "All producer jobs completed successfully.")?;
    } else {
        writeln!(
            writer,
            "{} job(s) failed during the producer run:",
            report.failures().len()
        )?;
        for failure in report.failures() {
            render_failure_to(failure, writer)?;
        }
    }

    Ok(())
}

fn render_failure_to<W: Write>(failure: &TargetFailure, writer: &mut W) -> io::Result<()> {
    let ob_id = &failure.ob_id;
    let address = ob_id.orderbook_address;
    let chain_id = ob_id.chain_id;
    let stage = failure.stage;
    let message = failure.error.to_readable_msg();
    let key = failure
        .orderbook_key
        .as_deref()
        .unwrap_or("<unknown-orderbook>");

    if chain_id == 0 && address.is_zero() {
        writeln!(writer, "- job {} failed at {:?}: {}", key, stage, message)
    } else {
        writeln!(
            writer,
            "- chain {} orderbook {:#x} ({}) failed at {:?}: {}",
            chain_id, address, key, stage, message
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::local_db::pipeline::runner::export::ExportMetadata;

    use super::*;
    use alloy::primitives::{address, Address};
    use rain_orderbook_common::local_db::pipeline::engine::SyncInputs;
    use rain_orderbook_common::local_db::pipeline::runner::utils::RunnerTarget;
    use rain_orderbook_common::local_db::pipeline::runner::{
        TargetFailure, TargetStage, TargetSuccess,
    };
    use rain_orderbook_common::local_db::pipeline::{
        FinalityConfig, SyncConfig, SyncOutcome, WindowOverrides,
    };
    use rain_orderbook_common::local_db::{FetchConfig, LocalDbError, OrderbookIdentifier};
    use std::collections::HashMap;

    #[test]
    fn default_out_root_is_local_db() {
        let command = RunPipeline::parse_from([
            "sync",
            "--settings-yaml",
            "test",
            "--api-token",
            "token",
            "--release-base-url",
            "https://example.com/releases",
        ]);
        assert_eq!(command.out_root, PathBuf::from("./local-db"));
    }

    fn sample_success_and_export(chain_id: u32) -> (TargetSuccess, ExportMetadata) {
        let ob_id = OrderbookIdentifier::new(
            chain_id,
            address!("0000000000000000000000000000000000000a11"),
        );
        let fetch = FetchConfig::new(10, 5, 2, 1, 0, 0).unwrap();
        let sync_config = SyncConfig {
            deployment_block: 100,
            fetch,
            finality: FinalityConfig { depth: 12 },
            window_overrides: WindowOverrides::default(),
        };
        let inputs = SyncInputs {
            ob_id: ob_id.clone(),
            metadata_rpcs: Vec::new(),
            cfg: sync_config,
            dump_str: None,
            block_number_threshold: 10000,
            manifest_end_block: 1,
        };
        let runner_target = RunnerTarget {
            orderbook_key: "test".to_string(),
            manifest_url: "https://example.com/manifest.yaml".parse().unwrap(),
            network_key: "anvil".to_string(),
            inputs,
        };

        let outcome = SyncOutcome {
            ob_id: runner_target.inputs.ob_id.clone(),
            start_block: 200,
            target_block: 400,
            fetched_logs: 123,
            decoded_events: 456,
        };
        let export = ExportMetadata {
            dump_path: PathBuf::from(format!(
                "./local-db/{}/{}-{}.sql.gz",
                chain_id, chain_id, runner_target.inputs.ob_id.orderbook_address
            )),
            end_block: 400,
            end_block_hash: "0xdeadbeef".to_string(),
            end_block_time_ms: 1_700_000_000,
        };

        (TargetSuccess { outcome }, export)
    }

    #[test]
    fn render_report_to_writes_success_summary() {
        let (success, export) = sample_success_and_export(42161);
        let mut exports = HashMap::new();
        exports.insert(success.outcome.ob_id.clone(), Some(export));
        let report = ProducerRunReport {
            successes: vec![success],
            failures: vec![],
            exports,
        };
        let mut buffer = Vec::new();
        render_report_to(&report, &mut buffer).expect("render succeeds");

        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("1 successful job"));
        assert!(output.contains("chain 42161"));
        assert!(output.contains("start 200 → target 400"));
        assert!(output.contains("logs 123"));
        assert!(output.contains("events 456"));
        assert!(output.contains("All producer jobs completed successfully."));
    }

    #[test]
    fn render_report_to_handles_missing_dump() {
        let (success, _) = sample_success_and_export(10);
        let mut exports = HashMap::new();
        exports.insert(success.outcome.ob_id.clone(), None);
        let report = ProducerRunReport {
            successes: vec![success],
            failures: vec![],
            exports,
        };

        let mut buffer = Vec::new();
        render_report_to(&report, &mut buffer).expect("render succeeds");

        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("dump <none>"));
    }

    #[test]
    fn render_report_to_lists_failures() {
        let orderbook_address = address!("0000000000000000000000000000000000000fA1");
        let failure = TargetFailure {
            ob_id: OrderbookIdentifier::new(1, orderbook_address),
            orderbook_key: Some("book".into()),
            stage: TargetStage::EngineRun,
            error: LocalDbError::CustomError("oh no".into()),
        };
        let report = ProducerRunReport {
            successes: Vec::new(),
            failures: vec![failure],
            exports: HashMap::new(),
        };

        let mut buffer = Vec::new();
        render_report_to(&report, &mut buffer).expect("render succeeds");

        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("No producer jobs completed successfully."));
        assert!(output.contains("1 job(s) failed"));
        assert!(output.contains(&format!("chain {} orderbook {:#x}", 1, orderbook_address)));
        assert!(output.contains("oh no"));
    }

    #[test]
    fn render_failure_to_handles_unknowns() {
        let failure = TargetFailure {
            ob_id: OrderbookIdentifier::new(0, Address::ZERO),
            orderbook_key: None,
            stage: TargetStage::EngineRun,
            error: LocalDbError::CustomError("boom".into()),
        };
        let mut buffer = Vec::new();
        render_failure_to(&failure, &mut buffer).expect("render succeeds");
        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("job <unknown-orderbook> failed"));
        assert!(output.contains("EngineRun"));
    }
}
