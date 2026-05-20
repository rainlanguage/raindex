use crate::commands::{Chart, Order, Subgraph, Trade, Vault, Words};
use crate::execute::Execute;
use anyhow::Result;
use clap::Subcommand;
use commands::local_db::LocalDbCommands;
use raindex_quote::cli::Quoter;

mod commands;
mod execute;
mod output;
mod status;
mod subgraph;
mod transaction;

#[derive(Subcommand)]
pub enum Raindex {
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
}

impl Raindex {
    pub async fn execute(self) -> Result<()> {
        match self {
            Raindex::Order(order) => order.execute().await,
            Raindex::Vault(vault) => vault.execute().await,
            Raindex::Trade(trade) => trade.execute().await,
            Raindex::Chart(chart) => chart.execute().await,
            Raindex::Quote(quote) => quote.execute().await,
            Raindex::Subgraph(subgraph) => subgraph.execute().await,
            Raindex::Words(words) => words.execute().await,
            Raindex::LocalDb(local_db) => local_db.execute().await,
        }
    }
}
