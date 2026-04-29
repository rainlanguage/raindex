use crate::{execute::Execute, subgraph::CliSubgraphArgs};
use anyhow::Result;
use clap::Args;
use raindex_common::{subgraph::SubgraphArgs, types::OrderDetailExtended};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderDetailArgs {
    #[arg(short = 'i', long, help = "ID of the Order")]
    order_id: String,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderDetailArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let order = subgraph_args
            .to_subgraph_client()?
            .order_detail(&self.order_id.clone().into())
            .await?;
        let order_extended: OrderDetailExtended = order.try_into()?;
        info!("{:#?}", order_extended);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{hex::encode_prefixed, primitives::B256, sol_types::SolValue};
    use httpmock::MockServer;
    use raindex_bindings::IRaindexV6::{OrderV4, IOV2};
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_execute_happy() {
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response());
        });

        let cli_order_detail_args = CliOrderDetailArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            order_id: encode_prefixed(B256::random()),
        };

        // should succeed
        assert!(cli_order_detail_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        // mock sg with corrupt response
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({"data": {"order": null}}));
        });

        let cli_order_detail_args = CliOrderDetailArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            order_id: encode_prefixed(B256::random()),
        };

        // should error
        assert!(cli_order_detail_args.execute().await.is_err());
    }

    // helper function that returns mocked sg response in json
    fn get_sg_response() -> Value {
        let io = IOV2::default();
        let order = OrderV4 {
            validInputs: vec![io.clone()],
            validOutputs: vec![io.clone()],
            ..Default::default()
        };

        json!({
            "data": {
                "order": {
                    "id": encode_prefixed(B256::random()),
                    "owner": encode_prefixed(order.owner),
                    "orderHash": encode_prefixed(B256::random()),
                    "orderBytes": encode_prefixed(order.abi_encode()),
                    "outputs": [{
                        "id": encode_prefixed(B256::random()),
                        "balance": "0",
                        "vaultId": io.vaultId.to_string(),
                        "token": {
                            "name": "T1",
                            "symbol": "T1",
                            "id": encode_prefixed(io.token),
                            "address": encode_prefixed(io.token),
                            "decimals": "0"
                        },
                        "raindex": { "id": encode_prefixed(B256::random()) },
                        "owner": encode_prefixed(order.owner),
                        "ordersAsOutput": [],
                        "ordersAsInput": [],
                        "balanceChanges": []
                    }],
                    "inputs": [{
                        "id": encode_prefixed(B256::random()),
                        "balance": "0",
                        "vaultId": io.vaultId.to_string(),
                        "token": {
                            "name": "T2",
                            "symbol": "T2",
                            "id": encode_prefixed(io.token),
                            "address": encode_prefixed(io.token),
                            "decimals": "0"
                        },
                        "raindex": { "id": encode_prefixed(B256::random()) },
                        "owner": encode_prefixed(order.owner),
                        "ordersAsOutput": [],
                        "ordersAsInput": [],
                        "balanceChanges": []
                    }],
                    "raindex": {
                        "id": encode_prefixed(B256::random()),
                    },
                    "meta": null,
                    "active": true,
                    "timestampAdded": "0",
                    "addEvents": [{
                        "transaction": {
                            "id": encode_prefixed(B256::random()),
                            "blockNumber": "0",
                            "timestamp": "0",
                            "from": encode_prefixed(alloy::primitives::Address::random())
                        }
                    }],
                    "trades": [],
                    "removeEvents": []
                }
            }
        })
    }
}
