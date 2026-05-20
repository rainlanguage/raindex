use crate::commands::{Chart, Order, StrategyBuilder, Subgraph, Trade, Vault, Words};
use crate::execute::Execute;
use anyhow::Result;
use clap::Subcommand;
use commands::local_db::LocalDbCommands;
use rain_orderbook_quote::cli::Quoter;

mod commands;
mod execute;
mod output;
mod status;
mod subgraph;
mod transaction;

#[derive(Subcommand)]
pub enum Orderbook {
    #[command(subcommand)]
    Order(Order),

    #[command(subcommand)]
    Vault(Vault),

    #[command(subcommand)]
    Trade(Trade),

    #[command(subcommand)]
    Subgraph(Subgraph),

    Chart(Chart),

    Quote(Quoter),

    Words(Words),

    #[command(name = "local-db", subcommand)]
    LocalDb(LocalDbCommands),

    #[command(
        name = "strategy-builder",
        about = "Generate deployment calldata from a registry strategy"
    )]
    StrategyBuilder(StrategyBuilder),
}

impl Orderbook {
    pub async fn execute(self) -> Result<()> {
        match self {
            Orderbook::Order(order) => order.execute().await,
            Orderbook::Vault(vault) => vault.execute().await,
            Orderbook::Trade(trade) => trade.execute().await,
            Orderbook::Chart(chart) => chart.execute().await,
            Orderbook::Quote(quote) => quote.execute().await,
            Orderbook::Subgraph(subgraph) => subgraph.execute().await,
            Orderbook::Words(words) => words.execute().await,
            Orderbook::LocalDb(local_db) => local_db.execute().await,
            Orderbook::StrategyBuilder(strategy_builder) => strategy_builder.execute().await,
        }
    }
}
