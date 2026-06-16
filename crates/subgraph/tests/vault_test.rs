use cynic::Id;
use insta::assert_snapshot;
use raindex_subgraph_client::types::common::*;
use raindex_subgraph_client::types::vault::SgVaultDetailQuery;

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = SgVaultDetailQuery::build(SgIdQueryVariables { id: &id });

    assert_snapshot!(request_body.query);
}
