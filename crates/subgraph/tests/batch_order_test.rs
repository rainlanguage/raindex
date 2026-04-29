use insta::assert_snapshot;
use raindex_subgraph_client::types::common::*;
use raindex_subgraph_client::types::order::{
    SgBatchOrderDetailQuery, SgBatchOrderDetailQueryVariables, SgOrderIdList,
};

#[test]
fn batch_order_query_gql_output() {
    use cynic::QueryBuilder;

    let id = SgBytes("1234".to_string());
    let request_body = SgBatchOrderDetailQuery::build(SgBatchOrderDetailQueryVariables {
        id_list: SgOrderIdList { id_in: vec![id] },
    });

    assert_snapshot!(request_body.query);
}
