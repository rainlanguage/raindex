use crate::execute::Execute;
use raindex_quote::cli::Quoter;

impl Execute for Quoter {
    async fn execute(&self) -> anyhow::Result<()> {
        self.run().await.map(|_| ())
    }
}
