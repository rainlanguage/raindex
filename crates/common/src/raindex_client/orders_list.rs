use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::prelude::*;

use crate::raindex_client::orders::RaindexOrder;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[wasm_bindgen]
pub struct RaindexOrders(Vec<RaindexOrder>);

impl RaindexOrders {
    pub fn inner(&self) -> &[RaindexOrder] {
        &self.0
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
impl RaindexOrders {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, order: &RaindexOrder) {
        self.0.push(order.clone());
    }

    #[wasm_bindgen(getter)]
    pub fn items(&self) -> Vec<RaindexOrder> {
        self.0.clone()
    }
}

#[cfg(not(target_family = "wasm"))]
impl RaindexOrders {
    pub fn new(orders: Vec<RaindexOrder>) -> Self {
        Self(orders)
    }

    pub fn push(&mut self, order: &RaindexOrder) {
        self.0.push(order.clone());
    }

    pub fn items(&self) -> Vec<RaindexOrder> {
        self.0.clone()
    }
}

#[cfg(test)]
#[cfg(not(target_family = "wasm"))]
mod tests {
    use super::*;
    use crate::local_db::OrderbookIdentifier;
    use crate::raindex_client::tests::{get_test_yaml, CHAIN_ID_1_ORDERBOOK_ADDRESS};
    use alloy::primitives::{b256, Address};
    use httpmock::MockServer;
    use serde_json::json;
    use std::str::FromStr;

    async fn make_test_order() -> RaindexOrder {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.path("/sg");
            then.status(200).json_body_obj(&json!({
                "data": {
                    "orders": [{
                        "id": "0x46891c626a8a188610b902ee4a0ce8a7e81915e1b922584f8168d14525899dfb",
                        "orderBytes": "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000005f6c104ca9812ef91fe2e26a2e7187b92d3b0e800000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000022009cd210f509c66e18fab61fd30f76fb17c6c6cd09f0972ce0815b5b7630a1b050000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae600000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000075000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c02020002011000000110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012f5bb1bfe104d351d99dcce1ccfb041ff244a2d3aaf83bd5c4f3fe20b3fceb372000000000000000000000000000000000000000000000000000000000000000100000000000000000000000012e605bc104e93b45e1ad99f9e555f659051c2bb0000000000000000000000000000000000000000000000000000000000000012f5bb1bfe104d351d99dcce1ccfb041ff244a2d3aaf83bd5c4f3fe20b3fceb372",
                        "orderHash": "0x283508c8f56f4de2f21ee91749d64ec3948c16bc6b4bfe4f8d11e4e67d76f4e0",
                        "owner": "0x0000000000000000000000000000000000000000",
                        "outputs": [{
                            "id": "0x0000000000000000000000000000000000000000",
                            "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                            "vaultId": "0x01",
                            "balance": "0x0000000000000000000000000000000000000000000000000000000000000000",
                            "token": {
                                "id": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                                "address": "0x12e605bc104e93b45e1ad99f9e555f659051c2bb",
                                "name": "sFLR", "symbol": "sFLR", "decimals": "18"
                            },
                            "orderbook": { "id": CHAIN_ID_1_ORDERBOOK_ADDRESS },
                            "ordersAsOutput": [], "ordersAsInput": [], "balanceChanges": []
                        }],
                        "inputs": [{
                            "id": "0x0000000000000000000000000000000000000000",
                            "owner": "0xf08bcbce72f62c95dcb7c07dcb5ed26acfcfbc11",
                            "vaultId": "0x01",
                            "balance": "0x0000000000000000000000000000000000000000000000000000000000000000",
                            "token": {
                                "id": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                "address": "0x1d80c49bbbcd1c0911346656b529df9e5c2f783d",
                                "name": "WFLR", "symbol": "WFLR", "decimals": "18"
                            },
                            "orderbook": { "id": CHAIN_ID_1_ORDERBOOK_ADDRESS },
                            "ordersAsOutput": [], "ordersAsInput": [], "balanceChanges": []
                        }],
                        "orderbook": { "id": CHAIN_ID_1_ORDERBOOK_ADDRESS },
                        "active": true, "timestampAdded": "0", "meta": null,
                        "addEvents": [], "trades": [], "removeEvents": []
                    }]
                }
            }));
        });

        let client = crate::raindex_client::RaindexClient::new(
            vec![get_test_yaml(
                &server.url("/sg"),
                "http://localhost:3000",
                "http://localhost:3000",
                "http://localhost:3000",
            )],
            None,
            None,
        )
        .await
        .unwrap();
        client
            .get_order_by_hash(
                &OrderbookIdentifier::new(
                    1,
                    Address::from_str(CHAIN_ID_1_ORDERBOOK_ADDRESS).unwrap(),
                ),
                b256!("0x0000000000000000000000000000000000000000000000000000000000000123"),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_new_and_items() {
        let order = make_test_order().await;
        let list = RaindexOrders::new(vec![order.clone()]);
        let items = list.items();
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    async fn test_new_empty() {
        let list = RaindexOrders::new(vec![]);
        assert!(list.items().is_empty());
        assert!(list.inner().is_empty());
    }

    #[tokio::test]
    async fn test_push() {
        let order = make_test_order().await;
        let mut list = RaindexOrders::new(vec![]);
        assert!(list.inner().is_empty());

        list.push(&order);
        assert_eq!(list.items().len(), 1);

        list.push(&order);
        assert_eq!(list.items().len(), 2);
    }

    #[tokio::test]
    async fn test_inner() {
        let order = make_test_order().await;
        let list = RaindexOrders::new(vec![order.clone(), order]);
        assert_eq!(list.inner().len(), 2);
    }
}
