mod cynic_client;
mod multi_raindex_client;
mod pagination;
pub mod performance;
mod raindex_client;
pub mod types;
pub mod utils;
pub mod validate;
mod vault_balance_changes_query;

#[cynic::schema("raindex")]
pub mod schema {}

pub use multi_raindex_client::{MultiRaindexSubgraphClient, MultiSubgraphArgs};
pub use pagination::{PageQueryClient, SgPaginationArgs};
pub use raindex_client::{RaindexSubgraphClient, RaindexSubgraphClientError};
