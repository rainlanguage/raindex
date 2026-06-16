use crate::{
    types::common::{
        SgErc20WithSubgraphName, SgOrderWithSubgraphName, SgOrdersListFilterArgs,
        SgVaultWithSubgraphName, SgVaultsListFilterArgs,
    },
    OrderbookSubgraphClient, OrderbookSubgraphClientError, SgPaginationArgs,
};
use futures::future::join_all;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
pub struct MultiSubgraphArgs {
    #[cfg_attr(target_family = "wasm", tsify(type = "string"))]
    pub url: Url,
    pub name: String,
}
impl_wasm_traits!(MultiSubgraphArgs);

pub struct MultiOrderbookSubgraphClient {
    subgraphs: Vec<MultiSubgraphArgs>,
}
impl MultiOrderbookSubgraphClient {
    pub fn new(subgraphs: Vec<MultiSubgraphArgs>) -> Self {
        Self { subgraphs }
    }

    fn get_orderbook_subgraph_client(&self, url: Url) -> OrderbookSubgraphClient {
        OrderbookSubgraphClient::new(url)
    }

    pub async fn orders_list(
        &self,
        filter_args: SgOrdersListFilterArgs,
        pagination_args: SgPaginationArgs,
    ) -> Vec<SgOrderWithSubgraphName> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            let pagination_args = pagination_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let orders = client.orders_list(filter_args, pagination_args).await?;
                let wrapped_orders: Vec<SgOrderWithSubgraphName> = orders
                    .into_iter()
                    .map(|order| SgOrderWithSubgraphName {
                        order,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_orders)
            }
        });

        let results = join_all(futures).await;

        let mut all_orders: Vec<SgOrderWithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        all_orders.sort_by(|a, b| {
            let a_timestamp = a.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            let b_timestamp = b.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            b_timestamp.cmp(&a_timestamp)
        });

        all_orders
    }

    pub async fn orders_count(
        &self,
        filter_args: SgOrdersListFilterArgs,
    ) -> Result<u32, OrderbookSubgraphClientError> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                client.orders_count(filter_args).await
            }
        });

        let results = join_all(futures).await;
        let mut total: u32 = 0;
        for result in results {
            total += result?;
        }
        Ok(total)
    }

    pub async fn vaults_list(
        &self,
        filter_args: SgVaultsListFilterArgs,
        pagination_args: SgPaginationArgs,
    ) -> Vec<SgVaultWithSubgraphName> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            let pagination_args = pagination_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let vaults = client.vaults_list(filter_args, pagination_args).await?;
                let wrapped_vaults: Vec<SgVaultWithSubgraphName> = vaults
                    .into_iter()
                    .map(|vault| SgVaultWithSubgraphName {
                        vault,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_vaults)
            }
        });

        let results = join_all(futures).await;

        let all_vaults: Vec<SgVaultWithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        all_vaults
    }

    pub async fn tokens_list(&self) -> Vec<SgErc20WithSubgraphName> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let tokens = client.tokens_list_all().await?;
                let wrapped_tokens: Vec<SgErc20WithSubgraphName> = tokens
                    .into_iter()
                    .map(|token| SgErc20WithSubgraphName {
                        token,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_tokens)
            }
        });

        let results = join_all(futures).await;

        let all_tokens: Vec<SgErc20WithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        all_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrder, SgOrderbook, SgOrdersListFilterArgs, SgVault,
    };
    use crate::utils::float::*;
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;

    fn sample_sg_order(id_suffix: &str, timestamp: &str) -> SgOrder {
        SgOrder {
            id: SgBytes(format!("0xorder_id_{}", id_suffix)),
            order_bytes: SgBytes("0x00".to_string()),
            order_hash: SgBytes(format!("0xhash_{}", id_suffix)),
            owner: SgBytes("0xdefault_owner".to_string()),
            outputs: vec![],
            inputs: vec![],
            orderbook: SgOrderbook {
                id: SgBytes("0xdefault_orderbook_id".to_string()),
            },
            active: true,
            timestamp_added: SgBigInt(timestamp.to_string()),
            meta: None,
            add_events: vec![],
            trades: vec![],
            remove_events: vec![],
        }
    }

    fn default_filter_args() -> SgOrdersListFilterArgs {
        SgOrdersListFilterArgs {
            owners: vec![],
            active: None,
            order_hash: None,
            tokens: None,
            orderbooks: vec![],
        }
    }

    fn default_pagination_args() -> SgPaginationArgs {
        SgPaginationArgs {
            page: 1,
            page_size: 10,
        }
    }

    #[tokio::test]
    async fn test_orders_list_no_subgraphs() {
        let client = MultiOrderbookSubgraphClient::new(vec![]);
        let result = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_orders_list_one_subgraph_returns_orders() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "subgraph_alpha";

        let order1_s1 = sample_sg_order("s1_1", "100");
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": [order1_s1]}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![MultiSubgraphArgs {
            url: sg1_url,
            name: sg1_name.to_string(),
        }]);

        let orders = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order.id, order1_s1.id);
        assert_eq!(orders[0].subgraph_name, sg1_name);
    }

    #[tokio::test]
    async fn test_orders_list_multiple_subgraphs_merge_and_sort() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_one";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_two";

        let order_a_s1 = sample_sg_order("s1_A", "100");
        let order_b_s2 = sample_sg_order("s2_B", "200");
        let order_c_s2 = sample_sg_order("s2_C", "50");

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": [order_a_s1]}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": [order_b_s2, order_c_s2]}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);

        let orders = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;

        assert_eq!(orders.len(), 3);
        assert_eq!(orders[0].order.id, order_b_s2.id);
        assert_eq!(orders[0].subgraph_name, sg2_name);
        assert_eq!(orders[1].order.id, order_a_s1.id);
        assert_eq!(orders[1].subgraph_name, sg1_name);
        assert_eq!(orders[2].order.id, order_c_s2.id);
        assert_eq!(orders[2].subgraph_name, sg2_name);
    }

    #[tokio::test]
    async fn test_orders_list_multiple_subgraphs_some_empty() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_one";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_two_empty";

        let order_a_s1 = sample_sg_order("s1_A", "100");
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": [order_a_s1]}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"orders": []}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);
        let orders = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order.id, order_a_s1.id);
        assert_eq!(orders[0].subgraph_name, sg1_name);
    }

    #[tokio::test]
    async fn test_orders_list_one_subgraph_errors_others_succeed() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_one_ok";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_two_error";

        let order_a_s1 = sample_sg_order("s1_A", "100");
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": [order_a_s1]}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);
        let orders = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order.id, order_a_s1.id);
        assert_eq!(orders[0].subgraph_name, sg1_name);
    }

    #[tokio::test]
    async fn test_orders_list_all_subgraphs_error() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_one_err";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_two_err";

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);
        let orders = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;
        assert!(orders.is_empty());
    }

    #[tokio::test]
    async fn test_orders_list_invalid_timestamp_string_sorts_as_zero() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_one";

        let order_a = sample_sg_order("A", "100");
        let order_b = sample_sg_order("B", "invalid_timestamp");
        let order_c = sample_sg_order("C", "50");

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": [order_a, order_b, order_c]}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![MultiSubgraphArgs {
            url: sg1_url,
            name: sg1_name.to_string(),
        }]);
        let orders = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;
        assert_eq!(orders.len(), 3);
        assert_eq!(orders[0].order.id, order_a.id);
        assert_eq!(orders[1].order.id, order_c.id);
        assert_eq!(orders[2].order.id, order_b.id);
    }

    #[tokio::test]
    async fn test_orders_list_sorts_various_timestamps_correctly() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_one";

        let order_a = sample_sg_order("A", "0");
        let order_b = sample_sg_order("B", "9999999999999");
        let order_c = sample_sg_order("C", "1");
        let order_d = sample_sg_order("D", "-10");
        let order_e = sample_sg_order("E", "another_invalid");

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(
                json!({"data": {"orders": [order_a, order_b, order_c, order_d, order_e]}}),
            );
        });

        let client = MultiOrderbookSubgraphClient::new(vec![MultiSubgraphArgs {
            url: sg1_url,
            name: sg1_name.to_string(),
        }]);
        let orders = client
            .orders_list(default_filter_args(), default_pagination_args())
            .await;
        assert_eq!(orders.len(), 5);

        assert_eq!(orders[0].order.id, order_b.id);
        assert_eq!(orders[1].order.id, order_c.id);

        let ids_for_ts_zero: Vec<&SgBytes> = orders
            .iter()
            .filter(|o| o.order.timestamp_added.0.parse::<i64>().unwrap_or(0) == 0)
            .map(|o| &o.order.id)
            .collect();
        assert!(ids_for_ts_zero.contains(&&order_a.id));
        assert!(ids_for_ts_zero.contains(&&order_e.id));
        assert_eq!(orders[4].order.id, order_d.id);

        let order_ids_sorted: Vec<SgBytes> = orders.into_iter().map(|o| o.order.id).collect();
        assert_eq!(order_ids_sorted[0], order_b.id);
        assert_eq!(order_ids_sorted[1], order_c.id);

        assert!(
            (order_ids_sorted[2] == order_a.id && order_ids_sorted[3] == order_e.id)
                || (order_ids_sorted[2] == order_e.id && order_ids_sorted[3] == order_a.id)
        );
        assert_eq!(order_ids_sorted[4], order_d.id);
    }

    #[tokio::test]
    async fn test_orders_count_no_subgraphs() {
        let client = MultiOrderbookSubgraphClient::new(vec![]);
        let count = client.orders_count(default_filter_args()).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_orders_count_one_subgraph() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();

        let orders: Vec<_> = (0..3)
            .map(|i| sample_sg_order(&format!("c_{}", i), "100"))
            .collect();
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": orders}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![MultiSubgraphArgs {
            url: sg1_url,
            name: "sg_one".to_string(),
        }]);

        let count = client.orders_count(default_filter_args()).await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_orders_count_multiple_subgraphs_sums() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();

        let orders_s1: Vec<_> = (0..2)
            .map(|i| sample_sg_order(&format!("s1_{}", i), "100"))
            .collect();
        let orders_s2: Vec<_> = (0..5)
            .map(|i| sample_sg_order(&format!("s2_{}", i), "200"))
            .collect();

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": orders_s1}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": orders_s2}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: "sg_one".to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: "sg_two".to_string(),
            },
        ]);

        let count = client.orders_count(default_filter_args()).await.unwrap();
        assert_eq!(count, 7);
    }

    #[tokio::test]
    async fn test_orders_count_one_subgraph_errors_propagates() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();

        let orders_s1: Vec<_> = (0..4)
            .map(|i| sample_sg_order(&format!("s1_{}", i), "100"))
            .collect();
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"orders": orders_s1}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: "sg_one".to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: "sg_two_err".to_string(),
            },
        ]);

        let result = client.orders_count(default_filter_args()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_orders_count_all_subgraphs_error() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: "sg_one_err".to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: "sg_two_err".to_string(),
            },
        ]);

        let result = client.orders_count(default_filter_args()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_orders_count_pagination_boundary() {
        use crate::orderbook_client::ALL_PAGES_QUERY_PAGE_SIZE;

        let server = MockServer::start_async().await;
        let sg_url = Url::parse(&server.url("")).unwrap();

        let page1_orders: Vec<_> = (0..ALL_PAGES_QUERY_PAGE_SIZE)
            .map(|i| sample_sg_order(&format!("p1_{}", i), "100"))
            .collect();
        let page2_orders: Vec<_> = (0..10)
            .map(|i| sample_sg_order(&format!("p2_{}", i), "100"))
            .collect();

        server.mock(|when, then| {
            when.method(POST).path("/").body_contains("\"skip\":0");
            then.status(200)
                .json_body(json!({"data": {"orders": page1_orders}}));
        });
        server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE));
            then.status(200)
                .json_body(json!({"data": {"orders": page2_orders}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![MultiSubgraphArgs {
            url: sg_url,
            name: "sg_one".to_string(),
        }]);

        let count = client.orders_count(default_filter_args()).await.unwrap();
        assert_eq!(count, ALL_PAGES_QUERY_PAGE_SIZE as u32 + 10);
    }

    fn sample_sg_erc20(id_suffix: &str) -> SgErc20 {
        SgErc20 {
            id: SgBytes(format!("0xtoken_id_{}", id_suffix)),
            address: SgBytes(format!("0xtoken_address_{}", id_suffix)),
            name: Some(format!("Token {}", id_suffix)),
            symbol: Some(format!("TKN{}", id_suffix)),
            decimals: Some(SgBigInt("18".to_string())),
        }
    }

    fn sample_sg_orderbook(id_suffix: &str) -> SgOrderbook {
        SgOrderbook {
            id: SgBytes(format!("0xorderbook_id_{}", id_suffix)),
        }
    }

    fn sample_sg_vault(id_suffix: &str) -> SgVault {
        SgVault {
            id: SgBytes(format!("0xvault_id_{}", id_suffix)),
            owner: SgBytes(format!("0xowner_vault_{}", id_suffix)),
            vault_id: SgBytes(format!(
                "{}",
                id_suffix
                    .chars()
                    .filter_map(|c| c.to_digit(10))
                    .fold(0, |acc, digit| acc * 10 + digit)
                    + 1000
            )),
            balance: SgBytes(F1.as_hex()),
            token: sample_sg_erc20(id_suffix),
            orderbook: sample_sg_orderbook(id_suffix),
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        }
    }

    fn default_vault_filter_args() -> SgVaultsListFilterArgs {
        SgVaultsListFilterArgs {
            owners: vec![],
            hide_zero_balance: false,
            tokens: vec![],
            orderbooks: vec![],
            only_active_orders: false,
        }
    }

    #[tokio::test]
    async fn test_vaults_list_no_subgraphs() {
        let client = MultiOrderbookSubgraphClient::new(vec![]);
        let result = client
            .vaults_list(default_vault_filter_args(), default_pagination_args())
            .await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_vaults_list_one_subgraph_returns_vaults() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "subgraph_gamma";

        let vault1_s1 = sample_sg_vault("s1_v1");
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vaults": [vault1_s1]}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![MultiSubgraphArgs {
            url: sg1_url,
            name: sg1_name.to_string(),
        }]);

        let vaults = client
            .vaults_list(default_vault_filter_args(), default_pagination_args())
            .await;
        assert_eq!(vaults.len(), 1);
        assert_eq!(vaults[0].vault.id, vault1_s1.id);
        assert_eq!(vaults[0].subgraph_name, sg1_name);
    }

    #[tokio::test]
    async fn test_vaults_list_multiple_subgraphs_merge() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_v_one";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_v_two";

        let vault_a_s1 = sample_sg_vault("s1_VA");
        let vault_b_s2 = sample_sg_vault("s2_VB");
        let vault_c_s2 = sample_sg_vault("s2_VC");

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vaults": [vault_a_s1]}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vaults": [vault_b_s2, vault_c_s2]}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);

        let vaults_with_names = client
            .vaults_list(default_vault_filter_args(), default_pagination_args())
            .await;

        assert_eq!(vaults_with_names.len(), 3);

        let mut expected_vault_ids_with_names = std::collections::HashSet::new();
        expected_vault_ids_with_names.insert((vault_a_s1.id.clone(), sg1_name.to_string()));
        expected_vault_ids_with_names.insert((vault_b_s2.id.clone(), sg2_name.to_string()));
        expected_vault_ids_with_names.insert((vault_c_s2.id.clone(), sg2_name.to_string()));

        let actual_vault_ids_with_names: std::collections::HashSet<_> = vaults_with_names
            .into_iter()
            .map(|v| (v.vault.id, v.subgraph_name))
            .collect();

        assert_eq!(actual_vault_ids_with_names, expected_vault_ids_with_names);
    }

    #[tokio::test]
    async fn test_vaults_list_multiple_subgraphs_some_empty() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_v_one";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_v_two_empty";

        let vault_a_s1 = sample_sg_vault("s1_VA");
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vaults": [vault_a_s1]}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({"data": {"vaults": []}}));
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);
        let vaults = client
            .vaults_list(default_vault_filter_args(), default_pagination_args())
            .await;
        assert_eq!(vaults.len(), 1);
        assert_eq!(vaults[0].vault.id, vault_a_s1.id);
        assert_eq!(vaults[0].subgraph_name, sg1_name);
    }

    #[tokio::test]
    async fn test_vaults_list_one_subgraph_errors_others_succeed() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_v_one_ok";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_v_two_error";

        let vault_a_s1 = sample_sg_vault("s1_VA");
        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({"data": {"vaults": [vault_a_s1]}}));
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);
        let vaults = client
            .vaults_list(default_vault_filter_args(), default_pagination_args())
            .await;
        assert_eq!(vaults.len(), 1);
        assert_eq!(vaults[0].vault.id, vault_a_s1.id);
        assert_eq!(vaults[0].subgraph_name, sg1_name);
    }

    #[tokio::test]
    async fn test_vaults_list_all_subgraphs_error() {
        let server1 = MockServer::start_async().await;
        let sg1_url = Url::parse(&server1.url("")).unwrap();
        let sg1_name = "sg_v_one_err";

        let server2 = MockServer::start_async().await;
        let sg2_url = Url::parse(&server2.url("")).unwrap();
        let sg2_name = "sg_v_two_err";

        server1.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });
        server2.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let client = MultiOrderbookSubgraphClient::new(vec![
            MultiSubgraphArgs {
                url: sg1_url,
                name: sg1_name.to_string(),
            },
            MultiSubgraphArgs {
                url: sg2_url,
                name: sg2_name.to_string(),
            },
        ]);
        let vaults = client
            .vaults_list(default_vault_filter_args(), default_pagination_args())
            .await;
        assert!(vaults.is_empty());
    }
}
