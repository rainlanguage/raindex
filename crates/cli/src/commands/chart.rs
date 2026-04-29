use crate::execute::Execute;
use anyhow::{anyhow, Result};
use clap::Args;
use rain_orderbook_common::fuzz::{FuzzRunner, FuzzRunnerContext};
use std::fs::read_to_string;
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Clone)]
pub struct Chart {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,
}

impl Execute for Chart {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let fuzzer = FuzzRunner::new(None)?;
        let mut context = FuzzRunnerContext::new(&dotrain, None, None)?;
        let chart_data = fuzzer.make_chart_data(&mut context).await?;

        info!("{:#?}", chart_data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_test_fixtures::LocalEvm;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn get_dotrain_prefix(subparser: &str, token1: &str, token2: &str) -> String {
        format!(
            r#"
tokens:
  token1:
    network: "flare"
    address: {token1}
    decimals: 18
  token2:
    network: "flare"
    address: {token2}
    decimals: 18
orders:
  flare:
    orderbook: "flare"
    inputs:
      - token: "token1"
    outputs:
      - token: "token2"
scenarios:
  flare:
    orderbook: "flare"
    runs: 1
    bindings:
      raindex-subparser: {subparser}
      fixed-io-output-token: {token1}
deployments:
  flare:
    order: flare
    scenario: flare
"#
        )
    }

    fn get_settings(rpc: &str, orderbook: &str, rainlang: &str) -> String {
        format!(
            r#"
version: {spec_version}
networks:
  flare:
    rpcs:
      - {rpc}
    chain-id: 14
    currency: "FLR"
subgraphs:
  flare: "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/2024-12-13-9dc7/gn"
metaboards:
  flare: "https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn"
orderbooks:
  flare:
    address: {orderbook}
    network: "flare"
    subgraph: "flare"
rainlangs:
  flare:
    address: {rainlang}
    network: "flare"
"#,
            spec_version = SpecVersion::current()
        )
    }

    const HAPPY_CHART: &str = r#"
charts:
  flare-chart:
    scenario: "flare"
    plots:
      my-plot:
        title: Test title
        subtitle: Test subtitle
        marks:
          - type: dot
            options:
              x: 1
              y: 2
              r: 3
              fill: "red"
              stroke: "blue"
              transform:
                type: hexbin
                content:
                  outputs:
                    x: 1
                    y: 2
                    r: 3
                    z: 4
                    stroke: "green"
                    fill: "blue"
                  options:
                    x: 1
                    y: 2
                    bin-width: 10
        x:
          label: Test x label
          anchor: start
          label-anchor: middle
          label-arrow: none
        y:
          label: Test y label
          anchor: end
          label-anchor: end
          label-arrow: arrow
        margin: 10
        margin-left: 20
        margin-right: 30
        margin-top: 40
        margin-bottom: 50
        inset: 2
    metrics:
      - label: Metric One
        description: Description for metric one
        value: 42
        precision: 0
      - label: Metric Two
        description: Description for metric two
        unit-prefix: "k"
        value: 3.14
        precision: 2
      - label: Metric Three
        description: Description for metric three
        unit-suffix: "ms"
        value: 100
        precision: 1
      - label: Metric Four
        description: Description for metric four
        unit-prefix: "%"
        value: 87
        precision: 0
"#;

    const RAINLANG: &str = r#"
#raindex-subparser !The subparser to use.
#fixed-io !The io ratio for the limit order.
#fixed-io-output-token !The output token that the fixed io is for. If this doesn't match the runtime output then the fixed-io will be inverted.
#calculate-io
using-words-from raindex-subparser
max-output: max-positive-value(),
io: fixed-io;
#handle-io
:;
#handle-add-order
:; 
"#;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_chart_execute_success() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let orderbook = &local_evm.orderbook;
        let orderbook_subparser = &local_evm.orderbook_subparser;
        let rainlang = local_evm.rainlang;
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let settings = get_settings(
            &local_evm.url(),
            &orderbook.address().to_string(),
            &rainlang.to_string(),
        );
        let dotrain_prefix = get_dotrain_prefix(
            &orderbook_subparser.address().to_string(),
            &token1.address().to_string(),
            &token2.address().to_string(),
        );
        let dotrain = format!(
            "{}\n{}\n{}\n---\n{}",
            settings, dotrain_prefix, HAPPY_CHART, RAINLANG
        );

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(dotrain.as_bytes()).unwrap();

        let chart_cmd = Chart {
            dotrain_file: temp_file.path().to_path_buf(),
        };

        let result = chart_cmd.execute().await;
        assert!(
            result.is_ok(),
            "Expected execution to succeed, but it failed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_chart_execute_file_not_found() {
        let chart_cmd = Chart {
            dotrain_file: PathBuf::from("non_existent_file.rain"),
        };

        let result = chart_cmd.execute().await;
        assert!(
            result.is_err(),
            "Expected execution to fail due to file not found"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_chart_execute_missing_scenario() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let orderbook = &local_evm.orderbook;
        let orderbook_subparser = &local_evm.orderbook_subparser;
        let rainlang = local_evm.rainlang;
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let settings = get_settings(
            &local_evm.url(),
            &orderbook.address().to_string(),
            &rainlang.to_string(),
        );
        let dotrain_prefix = get_dotrain_prefix(
            &orderbook_subparser.address().to_string(),
            &token1.address().to_string(),
            &token2.address().to_string(),
        );

        let dotrain = format!(
            "{}\n{}\n{}\n---\n{}",
            settings,
            dotrain_prefix,
            r#"
charts:
  flare-chart:
    scenario: missing
    plots:
"#,
            RAINLANG
        );

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(dotrain.as_bytes()).unwrap();

        let chart_cmd = Chart {
            dotrain_file: temp_file.path().to_path_buf(),
        };

        let result = chart_cmd.execute().await;
        assert!(
            result.is_err(),
            "Expected execution to fail due to missing scenario"
        );
        assert!(
            result
                .as_ref()
                .err()
                .unwrap()
                .to_string()
                .contains("Key 'missing' not found"),
            "Expected execution to fail due to missing scenario, got {:?}",
            result.err().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_chart_execute_missing_rpc_response() {
        let local_evm = LocalEvm::new_with_tokens(2).await;
        let orderbook = &local_evm.orderbook;
        let orderbook_subparser = &local_evm.orderbook_subparser;
        let rainlang = local_evm.rainlang;
        let token1 = local_evm.tokens[0].clone();
        let token2 = local_evm.tokens[1].clone();

        let settings = get_settings(
            "http://localhost:8545",
            &orderbook.address().to_string(),
            &rainlang.to_string(),
        );
        let dotrain_prefix = get_dotrain_prefix(
            &orderbook_subparser.address().to_string(),
            &token1.address().to_string(),
            &token2.address().to_string(),
        );
        let dotrain = format!(
            "{}\n{}\n{}\n---\n{}",
            settings, dotrain_prefix, HAPPY_CHART, RAINLANG
        );

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(dotrain.as_bytes()).unwrap();

        let chart_cmd = Chart {
            dotrain_file: temp_file.path().to_path_buf(),
        };

        let result = chart_cmd.execute().await;
        assert!(
            result.is_err(),
            "Expected execution to fail due to missing RPC response"
        );
        assert!(
            result
                .as_ref()
                .err()
                .unwrap()
                .to_string()
                .contains("error sending request"),
            "Expected execution to fail due to missing RPC response, got {:?}",
            result.err().unwrap()
        );
    }
}
