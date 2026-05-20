#[cfg(not(target_family = "wasm"))]
use crate::transaction::TransactionArgs;
use crate::transaction::TransactionArgsError;
use alloy::primitives::hex::FromHexError;
use alloy::sol_types::SolCall;
use alloy_ethers_typecast::WritableClientError;
#[cfg(not(target_family = "wasm"))]
use alloy_ethers_typecast::{WriteTransaction, WriteTransactionStatus};
use raindex_bindings::IRaindexV6::removeOrder3Call;
use raindex_subgraph_client::types::{common::SgOrder, order_detail_traits::OrderDetailError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RemoveOrderArgsError {
    #[error(transparent)]
    WritableClientError(#[from] WritableClientError),
    #[error(transparent)]
    TransactionArgs(#[from] TransactionArgsError),
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    OrderDetailError(#[from] OrderDetailError),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoveOrderArgs {
    pub order: SgOrder,
}

impl From<SgOrder> for RemoveOrderArgs {
    fn from(order: SgOrder) -> Self {
        Self { order }
    }
}

impl TryInto<removeOrder3Call> for RemoveOrderArgs {
    type Error = OrderDetailError;

    fn try_into(self) -> Result<removeOrder3Call, OrderDetailError> {
        Ok(removeOrder3Call {
            order: self.order.try_into()?,
            tasks: vec![],
        })
    }
}

impl RemoveOrderArgs {
    #[cfg(not(target_family = "wasm"))]
    pub async fn execute<S: Fn(WriteTransactionStatus<removeOrder3Call>)>(
        self,
        transaction_args: TransactionArgs,
        transaction_status_changed: S,
    ) -> Result<(), RemoveOrderArgsError> {
        let (ledger_client, _) = transaction_args.clone().try_into_ledger_client().await?;

        let remove_order_call: removeOrder3Call = self.try_into()?;
        let params = transaction_args.try_into_write_contract_parameters(
            remove_order_call,
            transaction_args.raindex_address,
        )?;

        WriteTransaction::new(ledger_client, params, 4, transaction_status_changed)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_rm_order_calldata(self) -> Result<Vec<u8>, RemoveOrderArgsError> {
        let remove_order_call: removeOrder3Call = self.try_into()?;
        Ok(remove_order_call.abi_encode())
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use raindex_bindings::IRaindexV6::removeOrder3Call;
    use raindex_subgraph_client::types::common::{SgBigInt, SgBytes, SgErc20, SgRaindex, SgVault};
    use raindex_subgraph_client::utils::float::*;

    fn get_order() -> SgOrder {
        SgOrder {
            id: SgBytes(
                "0x99db88d9726c5dbcea240be8eba022d2fd6dd40f6d947dc28cda692a9ba8e6ae".to_string(),
            ),
            order_bytes: SgBytes(
                "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000018a62a3ac2ca9f775a4a12380eda03245270b73e00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000002602d79fbdfab4699235b5e9d2144f3bab5e2d887c3f4df9d1a1a1bef1d9d5b81b20000000000000000000000005fb33d710f8b58de4c9fdec703b5c2487a5219d600000000000000000000000084c6e7f5a1e5dd89594cc25bef4722a1b8871ae6000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000ad00000000000000000000000000000000000000000000000000000000000000020000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000214e8348c4f0000000000000000000000000000000000000000000000000000000000000000002d0200000024080500021810000001100001361100000110000101100000031000041e12000022130000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000b38e83b86d491735feaa0a791f65c2b995353960000000000000000000000000000000000000000000000000000000000000006d23be6b1eb755ac469bba083cade61d48f156fc531662469291dc04a0782964500000000000000000000000000000000000000000000000000000000000000010000000000000000000000001d80c49bbbcd1c0911346656b529df9e5c2f783d0000000000000000000000000000000000000000000000000000000000000012d23be6b1eb755ac469bba083cade61d48f156fc531662469291dc04a07829645".to_string(),
            ),
            order_hash: SgBytes(
                "0x99db88d9726c5dbcea240be8eba022d2fd6dd40f6d947dc28cda692a9ba8e6ae".to_string(),
            ),
            owner: SgBytes("0x18a62a3ac2ca9f775a4a12380eda03245270b73e".to_string()),
            outputs: vec![
                SgVault {
                    id: SgBytes("0xfd661f641ed6f13210fa83be991d7afc8e290202473f1fa9548a8e5654984575".to_string()),
                    owner: SgBytes("0x18a62a3ac2ca9f775a4a12380eda03245270b73e".to_string()),
                    vault_id: SgBytes("95091534377674853556918913044061641909871616138258204934350492514947914962501".to_string()),
                    balance: SgBytes(F50.as_hex()),
                    token: SgErc20 {
                        id: SgBytes("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d".to_string()),
                        address: SgBytes("0x1d80c49bbbcd1c0911346656b529df9e5c2f783d".to_string()),
                        name: Some("Wrapped Flare".to_string()),
                        symbol: Some("WFLR".to_string()),
                        decimals: Some(SgBigInt("18".to_string())),
                    },
                    raindex: SgRaindex {
                        id: SgBytes("0xcee8cd002f151a536394e564b84076c41bbbcd4d".to_string()),
                    },
                    orders_as_output: vec![],
                    orders_as_input: vec![],
                    balance_changes: vec![],
                },
            ],
            inputs: vec![],
            raindex: SgRaindex {
                id: SgBytes("0xcee8cd002f151a536394e564b84076c41bbbcd4d".to_string()),
            },
            active: true,
            timestamp_added: SgBigInt("1745524403".to_string()),
            meta: None,
            add_events: vec![],
            remove_events: vec![],
            trades: vec![],
        }
    }

    #[tokio::test]
    async fn test_remove_order_calldata() {
        let remove_order_args = RemoveOrderArgs { order: get_order() };
        let calldata = remove_order_args.get_rm_order_calldata().await.unwrap();

        let remove_order_call = removeOrder3Call {
            order: get_order().try_into().unwrap(),
            tasks: vec![],
        };
        let expected_calldata = remove_order_call.abi_encode();

        assert_eq!(calldata, expected_calldata);
    }

    #[test]
    fn test_try_into_remove_order_call() {
        let remove_order_call = removeOrder3Call {
            order: get_order().try_into().unwrap(),
            tasks: vec![],
        };

        let args = TransactionArgs {
            rpcs: vec!["http://test.com".to_string()],
            raindex_address: Address::ZERO,
            derivation_index: Some(0_usize),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(200),
            max_fee_per_gas: Some(100),
        };

        let params = args
            .try_into_write_contract_parameters(remove_order_call.clone(), args.raindex_address)
            .unwrap();
        assert_eq!(params.address, args.raindex_address);
        assert_eq!(params.call, remove_order_call);
        assert_eq!(params.max_priority_fee_per_gas, Some(200));
        assert_eq!(params.max_fee_per_gas, Some(100));
    }
}
