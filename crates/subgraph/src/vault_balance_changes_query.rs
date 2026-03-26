use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PageQueryClient, PageQueryVariables};
use crate::types::common::*;
use crate::types::vault::SgVaultBalanceChangesListQuery;
use chrono::DateTime;
use reqwest::Url;
use std::cmp::Reverse;

pub struct VaultBalanceChangesListPageQueryClient {
    pub url: Url,
}

impl VaultBalanceChangesListPageQueryClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

impl CynicClient for VaultBalanceChangesListPageQueryClient {
    fn get_base_url(&self) -> &Url {
        &self.url
    }
}

impl PageQueryClient<SgVaultBalanceChangeType, SgPaginationWithIdQueryVariables>
    for VaultBalanceChangesListPageQueryClient
{
    async fn query_page(
        &self,
        variables: SgPaginationWithIdQueryVariables,
    ) -> Result<Vec<SgVaultBalanceChangeType>, CynicClientError> {
        let res: Result<SgVaultBalanceChangesListQuery, CynicClientError> = self
            .query::<SgVaultBalanceChangesListQuery, SgPaginationWithIdQueryVariables>(variables)
            .await;

        let list: Vec<SgVaultBalanceChangeType> = res?.vault_balance_changes;

        Ok(list)
    }

    fn sort_results(results: Vec<SgVaultBalanceChangeType>) -> Vec<SgVaultBalanceChangeType> {
        let mut sorted_results = results.clone();
        sorted_results.sort_by_key(|r| {
            Reverse(DateTime::from_timestamp(
                r.timestamp()
                    .map(|t| t.0.parse::<i64>().unwrap_or(0))
                    .unwrap_or(0),
                0,
            ))
        });

        sorted_results
    }
}

impl PageQueryVariables for SgPaginationWithIdQueryVariables {
    fn with_pagination(&self, skip: Option<i32>, first: Option<i32>) -> Self {
        Self {
            skip,
            first,
            id: self.id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cynic_client::CynicClientError;
    use crate::types::common::{
        SgBigInt, SgBytes, SgDeposit, SgErc20, SgOrderbook, SgPaginationWithIdQueryVariables,
        SgTransaction, SgVaultBalanceChangeVault, SgWithdrawal,
    };
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;

    fn setup_query_client(server: &MockServer) -> VaultBalanceChangesListPageQueryClient {
        let url = Url::parse(&server.url("")).unwrap();
        VaultBalanceChangesListPageQueryClient::new(url)
    }

    fn default_sg_bytes(val: &str) -> SgBytes {
        SgBytes(val.to_string())
    }

    fn default_sg_big_int(val: &str) -> SgBigInt {
        SgBigInt(val.to_string())
    }

    fn default_sg_erc20() -> SgErc20 {
        SgErc20 {
            id: default_sg_bytes("0xTokenId"),
            address: default_sg_bytes("0xTokenAddress"),
            name: Some("Test Token".to_string()),
            symbol: Some("TTK".to_string()),
            decimals: Some(default_sg_big_int("18")),
        }
    }

    fn default_sg_orderbook() -> SgOrderbook {
        SgOrderbook {
            id: default_sg_bytes("0xOrderbookId"),
        }
    }

    fn default_sg_transaction() -> SgTransaction {
        SgTransaction {
            id: default_sg_bytes("0xTransactionId"),
            from: default_sg_bytes("0xSenderAddress"),
            block_number: default_sg_big_int("100"),
            timestamp: default_sg_big_int("1700000000"),
        }
    }

    fn default_sg_vault_balance_change_vault_ref() -> SgVaultBalanceChangeVault {
        SgVaultBalanceChangeVault {
            id: default_sg_bytes("0xVaultIdForBalanceChange"),
            vault_id: default_sg_bytes("12345"),
            token: default_sg_erc20(),
        }
    }

    fn create_deposit(timestamp_str: &str, id_suffix: &str) -> SgVaultBalanceChangeType {
        SgVaultBalanceChangeType::Deposit(SgDeposit {
            id: default_sg_bytes(&format!("0xDepositId{}", id_suffix)),
            __typename: "Deposit".to_string(),
            amount: default_sg_bytes("500000000000000000"),
            new_vault_balance: default_sg_bytes("1500000000000000000"),
            old_vault_balance: default_sg_bytes("1000000000000000000"),
            vault: default_sg_vault_balance_change_vault_ref(),
            timestamp: default_sg_big_int(timestamp_str),
            transaction: SgTransaction {
                id: default_sg_bytes(&format!("0xTransactionId{}", id_suffix)),
                ..default_sg_transaction()
            },
            orderbook: default_sg_orderbook(),
        })
    }

    fn create_withdrawal(timestamp_str: &str, id_suffix: &str) -> SgVaultBalanceChangeType {
        SgVaultBalanceChangeType::Withdrawal(SgWithdrawal {
            id: default_sg_bytes(&format!("0xWithdrawalId{}", id_suffix)),
            __typename: "Withdrawal".to_string(),
            amount: default_sg_bytes("500000000000000000"),
            new_vault_balance: default_sg_bytes("1500000000000000000"),
            old_vault_balance: default_sg_bytes("1000000000000000000"),
            vault: default_sg_vault_balance_change_vault_ref(),
            timestamp: default_sg_big_int(timestamp_str),
            transaction: SgTransaction {
                id: default_sg_bytes(&format!("0xTransactionId{}", id_suffix)),
                ..default_sg_transaction()
            },
            orderbook: default_sg_orderbook(),
        })
    }

    #[test]
    fn test_new_client() {
        let url = Url::parse("http://localhost:8000/subgraphs/name/test").unwrap();
        let client = VaultBalanceChangesListPageQueryClient::new(url.clone());
        assert_eq!(client.url, url);
    }

    #[test]
    fn test_get_base_url() {
        let url = Url::parse("http://localhost:8000/subgraphs/name/test").unwrap();
        let client = VaultBalanceChangesListPageQueryClient::new(url.clone());
        assert_eq!(client.get_base_url(), &url);
    }

    #[tokio::test]
    async fn test_query_page_empty_result() {
        let server = MockServer::start_async().await;
        let client = setup_query_client(&server);

        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({ "data": { "vaultBalanceChanges": [] } }));
        });

