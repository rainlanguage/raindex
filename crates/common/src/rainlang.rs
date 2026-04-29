use crate::dotrain_add_order_lsp::LANG_SERVICES;
use dotrain::error::ComposeError;
use dotrain::RainDocument;
use dotrain::Rebind;
use std::collections::HashMap;

/// Compose to rainlang string by setting elided bindings to zero
pub fn compose_to_rainlang(
    dotrain: String,
    bindings: HashMap<String, String>,
    entrypoints: &[&str],
) -> Result<String, ComposeError> {
    let meta_store = LANG_SERVICES.meta_store();

    let rebinds = (!bindings.is_empty()).then_some(
        bindings
            .iter()
            .map(|(k, v)| Rebind(k.clone(), v.clone()))
            .collect(),
    );

    // compose a new RainDocument with final injected bindings
    RainDocument::create(dotrain, Some(meta_store), None, rebinds).compose(entrypoints)
}

#[cfg(test)]
mod tests {
    use dotrain::{error::ErrorCode, types::ast::Problem};

    use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;

    use super::*;

    #[test]
    fn test_compose_to_rainlang_err_empty_entrypoints() {
        let dotrain = r"
some front matter
---
/** this is test */
                                                                           

#const-binding 4e18
#elided-binding ! this elided, rebind before use
#exp-binding
_: opcode-1(0xabcd 456);
"
        .trim_start();

        let bindings = HashMap::new();
        let entrypoints = &[];

        let err = compose_to_rainlang(dotrain.to_string(), bindings, entrypoints).unwrap_err();
        assert_eq!(
            err,
            ComposeError::Reject("no entrypoints specified".to_owned())
        );
    }

