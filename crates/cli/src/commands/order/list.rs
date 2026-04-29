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
    types::{FlattenError, OrderFlattened, LIST_DELIMITER},
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderListArgs {
    #[clap(flatten)]
    pub pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,

    #[clap(flatten)]
    pub filter_args: CliFilterArgs,
}

impl Execute for CliOrderListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let csv_text = subgraph_args
                .to_subgraph_client()?
                .orders_list_all()
                .await?
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderFlattened>, FlattenError>>()?
                .try_into_csv()?;

            println!("{}", csv_text);
        } else {
            let table = build_table(
                subgraph_args
                    .to_subgraph_client()?
                    .orders_list(
                        self.filter_args.clone().into(),
                        self.pagination_args.clone().into(),
                    )
                    .await?
                    .into_iter()
                    .map(|o| o.try_into())
                    .collect::<Result<Vec<OrderFlattened>, FlattenError>>()?,
            )?;
            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(orders: Vec<OrderFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec![
            "Order ID",
            "Added At",
            "Active",
            "Owner",
            "Input Tokens",
            "Output Tokens",
            "Trades",
        ]);

    for order in orders.into_iter() {
        table.add_row(vec![
            order.id,
            order.timestamp_display,
            order.order_active.to_string(),
            order.owner.0,
            order.valid_inputs_token_symbols_display,
            order.valid_outputs_token_symbols_display,
            order
                .trades
                .split(LIST_DELIMITER)
                .collect::<Vec<&str>>()
                .len()
                .to_string(),
        ]);
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{hex::encode_prefixed, primitives::B256, sol_types::SolValue};
    use httpmock::MockServer;
    use raindex_bindings::IRaindexV6::{OrderV4, IOV2};
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_csv_execute_happy() {
        // mock subgraph with pagination
        let sg_server = MockServer::start();
        sg_server.mock(|when, then| {
            when.body_contains("\"skip\":0");
            then.json_body_obj(&get_sg_response(false));
        });
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({"data": {"orders": []}}));
        });

        let cli_order_list_args = CliOrderListArgs {
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
                hide_zero_balance: None,
                order_hash: None,
                tokens: vec![],
            },
        };

        // should succeed
        assert!(cli_order_list_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_no_csv_execute_happy() {
        // mock subgraph
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response(false));
        });

        let cli_order_list_args = CliOrderListArgs {
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
                hide_zero_balance: None,
                order_hash: None,
                tokens: vec![],
            },
        };

        // should succeed
        assert!(cli_order_list_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        // mock sg with corrupt response
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response(true));
        });

        let cli_order_list_args = CliOrderListArgs {
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
                hide_zero_balance: None,
                order_hash: None,
                tokens: vec![],
            },
        };

        // should error
        assert!(cli_order_list_args.execute().await.is_err());
    }

    // helper function that returns mocked sg response in json
    fn get_sg_response(corrupt: bool) -> Value {
        let io = IOV2::default();
        let order = OrderV4 {
            validInputs: vec![io.clone()],
            validOutputs: vec![io.clone()],
            ..Default::default()
        };
        json!({
            "data": {
                "orders": [{
                    "id": encode_prefixed(B256::random()),
                    "owner": encode_prefixed(order.owner),
                    "orderHash": encode_prefixed(B256::random()),
                    "orderBytes": if corrupt {
                        // set a corrupt order bytes
                        encode_prefixed(vec![])
                    } else {
                        // set a valid order bytes
                        encode_prefixed(order.abi_encode())
                    },
                    "outputs": [{
                        "id": encode_prefixed(B256::random()),
                        "balance": "0",
                        "vaultId": io.vaultId.to_string(),
                        "token": {
                            "name": "T1",
                            "symbol": "T1",
                            "id": encode_prefixed(io.token),
                            "address": encode_prefixed(io.token),
                            "decimals": "0",
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
                            "decimals": "0",
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
                }]
            }
        })
    }
}
