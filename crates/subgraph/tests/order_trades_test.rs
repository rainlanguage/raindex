use cynic::Id;
use insta::assert_snapshot;
use raindex_subgraph_client::types::common::*;
use raindex_subgraph_client::types::order_trade::SgOrderTradesListQuery;

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = SgOrderTradesListQuery::build(SgPaginationWithTimestampQueryVariables {
        id: SgBytes(id.inner().to_string()),
        skip: Some(0),
        first: Some(10),
        timestamp_gte: None,
        timestamp_lte: None,
    });

    assert_snapshot!(request_body.query);
}
