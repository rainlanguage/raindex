use clap::Args;
use raindex_common::subgraph::SubgraphArgs;
use raindex_subgraph_client::{
    types::common::{
        SgBytes, SgOrdersListFilterArgs, SgOrdersTokensFilterArgs, SgVaultsListFilterArgs,
    },
    SgPaginationArgs,
};

#[derive(Args, Clone)]
pub struct CliSubgraphArgs {
    #[arg(
        short,
        long,
        help = "Url of the hosted Subgraph for this Raindex deployment"
    )]
    pub subgraph_url: String,
}

impl From<CliSubgraphArgs> for SubgraphArgs {
    fn from(val: CliSubgraphArgs) -> Self {
        SubgraphArgs {
            url: val.subgraph_url,
        }
    }
}

#[derive(Args, Clone)]
pub struct CliPaginationArgs {
    #[arg(
        short,
        long,
        help = "Page number to query",
        default_value = "1",
        conflicts_with("csv")
    )]
    pub page: u16,

    #[arg(
        short = 'l',
        long,
        help = "Number of items per page",
        default_value = "25",
        conflicts_with("csv")
    )]
    pub page_size: u16,

    #[arg(
        long,
        help = "Output all items in CSV format (not paginated)",
        conflicts_with("page"),
        conflicts_with("page_size")
    )]
    pub csv: bool,
}

impl From<CliPaginationArgs> for SgPaginationArgs {
    fn from(val: CliPaginationArgs) -> Self {
        Self {
            page: val.page,
            page_size: val.page_size,
        }
    }
}

#[derive(Args, Clone)]
pub struct CliFilterArgs {
    #[arg(
        long,
        help = "Filter orders by owner addresses (comma-separated)",
        value_delimiter = ','
    )]
    pub owners: Vec<String>,

    #[arg(long, help = "Filter orders by active status", default_value = "true")]
    pub active: Option<bool>,

    #[arg(
        long,
        help = "Hide vaults with zero balance (default true)",
        default_value = "true"
    )]
    pub hide_zero_balance: Option<bool>,

    #[arg(long, help = "Filter orders by order hash")]
    pub order_hash: Option<String>,

    #[arg(
        long,
        help = "Filter orders and vaults by token addresses (comma-separated)",
        value_delimiter = ','
    )]
    pub tokens: Vec<String>,
}

impl From<CliFilterArgs> for SgOrdersListFilterArgs {
    fn from(val: CliFilterArgs) -> Self {
        let tokens = if val.tokens.is_empty() {
            None
        } else {
            Some(SgOrdersTokensFilterArgs {
                inputs: val.tokens.clone(),
                outputs: val.tokens.clone(),
            })
        };
        Self {
            owners: val.owners.into_iter().map(SgBytes).collect(),
            active: val.active,
            order_hash: val.order_hash.map(SgBytes),
            tokens,
            raindexes: vec![],
        }
    }
}

impl From<CliFilterArgs> for SgVaultsListFilterArgs {
    fn from(val: CliFilterArgs) -> Self {
        Self {
            owners: val.owners.into_iter().map(SgBytes).collect(),
            hide_zero_balance: val.hide_zero_balance.unwrap_or(true),
            tokens: val.tokens,
            raindexes: vec![],
            only_active_orders: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_cli_subgraph_args() {
        let cli_args = CliSubgraphArgs {
            subgraph_url: "https://example.com/subgraph".to_string(),
        };
        let subgraph_args: SubgraphArgs = cli_args.into();
        assert_eq!(subgraph_args.url, "https://example.com/subgraph");
    }

    #[test]
    fn test_from_cli_pagination_args() {
        let cli_args = CliPaginationArgs {
            page: 2,
            page_size: 50,
            csv: false,
        };
        let pagination_args: SgPaginationArgs = cli_args.into();
        assert_eq!(pagination_args.page, 2);
        assert_eq!(pagination_args.page_size, 50);
    }

    #[test]
    fn test_from_cli_filter_args_to_orders() {
        let owners = vec!["0x123".to_string(), "0x456".to_string()];
        let tokens = vec!["0xabc".to_string()];
        let cli_args = CliFilterArgs {
            owners: owners.clone(),
            active: Some(true),
            hide_zero_balance: Some(false),
            order_hash: Some("0x789".to_string()),
            tokens: tokens.clone(),
        };
        let filter_args: SgOrdersListFilterArgs = cli_args.into();
        assert_eq!(
            filter_args.owners,
            owners.into_iter().map(SgBytes).collect::<Vec<_>>()
        );
        assert_eq!(filter_args.active, Some(true));
        assert_eq!(filter_args.order_hash, Some(SgBytes("0x789".to_string())));
        assert!(filter_args.tokens.is_some());
        let tokens_filter = filter_args.tokens.unwrap();
        assert_eq!(tokens_filter.inputs, tokens);
        assert_eq!(tokens_filter.outputs, tokens_filter.inputs);
    }

    #[test]
    fn test_from_cli_filter_args_to_vaults() {
        let owners = vec!["0x123".to_string(), "0x456".to_string()];
        let tokens = vec!["0xabc".to_string()];
        let cli_args = CliFilterArgs {
            owners: owners.clone(),
            active: Some(true),
            hide_zero_balance: Some(false),
            order_hash: Some("0x789".to_string()),
            tokens: tokens.clone(),
        };
        let filter_args: SgVaultsListFilterArgs = cli_args.into();
        assert_eq!(
            filter_args.owners,
            owners.into_iter().map(SgBytes).collect::<Vec<_>>()
        );
        assert!(!filter_args.hide_zero_balance);
        assert_eq!(filter_args.tokens, tokens);
    }
}
