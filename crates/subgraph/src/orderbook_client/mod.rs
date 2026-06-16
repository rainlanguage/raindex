use crate::cynic_client::{CynicClient, CynicClientError};
use crate::pagination::{PaginationClient, PaginationClientError, SgPaginationArgs};
use crate::types::add_order::{SgTransactionAddOrdersQuery, TransactionAddOrdersVariables};
use crate::types::common::*;
use crate::types::order::{
    SgBatchOrderDetailQuery, SgBatchOrderDetailQueryVariables, SgOrderDetailByHashQuery,
    SgOrderDetailByHashQueryVariables, SgOrderDetailByIdQuery, SgOrderIdList, SgOrdersListQuery,
};
use crate::types::order_trade::{
    OwnerTradesVariables, SgOrderTradeDetailQuery, SgOrderTradesListQuery, SgOwnerTradesCountQuery,
    SgOwnerTradesListQuery, SgTransactionTradesQuery, TransactionTradesVariables,
};
use crate::types::remove_order::{
    SgTransactionRemoveOrdersQuery, TransactionRemoveOrdersVariables,
};
use crate::types::transaction::SgTransactionDetailQuery;
use crate::types::vault::{SgVaultDetailQuery, SgVaultsListQuery};
use crate::vault_balance_changes_query::VaultBalanceChangesListPageQueryClient;
use cynic::Id;
use reqwest::Url;
use thiserror::Error;
use wasm_bindgen_utils::prelude::*;

mod order;
mod order_trade;
mod performance;
mod token;
mod transaction;
mod vault;

pub const ALL_PAGES_QUERY_PAGE_SIZE: u16 = 200;

#[derive(Error, Debug)]
pub enum OrderbookSubgraphClientError {
    #[error(transparent)]
    CynicClientError(#[from] CynicClientError),
    #[error("Subgraph query returned no data")]
    Empty,
    #[error("Request timed out")]
    RequestTimedOut,
    #[error(transparent)]
    PaginationClientError(#[from] PaginationClientError),
    #[error(transparent)]
    ParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    PerformanceError(#[from] crate::performance::PerformanceError),
    #[cfg(target_family = "wasm")]
    #[error(transparent)]
    SerdeWasmBindgenError(#[from] serde_wasm_bindgen::Error),
    #[error("Failed to extend the order detail")]
    OrderDetailExtendError,
}

impl From<OrderbookSubgraphClientError> for JsValue {
    fn from(value: OrderbookSubgraphClientError) -> Self {
        JsError::new(&value.to_string()).into()
    }
}

#[derive(Debug)]
pub struct OrderbookSubgraphClient {
    url: Url,
}

impl CynicClient for OrderbookSubgraphClient {
    fn get_base_url(&self) -> &Url {
        &self.url
    }
}
impl PaginationClient for OrderbookSubgraphClient {}

impl OrderbookSubgraphClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
