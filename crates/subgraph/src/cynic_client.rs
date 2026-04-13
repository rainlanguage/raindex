use cynic::{
    serde::{Deserialize, Serialize},
    GraphQlError, GraphQlResponse, QueryBuilder, QueryFragment,
};
use reqwest::Url;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CynicClientError {
    #[error("Graphql errors: {}", .0.iter().map(|e| e.message.clone()).collect::<Vec<String>>().join(", "))]
    GraphqlError(Vec<GraphQlError>),
    #[error("Subgraph query returned no data")]
    Empty,
    #[error("Request Error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },
}

pub trait CynicClient {
    fn get_base_url(&self) -> &Url;

    async fn query<R: QueryFragment + QueryBuilder<V> + for<'a> Deserialize<'a>, V: Serialize>(
        &self,
        variables: V,
    ) -> Result<R, CynicClientError> {
        let request_body = R::build(variables);

        let response = reqwest::Client::new()
            .post(self.get_base_url().clone())
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(CynicClientError::HttpError {
                status: status.as_u16(),
                body,
            });
        }

        let response_deserialized: GraphQlResponse<R> =
            response.json::<GraphQlResponse<R>>().await?;

        match response_deserialized.errors {
            Some(errors) => Err(CynicClientError::GraphqlError(errors)),
            None => response_deserialized.data.ok_or(CynicClientError::Empty),
        }
    }
}
