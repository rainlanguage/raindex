#[cfg(test)]
use crate::injector::NoopInjector;
use crate::{
    error::Error,
    injector::SignedContextInjector,
    quote::{BatchQuoteTarget, QuoteTarget},
    OrderQuoteValue,
};
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use raindex_bindings::provider::mk_read_provider;
use raindex_bindings::IRaindexV6::{OrderV4, QuoteV2, SignedContextV1};
use raindex_subgraph_client::types::common::SgOrder;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, info, warn};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::Date;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

struct QuoteTiming {
    #[cfg(not(target_family = "wasm"))]
    started_at: std::time::Instant,
    #[cfg(target_family = "wasm")]
    started_at_ms: f64,
}

impl QuoteTiming {
    fn now() -> Self {
        Self {
            #[cfg(not(target_family = "wasm"))]
            started_at: std::time::Instant::now(),
            #[cfg(target_family = "wasm")]
            started_at_ms: Date::now(),
        }
    }

    fn elapsed_ms(&self) -> u64 {
        #[cfg(not(target_family = "wasm"))]
        {
            self.started_at.elapsed().as_millis() as u64
        }

        #[cfg(target_family = "wasm")]
        {
            let elapsed = Date::now() - self.started_at_ms;
            if elapsed.is_finite() && elapsed > 0.0 {
                elapsed as u64
            } else {
                0
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct BatchOrderQuotesResponse {
    pub pair: Pair,
    pub block_number: u64,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub data: Option<OrderQuoteValue>,
    pub success: bool,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub error: Option<String>,
    /// Composed signed context that was sent with the quote RPC: any
    /// oracle-fetched entries first, followed by injector-contributed entries
    /// (composition order: `[oracle..., injected...]`). This is propagated so
    /// downstream candidate construction can reuse the same context that the
    /// quote call saw, rather than re-fetching or re-composing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub signed_context: Vec<SignedContextV1>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(BatchOrderQuotesResponse);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub pair_name: String,
    pub input_index: u32,
    pub output_index: u32,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(Pair);

/// Get order quotes, automatically fetching signed oracle context from order
/// meta and appending any caller-supplied injector contexts.
///
/// For each order, if the meta contains a `RaindexSignedContextOracleV1`
/// entry, the oracle URL is extracted and signed context is fetched per IO
/// pair via POST. Any additional entries produced by `injector` are appended
/// after the oracle entries (composition order: `[oracle..., injected...]`),
/// and the composed list is attached to the `QuoteV2.signedContext` before
/// the multicall is issued. This matters for gated orders whose
/// `calculate-io` asserts on signed context during quoting.
pub async fn get_order_quotes(
    orders: Vec<SgOrder>,
    block_number: Option<u64>,
    rpcs: Vec<String>,
    chunk_size: Option<usize>,
    counterparty: Address,
    injector: &dyn SignedContextInjector,
) -> Result<Vec<BatchOrderQuotesResponse>, Error> {
    let started_at = QuoteTiming::now();
    info!(
        order_count = orders.len(),
        rpc_url_count = rpcs.len(),
        block_number = ?block_number,
        chunk_size = ?chunk_size,
        counterparty = %counterparty,
        "starting quote pair sweep"
    );

    let block_started_at = QuoteTiming::now();
    let req_block_number = match block_number {
        Some(block) => block,
        None => {
            let urls = rpcs
                .iter()
                .map(|rpc| rpc.parse())
                .collect::<Result<Vec<_>, _>>()
                .map_err(Error::UrlParseError)?;
            let provider = mk_read_provider(&urls).map_err(Error::ReadProviderError)?;
            provider
                .get_block_number()
                .await
                .map_err(|e| Error::TransportError(e.to_string()))?
        }
    };
    info!(
        requested_block_number = ?block_number,
        resolved_block_number = req_block_number,
        duration_ms = block_started_at.elapsed_ms(),
        "resolved quote block number"
    );

    // Responses are assembled in strict iteration order. Pairs whose oracle
    // fetch failed get a failure response stored immediately; quoted pairs
    // leave a hole that is filled in after the RPC batch returns. This
    // preserves per-order positional alignment for callers that re-slice the
    // flat response vector by per-order pair counts.
    let mut all_responses: Vec<Option<BatchOrderQuotesResponse>> = Vec::new();
    // Parallel tracking for the subset of iteration slots that were quoted,
    // so we can scatter RPC results back into `all_responses`.
    let mut all_pairs: Vec<Pair> = Vec::new();
    let mut all_quote_targets: Vec<QuoteTarget> = Vec::new();
    let mut all_signed_contexts: Vec<Vec<SignedContextV1>> = Vec::new();
    let mut quoted_slot_indices: Vec<usize> = Vec::new();
    let mut skipped_self_trade_pair_count = 0usize;
    let mut oracle_fetch_count = 0usize;
    let mut oracle_fetch_failure_count = 0usize;
    let mut injected_context_count = 0usize;

    let target_build_started_at = QuoteTiming::now();
    for order in &orders {
        let order_struct: OrderV4 = order.clone().try_into()?;
        let raindex = Address::from_str(&order.raindex.id.0)?;
        let oracle_url = crate::oracle::extract_oracle_url(order);

        for (input_index, input) in order_struct.validInputs.iter().enumerate() {
            for (output_index, output) in order_struct.validOutputs.iter().enumerate() {
                if input.token == output.token {
                    skipped_self_trade_pair_count += 1;
                    continue;
                }

                let pair_name = format!(
                    "{}/{}",
                    order
                        .inputs
                        .iter()
                        .find_map(|v| {
                            Address::from_str(&v.token.address.0).ok().and_then(|add| {
                                add.eq(&input.token).then_some(
                                    v.token.symbol.clone().unwrap_or("UNKNOWN".to_string()),
                                )
                            })
                        })
                        .unwrap_or("UNKNOWN".to_string()),
                    order
                        .outputs
                        .iter()
                        .find_map(|v| {
                            Address::from_str(&v.token.address.0).ok().and_then(|add| {
                                add.eq(&output.token).then_some(
                                    v.token.symbol.clone().unwrap_or("UNKNOWN".to_string()),
                                )
                            })
                        })
                        .unwrap_or("UNKNOWN".to_string())
                );

                // Fetch signed oracle context for this pair if oracle URL is present.
                // NOTE: Oracle context is always fetched live (current price), even when
                // the quote is pinned to a historical block. This means historical quotes
                // for oracle-backed orders reflect current oracle prices against past chain
                // state, which may not represent a quote that actually existed.
                let oracle_context = if let Some(ref url) = oracle_url {
                    oracle_fetch_count += 1;
                    let oracle_started_at = QuoteTiming::now();
                    let body = crate::oracle::encode_oracle_body(
                        &order_struct,
                        input_index as u32,
                        output_index as u32,
                        counterparty,
                    );
                    match crate::oracle::fetch_signed_context(url, body).await {
                        Ok(ctx) => {
                            debug!(
                                raindex = %raindex,
                                input_index,
                                output_index,
                                duration_ms = oracle_started_at.elapsed_ms(),
                                "fetched quote oracle context"
                            );
                            Ok(vec![ctx])
                        }
                        Err(e) => {
                            oracle_fetch_failure_count += 1;
                            warn!(
                                raindex = %raindex,
                                input_index,
                                output_index,
                                duration_ms = oracle_started_at.elapsed_ms(),
                                error = %e,
                                "quote oracle context fetch failed"
                            );
                            Err(format!(
                                "Oracle fetch failed for pair ({}, {}): {}",
                                input_index, output_index, e
                            ))
                        }
                    }
                } else {
                    Ok(vec![])
                };

                let pair = Pair {
                    pair_name,
                    input_index: input_index as u32,
                    output_index: output_index as u32,
                };

                let slot_idx = all_responses.len();
                match oracle_context {
                    Ok(oracle_ctx) => {
                        // Append injector-contributed contexts after the oracle
                        // context. Gated orders that verify signed context by
                        // index must know the composition order.
                        let injected = injector
                            .contexts_for(
                                &order_struct,
                                input_index as u32,
                                output_index as u32,
                                counterparty,
                            )
                            .await;
                        injected_context_count += injected.len();
                        let composed: Vec<SignedContextV1> =
                            oracle_ctx.into_iter().chain(injected).collect();

                        all_responses.push(None);
                        all_pairs.push(pair);
                        all_signed_contexts.push(composed.clone());
                        all_quote_targets.push(QuoteTarget {
                            raindex,
                            quote_config: QuoteV2 {
                                order: order_struct.clone(),
                                inputIOIndex: U256::from(input_index),
                                outputIOIndex: U256::from(output_index),
                                signedContext: composed,
                            },
                        });
                        quoted_slot_indices.push(slot_idx);
                    }
                    Err(e) => {
                        all_responses.push(Some(BatchOrderQuotesResponse {
                            pair,
                            block_number: req_block_number,
                            success: false,
                            data: None,
                            error: Some(e),
                            signed_context: vec![],
                        }));
                    }
                }
            }
        }
    }
    info!(
        order_count = orders.len(),
        target_count = all_quote_targets.len(),
        response_slot_count = all_responses.len(),
        skipped_self_trade_pair_count,
        oracle_fetch_count,
        oracle_fetch_failure_count,
        injected_context_count,
        duration_ms = target_build_started_at.elapsed_ms(),
        "built quote targets"
    );

    let batch_quote_started_at = QuoteTiming::now();
    let quote_results: Vec<BatchOrderQuotesResponse> = match BatchQuoteTarget(all_quote_targets)
        .do_quote(rpcs, Some(req_block_number), counterparty, chunk_size)
        .await
    {
        Ok(quote_values) => {
            let failed_quote_count = quote_values.iter().filter(|result| result.is_err()).count();
            let successful_quote_count = quote_values.len().saturating_sub(failed_quote_count);
            info!(
                successful_quote_count,
                failed_quote_count,
                duration_ms = batch_quote_started_at.elapsed_ms(),
                "completed quote batch RPC"
            );
            quote_values
                .into_iter()
                .zip(all_pairs)
                .zip(all_signed_contexts)
                .map(
                    |((quote_result, pair), signed_context)| match quote_result {
                        Ok(data) => BatchOrderQuotesResponse {
                            pair,
                            block_number: req_block_number,
                            success: true,
                            data: Some(data),
                            error: None,
                            signed_context,
                        },
                        Err(e) => BatchOrderQuotesResponse {
                            pair,
                            block_number: req_block_number,
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                            signed_context,
                        },
                    },
                )
                .collect()
        }
        Err(e) => {
            let error = e.to_string();
            warn!(
                quoted_pair_count = all_pairs.len(),
                duration_ms = batch_quote_started_at.elapsed_ms(),
                error = %error,
                "quote batch RPC failed"
            );
            all_pairs
                .into_iter()
                .zip(all_signed_contexts)
                .map(|(pair, signed_context)| BatchOrderQuotesResponse {
                    pair,
                    block_number: req_block_number,
                    success: false,
                    data: None,
                    error: Some(error.clone()),
                    signed_context,
                })
                .collect()
        }
    };

    // Scatter quote results back into the iteration-ordered response vector.
    for (slot_idx, response) in quoted_slot_indices.into_iter().zip(quote_results) {
        all_responses[slot_idx] = Some(response);
    }

    let responses: Vec<_> = all_responses.into_iter().map(|r| r.unwrap()).collect();
    let successful_quote_count = responses.iter().filter(|response| response.success).count();
    let failed_quote_count = responses.len().saturating_sub(successful_quote_count);
    info!(
        order_count = orders.len(),
        response_count = responses.len(),
        successful_quote_count,
        failed_quote_count,
        resolved_block_number = req_block_number,
        duration_ms = started_at.elapsed_ms(),
        "completed quote pair sweep"
    );

    Ok(responses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        hex::{encode_prefixed, FromHexError},
        primitives::B256,
        providers::Provider,
        sol_types::{SolCall, SolValue},
    };
    use rain_math_float::Float;
    use raindex_app_settings::spec_version::SpecVersion;
    use raindex_common::{add_order::AddOrderArgs, dotrain_order::DotrainOrder};
    use raindex_subgraph_client::types::{
        common::{SgBigInt, SgBytes, SgErc20, SgRaindex, SgVault},
        order_detail_traits::OrderDetailError,
    };
    use raindex_subgraph_client::utils::float::*;
    use raindex_test_fixtures::LocalEvm;

    struct TestSetup {
        local_evm: LocalEvm,
        owner: Address,
        token1: SgErc20,
        token2: SgErc20,
        raindex: Address,
    }

    async fn setup_test() -> TestSetup {
        let mut local_evm = LocalEvm::new().await;
        let owner = local_evm.signer_wallets[0].default_signer().address();

        let token1 = local_evm
            .deploy_new_token("Token1", "Token1", 18, U256::MAX, owner)
            .await;
        let token2 = local_evm
            .deploy_new_token("Token2", "Token2", 18, U256::MAX, owner)
            .await;
        let raindex = *local_evm.raindex.address();

        TestSetup {
            local_evm,
            owner,
            token1: SgErc20 {
                id: SgBytes(token1.address().to_string()),
                address: SgBytes(token1.address().to_string()),
                name: Some("Token1".to_string()),
                symbol: Some("Token1".to_string()),
                decimals: Some(SgBigInt(18.to_string())),
            },
            token2: SgErc20 {
                id: SgBytes(token2.address().to_string()),
                address: SgBytes(token2.address().to_string()),
                name: Some("Token2".to_string()),
                symbol: Some("Token2".to_string()),
                decimals: Some(SgBigInt(18.to_string())),
            },
            raindex,
        }
    }

    fn create_dotrain_config(setup: &TestSetup) -> String {
        format!(
            r#"
version: {spec_version}
networks:
    some-key:
        rpcs:
            - {rpc_url}
        chain-id: 123
        network-id: 123
        currency: ETH
rainlangs:
    some-key:
        address: {rainlang_address}
tokens:
    t2:
        network: some-key
        address: {token2}
        decimals: 18
        label: Token2
        symbol: Token2
    t1:
        network: some-key
        address: {token1}
        decimals: 18
        label: Token1
        symbol: token1
raindex:
    some-key:
        address: {raindex}
orders:
    some-key:
        inputs:
            - token: t1
            - token: t2
        outputs:
            - token: t1
              vault-id: 0x01
            - token: t2
              vault-id: 0x01
scenarios:
    some-key:
        rainlang: some-key
        bindings:
            key1: 10
deployments:
    some-key:
        scenario: some-key
        order: some-key
---
#key1 !Test binding
#calculate-io
amount price: 2 3;
#handle-add-order
:;
#handle-io
:;
"#,
            rpc_url = setup.local_evm.url(),
            raindex = setup.raindex,
            rainlang_address = setup.local_evm.rainlang,
            token1 = setup.token1.address.0,
            token2 = setup.token2.address.0,
            spec_version = SpecVersion::current(),
        )
    }

    async fn create_order(setup: &TestSetup, dotrain: String) -> String {
        let dotrain_order = DotrainOrder::create(dotrain.clone(), None).await.unwrap();
        let deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment("some-key")
            .unwrap();
        let calldata = AddOrderArgs::new_from_deployment(dotrain, deployment, None)
            .await
            .unwrap()
            .try_into_call(vec![setup.local_evm.url()])
            .await
            .unwrap()
            .abi_encode();

        encode_prefixed(
            setup
                .local_evm
                .add_order(&calldata, setup.owner)
                .await
                .0
                .order
                .abi_encode(),
        )
    }

    fn create_vault(vault_id: B256, setup: &TestSetup, token: &SgErc20) -> SgVault {
        SgVault {
            id: SgBytes(vault_id.to_string()),
            token: token.clone(),
            balance: SgBytes(F6.as_hex()),
            vault_id: SgBytes(vault_id.to_string()),
            owner: SgBytes(setup.local_evm.anvil.addresses()[0].to_string()),
            raindex: SgRaindex {
                id: SgBytes(setup.raindex.to_string()),
            },
            orders_as_input: vec![],
            orders_as_output: vec![],
            balance_changes: vec![],
        }
    }

    fn create_sg_order(
        setup: &TestSetup,
        order_bytes: String,
        inputs: Vec<SgVault>,
        outputs: Vec<SgVault>,
    ) -> SgOrder {
        SgOrder {
            id: SgBytes(B256::random().to_string()),
            raindex: SgRaindex {
                id: SgBytes(setup.raindex.to_string()),
            },
            order_bytes: SgBytes(order_bytes),
            order_hash: SgBytes(B256::random().to_string()),
            owner: SgBytes(setup.local_evm.anvil.addresses()[0].to_string()),
            outputs,
            inputs,
            active: true,
            add_events: vec![],
            meta: None,
            timestamp_added: SgBigInt(0.to_string()),
            trades: vec![],
            remove_events: vec![],
        }
    }

    #[tokio::test]
    async fn test_get_order_quotes_ok() {
        let setup = setup_test().await;

        let vault_id_const = B256::from(U256::from(1u64));
        let vault_id1 = vault_id_const; // for token1
        let vault_id2 = vault_id_const; // for token2

        // Deposit in token1 and token2 vaults
        setup
            .local_evm
            .deposit(
                setup.owner,
                Address::from_str(&setup.token1.address.0).unwrap(),
                U256::from(10).pow(U256::from(66)),
                18,
                vault_id1,
            )
            .await;
        setup
            .local_evm
            .deposit(
                setup.owner,
                Address::from_str(&setup.token2.address.0).unwrap(),
                U256::from(10).pow(U256::from(66)),
                18,
                vault_id2,
            )
            .await;

        let dotrain = create_dotrain_config(&setup);
        let order = create_order(&setup, dotrain).await;

        let vault1 = create_vault(vault_id1, &setup, &setup.token1);
        let vault2 = create_vault(vault_id2, &setup, &setup.token2);

        // does not follow the actual original order's io order
        let inputs = vec![vault2.clone(), vault1.clone()];
        let outputs = vec![vault2.clone(), vault1.clone()];

        let order = create_sg_order(&setup, order, inputs, outputs);

        let result = get_order_quotes(
            vec![order],
            None,
            vec![setup.local_evm.url()],
            None,
            Address::ZERO,
            &NoopInjector,
        )
        .await
        .unwrap();

        let expected_max_output = Float::parse("2".to_string()).unwrap();
        let expected_ratio = Float::parse("3".to_string()).unwrap();

        let block_number = setup.local_evm.provider.get_block_number().await.unwrap();
        let expected = [
            BatchOrderQuotesResponse {
                pair: Pair {
                    pair_name: "Token1/Token2".to_string(),
                    input_index: 0,
                    output_index: 1,
                },
                block_number,
                data: Some(OrderQuoteValue {
                    max_output: expected_max_output,
                    ratio: expected_ratio,
                }),
                success: true,
                error: None,
                signed_context: vec![],
            },
            BatchOrderQuotesResponse {
                pair: Pair {
                    pair_name: "Token2/Token1".to_string(),
                    input_index: 1,
                    output_index: 0,
                },
                block_number,
                data: Some(OrderQuoteValue {
                    max_output: expected_max_output,
                    ratio: expected_ratio,
                }),
                success: true,
                error: None,
                signed_context: vec![],
            },
        ];

        assert_eq!(result.len(), expected.len());

        for (res, exp) in result.iter().zip(expected.iter()) {
            assert_eq!(res.pair, exp.pair);
            assert_eq!(res.block_number, exp.block_number);
            assert_eq!(res.success, exp.success);
            assert_eq!(res.error, exp.error);

            let actual_data = res.data.unwrap();
            let expected_data = exp.data.unwrap();

            assert!(
                actual_data.max_output.eq(expected_data.max_output).unwrap(),
                "actual_data.max_output: {}, expected_data.max_output: {}",
                actual_data.max_output.format().unwrap(),
                expected_data.max_output.format().unwrap()
            );
            assert!(
                actual_data.ratio.eq(expected_data.ratio).unwrap(),
                "actual_data.ratio: {}, expected_data.ratio: {}",
                actual_data.ratio.format().unwrap(),
                expected_data.ratio.format().unwrap()
            );
        }
    }

    #[tokio::test]
    async fn test_get_order_quotes_err() {
        let setup = setup_test().await;
        let dotrain = create_dotrain_config(&setup);
        let order = create_order(&setup, dotrain).await;

        // Test invalid raindex address
        let mut invalid_order = create_sg_order(&setup, order.clone(), vec![], vec![]);
        invalid_order.raindex.id = SgBytes("invalid_address".to_string());

        let err = get_order_quotes(
            vec![invalid_order],
            None,
            vec![setup.local_evm.url()],
            None,
            Address::ZERO,
            &NoopInjector,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, Error::FromHexError(FromHexError::OddLength)));

        // Test invalid order bytes
        let invalid_order = create_sg_order(&setup, B256::random().to_string(), vec![], vec![]);

        let err = get_order_quotes(
            vec![invalid_order],
            None,
            vec![setup.local_evm.url()],
            None,
            Address::ZERO,
            &NoopInjector,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            Error::OrderDetailError(OrderDetailError::AbiDecode(_))
        ));

        // Test invalid RPC URL
        let valid_order = create_sg_order(&setup, order, vec![], vec![]);

        let err = get_order_quotes(
            vec![valid_order],
            None,
            vec!["invalid_rpc_url".to_string()],
            None,
            Address::ZERO,
            &NoopInjector,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, Error::UrlParseError(_)));
    }
}
