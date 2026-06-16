use anyhow::Result;
use clap::Subcommand;
use commands::local_db::LocalDbCommands;

mod commands;

#[derive(Subcommand)]
pub enum Raindex {
    #[command(name = "local-db", subcommand)]
    LocalDb(LocalDbCommands),
}

impl Raindex {
    pub async fn execute(self) -> Result<()> {
        match self {
            Raindex::LocalDb(local_db) => local_db.execute().await,
        }
    }
}
