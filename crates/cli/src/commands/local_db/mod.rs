pub mod cli;
pub mod executor;
pub mod manifest_urls;
pub mod pipeline;

use anyhow::Result;
use clap::Subcommand;
use cli::RunPipeline;
use manifest_urls::ManifestUrls;

#[derive(Subcommand)]
#[command(about = "Local database operations")]
pub enum LocalDbCommands {
    #[command(name = "sync")]
    Sync(RunPipeline),

    #[command(name = "manifest-urls")]
    ManifestUrls(ManifestUrls),
}

impl LocalDbCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            LocalDbCommands::Sync(cmd) => cmd.execute().await,
            LocalDbCommands::ManifestUrls(cmd) => cmd.execute(),
        }
    }
}
