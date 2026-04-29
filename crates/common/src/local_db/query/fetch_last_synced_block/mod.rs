use super::SqlValue;
use crate::local_db::{query::SqlStatement, RaindexIdentifier};
use serde::{Deserialize, Serialize};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

pub const FETCH_LAST_SYNCED_BLOCK_SQL: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SyncStatusResponse {
    #[serde(alias = "chainId")]
    pub chain_id: u32,
    #[serde(alias = "raindexAddress")]
    pub raindex_address: String,
    #[serde(alias = "lastSyncedBlock")]
    pub last_synced_block: u64,
    #[serde(alias = "updatedAt")]
    pub updated_at: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(SyncStatusResponse);

pub fn fetch_last_synced_block_stmt(raindex_id: &RaindexIdentifier) -> SqlStatement {
    SqlStatement::new_with_params(
        FETCH_LAST_SYNCED_BLOCK_SQL,
        [
            SqlValue::from(raindex_id.chain_id as u64),
            SqlValue::from(raindex_id.raindex_address.to_string()),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::SqlValue;
    use alloy::{hex, primitives::Address};

    #[test]
    fn stmt_is_static_and_param_free() {
        let addr = Address::from([0x22u8; 20]);
        let stmt = fetch_last_synced_block_stmt(&RaindexIdentifier::new(42161, addr));
        assert_eq!(stmt.sql, FETCH_LAST_SYNCED_BLOCK_SQL);
        assert_eq!(stmt.params.len(), 2);
        assert_eq!(stmt.params[0], SqlValue::U64(42161));
        assert_eq!(stmt.params[1], SqlValue::Text(hex::encode_prefixed(addr)));
        assert!(stmt.sql.to_lowercase().contains("from sync_status"));
    }
}
