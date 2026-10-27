use super::vaults::RaindexVault;
use super::{RaindexClient, RaindexError};
use crate::transaction::{read_call, TransactionArgsError};
use alloy::primitives::{Address, Bytes, B256};
use alloy::sol_types::SolCall;
use futures::future::try_join_all;
use rain_math_float::Float;
use raindex_bindings::{IRaindexV6::vaultBalance2Call, Raindex::multicallCall};
use std::collections::{BTreeMap, BTreeSet};

const VAULTLESS_BALANCE_CHUNK_SIZE: usize = 100;

type OwnerToken = (Address, Address);

async fn read_chunk(
    rpcs: &[String],
    raindex: Address,
    keys: &[OwnerToken],
) -> Result<Vec<(OwnerToken, Float)>, RaindexError> {
    let data = keys
        .iter()
        .map(|&(owner, token)| {
            Bytes::from(
                vaultBalance2Call {
                    owner,
                    token,
                    vaultId: B256::ZERO,
                }
                .abi_encode(),
            )
        })
        .collect();
    let results = read_call(rpcs, raindex, multicallCall { data }).await?;
    if results.len() != keys.len() {
        return Err(TransactionArgsError::AbiDecode(format!(
            "vaultBalance2 multicall returned {} results for {} calls",
            results.len(),
            keys.len()
        ))
        .into());
    }
    keys.iter()
        .zip(results)
        .map(|(key, result)| {
            let raw = vaultBalance2Call::abi_decode_returns(&result)
                .map_err(|e| TransactionArgsError::AbiDecode(e.to_string()))?;
            Ok((*key, Float::from_raw(raw)))
        })
        .collect()
}

async fn read_raindex(
    rpcs: Vec<String>,
    chain_id: u32,
    raindex: Address,
    keys: Vec<OwnerToken>,
) -> Result<Vec<((u32, Address, OwnerToken), Float)>, RaindexError> {
    let chunks = try_join_all(
        keys.chunks(VAULTLESS_BALANCE_CHUNK_SIZE)
            .map(|chunk| read_chunk(&rpcs, raindex, chunk)),
    )
    .await?;
    Ok(chunks
        .into_iter()
        .flatten()
        .map(|(key, balance)| ((chain_id, raindex, key), balance))
        .collect())
}

