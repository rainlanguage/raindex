#[cfg(not(target_family = "wasm"))]
use crate::add_order::RAINDEX_ORDER_ENTRYPOINTS;
#[cfg(not(target_family = "wasm"))]
use crate::rainlang::parse_rainlang_on_fork;
#[cfg(not(target_family = "wasm"))]
use alloy::primitives::Address;
use dotrain::Rebind;
#[cfg(not(target_family = "wasm"))]
use dotrain::{
    error::{ComposeError, ErrorCode},
    types::ast::Problem,
    RainDocument,
};
use dotrain_lsp::{
    lsp_types::{CompletionItem, Hover, Position, TextDocumentItem},
    RainLanguageServices,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
#[cfg(not(target_family = "wasm"))]
use std::collections::HashSet;

/// Collects the union of every binding key that is set under any scenario in the
/// dotrain front matter (recursing through nested scenarios).
///
/// dotrain parses only the body after the front matter splitter, so an elided
/// binding that is actually provided by a front matter scenario is still flagged
/// as an unresolved elided binding. This set lets [`DotrainAddOrderLsp::problems`]
/// suppress those false positives while leaving genuinely unset elided bindings
/// reported.
#[cfg(not(target_family = "wasm"))]
fn frontmatter_scenario_binding_keys(text: &str) -> HashSet<String> {
    let mut keys = HashSet::new();
    let Some(front_matter) = RainDocument::get_front_matter(text) else {
        return keys;
    };
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(front_matter) else {
        return keys;
    };
    if let Some(scenarios) = value.get("scenarios") {
        collect_binding_keys(scenarios, &mut keys);
    }
    keys
}

/// Recursively walks a scenarios YAML subtree collecting the keys of every
/// `bindings` mapping it encounters.
#[cfg(not(target_family = "wasm"))]
fn collect_binding_keys(value: &serde_yaml::Value, keys: &mut HashSet<String>) {
    let serde_yaml::Value::Mapping(mapping) = value else {
        return;
    };
    for (key, child) in mapping {
        if key.as_str() == Some("bindings") {
            if let serde_yaml::Value::Mapping(bindings) = child {
                for binding_key in bindings.keys() {
                    if let Some(binding_key) = binding_key.as_str() {
                        keys.insert(binding_key.to_string());
                    }
                }
            }
        } else {
            collect_binding_keys(child, keys);
        }
    }
}

/// Extracts the binding name from an [`ErrorCode::ElidedBinding`] problem message.
///
/// The message is formatted as `elided binding '<name>': <description>`, so the
/// name is the text between the first pair of single quotes.
#[cfg(not(target_family = "wasm"))]
fn elided_binding_name(msg: &str) -> Option<&str> {
    let rest = msg.strip_prefix("elided binding '")?;
    let end = rest.find('\'')?;
    Some(&rest[..end])
}

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
        let provided_binding_keys = frontmatter_scenario_binding_keys(&self.text_document.text);
        let is_elided_but_provided = |problem: &Problem| {
            problem.code == ErrorCode::ElidedBinding
                && elided_binding_name(&problem.msg)
                    .is_some_and(|name| provided_binding_keys.contains(name))
        };

        let rain_document =
            LANG_SERVICES.new_rain_document(&self.text_document, self.rebinds.clone());
        let mut bindings_problems = rain_document
            .bindings_problems()
            .iter()
            .filter(|v| !is_elided_but_provided(v))
            .map(|&v| v.clone())
            .collect::<Vec<_>>();
        let top_problems = rain_document.problems();
        if !top_problems.is_empty() {
            bindings_problems.extend(top_problems.to_vec());
            bindings_problems
        } else {
            let rainlang = match rain_document.compose(&RAINDEX_ORDER_ENTRYPOINTS) {
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
                            if !is_elided_but_provided(&p)
                                && bindings_problems.iter().all(|v| p != *v)
                            {
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
    use raindex_app_settings::spec_version::SpecVersion;
    use raindex_test_fixtures::LocalEvm;
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
    raindex: flare
    inputs:
      - token: token1
    outputs:
      - token: token2
  flareaaaaa:
    raindex: flare
    inputs:
      - token: token1
    outputs:
      - token: token1
  flare2:
    raindex: flare
    inputs:
      - token: token2
    outputs:
      - token: token1

scenarios:
  flare:
    raindex: flare
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

        // `fixed-io-output-token` is an elided binding (`#fixed-io-output-token !...`)
        // that IS set in the `flare` scenario front matter bindings, so its elided
        // binding error must be suppressed. `fixed-io` is elided AND never set in the
        // front matter, so its elided binding error must still be reported.
        let expected_msgs = [
            "invalid reference to binding: raindex-subparser, only literal bindings can be referenced",
            "invalid expression line",
            "invalid expression line",
            "invalid word pattern: ABC",
            "elided binding 'fixed-io': The io ratio for the limit order.",
            "undefined word: test",
            "unexpected token",
        ];
        let actual_msgs: Vec<String> = problems.iter().map(|p| p.msg.clone()).collect();

        assert_eq!(
            actual_msgs, expected_msgs,
            "elided-but-provided binding 'fixed-io-output-token' should be suppressed, \
             unset elided binding 'fixed-io' should remain"
        );

        // The suppressed binding must be gone entirely, not merely reordered.
        assert!(
            !actual_msgs
                .iter()
                .any(|m| m.contains("fixed-io-output-token")),
            "elided binding error for front-matter-provided 'fixed-io-output-token' leaked through"
        );
    }

    #[test]
    fn test_frontmatter_scenario_binding_keys_unions_nested_scenarios() {
        let text = r#"
scenarios:
  parent:
    bindings:
      top-level-binding: 1
    scenarios:
      child:
        bindings:
          nested-binding: 2
          another-nested: ${order.outputs.0.token.address}
---
#top-level-binding !desc
:;
"#;
        let keys = frontmatter_scenario_binding_keys(text);
        assert_eq!(
            keys,
            HashSet::from([
                "top-level-binding".to_string(),
                "nested-binding".to_string(),
                "another-nested".to_string(),
            ]),
            "binding keys from every (including nested) scenario must be collected"
        );
    }

    #[test]
    fn test_frontmatter_scenario_binding_keys_ignores_non_binding_keys() {
        // Keys that are not under a `bindings` map (scenario names, network keys,
        // runs, etc.) must NOT be treated as provided bindings, otherwise unrelated
        // elided bindings would be silently suppressed.
        let text = r#"
scenarios:
  flare:
    raindex: flare
    runs: 1
    bindings:
      only-real-binding: 0xabc
---
#flare !this is a scenario name, not a binding
#runs !this is a reserved key, not a binding
#only-real-binding !desc
:;
"#;
        let keys = frontmatter_scenario_binding_keys(text);
        assert_eq!(keys, HashSet::from(["only-real-binding".to_string()]));
    }

    #[test]
    fn test_elided_binding_name_extraction() {
        assert_eq!(
            elided_binding_name("elided binding 'fixed-io': The io ratio for the limit order."),
            Some("fixed-io")
        );
        // A binding name may itself contain hyphens and the description may contain
        // single quotes; only the first quoted segment is the name.
        assert_eq!(
            elided_binding_name(
                "elided binding 'fixed-io-output-token': don't match this 'quote'."
            ),
            Some("fixed-io-output-token")
        );
        assert_eq!(elided_binding_name("undefined word: test"), None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_elided_binding_set_in_frontmatter_is_not_reported() {
        let local_evm = LocalEvm::new().await;

        // `bound` is elided in the body but provided by the `flare` scenario, so it
        // must compose cleanly with zero problems. `unbound` would be a counter-case
        // (it is exercised by test_problems / the assertion below).
        let dotrain = format!(
            r#"
version: {spec_version}

networks:
  flare:
    rpcs:
      - https://rpc.ankr.com/flare
    chain-id: 14

scenarios:
  flare:
    runs: 1
    bindings:
      bound: 0x1234567890abcdef
---
#bound !The output token the fixed io is for.
#calculate-io
_ _: 0 bound;
#handle-io
:;
#handle-add-order
:;
"#,
            spec_version = SpecVersion::current()
        );

        let lsp = DotrainAddOrderLsp::new(get_text_document(&dotrain), HashMap::new());
        let problems = lsp
            .problems(&vec![local_evm.url()], None, Some(local_evm.rainlang))
            .await;

        assert_eq!(
            problems.iter().map(|p| p.msg.clone()).collect::<Vec<_>>(),
            Vec::<String>::new(),
            "elided binding 'bound' is set in the front matter scenario, so no problems expected"
        );
    }
}
