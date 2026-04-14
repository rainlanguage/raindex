//! `--tokens` mode: lists all tokens available for `--select-token` on a
//! given strategy + deployment, as markdown.

use anyhow::Result;
use rain_orderbook_common::raindex_order_builder::RaindexOrderBuilder;
use rain_orderbook_js_api::registry::DotrainRegistry;
use std::fmt::Write;

pub async fn run_tokens(registry_url: &str, strategy: &str, deployment: &str) -> Result<()> {
    let registry = DotrainRegistry::new(registry_url.to_string())
        .await
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    let dotrain = registry
        .orders()
        .0
        .get(strategy)
        .ok_or_else(|| {
            let available = registry.get_order_keys().unwrap_or_default();
            anyhow::anyhow!("strategy '{strategy}' not found. Available: {available:?}")
        })?
        .clone();

    let settings = registry_settings(&registry);

    let builder =
        RaindexOrderBuilder::new_with_deployment(dotrain, settings, deployment.to_string())
            .await
            .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    let tokens = builder
        .get_all_tokens(None)
        .await
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    let mut out = String::new();
    writeln!(out, "# Available tokens — `{strategy}` / `{deployment}`")?;
    writeln!(out)?;

    if tokens.is_empty() {
        writeln!(out, "_No tokens registered for this deployment._")?;
    } else {
        writeln!(
            out,
            "{} tokens. Use any address as the value of a `--select-token KEY=<address>` flag.",
            tokens.len()
        )?;
        writeln!(out)?;
        writeln!(out, "| Symbol | Name | Address | Decimals |")?;
        writeln!(out, "|--------|------|---------|----------|")?;
        for token in tokens {
            writeln!(
                out,
                "| `{}` | {} | `{}` | {} |",
                token.symbol, token.name, token.address, token.decimals
            )?;
        }
    }

    println!("{out}");
    Ok(())
}

fn registry_settings(registry: &DotrainRegistry) -> Option<Vec<String>> {
    let content = registry.settings();
    if content.is_empty() {
        None
    } else {
        Some(vec![content])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_tokens_for_a_deployment() {
        let server = httpmock::MockServer::start();

        let settings = r#"version: 4
networks:
  base:
    rpcs:
      - https://base-rpc.publicnode.com
    chain-id: 8453
    network-id: 8453
    currency: ETH
orderbooks:
  base:
    address: 0xe522cB4a5fCb2eb31a52Ff41a4653d85A4fd7C9D
    network: base
deployers:
  base:
    address: 0xd905B56949284Bb1d28eeFC05be78Af69cCf3668
    network: base
tokens:
  test-usdc:
    network: base
    address: 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913
    decimals: 6
    label: USD Coin
    symbol: USDC
"#;

        let strategy = r#"version: 4
orders:
  base:
    orderbook: base
    inputs:
      - token: token1
    outputs:
      - token: token2
scenarios:
  base:
    orderbook: base
    runs: 1
    bindings: {}
deployments:
  base:
    order: base
    scenario: base
builder:
  name: Test
  description: Test
  short-description: Test
  deployments:
    base:
      name: Base
      description: Test
      deposits: []
      fields: []
      select-tokens:
        - key: token1
        - key: token2
---
#calculate-io
max-output: max-positive-value(),
io: 1;
#handle-io
:;
#handle-add-order
:;
"#;

        let settings_url = format!("{}/settings.yaml", server.base_url());
        let strategy_url = format!("{}/test.rain", server.base_url());
        let registry_url = format!("{}/registry", server.base_url());

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/registry");
            then.status(200)
                .body(format!("{settings_url}\ntest {strategy_url}\n"));
        });
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/settings.yaml");
            then.status(200).body(settings);
        });
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test.rain");
            then.status(200).body(strategy);
        });

        let result = run_tokens(&registry_url, "test", "base").await;
        assert!(result.is_ok(), "tokens failed: {result:?}");
    }
}