impl RaindexClient {
    /// Replaces the indexed balance of every vaultless vault with its live
    /// `vaultBalance2(owner, token, 0)`, one raindex `multicall` per raindex.
    pub(crate) async fn read_vaultless_balances<'a>(
        &self,
        vaults: impl IntoIterator<Item = &'a mut RaindexVault>,
    ) -> Result<(), RaindexError> {
        let vaults: Vec<&mut RaindexVault> =
            vaults.into_iter().filter(|v| v.is_vaultless()).collect();
        if vaults.is_empty() {
            return Ok(());
        }

        let mut grouped: BTreeMap<(u32, Address), BTreeSet<OwnerToken>> = BTreeMap::new();
        for vault in &vaults {
            grouped
                .entry((vault.chain_id_raw(), vault.raindex_address()))
                .or_default()
                .insert((vault.owner_address(), vault.token_address()));
        }

        let reads = grouped
            .into_iter()
            .map(|((chain_id, raindex), keys)| {
                let rpcs = self
                    .get_rpc_urls_for_chain(chain_id)?
                    .iter()
                    .map(|rpc| rpc.to_string())
                    .collect();
                Ok(read_raindex(
                    rpcs,
                    chain_id,
                    raindex,
                    keys.into_iter().collect(),
                ))
            })
            .collect::<Result<Vec<_>, RaindexError>>()?;
        let balances: BTreeMap<_, _> = try_join_all(reads).await?.into_iter().flatten().collect();

        for vault in vaults {
            let key = (
                vault.chain_id_raw(),
                vault.raindex_address(),
                (vault.owner_address(), vault.token_address()),
            );
            if let Some(balance) = balances.get(&key) {
                vault.set_balance(*balance)?;
            }
        }
        Ok(())
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::raindex_client::tests::get_test_yaml;
    use crate::raindex_client::ClientRef;
    use alloy::hex::encode_prefixed;
    use alloy::primitives::U256;
    use httpmock::MockServer;
    use raindex_subgraph_client::types::common::{SgBigInt, SgBytes, SgErc20, SgRaindex, SgVault};
    use raindex_test_fixtures::LocalEvm;
    use serde_json::json;

    async fn client(rpc1: &str) -> ClientRef {
        ClientRef::new(
            RaindexClient::new(
                vec![get_test_yaml(
                    "http://sg1",
                    "http://sg2",
                    rpc1,
                    "http://rpc2",
                )],
                None,
                None,
            )
            .await
            .unwrap(),
        )
    }

    fn vault(
        client: &ClientRef,
        raindex: Address,
        owner: Address,
        vault_id: u64,
        token: Address,
        indexed: &str,
    ) -> RaindexVault {
        RaindexVault::try_from_sg_vault(
            client.clone(),
            1,
            SgVault {
                id: SgBytes(encode_prefixed(B256::from(U256::from(vault_id)))),
                owner: SgBytes(owner.to_string()),
                vault_id: SgBytes(vault_id.to_string()),
                balance: SgBytes(float(indexed).as_hex()),
                token: SgErc20 {
                    id: SgBytes(token.to_string()),
                    address: SgBytes(token.to_string()),
                    name: Some("T".to_string()),
                    symbol: Some("T".to_string()),
                    decimals: Some(SgBigInt("18".to_string())),
                },
                raindex: SgRaindex {
                    id: SgBytes(raindex.to_string()),
                },
                orders_as_output: vec![],
                orders_as_input: vec![],
                balance_changes: vec![],
            },
            None,
        )
        .unwrap()
    }

    fn float(value: &str) -> Float {
        Float::parse(value.to_string()).unwrap()
    }

    fn units(value: u64) -> U256 {
        U256::from(value) * U256::from(10).pow(U256::from(18))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn vaultless_balance_equals_onchain_vault_balance2_capped_by_wallet_or_allowance() {
        let mut local_evm = LocalEvm::new().await;
        let funder = local_evm.anvil.addresses()[0];
        let owner = local_evm.anvil.addresses()[1];
        let raindex = *local_evm.raindex.address();
        let wallet_capped = *local_evm
            .deploy_new_token("A", "A", 18, units(1000), funder)
            .await
            .address();
        let allowance_capped = *local_evm
            .deploy_new_token("B", "B", 18, units(1000), funder)
            .await
            .address();

        for (token, wallet, allowance) in [(wallet_capped, 5, 9), (allowance_capped, 7, 2)] {
            let contract = local_evm
                .tokens
                .iter()
                .find(|t| *t.address() == token)
                .unwrap();
            local_evm
                .send_transaction(
                    contract
                        .transfer(owner, units(wallet))
                        .from(funder)
                        .into_transaction_request(),
                )
                .await
                .unwrap();
            local_evm
                .send_transaction(
                    contract
                        .approve(raindex, units(allowance))
                        .from(owner)
                        .into_transaction_request(),
                )
                .await
                .unwrap();
        }

        let client = client(&local_evm.url()).await;
        let mut vaults = [
            vault(&client, raindex, owner, 0, wallet_capped, "0"),
            vault(&client, raindex, owner, 0, allowance_capped, "0"),
        ];
        client
            .read_vaultless_balances(vaults.iter_mut())
            .await
            .unwrap();

        for (vault, expected) in vaults.iter().zip(["5", "2"]) {
            let onchain = Float::from_raw(
                local_evm
                    .raindex
                    .vaultBalance2(owner, vault.token_address(), B256::ZERO)
                    .call()
                    .await
                    .unwrap(),
            );
            assert!(vault.balance().eq(onchain).unwrap());
            assert!(vault.balance().eq(float(expected)).unwrap());
            assert_eq!(vault.formatted_balance(), expected);
        }
    }

    // At rest the vault-0 internal slot is zero on a real raindex, so a nonzero
    // internal component can only be served by a mocked `vaultBalance2`.
    #[tokio::test]
    async fn vaultless_balance_includes_nonzero_internal_balance() {
        let server = MockServer::start_async().await;
        let raindex = Address::repeat_byte(0x11);
        let owner = Address::repeat_byte(0x22);
        let token = Address::repeat_byte(0x33);
        // min(wallet 3, allowance 4) + internal 1.5
        let live = float("4.5");
        let call = encode_prefixed(
            vaultBalance2Call {
                owner,
                token,
                vaultId: B256::ZERO,
            }
            .abi_encode(),
        );
        let result = encode_prefixed(multicallCall::abi_encode_returns(&vec![Bytes::from(
            live.get_inner().to_vec(),
        )]));
        let rpc = server.mock(|when, then| {
            when.path("/rpc1").body_contains(&call[2..]);
            then.status(200)
                .json_body(json!({ "jsonrpc": "2.0", "id": 1, "result": result }));
        });

        let client = client(&server.url("/rpc1")).await;
        let mut vaults = [vault(&client, raindex, owner, 0, token, "0")];
        client
            .read_vaultless_balances(vaults.iter_mut())
            .await
            .unwrap();

        rpc.assert_hits(1);
        assert!(vaults[0].balance().eq(live).unwrap());
        assert_eq!(vaults[0].formatted_balance(), "4.5");
    }

    #[tokio::test]
    async fn non_vaultless_vault_keeps_indexed_balance_without_rpc() {
        let server = MockServer::start_async().await;
        let rpc = server.mock(|when, then| {
            when.path("/rpc1");
            then.status(500);
        });
        let client = client(&server.url("/rpc1")).await;
        let mut vaults = [vault(
            &client,
            Address::repeat_byte(0x11),
            Address::repeat_byte(0x22),
            7,
            Address::repeat_byte(0x33),
            "12.25",
        )];

        client
            .read_vaultless_balances(vaults.iter_mut())
            .await
            .unwrap();

        rpc.assert_hits(0);
        assert!(vaults[0].balance().eq(float("12.25")).unwrap());
        assert_eq!(vaults[0].formatted_balance(), "12.25");
    }

    #[tokio::test]
    async fn vaultless_balance_rpc_failure_is_an_error() {
        let server = MockServer::start_async().await;
        server.mock(|when, then| {
            when.path("/rpc1");
            then.status(500);
        });
        let client = client(&server.url("/rpc1")).await;
        let mut vaults = [vault(
            &client,
            Address::repeat_byte(0x11),
            Address::repeat_byte(0x22),
            0,
            Address::repeat_byte(0x33),
            "99",
        )];

        let err = client
            .read_vaultless_balances(vaults.iter_mut())
            .await
            .unwrap_err();
        assert!(matches!(err, RaindexError::TransactionArgsError(_)));
    }
}
