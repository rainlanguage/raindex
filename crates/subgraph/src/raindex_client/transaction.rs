use super::*;

impl RaindexSubgraphClient {
    pub async fn transaction_detail(
        &self,
        id: Id,
    ) -> Result<SgTransaction, RaindexSubgraphClientError> {
        let data = self
            .query::<SgTransactionDetailQuery, SgIdQueryVariables>(SgIdQueryVariables { id: &id })
            .await?;
        let transaction = data.transaction.ok_or(RaindexSubgraphClientError::Empty)?;
        Ok(transaction)
    }

    /// Fetch all add orders for a given transaction
    pub async fn transaction_add_orders(
        &self,
        id: Id,
    ) -> Result<Vec<SgAddOrderWithOrder>, RaindexSubgraphClientError> {
        let data = self
            .query::<SgTransactionAddOrdersQuery, TransactionAddOrdersVariables>(
                TransactionAddOrdersVariables {
                    id: SgBytes(id.inner().to_string()),
                },
            )
            .await?;

        if data.add_orders.is_empty() {
            return Err(RaindexSubgraphClientError::Empty);
        }

        Ok(data.add_orders)
    }

    /// Fetch all remove orders for a given transaction
    pub async fn transaction_remove_orders(
        &self,
        id: Id,
    ) -> Result<Vec<SgRemoveOrderWithOrder>, RaindexSubgraphClientError> {
        let data = self
            .query::<SgTransactionRemoveOrdersQuery, TransactionRemoveOrdersVariables>(
                TransactionRemoveOrdersVariables {
                    id: SgBytes(id.inner().to_string()),
                },
            )
            .await?;

        if data.remove_orders.is_empty() {
            return Err(RaindexSubgraphClientError::Empty);
        }

        Ok(data.remove_orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{
        SgAddOrderWithOrder, SgBigInt, SgBytes, SgErc20, SgOrder, SgRaindex,
        SgRemoveOrderWithOrder, SgTransaction, SgVault,
    };
    use crate::utils::float::*;
    use cynic::Id;
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;

    fn setup_client(server: &MockServer) -> RaindexSubgraphClient {
        let url = Url::parse(&server.url("")).unwrap();
        RaindexSubgraphClient::new(url)
    }

    fn default_sg_transaction(id_str: &str) -> SgTransaction {
        SgTransaction {
            id: SgBytes(id_str.to_string()),
            from: SgBytes("0xfrom_default_address".to_string()),
            block_number: SgBigInt("123456".to_string()),
            timestamp: SgBigInt("1600000000".to_string()),
        }
    }

    fn default_sg_erc20(id_suffix: &str) -> SgErc20 {
        SgErc20 {
            id: SgBytes(format!("0xtoken_id_{}", id_suffix)),
            address: SgBytes(format!("0xtoken_address_{}", id_suffix)),
            name: Some(format!("Token {}", id_suffix.to_uppercase())),
            symbol: Some(format!("TKN{}", id_suffix.to_uppercase())),
            decimals: Some(SgBigInt("18".to_string())),
        }
    }

    fn default_sg_order(order_id_str: &str) -> SgOrder {
        SgOrder {
            id: SgBytes(order_id_str.to_string()),
            order_hash: SgBytes(format!("0xhash_{}", order_id_str)),
            owner: SgBytes("0xowner_default_order".to_string()),
            order_bytes: SgBytes("0xorderbytes_default_order".to_string()),
            timestamp_added: SgBigInt("1600000100".to_string()),
            active: true,
            raindex: SgRaindex {
                id: SgBytes("0xraindex_default_order".to_string()),
            },
            inputs: vec![SgVault {
                id: SgBytes("input_vault_id_order".to_string()),
                owner: SgBytes("0xowner_default_order".to_string()),
                vault_id: SgBytes("input_vault_sg_id_order".to_string()),
                balance: SgBytes(F1000.as_hex()),
                token: default_sg_erc20("input_order"),
                raindex: SgRaindex {
                    id: SgBytes("0xraindex_default_order".to_string()),
                },
                orders_as_output: vec![],
                orders_as_input: vec![],
                balance_changes: vec![],
            }],
            outputs: vec![SgVault {
                id: SgBytes("output_vault_id_order".to_string()),
                owner: SgBytes("0xowner_default_order".to_string()),
                vault_id: SgBytes("output_vault_sg_id_order".to_string()),
                balance: SgBytes(F0.as_hex()),
                token: default_sg_erc20("output_order"),
                raindex: SgRaindex {
                    id: SgBytes("0xraindex_default_order".to_string()),
                },
                orders_as_output: vec![],
                orders_as_input: vec![],
                balance_changes: vec![],
            }],
            meta: None,
            add_events: vec![],
            trades: vec![],
            remove_events: vec![],
        }
    }

    fn default_sg_add_order_with_order(tx_id_str: &str, order_id_str: &str) -> SgAddOrderWithOrder {
        SgAddOrderWithOrder {
            transaction: default_sg_transaction(tx_id_str),
            order: default_sg_order(order_id_str),
        }
    }

    fn default_sg_remove_order_with_order(
        tx_id_str: &str,
        order_id_str: &str,
    ) -> SgRemoveOrderWithOrder {
        SgRemoveOrderWithOrder {
            transaction: default_sg_transaction(tx_id_str),
            order: default_sg_order(order_id_str),
        }
    }

    fn assert_sg_transaction_eq(actual: &SgTransaction, expected: &SgTransaction) {
        assert_eq!(actual.id, expected.id, "Transaction ID mismatch");
        assert_eq!(actual.from, expected.from, "Transaction from mismatch");
        assert_eq!(
            actual.block_number, expected.block_number,
            "Transaction block_number mismatch"
        );
        assert_eq!(
            actual.timestamp, expected.timestamp,
            "Transaction timestamp mismatch"
        );
    }

    fn assert_sg_order_eq_brief(actual: &SgOrder, expected: &SgOrder) {
        assert_eq!(actual.id, expected.id, "Order ID mismatch in event");
        assert_eq!(
            actual.order_hash, expected.order_hash,
            "Order hash mismatch in event"
        );
    }

    fn assert_sg_add_order_with_order_eq(
        actual: &SgAddOrderWithOrder,
        expected: &SgAddOrderWithOrder,
    ) {
        assert_sg_transaction_eq(&actual.transaction, &expected.transaction);
        assert_sg_order_eq_brief(&actual.order, &expected.order);
    }

    fn assert_sg_remove_order_with_order_eq(
        actual: &SgRemoveOrderWithOrder,
        expected: &SgRemoveOrderWithOrder,
    ) {
        assert_sg_transaction_eq(&actual.transaction, &expected.transaction);
        assert_sg_order_eq_brief(&actual.order, &expected.order);
    }

    #[tokio::test]
    async fn test_transaction_detail_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id_str = "0xtx123";
        let tx_id = Id::new(tx_id_str);
        let expected_tx = default_sg_transaction(tx_id_str);

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"transaction": expected_tx}}));
        });

        let result = client.transaction_detail(tx_id).await;
        assert!(result.is_ok(), "Result should be Ok");
        assert_sg_transaction_eq(&result.unwrap(), &expected_tx);
    }

    #[tokio::test]
    async fn test_transaction_detail_not_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id = Id::new("0xtx_notfound");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"transaction": null}}));
        });

        let result = client.transaction_detail(tx_id).await;
        assert!(matches!(result, Err(RaindexSubgraphClientError::Empty)));
    }

    #[tokio::test]
    async fn test_transaction_detail_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id = Id::new("0xtx_network_error");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.transaction_detail(tx_id).await;
        assert!(matches!(
            result,
            Err(RaindexSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_transaction_add_orders_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id_str = "0xtx_add_orders_1";
        let tx_id = Id::new(tx_id_str);
        let expected_add_orders = vec![
            default_sg_add_order_with_order(tx_id_str, "order1"),
            default_sg_add_order_with_order(tx_id_str, "order2"),
        ];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", tx_id_str)); // Check variable being sent
            then.status(200)
                .json_body(json!({"data": {"addOrders": expected_add_orders}}));
        });

        let result = client.transaction_add_orders(tx_id).await;
        assert!(result.is_ok(), "Result was: {:?}", result);
        let add_orders = result.unwrap();
        assert_eq!(add_orders.len(), expected_add_orders.len());
        for (actual, expected) in add_orders.iter().zip(expected_add_orders.iter()) {
            assert_sg_add_order_with_order_eq(actual, expected);
        }
    }

    #[tokio::test]
    async fn test_transaction_add_orders_empty_result_returns_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id = Id::new("0xtx_add_orders_empty");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"addOrders": []}}));
        });

        let result = client.transaction_add_orders(tx_id).await;
        assert!(matches!(result, Err(RaindexSubgraphClientError::Empty)));
    }

    #[tokio::test]
    async fn test_transaction_add_orders_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id = Id::new("0xtx_add_orders_network_err");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.transaction_add_orders(tx_id).await;
        assert!(matches!(
            result,
            Err(RaindexSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_transaction_remove_orders_found() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id_str = "0xtx_remove_orders_1";
        let tx_id = Id::new(tx_id_str);
        let expected_remove_orders = vec![
            default_sg_remove_order_with_order(tx_id_str, "order3"),
            default_sg_remove_order_with_order(tx_id_str, "order4"),
        ];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", tx_id_str)); // Check variable
            then.status(200)
                .json_body(json!({"data": {"removeOrders": expected_remove_orders}}));
        });

        let result = client.transaction_remove_orders(tx_id).await;
        assert!(result.is_ok(), "Result was: {:?}", result);
        let remove_orders = result.unwrap();
        assert_eq!(remove_orders.len(), expected_remove_orders.len());
        for (actual, expected) in remove_orders.iter().zip(expected_remove_orders.iter()) {
            assert_sg_remove_order_with_order_eq(actual, expected);
        }
    }

    #[tokio::test]
    async fn test_transaction_remove_orders_empty_result_returns_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id = Id::new("0xtx_remove_orders_empty");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"removeOrders": []}}));
        });

        let result = client.transaction_remove_orders(tx_id).await;
        assert!(matches!(result, Err(RaindexSubgraphClientError::Empty)));
    }

    #[tokio::test]
    async fn test_transaction_remove_orders_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let tx_id = Id::new("0xtx_remove_orders_network_err");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.transaction_remove_orders(tx_id).await;
        assert!(matches!(
            result,
            Err(RaindexSubgraphClientError::CynicClientError(_))
        ));
    }
}
