use crate::execute::Execute;
use anyhow::Result;
use clap::Parser;
use raindex_subgraph_client::validate::validate_subgraph_schema;

#[derive(Parser)]
pub enum Subgraph {
    #[command(about = "Validates a subgraph schema against this apps subgraph schema")]
    Validate {
        /// Subgraph url to validate
        subgraph_url: String,
    },
}

impl Execute for Subgraph {
    async fn execute(&self) -> Result<()> {
        match self {
            Subgraph::Validate { subgraph_url } => Ok(validate_subgraph_schema(subgraph_url)
                .await
                .map(|v| println!("--- {}valid subgraph ---", if v { "" } else { "in" }))?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_command() {
        Subgraph::command().debug_assert();
    }
}
