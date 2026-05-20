use crate::{
    execute::Execute,
    subgraph::{CliFilterArgs, CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use raindex_common::{
    csv::TryIntoCsv,
    subgraph::SubgraphArgs,
    types::{FlattenError, TokenVaultFlattened, NO_SYMBOL},
};
use raindex_subgraph_client::SgPaginationArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultListArgs {
    #[clap(flatten)]
    pub pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,

    #[clap(flatten)]
    pub filter_args: CliFilterArgs,
}

impl Execute for CliVaultListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let vaults = subgraph_args
                .to_subgraph_client()?
                .vaults_list_all()
                .await?;
            let vaults_flattened: Vec<TokenVaultFlattened> = vaults
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<TokenVaultFlattened>, FlattenError>>()?;

            let csv_text = vaults_flattened.try_into_csv()?;
            println!("{}", csv_text);
        } else {
            let pagination_args: SgPaginationArgs = self.pagination_args.clone().into();
            let filter_args = self.filter_args.clone().into();
            let vaults = subgraph_args
                .to_subgraph_client()?
                .vaults_list(filter_args, pagination_args)
                .await?;
            let vaults_flattened: Vec<TokenVaultFlattened> = vaults
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<TokenVaultFlattened>, FlattenError>>()?;

            let table = build_table(vaults_flattened)?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(vaults: Vec<TokenVaultFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Owner", "Token", "Balance"]);

    for vault in vaults.iter() {
        table.add_row(vec![
            vault.id.clone(),
            format!("{}", vault.clone().owner.0),
            vault
                .clone()
                .token_symbol
                .unwrap_or(NO_SYMBOL.into())
                .clone(),
            format!("{}", vault.balance_display),
        ]);
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex::encode_prefixed,
        primitives::{Address, B256},
    };
    use httpmock::MockServer;
    use raindex_subgraph_client::utils::float::*;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_csv_execute_happy() {
        // mock subgraph with pagination
        let sg_server = MockServer::start();
        sg_server.mock(|when, then| {
            when.body_contains("\"skip\":0");
            then.json_body_obj(&get_sg_response());
        });
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({"data": {"vaults": []}}));
        });

        let cli_vault_list_args = CliVaultListArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            pagination_args: CliPaginationArgs {
                csv: true,
                page_size: 25,
                page: 1,
            },
            filter_args: CliFilterArgs {
                owners: vec!["addr1".to_string()],
                active: Some(true),
                hide_zero_balance: Some(true),
                order_hash: None,
                tokens: vec![],
            },
        };

        // should succeed
        cli_vault_list_args.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_no_csv_execute_happy() {
        // mock subgraph
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response());
        });

        let cli_vault_list_args = CliVaultListArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            pagination_args: CliPaginationArgs {
                csv: false,
                page_size: 25,
                page: 1,
            },
            filter_args: CliFilterArgs {
                owners: vec!["addr1".to_string()],
                active: Some(true),
                hide_zero_balance: Some(true),
                order_hash: None,
                tokens: vec![],
            },
        };

        // should succeed
        cli_vault_list_args.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        let cli_vault_list_args = CliVaultListArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: "https://bad-url".to_string(),
            },
            pagination_args: CliPaginationArgs {
                csv: false,
                page_size: 25,
                page: 1,
            },
            filter_args: CliFilterArgs {
                owners: vec!["addr1".to_string()],
                active: Some(true),
                hide_zero_balance: Some(true),
                order_hash: None,
                tokens: vec![],
            },
        };

        // should error
        cli_vault_list_args.execute().await.unwrap_err();
    }

    // helper function that returns mocked sg response in json
    fn get_sg_response() -> Value {
        json!({
            "data": {
                "vaults": [{
                    "id": encode_prefixed(B256::random()),
                    "vaultId": encode_prefixed(B256::random()),
                    "owner": encode_prefixed(Address::random()),
                    "balance": F0,
                    "token": {
                        "name": "T1",
                        "symbol": "T1",
                        "id": encode_prefixed(Address::random()),
                        "address": encode_prefixed(Address::random()),
                        "decimals": "6"
                    },
                    "ordersAsInput": [{
                        "id": encode_prefixed(B256::random()),
                        "orderHash": encode_prefixed(B256::random()),
                        "active": true,
                    }],
                    "ordersAsOutput": [{
                        "id": encode_prefixed(B256::random()),
                        "orderHash": encode_prefixed(B256::random()),
                        "active": true,
                    }],
                    "raindex": {
                        "id": encode_prefixed(B256::random()),
                    },
                    "balanceChanges": []
                }]
            }
        })
    }
}
