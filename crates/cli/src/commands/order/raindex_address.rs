use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Parser;
use raindex_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser, Clone)]
pub struct RaindexAddress {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    #[arg(short = 'e', long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for RaindexAddress {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };
        let dotrain_order = DotrainOrder::create(dotrain, settings.map(|v| vec![v])).await?;
        let deployment_ref = dotrain_order
            .dotrain_yaml()
            .get_deployment(&self.deployment)?;

        let raindex_address = if let Some(v) = &deployment_ref.order.raindex {
            v.address
        } else {
            let network_key = &deployment_ref.scenario.rainlang.network.key;
            let mut raindex_address = None;
            for key in dotrain_order.raindex_yaml().get_raindex_keys()? {
                let raindex = dotrain_order.raindex_yaml().get_raindex(&key)?;
                if key == *network_key || raindex.network.key == *network_key {
                    raindex_address = Some(raindex.address);
                }
            }
            if raindex_address.is_none() {
                return Err(anyhow!("specified raindex is undefined!"));
            }
            raindex_address.unwrap()
        };
        output(&None, self.encoding.clone(), raindex_address.as_slice())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use raindex_app_settings::spec_version::SpecVersion;
    use std::str::FromStr;
    use tempfile::NamedTempFile;

    #[test]
    fn verify_cli() {
        RaindexAddress::command().debug_assert();
    }

    #[test]
    fn test_cli_args() {
        let dotrain_file = PathBuf::from_str("./some/dotrain_file.rain").unwrap();
        let settings_file = PathBuf::from_str("./some/settings_file.rain").unwrap();
        let deployment_str = "some-deployment";
        let output_str = "hex";

        let cmd = RaindexAddress::command();
        let result = cmd.get_matches_from(vec![
            "cmd",
            "-f",
            dotrain_file.to_str().unwrap(),
            "-c",
            settings_file.to_str().unwrap(),
            "-e",
            deployment_str,
            "-o",
            output_str,
        ]);
        assert_eq!(
            result.get_one::<PathBuf>("dotrain_file"),
            Some(&dotrain_file)
        );
        assert_eq!(
            result.get_one::<PathBuf>("settings_file"),
            Some(&settings_file)
        );
        assert_eq!(
            result.get_one::<String>("deployment"),
            Some(&deployment_str.to_string())
        );
        assert_eq!(
            result.get_one::<SupportedOutputEncoding>("encoding"),
            Some(&SupportedOutputEncoding::Hex)
        );
    }

    fn get_test_dotrain(raindex_key: &str) -> String {
        format!(
            "
version: {spec_version}

networks:
    some-network:
        rpcs:
            - https://some-url.com
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
    {}:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
        local-db-remote: some-remote
        deployment-block: 12345

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: T2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
        bindings:
            key1: 10

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token2
              vault-id: 1
        rainlang: some-rainlang

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#key !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;",
            raindex_key,
            spec_version = SpecVersion::current()
        )
    }

    #[tokio::test]
    async fn test_execute_diff_name() {
        let dotrain = get_test_dotrain("some-raindex");

        let dotrain_file = NamedTempFile::new().unwrap();
        std::fs::write(dotrain_file.path(), dotrain).unwrap();

        let raindex_address = RaindexAddress {
            dotrain_file: dotrain_file.path().into(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should succeed without err
        raindex_address.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_same_name() {
        let dotrain = get_test_dotrain("some-network");

        let dotrain_file = NamedTempFile::new().unwrap();
        std::fs::write(dotrain_file.path(), dotrain).unwrap();

        let raindex_address = RaindexAddress {
            dotrain_file: dotrain_file.path().into(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should succeed without err
        raindex_address.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_non_existing_deployment() {
        let dotrain = get_test_dotrain("some-network");

        let dotrain_file = NamedTempFile::new().unwrap();
        std::fs::write(dotrain_file.path(), dotrain).unwrap();

        let raindex_address = RaindexAddress {
            dotrain_file: dotrain_file.path().into(),
            settings_file: None,
            deployment: "non-existing-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should return err
        raindex_address.execute().await.unwrap_err();
    }

    #[tokio::test]
    async fn test_execute_non_existing_file() {
        let raindex_address = RaindexAddress {
            dotrain_file: PathBuf::from_str("non-existing-file.rain").unwrap(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should return err
        raindex_address.execute().await.unwrap_err();
    }

    #[tokio::test]
    async fn test_execute_empty_dotrain() {
        let dotrain_file = NamedTempFile::new().unwrap();

        let raindex_address = RaindexAddress {
            dotrain_file: dotrain_file.path().into(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should return err
        raindex_address.execute().await.unwrap_err();
    }
}
