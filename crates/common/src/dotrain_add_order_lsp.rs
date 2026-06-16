#[cfg(not(target_family = "wasm"))]
use crate::add_order::ORDERBOOK_ORDER_ENTRYPOINTS;
#[cfg(not(target_family = "wasm"))]
use crate::rainlang::parse_rainlang_on_fork;
#[cfg(not(target_family = "wasm"))]
use alloy::primitives::Address;
use dotrain::Rebind;
#[cfg(not(target_family = "wasm"))]
use dotrain::{
    error::{ComposeError, ErrorCode},
    types::ast::Problem,
};
use dotrain_lsp::{
    lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
    RainLanguageServices,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// static lang services instance
/// meta store instance can be taken from this for shared access to a unfied meta store across
/// all the dotrain usage in this crate
pub static LANG_SERVICES: Lazy<RainLanguageServices> = Lazy::new(RainLanguageServices::default);

pub struct DotrainAddOrderLsp {
    text_document: TextDocumentItem,
    rebinds: Option<Vec<Rebind>>,
}

impl DotrainAddOrderLsp {
    pub fn new(text_document: TextDocumentItem, bindings: HashMap<String, String>) -> Self {
        let rebinds = if !bindings.is_empty() {
            Some(
                bindings
                    .iter()
                    .map(|(key, value)| Rebind(key.clone(), value.clone()))
                    .collect(),
            )
        } else {
            None
        };

        Self {
            text_document: text_document.clone(),
            rebinds,
        }
    }

    /// get hover for a given text document item
    pub fn hover(&self, position: Position) -> Option<Hover> {
        LANG_SERVICES.do_hover(&self.text_document, position, None, self.rebinds.clone())
    }

    /// get completion items for a given text document item
    pub fn completion(&self, position: Position) -> Option<Vec<CompletionItem>> {
        LANG_SERVICES.do_complete(&self.text_document, position, None, self.rebinds.clone())
    }

    /// get problems for a given text document item
    #[cfg(not(target_family = "wasm"))]
    pub async fn problems(
        &self,
        rpcs: &Vec<String>,
        block_number: Option<u64>,
        rainlang_address: Option<Address>,
    ) -> Vec<Problem> {
        let rain_document =
            LANG_SERVICES.new_rain_document(&self.text_document, self.rebinds.clone());
        let mut bindings_problems = rain_document
            .bindings_problems()
            .iter()
            .map(|&v| v.clone())
            .collect::<Vec<_>>();
        let top_problems = rain_document.problems();
        if !top_problems.is_empty() {
            bindings_problems.extend(top_problems.to_vec());
            bindings_problems
        } else {
            let rainlang = match rain_document.compose(&ORDERBOOK_ORDER_ENTRYPOINTS) {
                Ok(v) => v,
                Err(e) => match e {
                    ComposeError::Reject(msg) => {
                        bindings_problems.push(Problem {
                            msg,
                            position: [0, 0],
                            code: ErrorCode::NativeParserError,
                        });
                        return bindings_problems;
                    }
                    ComposeError::Problems(problems) => {
                        for p in problems {
                            if bindings_problems.iter().all(|v| p != *v) {
                                bindings_problems.push(p)
                            }
                        }
                        return bindings_problems;
                    }
                },
            };

            if let Some(rainlang_addr) = rainlang_address {
                if let Err(e) =
                    parse_rainlang_on_fork(&rainlang, rpcs, block_number, rainlang_addr).await
                {
                    bindings_problems.push(Problem {
                        msg: e.to_string(),
                        position: [0, 0],
                        code: ErrorCode::NativeParserError,
                    })
                };
            } else {
                bindings_problems.push(Problem {
                    msg: "Choose a deployment to get Rainlang diagnostics".to_owned(),
                    position: [0, 0],
                    code: ErrorCode::NativeParserError,
                });
            }
            bindings_problems
        }
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_test_fixtures::LocalEvm;
    use url::Url;

    fn get_text() -> String {
        format!(
            r#"
version: {spec_version}

tokens:
  token1:
    network: flare
    address: 0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d
    decimals: 18
  token2:
    network: flare
    address: 0x12e605bc104e93B45e1aD99F9e555f659051c2BB
    decimals: 18

orders:
  flare1:
    orderbook: flare
    inputs:
      - token: token1
    outputs:
      - token: token2
  flareaaaaa:
    orderbook: flare
    inputs:
      - token: token1
    outputs:
      - token: token1
  flare2:
    orderbook: flare
    inputs:
      - token: token2
    outputs:
      - token: token1

scenarios:
  flare:
    orderbook: flare
    runs: 1
    bindings:
      raindex-subparser: 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
      fixed-io-output-token: ${{order.outputs.0.token.address}}

deployments:
  flare1:
    order: flare1
    scenario: flare
    
---
"test"

#raindex-subparser !The subparser to use.

#fixed-io !The io ratio for the limit order.
#fixed-io-output-token !The output token that the fixed io is for. If this doesn't match the runtime output then the fixed-io will be inverted.

#calculate-io

123;
invalid-text;

using-words-from raindex-subparser

_: ensure(ABC),

max-output: max-positive-value(),
io: if(
  equal-to(
    output-token()
    fixed-io-output-token
  )
  fixed-io
  inv(test)
);

#handle-io
:;

#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        )
    }

    fn get_text_document(text: &str) -> TextDocumentItem {
        TextDocumentItem {
            uri: Url::parse("file:///temp.rain").unwrap(),
            language_id: "rainlang".to_string(),
            version: 0,
            text: text.to_string(),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_no_problems() {
        let local_evm = LocalEvm::new().await;

        let rainlang = r#"
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;
"#;
        let lsp = DotrainAddOrderLsp::new(get_text_document(rainlang), HashMap::new());
        let problems = lsp
            .problems(&vec![local_evm.url()], None, Some(local_evm.rainlang))
            .await;
        assert_eq!(problems.len(), 0);
    }

    #[tokio::test]
    async fn test_problems() {
        let lsp = DotrainAddOrderLsp::new(get_text_document(&get_text()), HashMap::new());
        let problems = lsp
            .problems(&vec!["https://some-rpc-url.com".to_string()], None, None)
            .await;

        let expected_msgs = [
            "invalid reference to binding: raindex-subparser, only literal bindings can be referenced",
            "invalid expression line",
            "invalid expression line",
            "invalid word pattern: ABC",
            "elided binding 'fixed-io-output-token': The output token that the fixed io is for. If this doesn't match the runtime output then the fixed-io will be inverted.",
            "elided binding 'fixed-io': The io ratio for the limit order.",
            "undefined word: test",
            "unexpected token",
        ];
        let actual_msgs: Vec<String> = problems.iter().map(|p| p.msg.clone()).collect();

        assert_eq!(problems.len(), 8);

        for (actual, expected) in actual_msgs.iter().zip(expected_msgs.iter()) {
            assert_eq!(actual, expected);
        }
    }
}
