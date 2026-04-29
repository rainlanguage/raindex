use alloy::primitives::Address;
use clap::Args;
use raindex_common::transaction::TransactionArgs;

#[derive(Args, Clone)]
pub struct CliTransactionArgs {
    #[arg(long, help = "Raindex contract address")]
    pub raindex_address: Address,

    #[arg(
        short,
        long,
        help = "Derivation index of the Ledger wallet address to use",
        default_value = "0"
    )]
    pub derivation_index: Option<usize>,

    #[arg(short, long, help = "Chain ID of the network")]
    pub chain_id: Option<u64>,

    #[arg(short, long, help = "RPC URLs", num_args = 1..)]
    pub rpcs: Vec<String>,

    #[arg(short = 'p', long, help = "Max priority fee per gas (in wei)")]
    pub max_priority_fee_per_gas: Option<u128>,

    #[arg(short, long, help = "Max fee per gas (in wei)")]
    pub max_fee_per_gas: Option<u128>,
}

impl From<CliTransactionArgs> for TransactionArgs {
    fn from(val: CliTransactionArgs) -> Self {
        TransactionArgs {
            raindex_address: val.raindex_address,
            derivation_index: val.derivation_index,
            chain_id: val.chain_id,
            rpcs: val.rpcs,
            max_priority_fee_per_gas: val.max_priority_fee_per_gas,
            max_fee_per_gas: val.max_fee_per_gas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_cli_transaction_args() {
        let cli_args = CliTransactionArgs {
            raindex_address: Address::ZERO,
            derivation_index: Some(1),
            chain_id: Some(2),
            rpcs: vec!["http://localhost:8545".to_string()],
            max_priority_fee_per_gas: Some(100),
            max_fee_per_gas: Some(1000),
        };

        let transaction_args: TransactionArgs = cli_args.into();

        assert_eq!(transaction_args.raindex_address, Address::ZERO);
        assert_eq!(transaction_args.derivation_index, Some(1));
        assert_eq!(transaction_args.chain_id, Some(2));
        assert_eq!(transaction_args.rpcs, vec!["http://localhost:8545"]);
        assert_eq!(transaction_args.max_priority_fee_per_gas, Some(100));
        assert_eq!(transaction_args.max_fee_per_gas, Some(1000));

        let raindex_address = Address::random();
        let cli_args = CliTransactionArgs {
            raindex_address,
            derivation_index: None,
            chain_id: None,
            rpcs: vec!["http://localhost:8545".to_string()],
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        };

        let transaction_args: TransactionArgs = cli_args.into();

        assert_eq!(transaction_args.raindex_address, raindex_address);
        assert_eq!(transaction_args.derivation_index, None);
        assert_eq!(transaction_args.chain_id, None);
        assert_eq!(transaction_args.rpcs, vec!["http://localhost:8545"]);
        assert_eq!(transaction_args.max_priority_fee_per_gas, None);
        assert_eq!(transaction_args.max_fee_per_gas, None);
    }
}
