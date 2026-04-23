use crate::{
    execute::Execute,
    subgraph::{CliPaginationArgs, CliSubgraphArgs},
};
use anyhow::Result;
use clap::Args;
use comfy_table::Table;
use rain_orderbook_common::{
    csv::TryIntoCsv,
    subgraph::SubgraphArgs,
    types::{FlattenError, OrderTakeFlattened, NO_SYMBOL},
};
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderTradesListArgs {
    #[arg(short = 'i', long, help = "ID of the Order")]
    order_id: String,

    #[clap(flatten)]
    pagination_args: CliPaginationArgs,

    #[clap(flatten)]
    subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderTradesListArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();

        if self.pagination_args.csv {
            let csv_text = subgraph_args
                .to_subgraph_client()?
                .order_trades_list_all(self.order_id.clone().into(), None, None)
                .await?
                .into_iter()
                .map(|o| o.try_into())
                .collect::<Result<Vec<OrderTakeFlattened>, FlattenError>>()?
                .try_into_csv()?;

            println!("{}", csv_text);
        } else {
            let table = build_table(
                subgraph_args
                    .to_subgraph_client()?
                    .order_trades_list(
                        self.order_id.clone().into(),
                        self.pagination_args.clone().into(),
                        None,
                        None,
                    )
                    .await?
                    .into_iter()
                    .map(|o| o.try_into())
                    .collect::<Result<Vec<OrderTakeFlattened>, FlattenError>>()?,
            )?;

            info!("\n{}", table);
        }

        Ok(())
    }
}

fn build_table(order_take: Vec<OrderTakeFlattened>) -> Result<Table> {
    let mut table = comfy_table::Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Taken At", "Sender", "Input", "Output"]);

    for order_take in order_take.into_iter() {
        table.add_row(vec![
            order_take.id,
            order_take.timestamp_display,
            order_take.sender.0,
            format!(
                "{} {}",
                order_take.input_display,
                order_take.input_token_symbol.unwrap_or(NO_SYMBOL.into())
            ),
            format!(
                "{} {}",
                order_take.output_display,
                order_take.output_token_symbol.unwrap_or(NO_SYMBOL.into())
            ),
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
    use rain_orderbook_subgraph_client::utils::float::*;
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
            then.json_body_obj(&json!({"data": {"trades": []}}));
        });

        let cli_order_take_list_args = CliOrderTradesListArgs {
            order_id: encode_prefixed(B256::random()),
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
        assert!(cli_order_take_list_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_no_csv_execute_happy() {
        // mock subgraph
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&get_sg_response());
        });

        let cli_order_take_list_args = CliOrderTradesListArgs {
            order_id: encode_prefixed(B256::random()),
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
        cli_order_take_list_args.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        let cli_order_take_list_args = CliOrderTradesListArgs {
            order_id: encode_prefixed(B256::random()),
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
        cli_order_take_list_args.execute().await.unwrap_err();
    }

    // helper function that returns mocked sg response in json
    fn get_sg_response() -> Value {
        json!({
            "data": {
                "trades": [{
                    "id": encode_prefixed(B256::random()),
                    "order": {
                        "id": encode_prefixed(B256::random()),
                        "owner": encode_prefixed(B256::random()),
                        "orderHash": encode_prefixed(B256::random()),
                        "orderBytes": encode_prefixed(B256::random()),
                        "outputs": [],
                        "inputs": [],
                        "orderbook": {
                            "id": encode_prefixed(B256::random()),
                        },
                        "meta": null,
                        "active": true,
                        "timestampAdded": "0",
                        "addEvents": [],
                    },
                    "outputVaultBalanceChange": {
                        "id": encode_prefixed(B256::random()),
                        "amount": F0,
                        "__typename": "TradeVaultBalanceChange",
                        "vault": {
                            "id": encode_prefixed(B256::random()),
                            "vaultId": encode_prefixed(B256::random()),
                            "token": {
                                "name": "T1",
                                "symbol": "T1",
                                "id": encode_prefixed(Address::random()),
                                "address": encode_prefixed(Address::random()),
                                "decimals": "6"
                            }
                        },
                        "newVaultBalance": F0,
                        "oldVaultBalance": F0,
                        "timestamp": "0",
                        "transaction": {
                            "id": encode_prefixed(B256::random()),
                            "blockNumber": "0",
                            "timestamp": "0",
                            "from": encode_prefixed(Address::random())
                        },
                        "orderbook": {
                            "id": encode_prefixed(B256::random()),
                        },
                        "trade": {
                            "tradeEvent": {
                                "__typename": "TakeOrder"
                            }
                        },
                    },
                    "inputVaultBalanceChange": {
                        "id": encode_prefixed(B256::random()),
                        "amount": F0,
                        "__typename": "TradeVaultBalanceChange",
                        "vault": {
                            "id": encode_prefixed(B256::random()),
                            "vaultId": encode_prefixed(B256::random()),
                            "token": {
                                "name": "T2",
                                "symbol": "T2",
                                "id": encode_prefixed(Address::random()),
                                "address": encode_prefixed(Address::random()),
                                "decimals": "18"
                            }
                        },
                        "newVaultBalance": F0,
                        "oldVaultBalance": F0,
                        "timestamp": "0",
                        "transaction": {
                            "id": encode_prefixed(B256::random()),
                            "blockNumber": "0",
                            "timestamp": "0",
                            "from": encode_prefixed(Address::random())
                        },
                        "orderbook": {
                            "id": encode_prefixed(B256::random()),
                        },
                        "trade": {
                            "tradeEvent": {
                                "__typename": "TakeOrder"
                            }
                        },
                    },
                    "timestamp": "0",
                    "tradeEvent": {
                        "__typename": "TakeOrder",
                        "sender": encode_prefixed(Address::random()),
                        "transaction": {
                            "id": encode_prefixed(B256::random()),
                            "blockNumber": "0",
                            "timestamp": "0",
                            "from": encode_prefixed(Address::random())
                        },
                        "trades": []
                    },
                    "orderbook": {
                        "id": encode_prefixed(B256::random()),
                    },
                }]
            }
        })
    }
}
