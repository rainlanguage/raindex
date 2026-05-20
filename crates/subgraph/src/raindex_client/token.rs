use super::*;
use crate::types::common::SgTokensListAllQuery;

impl RaindexSubgraphClient {
    /// Fetch all tokens directly from ERC20 entities
    pub async fn tokens_list_all(&self) -> Result<Vec<SgErc20>, RaindexSubgraphClientError> {
        let data = self.query::<SgTokensListAllQuery, ()>(()).await?;

        Ok(data.tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{SgBigInt, SgBytes, SgErc20};
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;

    fn setup_client(server: &MockServer) -> RaindexSubgraphClient {
        let url = Url::parse(&server.url("")).unwrap();
        RaindexSubgraphClient::new(url)
    }

    #[tokio::test]
    async fn test_tokens_list_all_success() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);

        let token1 = SgErc20 {
            id: SgBytes("0xToken1".to_string()),
            address: SgBytes("0xToken1Address".to_string()),
            name: Some("Token 1".to_string()),
            symbol: Some("TK1".to_string()),
            decimals: Some(SgBigInt("18".to_string())),
        };

        let token2 = SgErc20 {
            id: SgBytes("0xToken2".to_string()),
            address: SgBytes("0xToken2Address".to_string()),
            name: Some("Token 2".to_string()),
            symbol: Some("TK2".to_string()),
            decimals: Some(SgBigInt("6".to_string())),
        };

        let tokens = vec![token1.clone(), token2.clone()];

        // Mock GraphQL response
        sg_server.mock(|when, then| {
            when.method(POST).path("/").body_contains("erc20S");
            then.status(200)
                .json_body(json!({"data": {"erc20S": tokens}}));
        });

        let result = client.tokens_list_all().await;
        assert!(result.is_ok(), "Result was: {:?}", result.err());

        let returned_tokens = result.unwrap();
        assert_eq!(returned_tokens.len(), 2, "Should return all tokens");
        assert_eq!(returned_tokens[0].id, token1.id);
        assert_eq!(returned_tokens[1].id, token2.id);
    }

    #[tokio::test]
    async fn test_tokens_list_all_empty_result() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);

        // Mock empty response
        sg_server.mock(|when, then| {
            when.method(POST).path("/").body_contains("erc20S");
            then.status(200).json_body(json!({"data": {"erc20S": []}}));
        });

        let result = client.tokens_list_all().await;
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_empty(),
            "Should return empty list when no tokens exist"
        );
    }

    #[tokio::test]
    async fn test_tokens_list_all_network_error() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);

        // Mock network error
        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.tokens_list_all().await;
        assert!(
            matches!(result, Err(RaindexSubgraphClientError::CynicClientError(_))),
            "Should return network error when GraphQL request fails"
        );
    }
}
