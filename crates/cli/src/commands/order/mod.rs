mod add;
mod calldata;
mod compose;
mod detail;
mod list;
mod listorderfrontmatterkeys;
mod raindex_address;
mod remove;

use crate::commands::order::raindex_address::RaindexAddress;
use crate::execute::Execute;
use add::CliOrderAddArgs;
use anyhow::Result;
use calldata::AddOrderCalldata;
use clap::Parser;
use compose::Compose;
use listorderfrontmatterkeys::ListOrderFrontmatterKeys;

use detail::CliOrderDetailArgs;
use list::CliOrderListArgs;
use remove::CliOrderRemoveArgs;

#[derive(Parser)]
pub enum Order {
    #[command(about = "List all Orders", alias = "ls")]
    List(CliOrderListArgs),

    #[command(about = "View an Order", alias = "view")]
    Detail(CliOrderDetailArgs),

    #[command(about = "Create an Order", alias = "add")]
    Create(CliOrderAddArgs),

    #[command(about = "Remove an Order", alias = "rm")]
    Remove(CliOrderRemoveArgs),

    #[command(about = "Compose a .rain order file to Rainlang", alias = "comp")]
    Compose(Compose),

    #[command(
        about = "Generate calldata for addOrder from a composition",
        alias = "call"
    )]
    Calldata(AddOrderCalldata),

    #[command(
        about = "Get the raindex address for a given order",
        alias = "raindex-addr"
    )]
    RaindexAddress(RaindexAddress),

    #[command(about = "Get frontmatter keys from a dotrain file", alias = "keys")]
    ListOrderFrontmatterKeys(ListOrderFrontmatterKeys),
}

impl Execute for Order {
    async fn execute(&self) -> Result<()> {
        match self {
            Order::List(list) => list.execute().await,
            Order::Detail(detail) => detail.execute().await,
            Order::Create(create) => create.execute().await,
            Order::Remove(remove) => remove.execute().await,
            Order::Compose(compose) => compose.execute().await,
            Order::Calldata(calldata) => calldata.execute().await,
            Order::RaindexAddress(raindex_address) => raindex_address.execute().await,
            Order::ListOrderFrontmatterKeys(keys) => keys.execute().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_command() {
        Order::command().debug_assert();
    }
}
