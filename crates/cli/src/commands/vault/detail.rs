use crate::{execute::Execute, subgraph::CliSubgraphArgs};
use anyhow::Result;
use clap::Args;
use raindex_common::subgraph::SubgraphArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliVaultDetailArgs {
    #[arg(short = 'i', long, help = "ID of the Vault")]
    vault_id: String,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliVaultDetailArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let vault = subgraph_args
            .to_subgraph_client()?
            .vault_detail(self.vault_id.clone().into())
            .await?;
        info!("{:#?}", vault);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex::encode_prefixed,
        primitives::{Address, B256},
    };
    use httpmock::MockServer;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_execute_happy() {
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response());
        });

        let cli_vault_detail_args = CliVaultDetailArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            vault_id: encode_prefixed(B256::random()),
        };

        // should succeed
        assert!(cli_vault_detail_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        // mock sg with corrupt response
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({"data": {"vault": null}}));
        });

        let cli_vault_detail_args = CliVaultDetailArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            vault_id: encode_prefixed(B256::random()),
        };

        // should error
        assert!(cli_vault_detail_args.execute().await.is_err());
    }

    // helper function that returns mocked sg response in json
    fn get_sg_response() -> Value {
        json!({
            "data": {
                "vault": {
                    "id": encode_prefixed(B256::random()),
                    "vaultId": encode_prefixed(B256::random()),
                    "owner": encode_prefixed(Address::random()),
                    "balance": "0",
                    "token": {
                        "name": "T1",
                        "symbol": "T1",
                        "id": encode_prefixed(Address::random()),
                        "address": encode_prefixed(Address::random()),
                        "decimals": "6"
                    },
                    "ordersAsInput": [{
                        "active": true,
                        "id": encode_prefixed(B256::random()),
                        "orderHash": encode_prefixed(B256::random()),
                    }],
                    "ordersAsOutput": [{
                        "active": true,
                        "id": encode_prefixed(B256::random()),
                        "orderHash": encode_prefixed(B256::random()),
                    }],
                    "balanceChanges": [],
                    "raindex": {
                        "id": encode_prefixed(B256::random()),
                    },
                    "balanceChanges": []
                }
            }
        })
    }
}
