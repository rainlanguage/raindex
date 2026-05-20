use crate::{
    execute::Execute, status::display_write_transaction_status, subgraph::CliSubgraphArgs,
    transaction::CliTransactionArgs,
};
use anyhow::Result;
use clap::Args;
use raindex_common::remove_order::RemoveOrderArgs;
use raindex_common::subgraph::SubgraphArgs;
use raindex_common::transaction::TransactionArgs;
use tracing::info;

#[derive(Args, Clone)]
pub struct CliOrderRemoveArgs {
    #[arg(short = 'i', long, help = "ID of the Order")]
    order_id: String,

    #[clap(flatten)]
    pub subgraph_args: CliSubgraphArgs,

    #[clap(flatten)]
    pub transaction_args: CliTransactionArgs,
}

impl Execute for CliOrderRemoveArgs {
    async fn execute(&self) -> Result<()> {
        let subgraph_args: SubgraphArgs = self.subgraph_args.clone().into();
        let order = subgraph_args
            .to_subgraph_client()?
            .order_detail(&self.order_id.clone().into())
            .await?;
        let remove_order_args: RemoveOrderArgs = order.into();

        let mut tx_args: TransactionArgs = self.transaction_args.clone().into();
        tx_args.try_fill_chain_id().await?;

        info!("----- Remove Order -----");
        remove_order_args
            .execute(tx_args, |status| {
                display_write_transaction_status(status);
            })
            .await?;

        Ok(())
    }
}
