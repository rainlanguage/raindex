use crate::{execute::Execute, subgraph::CliSubgraphArgs};
use anyhow::Result;
use clap::Args;

use rain_orderbook_common::subgraph::SubgraphArgs;

use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderTradeDetailArgs {
    #[arg(short = 'i', long, help = "Subgraph ID of the Order")]
    id: String,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,
}

impl Execute for CliOrderTradeDetailArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let order_take = subgraph_args
            .to_subgraph_client()?
            .order_trade_detail(self.id.clone().into())
            .await?;
        info!("{:#?}", order_take);

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

        let cli_order_take_detail_args = CliOrderTradeDetailArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            id: encode_prefixed(B256::random()),
        };

        // should succeed
        assert!(cli_order_take_detail_args.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        // mock sg with corrupt response
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({"data": {"trade": null}}));
        });

        let cli_order_take_detail_args = CliOrderTradeDetailArgs {
            subgraph_args: CliSubgraphArgs {
                subgraph_url: sg_server.url("/sg"),
            },
            id: encode_prefixed(B256::random()),
        };

        // should error
        assert!(cli_order_take_detail_args.execute().await.is_err());
    }

    // helper function that returns mocked sg response in json
    fn get_sg_response() -> Value {
        json!({
            "data": {
                "trade": {
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
                        "amount": "0",
                        "__typename": "Withdraw",
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
                        "newVaultBalance": "0",
                        "oldVaultBalance": "0",
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
                        "amount": "0",
                        "__typename": "Withdraw",
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
                        "newVaultBalance": "0",
                        "oldVaultBalance": "0",
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
                }
            }
        })
    }
}
