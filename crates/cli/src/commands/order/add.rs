use crate::{
    execute::Execute, status::display_write_transaction_status, transaction::CliTransactionArgs,
};
use anyhow::{anyhow, Result};
use clap::{ArgAction, Args};
use raindex_app_settings::yaml::dotrain::{DotrainYaml, DotrainYamlValidation};
use raindex_app_settings::yaml::YamlParsable;
use raindex_common::add_order::AddOrderArgs;
use raindex_common::dotrain::RainDocument;
use raindex_common::transaction::TransactionArgs;
use std::fs::read_to_string;
use std::path::PathBuf;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderAddArgs {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'e', long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,

    /// Do NOT broadcast the transaction to the network, only simulate the transaction
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_broadcast: bool,
}

impl CliOrderAddArgs {
    async fn to_add_order_args(&self) -> Result<AddOrderArgs> {
        let text = read_to_string(&self.dotrain_file).map_err(|e| anyhow!(e))?;
        let dotrain_yaml = DotrainYaml::new(
            vec![RainDocument::get_front_matter(text.as_str())
                .unwrap_or("")
                .to_string()],
            DotrainYamlValidation::default(),
        )?;
        let config_deployment = dotrain_yaml.get_deployment(&self.deployment)?;

        Ok(
            AddOrderArgs::new_from_deployment(text.clone(), config_deployment.clone(), None)
                .await?,
        )
    }
}

impl Execute for CliOrderAddArgs {
    async fn execute(&self) -> Result<()> {
        let add_order_args: AddOrderArgs = self.clone().to_add_order_args().await?;
        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;

        info!("----- Simulating Transaction -----");
        add_order_args
            .simulate_execute(tx_args.clone(), None)
            .await?;
        info!("----- Finished Simulation Successfully -----");

        if !self.no_broadcast {
            info!("----- Add Order -----");
            add_order_args
                .execute(tx_args, |status| {
                    display_write_transaction_status(status);
                })
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, Address, B256, U256};
    use raindex_app_settings::spec_version::SpecVersion;
    use raindex_bindings::IRaindexV6::IOV2;
    use std::{collections::HashMap, str::FromStr};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_to_add_order_args() {
        let dotrain = get_dotrain();

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain).unwrap();

        let cli_order_add_args = CliOrderAddArgs {
            no_broadcast: false,
            dotrain_file: dotrain_path,
            deployment: "some-deployment".to_string(),
            transaction_args: CliTransactionArgs {
                raindex_address: Address::random(),
                derivation_index: None,
                chain_id: Some(123),
                rpcs: vec!["https://some-rpc.com".to_string()],
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
            },
        };

        let result = cli_order_add_args.to_add_order_args().await.unwrap();
        let expected = AddOrderArgs {
            dotrain: get_dotrain(),
            inputs: vec![IOV2 {
                token: address!("0xc2132d05d31c914a87c6611c10748aeb04b58e8f"),
                vaultId: B256::from(U256::from(1)),
            }],
            outputs: vec![IOV2 {
                token: address!("0x8f3cf7ad23cd3cadbd9735aff958023239c6a063"),
                vaultId: B256::from(U256::from(1)),
            }],
            rainlang: Address::from_str("0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba").unwrap(),
            bindings: HashMap::new(),
            additional_meta: None,
        };
        assert_eq!(result, expected);
    }

    fn get_dotrain() -> String {
        format!(
            r#"
version: {}
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

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token2
              vault-id: 1
        rainlang: some-rainlang
        raindex: some-raindex

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#,
            SpecVersion::current()
        )
    }
}
