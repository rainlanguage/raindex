use crate::local_db::query::{SqlBuildError, SqlStatement, SqlValue};
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

const QUERY_TEMPLATE: &str = include_str!("query.sql");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalDbVault {
    pub chain_id: u32,
    pub vault_id: U256,
    pub token: Address,
    pub owner: Address,
    pub raindex_address: Address,
    pub token_name: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub balance: String,
    pub input_orders: Option<String>,
    pub output_orders: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FetchVaultsArgs {
    pub chain_ids: Vec<u32>,
    pub raindex_addresses: Vec<Address>,
    pub owners: Vec<Address>,
    pub tokens: Vec<Address>,
    pub hide_zero_balance: bool,
    pub only_active_orders: bool,
    pub page: Option<u16>,
    pub page_size: Option<u16>,
}

const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
const OWNERS_CLAUSE_BODY: &str = "\nAND o.owner IN ({list})\n";

const TOKENS_CLAUSE: &str = "/*TOKENS_CLAUSE*/";
const TOKENS_CLAUSE_BODY: &str = "\nAND o.token IN ({list})\n";

const CHAIN_IDS_CLAUSE: &str = "/*CHAIN_IDS_CLAUSE*/";
const CHAIN_IDS_BODY: &str = "AND rvb.chain_id IN ({list})";

const RAINDEXES_CLAUSE: &str = "/*RAINDEXES_CLAUSE*/";
const RAINDEXES_BODY: &str = "AND rvb.raindex_address IN ({list})";

const HIDE_ZERO_BALANCE_CLAUSE: &str = "/*HIDE_ZERO_BALANCE*/";
const HIDE_ZERO_BALANCE_BODY: &str = "\nAND NOT FLOAT_IS_ZERO(o.balance)\n";

const ONLY_ACTIVE_ORDERS_CLAUSE: &str = "/*ONLY_ACTIVE_ORDERS_CLAUSE*/";
const ONLY_ACTIVE_ORDERS_BODY: &str = "\nAND EXISTS (
  SELECT 1 FROM order_io_items oii
  WHERE oii.chain_id = o.chain_id
    AND oii.raindex_address = o.raindex_address
    AND oii.owner = o.owner
    AND oii.token = o.token
    AND oii.vault_id = o.vault_id
    AND substr(oii.item, -1) = '1'
)\n";

const INNER_CHAIN_IDS_CLAUSE: &str = "/*INNER_CHAIN_IDS_CLAUSE*/";
const INNER_CHAIN_IDS_BODY: &str = "AND chain_id IN ({list})";
const INNER_RAINDEXES_CLAUSE: &str = "/*INNER_RAINDEXES_CLAUSE*/";
const INNER_RAINDEXES_BODY: &str = "AND raindex_address IN ({list})";

const OIO_CHAIN_IDS_CLAUSE: &str = "/*OIO_CHAIN_IDS_CLAUSE*/";
const OIO_CHAIN_IDS_BODY: &str = "AND io.chain_id IN ({list})";
const OIO_RAINDEXES_CLAUSE: &str = "/*OIO_RAINDEXES_CLAUSE*/";
const OIO_RAINDEXES_BODY: &str = "AND io.raindex_address IN ({list})";
const PAGINATION_CLAUSE: &str = "/*PAGINATION_CLAUSE*/";
const ORDER_BY_CLAUSE: &str =
    "ORDER BY o.chain_id, o.raindex_address, o.owner, o.token, o.vault_id";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalDbVaultsCountRow {
    pub vaults_count: u32,
}

