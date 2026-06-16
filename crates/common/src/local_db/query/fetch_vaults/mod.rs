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
    pub orderbook_address: Address,
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
    pub orderbook_addresses: Vec<Address>,
    pub owners: Vec<Address>,
    pub tokens: Vec<Address>,
    pub hide_zero_balance: bool,
    pub only_active_orders: bool,
}

const OWNERS_CLAUSE: &str = "/*OWNERS_CLAUSE*/";
const OWNERS_CLAUSE_BODY: &str = "\nAND o.owner IN ({list})\n";

const TOKENS_CLAUSE: &str = "/*TOKENS_CLAUSE*/";
const TOKENS_CLAUSE_BODY: &str = "\nAND o.token IN ({list})\n";

const CHAIN_IDS_CLAUSE: &str = "/*CHAIN_IDS_CLAUSE*/";
const CHAIN_IDS_BODY: &str = "AND rvb.chain_id IN ({list})";

const ORDERBOOKS_CLAUSE: &str = "/*ORDERBOOKS_CLAUSE*/";
const ORDERBOOKS_BODY: &str = "AND rvb.orderbook_address IN ({list})";

const HIDE_ZERO_BALANCE_CLAUSE: &str = "/*HIDE_ZERO_BALANCE*/";
const HIDE_ZERO_BALANCE_BODY: &str = "\nAND NOT FLOAT_IS_ZERO(o.balance)\n";

const ONLY_ACTIVE_ORDERS_CLAUSE: &str = "/*ONLY_ACTIVE_ORDERS_CLAUSE*/";
const ONLY_ACTIVE_ORDERS_BODY: &str = "\nAND EXISTS (
  SELECT 1 FROM order_io_items oii
  WHERE oii.chain_id = o.chain_id
    AND oii.orderbook_address = o.orderbook_address
    AND oii.owner = o.owner
    AND oii.token = o.token
    AND oii.vault_id = o.vault_id
    AND substr(oii.item, -1) = '1'
)\n";

const INNER_CHAIN_IDS_CLAUSE: &str = "/*INNER_CHAIN_IDS_CLAUSE*/";
const INNER_CHAIN_IDS_BODY: &str = "AND chain_id IN ({list})";
const INNER_ORDERBOOKS_CLAUSE: &str = "/*INNER_ORDERBOOKS_CLAUSE*/";
const INNER_ORDERBOOKS_BODY: &str = "AND orderbook_address IN ({list})";

const OIO_CHAIN_IDS_CLAUSE: &str = "/*OIO_CHAIN_IDS_CLAUSE*/";
const OIO_CHAIN_IDS_BODY: &str = "AND io.chain_id IN ({list})";
const OIO_ORDERBOOKS_CLAUSE: &str = "/*OIO_ORDERBOOKS_CLAUSE*/";
const OIO_ORDERBOOKS_BODY: &str = "AND io.orderbook_address IN ({list})";

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

    let mut orderbooks = args.orderbook_addresses.clone();
    orderbooks.sort();
    orderbooks.dedup();
    let orderbooks_iter = || orderbooks.iter().copied().map(SqlValue::from);
    stmt.bind_list_clause(ORDERBOOKS_CLAUSE, ORDERBOOKS_BODY, orderbooks_iter())?;
    stmt.bind_list_clause(
        INNER_ORDERBOOKS_CLAUSE,
        INNER_ORDERBOOKS_BODY,
        orderbooks_iter(),
    )?;
    stmt.bind_list_clause(
        OIO_ORDERBOOKS_CLAUSE,
        OIO_ORDERBOOKS_BODY,
        orderbooks_iter(),
    )?;

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

    Ok(stmt)
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
        assert!(!stmt.sql.contains(OIO_ORDERBOOKS_CLAUSE));
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
        args.orderbook_addresses = vec![
            address!("0xabc0000000000000000000000000000000000000"),
            address!("0xdef0000000000000000000000000000000000000"),
        ];
        let stmt = build_fetch_vaults_stmt(&args).unwrap();

        // Clauses inserted
        assert!(!stmt.sql.contains(OWNERS_CLAUSE));
        assert!(!stmt.sql.contains(TOKENS_CLAUSE));
        assert!(!stmt.sql.contains(HIDE_ZERO_BALANCE_CLAUSE));
        assert!(!stmt.sql.contains(OIO_CHAIN_IDS_CLAUSE));
        assert!(!stmt.sql.contains(OIO_ORDERBOOKS_CLAUSE));
        assert!(stmt.sql.contains("AND NOT FLOAT_IS_ZERO("));
        assert!(stmt.sql.contains("rvb.chain_id IN ("));
        assert!(stmt.sql.contains("rvb.orderbook_address IN ("));
        assert!(stmt.sql.contains("io.chain_id IN ("));
        assert!(stmt.sql.contains("io.orderbook_address IN ("));
        // Params include chain ids, orderbooks, owners, and tokens
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
        args.orderbook_addresses = vec![address!("0xabc0000000000000000000000000000000000000")];
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.sql.contains("AND NOT FLOAT_IS_ZERO("));
        assert!(stmt.sql.contains("order_io_items oii"));
        assert!(stmt.sql.contains("o.owner IN ("));
        assert!(stmt.sql.contains("o.token IN ("));
        assert!(stmt.sql.contains("rvb.chain_id IN ("));
        assert!(stmt.sql.contains("io.chain_id IN ("));
        assert!(stmt.sql.contains("io.orderbook_address IN ("));
    }

    #[test]
    fn owner_filtering_threaded_through_query() {
        let args = mk_args();
        let stmt = build_fetch_vaults_stmt(&args).unwrap();

        assert!(stmt
            .sql
            .contains("SELECT DISTINCT chain_id, orderbook_address, owner, token, vault_id"));
        assert!(stmt.sql.contains("rv.owner = oe.order_owner"));
        assert!(stmt
            .sql
            .contains("GROUP BY chain_id, orderbook_address, owner, token, vault_id, io_type"));
        assert!(stmt
            .sql
            .contains("GROUP BY chain_id, orderbook_address, owner, token, vault_id\n"));
        assert!(stmt.sql.contains("vol.owner = o.owner"));
    }

    #[test]
    fn active_orders_filters_by_owner() {
        let mut args = mk_args();
        args.only_active_orders = true;
        let stmt = build_fetch_vaults_stmt(&args).unwrap();
        assert!(stmt.sql.contains("oii.owner = o.owner"));
    }
}
