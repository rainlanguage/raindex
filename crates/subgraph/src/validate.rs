use cynic::{GraphQlResponse, QueryBuilder};
use cynic_introspection::{CapabilitiesQuery, IntrospectionQuery, SchemaError};
use std::fs::read_to_string;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaValidationError {
    #[error("undefined schema")]
    UndefinedSchema,
    #[error("undefined capabilities")]
    UndefinedCapabilities,
    #[error(transparent)]
    ReadMainSchemaError(#[from] std::io::Error),
    #[error(transparent)]
    SchemaError(#[from] SchemaError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

/// checks if a schema is equal to this crate's schema
fn check_schema(schema: String) -> Result<bool, SchemaValidationError> {
    let main_schema = read_to_string("./schema/raindex.graphql")?;
    Ok(main_schema == schema)
}

/// Gets a subgraph schema given a url
pub async fn get_schema(subgraph_url: &str) -> Result<String, SchemaValidationError> {
    let client = reqwest::Client::new();
    let capabilities = client
        .post(subgraph_url)
        .json(&CapabilitiesQuery::build(()))
        .send()
        .await?
        .json::<GraphQlResponse<CapabilitiesQuery>>()
        .await?
        .data
        .ok_or(SchemaValidationError::UndefinedCapabilities)?
        .capabilities();

    Ok(client
        .post(subgraph_url)
        .json(&IntrospectionQuery::with_capabilities(capabilities))
        .send()
        .await?
        .json::<GraphQlResponse<IntrospectionQuery>>()
        .await?
        .data
        .ok_or(SchemaValidationError::UndefinedSchema)?
        .into_schema()?
        .to_sdl())
}

/// Validates a subgraph's schema by fetching it from the
/// given url and comparing it against this crate's schema
pub async fn validate_subgraph_schema(subgraph_url: &str) -> Result<bool, SchemaValidationError> {
    check_schema(get_schema(subgraph_url).await?)
}

#[cfg(test)]
mod test {
    use super::*;
    use httpmock::MockServer;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_schema_happy() {
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({
                "data": {
                    "__type": null,
                    "__schema": {
                        "queryType": { "name": "Query" },
                        "mutationType": null,
                        "subscriptionType": { "name": "Subscription" },
                        "types": [{
                            "kind": "ENUM",
                            "name": "Aggregation_interval",
                            "description": null,
                            "fields": null,
                            "inputFields": null,
                            "interfaces": null,
                            "enumValues": [
                              {
                                "name": "hour",
                                "description": null,
                                "isDeprecated": false,
                                "deprecationReason": null
                              },
                              {
                                "name": "day",
                                "description": null,
                                "isDeprecated": false,
                                "deprecationReason": null
                              }
                            ],
                            "possibleTypes": null
                          }],
                        "directives": []
                    }
                }
            }));
        });
        let result = get_schema(&sg_server.url("/sg")).await.unwrap();
        let expected = "enum Aggregation_interval {\n  hour\n  day\n}\n\n";
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_get_schema_unhappy() {
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({
                "data": {
                    "__type": null,
                    "__schema": {
                        "mutationType": null,
                        "types": [],
                        "directives": []
                    }
                }
            }));
        });
        assert!(get_schema(&sg_server.url("/sg")).await.is_err());
    }

    #[test]
    fn test_check_schema_happy() {
        let schema = read_to_string("./schema/raindex.graphql").unwrap();
        let result = check_schema(schema).unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_check_schema_unhappy() {
        let sg_server = MockServer::start();
        sg_server.mock(|_when, then| {
            then.json_body_obj(&json!({
                "data": {
                    "__type": null,
                    "__schema": {
                        "queryType": { "name": "Query" },
                        "mutationType": null,
                        "subscriptionType": { "name": "Subscription" },
                        "types": [{
                            "kind": "ENUM",
                            "name": "Aggregation_interval",
                            "description": null,
                            "fields": null,
                            "inputFields": null,
                            "interfaces": null,
                            "enumValues": [
                              {
                                "name": "hour",
                                "description": null,
                                "isDeprecated": false,
                                "deprecationReason": null
                              },
                              {
                                "name": "day",
                                "description": null,
                                "isDeprecated": false,
                                "deprecationReason": null
                              }
                            ],
                            "possibleTypes": null
                          }],
                        "directives": []
                    }
                }
            }));
        });
        let schema = get_schema(&sg_server.url("/sg")).await.unwrap();
        let result = check_schema(schema).unwrap();
        assert!(!result);
    }
}
