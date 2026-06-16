use insta::assert_snapshot;
use raindex_subgraph_client::types::common::*;
use raindex_subgraph_client::types::order::SgOrdersListQuery;

#[test]
fn orders_query_gql_output() {
    use cynic::QueryBuilder;

    let request_body = SgOrdersListQuery::build(SgOrdersListQueryVariables {
        skip: Some(0),
        first: Some(10),
        filters: None,
    });

    assert_snapshot!(request_body.query);
}
