use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Args;
use raindex_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Args, Clone)]
pub struct Compose {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    // path to the settings yaml
    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    // the name of the scenrio to use
    #[arg(short = 's', long, help = "The name of the scenario to use")]
    scenario: String,

    // whether to compose the post task
    #[arg(short = 'p', long, help = "Compose the post task")]
    post: bool,

    // supported encoding
    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for Compose {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };

        let dotrain_order = DotrainOrder::create(dotrain, settings.map(|v| vec![v])).await?;

        let rainlang = if self.post {
            dotrain_order
                .compose_scenario_to_post_task_rainlang(self.scenario.clone())
                .await?
        } else {
            dotrain_order
                .compose_scenario_to_rainlang(self.scenario.clone())
                .await?
        };

        output(&None, self.encoding.clone(), rainlang.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use raindex_app_settings::spec_version::SpecVersion;
    use tempfile::NamedTempFile;

    use super::*;

    #[tokio::test]
    async fn test_execute_happy() {
        let dotrain = get_dotrain();

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain).unwrap();

        let compose = Compose {
            dotrain_file: dotrain_path,
            settings_file: None,
            scenario: "some-scenario".to_string(),
            encoding: SupportedOutputEncoding::Hex,
            post: false,
        };

        assert!(compose.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        let dotrain = get_dotrain();

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain).unwrap();

        let compose = Compose {
            dotrain_file: dotrain_path,
            settings_file: None,
            scenario: "some-other-scenario".to_string(),
            encoding: SupportedOutputEncoding::Hex,
            post: false,
        };

        assert!(compose.execute().await.is_err());
    }

    fn get_dotrain() -> String {
        format!(
            r#"
version: {spec_version}
networks:
    some-network:
        rpcs:
            - https://some-rpc.com
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

raindexes:
    some-raindex:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg

scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
        bindings:
            key1: 10

tokens:
    token-one:
        network: some-network
        address: 0x1234567890123456789012345678901234567890

orders:
    some-order:
        inputs:
            - token: token-one
        outputs:
            - token: token-one
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#,
            spec_version = SpecVersion::current()
        )
    }
}
