use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use raindex_common::{
    csv::TryIntoCsv,
    subgraph::SubgraphArgs,
    types::{FlattenError, VaultBalanceChangeFlattened},
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultBalanceChangesList {
    #[arg(short = 'i', long, help = "ID of the Vault")]
    vault_id: String,

    #[clap(flatten)]
    pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultBalanceChangesList {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let csv_text = subgraph_args
                .to_subgraph_client()?
                .vault_balance_changes_list_all(self.vault_id.clone().into(), None)
                .await?
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<VaultBalanceChangeFlattened>, FlattenError>>()?
                .try_into_csv()?;

            println!("{}", csv_text);
        } else {
            let table = build_table(
                subgraph_args
                    .to_subgraph_client()?
                    .vault_balance_changes_list(
                        self.vault_id.clone().into(),
                        self.pagination_args.clone().into(),
                        None,
                    )
                    .await?
                    .into_iter()
                    .map(|o| o.try_into())
                    .collect::<Result<Vec<VaultBalanceChangeFlattened>, FlattenError>>()?,
            )?;

            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(balance_change: Vec<VaultBalanceChangeFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "Changed At",
            "Sender",
            "Balance Change",
            "Change Type",
        ]);

    for balance_change in balance_change.into_iter() {
        table.add_row(vec![
            balance_change.timestamp_display,
            balance_change.from.0,
            balance_change.amount_display_signed,
            balance_change.change_type_display,
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
            then.json_body_obj(&json!({"data": {"vaultBalanceChanges": []}}));
        });

        let cli_vault_balance_changes_list_args = CliVaultBalanceChangesList {
            vault_id: encode_prefixed(B256::random()),
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            pagination_args: CliPaginationArgs {
                csv: true,
                page_size: 25,
                page: 1,
            },
        };

        // should succeed
        assert!(cli_vault_balance_changes_list_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_no_csv_execute_happy() {
        // mock subgraph
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response());
        });

        let cli_vault_balance_changes_list_args = CliVaultBalanceChangesList {
            vault_id: encode_prefixed(B256::random()),
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            pagination_args: CliPaginationArgs {
                csv: false,
                page_size: 25,
                page: 1,
            },
        };

        // should succeed
        cli_vault_balance_changes_list_args.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        let cli_vault_balance_changes_list_args = CliVaultBalanceChangesList {
            vault_id: encode_prefixed(B256::random()),
            subgraph_args: CliSubgraphArgs {
                subgraph_url: "https://bad-url".to_string(),
            },
            pagination_args: CliPaginationArgs {
                csv: false,
                page_size: 25,
                page: 1,
            },
        };

        // should error
        assert!(cli_vault_balance_changes_list_args.execute().await.is_err());
    }

    fn get_sg_response() -> Value {
        json!({
            "data": {
                "vaultBalanceChanges": [{
                    "id": encode_prefixed(B256::random()),
                    "__typename": "Deposit",
                    "amount": F0,
                    "newVaultBalance": F0,
                    "oldVaultBalance": F0,
                    "vault": {
                        "id": encode_prefixed(B256::random()),
                        "vaultId": encode_prefixed(B256::random()),
                        "token": {
                            "name": "T1",
                            "symbol": "T1",
                            "id": encode_prefixed(Address::random()),
                            "address": encode_prefixed(Address::random()),
                            "decimals": "6"
                        },
                    },
                    "transaction": {
                        "id": encode_prefixed(B256::random()),
                        "blockNumber": "0",
                        "timestamp": "0",
                        "from": encode_prefixed(Address::random())
                    },
                    "timestamp": "0",
                    "raindex": {
                        "id": encode_prefixed(B256::random()),
                    },
                }]
            }
        })
    }
}
