use crate::local_db::{
    query::{SqlStatement, SqlValue},
    RaindexIdentifier,
};
use alloy::primitives::Bytes;

const QUERY_TEMPLATE: &str = include_str!("query.sql");

pub fn upsert_target_watermark_stmt(
    raindex_id: &RaindexIdentifier,
    last_block: u64,
    last_hash: Bytes,
) -> SqlStatement {
    SqlStatement::new_with_params(
        QUERY_TEMPLATE,
        [
            SqlValue::from(raindex_id.chain_id),
            SqlValue::from(raindex_id.raindex_address),
            SqlValue::from(last_block),
            SqlValue::from(last_hash),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use std::str::FromStr;

    #[test]
    fn upsert_stmt_binds_all_params() {
        let stmt = upsert_target_watermark_stmt(
            &RaindexIdentifier::new(10, Address::from([0xabu8; 20])),
            123,
            Bytes::from_str("0xbeef").unwrap(),
        );
        assert!(stmt.sql().to_lowercase().contains("on conflict"));
        assert_eq!(stmt.params().len(), 4);
    }

    #[test]
    fn upsert_stmt_sql_matches_template_and_columns() {
        let stmt = upsert_target_watermark_stmt(
            &RaindexIdentifier::new(1, Address::ZERO),
            0,
            Bytes::from_str("0xbeef").unwrap(),
        );
        assert_eq!(stmt.sql(), QUERY_TEMPLATE);
        let lower = stmt.sql().to_lowercase();
        assert!(lower.contains("insert into target_watermarks"));
        assert!(lower.contains("(chain_id, raindex_address, last_block, last_hash)"));
        assert!(lower.contains("values (?1, ?2, ?3, ?4)"));
        assert!(lower.contains("on conflict(chain_id, raindex_address)"));
        assert!(lower.contains("updated_at = (cast(strftime('%s', 'now') as integer) * 1000)"));
    }

    #[test]
    fn upsert_stmt_param_order_and_values() {
        let chain_id = 100u32;
        let raindex = Address::from([0x11u8; 20]);
        let last_block = 42u64;
        let last_hash = Bytes::from_str("0xdeadbeef").unwrap();
        let stmt = upsert_target_watermark_stmt(
            &RaindexIdentifier::new(chain_id, raindex),
            last_block,
            last_hash.clone(),
        );

        let params = stmt.params();
        assert_eq!(params.len(), 4);
        assert_eq!(params[0], SqlValue::U64(chain_id as u64));
        assert_eq!(params[1], SqlValue::Text(raindex.to_string()));
        assert_eq!(params[2], SqlValue::U64(last_block));
        assert_eq!(params[3], SqlValue::Text(last_hash.to_string()));
    }
}