pub fn build_fetch_vaults_stmt(args: &FetchVaultsArgs) -> Result<SqlStatement, SqlBuildError> {
    let mut stmt = SqlStatement::new(QUERY_TEMPLATE);

    let mut chain_ids = args.chain_ids.clone();
    chain_ids.sort();
    chain_ids.dedup();
    let chain_ids_iter = || chain_ids.iter().copied().map(SqlValue::from);
    stmt.bind_list_clause(CHAIN_IDS_CLAUSE, CHAIN_IDS_BODY, chain_ids_iter())?;
    stmt.bind_list_clause(
        INNER_CHAIN_IDS_CLAUSE,
        INNER_CHAIN_IDS_BODY,
        chain_ids_iter(),
    )?;
    stmt.bind_list_clause(OIO_CHAIN_IDS_CLAUSE, OIO_CHAIN_IDS_BODY, chain_ids_iter())?;

    let mut raindexes = args.raindex_addresses.clone();
    raindexes.sort();
    raindexes.dedup();
    let raindexes_iter = || raindexes.iter().copied().map(SqlValue::from);
    stmt.bind_list_clause(RAINDEXES_CLAUSE, RAINDEXES_BODY, raindexes_iter())?;
    stmt.bind_list_clause(
        INNER_RAINDEXES_CLAUSE,
        INNER_RAINDEXES_BODY,
        raindexes_iter(),
    )?;
    stmt.bind_list_clause(OIO_RAINDEXES_CLAUSE, OIO_RAINDEXES_BODY, raindexes_iter())?;

    stmt.bind_list_clause(
        OWNERS_CLAUSE,
        OWNERS_CLAUSE_BODY,
        args.owners.iter().cloned().map(SqlValue::from),
    )?;

    stmt.bind_list_clause(
        TOKENS_CLAUSE,
        TOKENS_CLAUSE_BODY,
        args.tokens.iter().cloned().map(SqlValue::from),
    )?;

    // Hide zero balance clause
    if args.hide_zero_balance {
        stmt.replace(HIDE_ZERO_BALANCE_CLAUSE, HIDE_ZERO_BALANCE_BODY)?;
    } else {
        stmt.replace(HIDE_ZERO_BALANCE_CLAUSE, "")?;
    }

    // Only active orders clause
    if args.only_active_orders {
        stmt.replace(ONLY_ACTIVE_ORDERS_CLAUSE, ONLY_ACTIVE_ORDERS_BODY)?;
    } else {
        stmt.replace(ONLY_ACTIVE_ORDERS_CLAUSE, "")?;
    }

    if let (Some(page), Some(page_size)) = (args.page, args.page_size) {
        let offset = (page.saturating_sub(1) as u64) * (page_size as u64);
        let limit_placeholder = format!("?{}", stmt.params.len() + 1);
        let offset_placeholder = format!("?{}", stmt.params.len() + 2);
        let pagination = format!("LIMIT {} OFFSET {}", limit_placeholder, offset_placeholder);
        stmt.replace(PAGINATION_CLAUSE, &pagination)?;
        stmt.push(SqlValue::U64(page_size as u64));
        stmt.push(SqlValue::U64(offset));
    } else {
        stmt.replace(PAGINATION_CLAUSE, "")?;
    }

    Ok(stmt)
}

pub fn build_fetch_vaults_count_stmt(
    args: &FetchVaultsArgs,
) -> Result<SqlStatement, SqlBuildError> {
    let count_args = FetchVaultsArgs {
        page: None,
        page_size: None,
        ..args.clone()
    };
    let mut stmt = build_fetch_vaults_stmt(&count_args)?;
    let inner_sql = stmt
        .sql
        .replace(ORDER_BY_CLAUSE, "")
        .trim_end()
        .trim_end_matches(';')
        .to_string();
    stmt.sql = format!("SELECT COUNT(*) AS vaults_count FROM ({inner_sql})");
    Ok(stmt)
}

