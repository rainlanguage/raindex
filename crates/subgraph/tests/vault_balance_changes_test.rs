use cynic::Id;
use insta::assert_snapshot;
use raindex_subgraph_client::types::common::*;
use raindex_subgraph_client::types::vault::SgVaultBalanceChangesListQuery;

#[test]
fn vault_balance_changes_list_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = SgVaultBalanceChangesListQuery::build(SgPaginationWithIdQueryVariables {
        id: SgBytes(id.inner().to_string()),
        skip: None,
        first: None,
    });

    assert_snapshot!(request_body.query);
}