    #[test]
    fn test_compose_to_rainlang_ok_empty_bindings() {
        let dotrain = r"
some front matter
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 1 2;
"
        .trim_start();

        let expected = r"
/* 0. calculate-io */ 
_ _: 0 0;

/* 1. handle-io */ 
:;"
        .trim_start();

        let actual = compose_to_rainlang(
            dotrain.to_string(),
            HashMap::new(),
            &ORDERBOOK_ORDER_ENTRYPOINTS,
        )
        .unwrap();

        assert_eq!(actual, expected);

        let expected = r"
/* 0. handle-add-order */ 
_ _: 1 2;"
            .trim_start();

        let actual =
            compose_to_rainlang(dotrain.to_string(), HashMap::new(), &["handle-add-order"])
                .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_compose_to_rainlang_ok_with_bindings() {
        let dotrain = r"
some front matter
---
/** this is test */
                                                                           

#const-binding 4e18
#elided-binding ! this elided, rebind before use
#exp-binding
_ _: const-binding elided-binding;"
            .trim_start();

        let bindings = HashMap::from([(
            "elided-binding".to_string(),
            "0x1234567890abcdef".to_string(),
        )]);

        let expected = r"
/* 0. exp-binding */ 
_ _: 4e18 0x1234567890abcdef;"
            .trim_start();

        let actual = compose_to_rainlang(dotrain.to_string(), bindings, &["exp-binding"]).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_compose_to_rainlang_err_with_bindings() {
        let dotrain = r"
some front matter
---
/** this is test */
                                                                           

#const-binding 4e18
#elided-binding ! this elided, rebind before use
#exp-binding
_ _: const-binding elided-binding;"
            .trim_start();

        let bindings = HashMap::new();

        let err = compose_to_rainlang(dotrain.to_string(), bindings, &["exp-binding"]).unwrap_err();

        assert!(matches!(
            err,
            ComposeError::Problems(problems) if problems.len() == 1 && matches!(&problems[0], Problem {
                msg,
                code: ErrorCode::ElidedBinding,
                ..
            } if msg == "elided binding 'elided-binding': this elided, rebind before use")
        ));
    }
}

#[cfg(not(target_family = "wasm"))]
pub use fork_parse::*;

#[cfg(not(target_family = "wasm"))]
mod fork_parse {
    use alloy::primitives::{bytes::Bytes, Address};
    use alloy_ethers_typecast::{ReadableClient, ReadableClientError};
    use once_cell::sync::OnceCell;
    use rain_error_decoding::AbiDecodedErrorType;
    use rain_interpreter_eval::error::ForkCallError;
    use rain_interpreter_eval::eval::ForkParseArgs;
    use rain_interpreter_eval::fork::Forker;
    use rain_interpreter_eval::fork::NewForkedEvm;
    use std::sync::Arc;
    use thiserror::Error;
    use tokio::sync::Mutex;

    /// Lazily-initialised shared [`Forker`] wrapped in an `Arc<Mutex<..>>`.
    ///
    /// The instance is created on first access via `get_or_try_init`,
    /// allowing the caller to handle any initialisation errors instead of
    /// unwrapping (panicking) at start-up.
    pub static FORKER: OnceCell<Arc<Mutex<Forker>>> = OnceCell::new();

    #[derive(Debug, Error)]
    pub enum ForkParseError {
        #[error("Fork Cache Poisoned")]
        ForkCachePoisoned,
        #[error(transparent)]
        ForkerError(Box<ForkCallError>),
        #[error("Fork Call Reverted: {0}")]
        ForkCallReverted(#[from] AbiDecodedErrorType),
        #[error(transparent)]
        ReadableClientError(#[from] ReadableClientError),
        #[error("Failed to read Parser address from rainlang")]
        ReadParserAddressFailed,
        #[error("Invalid input args: {0}")]
        InvalidArgs(String),
    }

    impl From<ForkCallError> for ForkParseError {
        fn from(value: ForkCallError) -> Self {
            match value {
                ForkCallError::AbiDecodedError(v) => Self::ForkCallReverted(v),
                other => Self::ForkerError(Box::new(other)),
            }
        }
    }

    /// checks the front matter validity and parses the given rainlang string
    /// with the rainlang address parsed from the front matter
    /// returns abi encoded expression config on Ok variant
    pub async fn parse_rainlang_on_fork(
        rainlang: &str,
        rpcs: &Vec<String>,
        block_number: Option<u64>,
        rainlang_address: Address,
    ) -> Result<Bytes, ForkParseError> {
        // Prepare evm fork
        let block_number_val = match block_number {
            Some(b) => b,
            None => {
                let client = ReadableClient::new_from_http_urls(rpcs.clone())?;
                client.get_block_number().await?
            }
        };

        // Lazily initialize the global `FORKER` (if required) and obtain a lock.
        let fork_arc = FORKER
            .get_or_try_init(|| {
                Forker::new()
                    .map_err(ForkCallError::from)
                    .map(|f| Arc::new(Mutex::new(f)))
            })
            .map_err(ForkParseError::from)?
            .clone();

        let mut forker = fork_arc.lock().await;
        let mut err: Option<ForkParseError> = None;

        if rpcs.is_empty() {
            return Err(ForkParseError::InvalidArgs("rpcs cannot be empty".into()));
        }

        for rpc in rpcs {
            let args = NewForkedEvm {
                fork_url: rpc.to_owned(),
                fork_block_number: Some(block_number_val),
            };
            match forker.add_or_select(args, None).await {
                Ok(_) => {
                    err = None;
                    break;
                }
                Err(e) => {
                    err = Some(ForkParseError::ForkerError(Box::new(e)));
                }
            }
        }
        if let Some(err) = err {
            return Err(err);
        };

        let parse_args = ForkParseArgs {
            rainlang_string: rainlang.to_owned(),
            rainlang: rainlang_address,
            decode_errors: true,
        };
        let result = forker.fork_parse(parse_args).await?;

        Ok(result.raw.result.0)
    }

    #[cfg(test)]
    mod tests {
        use alloy::primitives::hex::encode_prefixed;
        use httpmock::MockServer;
        use std::collections::HashMap;

        use super::*;
        use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
        use rain_orderbook_test_fixtures::LocalEvm;

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_parse_rainlang_on_fork_ok() {
            let local_evm = LocalEvm::new().await;
            let rainlang_addr = local_evm.rainlang;
            let rpc_url = local_evm.url();

            let dotrain = r"
some front matter
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 1 2;
"
            .trim_start();

            let rainlang = super::super::compose_to_rainlang(
                dotrain.to_string(),
                HashMap::new(),
                &ORDERBOOK_ORDER_ENTRYPOINTS,
            )
            .unwrap();

            let bytes = parse_rainlang_on_fork(&rainlang, &vec![rpc_url], None, rainlang_addr)
                .await
                .unwrap();

            let expected = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000075000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000015020000000c020200020110000001100000000000000000000000000000000000";

            assert_eq!(encode_prefixed(&bytes), expected);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_parse_rainlang_on_fork_err_parsing_dotrain_instead_of_rainlang() {
            let local_evm = LocalEvm::new().await;
            let rainlang_addr = local_evm.rainlang;
            let rpc_url = local_evm.url();

            let dotrain = r"
some front matter
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 1 2;
";

            let err = parse_rainlang_on_fork(dotrain, &vec![rpc_url], None, rainlang_addr)
                .await
                .unwrap_err();

            assert!(matches!(
                err,
                ForkParseError::ForkCallReverted(AbiDecodedErrorType::Known { name, .. }) if name == "UnexpectedLHSChar"
            ));
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn test_parse_rainlang_on_fork_err_bad_rpc() {
            let server = MockServer::start();

            server.mock(|when, then| {
                when.path("/rpc");
                then.status(500);
            });

            let rpc_url = server.url("/rpc");
            let rainlang_addr = Address::ZERO;

            let dotrain = r"
some front matter
---
#key1 !Test binding
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
_ _: 1 2;
"
            .trim_start();

            let rainlang = super::super::compose_to_rainlang(
                dotrain.to_string(),
                HashMap::new(),
                &ORDERBOOK_ORDER_ENTRYPOINTS,
            )
            .unwrap();

            let err =
                parse_rainlang_on_fork(&rainlang, &vec![rpc_url.clone()], None, rainlang_addr)
                    .await
                    .unwrap_err();

            assert!(
                matches!(&err,
                    ForkParseError::ReadableClientError(ReadableClientError::AllProvidersFailed(ref msg))
                    if msg.get(&rpc_url).is_some()
                        && matches!(
                            msg.get(&rpc_url).unwrap(),
                            ReadableClientError::ReadBlockNumberError(_)
                        )
                ),
                "unexpected error variant: {err:?}"
            );
        }
    }
}