pub fn extract_vaults_count(rows: &[LocalDbVaultsCountRow]) -> u32 {
    rows.first().map(|row| row.vaults_count).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    fn mk_args() -> FetchVaultsArgs {
        FetchVaultsArgs::default()
    }

    #[test]
    fn chain_id_and_no_filters() {
        let args = mk_args();
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.sql.contains("ORDER BY o.chain_id"));
        assert!(stmt.sql.contains("AS chainId"));
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert!(!stmt.sql.contains(ONLY_ACTIVE_ORDERS_CLAUSE));
        assert!(!stmt.sql.contains(OIO_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(OIO_RAINDEXES_CLAUSE));
        assert!(!stmt.sql.contains(PAGINATION_CLAUSE));
        assert!(stmt.params.is_empty());
    }

    #[test]
    fn owners_tokens_and_hide_zero() {
        let mut args = mk_args();
        args.owners = vec![
            address!("0x87d08841bdAd4aB82883a322D2c0eF557EC154fE"),
            address!("0x632ffCd874c1dDD5aCf9c26918D31CA3c96c0ec8"),
        ];
        args.tokens = vec![address!("0x1AC6F2786A51b20d47050f3f9E4B0e831427B498")];
        args.hide_zero_balance = true;
        args.chain_ids = vec![137, 1, 137];
        args.raindex_addresses = vec![
            address!("0xabc0000000000000000000000000000000000000"),
            address!("0xdef0000000000000000000000000000000000000"),
        ];
        let stmt = build_fetch_vaults_stmt(&args).unwrap();

        // Clauses inserted
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert!(!stmt.sql.contains(OIO_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(OIO_RAINDEXES_CLAUSE));
        assert!(stmt.sql.contains("AND NOT FLOAT_IS_ZERO("));
        assert!(stmt.sql.contains("rvb.chain_id IN ("));
        assert!(stmt.sql.contains("rvb.raindex_address IN ("));
        assert!(stmt.sql.contains("io.chain_id IN ("));
        assert!(stmt.sql.contains("io.raindex_address IN ("));
        // Params include chain ids, raindexes, owners, and tokens
        assert!(!stmt.params.is_empty());
    }

    #[test]
    fn missing_hide_zero_marker_yields_error() {
        // Remove the HIDE_ZERO_BALANCE marker to simulate template drift.
        let bad_template = QUERY_TEMPLATE.replace(HIDE_ZERO_BALANCE_CLAUSE, "");
        let mut stmt = SqlStatement::new(bad_template);
        // replace should error because the marker is absent
        let err = stmt
            .replace(HIDE_ZERO_BALANCE_CLAUSE, HIDE_ZERO_BALANCE_BODY)
            .unwrap_err();
        assert!(matches!(err, SqlBuildError::MissingMarker { .. }));
    }

    #[test]
    fn hide_zero_clause_without_filters_has_no_placeholders() {
        let mut args = mk_args();
        args.hide_zero_balance = true;
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.params.is_empty());
        assert!(!stmt.sql.contains("?1"));
        assert!(!stmt.sql.contains("?2"));
    }

    #[test]
    fn only_active_orders_clause_when_true() {
        let mut args = mk_args();
        args.only_active_orders = true;
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(!stmt.sql.contains(ONLY_ACTIVE_ORDERS_CLAUSE));
        assert!(stmt.sql.contains("AND EXISTS ("));
        assert!(stmt.sql.contains("order_io_items oii"));
        assert!(stmt.sql.contains("substr(oii.item, -1) = '1'"));
        assert!(stmt.params.is_empty());
    }

    #[test]
    fn only_active_orders_clause_omitted_when_false() {
        let mut args = mk_args();
        args.only_active_orders = false;
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(!stmt.sql.contains(ONLY_ACTIVE_ORDERS_CLAUSE));
        assert!(!stmt.sql.contains("order_io_items oii"));
        assert!(!stmt.sql.contains("substr(oii.item, -1) = '1'"));
    }

    #[test]
    fn combined_filters_with_only_active_orders() {
        let mut args = mk_args();
        args.owners = vec![address!("0x87d08841bdAd4aB82883a322D2c0eF557EC154fE")];
        args.tokens = vec![address!("0x1AC6F2786A51b20d47050f3f9E4B0e831427B498")];
        args.hide_zero_balance = true;
        args.only_active_orders = true;
        args.chain_ids = vec![137];
        args.raindex_addresses = vec![address!("0xabc0000000000000000000000000000000000000")];
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND NOT FLOAT_IS_ZERO("));
        assert!(stmt.sql.contains("order_io_items oii"));
        assert!(stmt.sql.contains("o.owner IN ("));
        assert!(stmt.sql.contains("o.token IN ("));
        assert!(stmt.sql.contains("rvb.chain_id IN ("));
        assert!(stmt.sql.contains("io.chain_id IN ("));
        assert!(stmt.sql.contains("io.raindex_address IN ("));
    }

    #[test]
    fn owner_filtering_threaded_through_query() {
        let args = mk_args();
        let stmt = build_fetch_vaults_stmt(&args).unwrap();

        assert!(stmt
            .sql
            .contains("SELECT DISTINCT chain_id, raindex_address, owner, token, vault_id"));
        assert!(stmt.sql.contains("rv.owner = oe.order_owner"));
        assert!(stmt
            .sql
            .contains("GROUP BY chain_id, raindex_address, owner, token, vault_id, io_type"));
        assert!(stmt
            .sql
            .contains("GROUP BY chain_id, raindex_address, owner, token, vault_id\n"));
        assert!(stmt.sql.contains("vol.owner = o.owner"));
    }

    #[test]
    fn active_orders_filters_by_owner() {
        let mut args = mk_args();
        args.only_active_orders = true;
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.sql.contains("oii.owner = o.owner"));
    }

    #[test]
    fn pagination_clause_uses_page_and_page_size() {
        let mut args = mk_args();
        args.page = Some(3);
        args.page_size = Some(25);
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.sql.contains("LIMIT ?1 OFFSET ?2"));
        assert_eq!(stmt.params, vec![SqlValue::U64(25), SqlValue::U64(50)]);
    }

    #[test]
    fn count_query_omits_pagination_and_counts_wrapped_vaults() {
        let mut args = mk_args();
        args.page = Some(2);
        args.page_size = Some(10);
        let stmt = build_fetch_vaults_count_stmt(&args).unwrap();
        assert!(stmt
            .sql
            .starts_with("SELECT COUNT(*) AS vaults_count FROM ("));
        assert!(!stmt.sql.contains(ORDER_BY_CLAUSE));
        assert!(!stmt.sql.contains("LIMIT"));
        assert!(stmt.params.is_empty());
    }

    #[test]
    fn extract_vaults_count_returns_zero_for_empty_rows() {
        assert_eq!(extract_vaults_count(&[]), 0);
        assert_eq!(
            extract_vaults_count(&[LocalDbVaultsCountRow { vaults_count: 7 }]),
            7
        );
    }
}