        let variables = SgPaginationWithIdQueryVariables {
            id: default_sg_bytes("some-vault-id"),
            skip: Some(0),
            first: Some(10),
        };

        let result = client.query_page(variables).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_query_page_http_error() {
        let server = MockServer::start_async().await;
        let client = setup_query_client(&server);

        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let variables = SgPaginationWithIdQueryVariables {
            id: default_sg_bytes("some-vault-id"),
            skip: Some(0),
            first: Some(10),
        };

        let result = client.query_page(variables).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            CynicClientError::HttpError { status: 500, .. }
        ));
    }

    #[tokio::test]
    async fn test_query_page_graphql_error() {
        let server = MockServer::start_async().await;
        let client = setup_query_client(&server);

        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200)
                .json_body(json!({ "errors": [{"message": "Some GraphQL error"}] }));
        });

        let variables = SgPaginationWithIdQueryVariables {
            id: default_sg_bytes("some-vault-id"),
            skip: Some(0),
            first: Some(10),
        };

        let result = client.query_page(variables).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            CynicClientError::GraphqlError(_)
        ));
    }

    #[test]
    fn test_sort_results() {
        let item1 = create_deposit("100", "a");
        let item2 = create_withdrawal("200", "b");
        let item3 = create_deposit("50", "c");

        let results1 = vec![item1.clone(), item2.clone(), item3.clone()];
        let sorted1 = VaultBalanceChangesListPageQueryClient::sort_results(results1);
        assert_eq!(sorted1[0].timestamp().unwrap().0, "200");
        assert_eq!(sorted1[1].timestamp().unwrap().0, "100");
        assert_eq!(sorted1[2].timestamp().unwrap().0, "50");

        let results_empty: Vec<SgVaultBalanceChangeType> = vec![];
        let sorted_empty = VaultBalanceChangesListPageQueryClient::sort_results(results_empty);
        assert!(sorted_empty.is_empty());

        let results_single = vec![item1.clone()];
        let sorted_single = VaultBalanceChangesListPageQueryClient::sort_results(results_single);
        assert_eq!(sorted_single.len(), 1);
        assert_eq!(sorted_single[0].timestamp().unwrap().0, "100");

        let results_already_sorted = vec![item2.clone(), item1.clone(), item3.clone()];
        let sorted_already =
            VaultBalanceChangesListPageQueryClient::sort_results(results_already_sorted);
        assert_eq!(sorted_already[0].timestamp().unwrap().0, "200");
        assert_eq!(sorted_already[1].timestamp().unwrap().0, "100");
        assert_eq!(sorted_already[2].timestamp().unwrap().0, "50");
    }

    #[test]
    fn test_with_pagination() {
        let original_vars = SgPaginationWithIdQueryVariables {
            id: default_sg_bytes("vault-abc"),
            skip: Some(0),
            first: Some(10),
        };

        let vars1 = original_vars.with_pagination(Some(10), Some(20));
        assert_eq!(vars1.id.0, "vault-abc");
        assert_eq!(vars1.skip, Some(10));
        assert_eq!(vars1.first, Some(20));

        let vars2 = original_vars.with_pagination(Some(5), original_vars.first);
        assert_eq!(vars2.id.0, "vault-abc");
        assert_eq!(vars2.skip, Some(5));
        assert_eq!(vars2.first, Some(10));

        let vars3 = original_vars.with_pagination(original_vars.skip, Some(15));
        assert_eq!(vars3.id.0, "vault-abc");
        assert_eq!(vars3.skip, Some(0));
        assert_eq!(vars3.first, Some(15));

        let vars4 = original_vars.with_pagination(None, Some(5));
        assert_eq!(vars4.id.0, "vault-abc");
        assert_eq!(vars4.skip, None);
        assert_eq!(vars4.first, Some(5));

        let vars5 = original_vars.with_pagination(Some(5), None);
        assert_eq!(vars5.id.0, "vault-abc");
        assert_eq!(vars5.skip, Some(5));
        assert_eq!(vars5.first, None);

        let vars6 = original_vars.with_pagination(None, None);
        assert_eq!(vars6.id.0, "vault-abc");
        assert_eq!(vars6.skip, None);
        assert_eq!(vars6.first, None);

        assert_eq!(original_vars.skip, Some(0));
        assert_eq!(original_vars.first, Some(10));
    }
}
