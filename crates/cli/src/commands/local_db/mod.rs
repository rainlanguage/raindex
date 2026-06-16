pub mod cli;
pub mod executor;
pub mod pipeline;

use anyhow::Result;
use clap::Subcommand;
use cli::RunPipeline;

#[derive(Subcommand)]
#[command(about = "Local database operations")]
pub enum LocalDbCommands {
    #[command(name = "sync")]
    Sync(RunPipeline),
}

impl LocalDbCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            LocalDbCommands::Sync(cmd) => cmd.execute().await,
        }
    }
}
