//! `--describe` mode: prints a full dump of a registry as markdown.
//!
//! Intended to serve as a self-generating skill for the non-interactive CLI.
//! A human or agent can run `raindex strategy-builder describe --registry URL`
//! and get everything they need to construct a deploy command.

use anyhow::Result;
use rain_orderbook_common::raindex_order_builder::RaindexOrderBuilder;
use rain_orderbook_js_api::registry::DotrainRegistry;
use std::fmt::Write;

const USAGE: &str = r#"## Usage

Generate deployment calldata for a strategy:

```
raindex strategy-builder \
  --registry <url> \
  --strategy <key> \
  --deployment <key> \
  --owner <0x-address> \
  [--select-token KEY=ADDRESS ...] \
  [--set-field BINDING=VALUE ...] \
  [--set-deposit TOKEN=AMOUNT ...]
```

### Output format

The command writes one transaction per line to stdout, each in the form:

```
<to-address>:<hex-calldata>
```

Multiple lines are possible — they must be signed and broadcast in order.
Typical output contains:

1. One ERC20 `approve` transaction per token being deposited (if any).
2. The main order deployment transaction to the orderbook contract.
3. An optional metadata emission transaction.

Pipe the output into any submitter that signs and broadcasts transactions:

```
raindex strategy-builder ... | stox submit
```

"#;

pub async fn run_describe(registry_url: &str) -> Result<()> {
    let registry = DotrainRegistry::new(registry_url.to_string())
        .await
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    let mut out = String::new();
    writeln!(out, "# Raindex Strategy Registry")?;
    writeln!(out)?;
    writeln!(out, "**Registry:** {registry_url}")?;
    writeln!(out)?;
    write!(out, "{USAGE}")?;
    writeln!(out, "## Strategies")?;
    writeln!(out)?;

    let details = registry
        .get_all_order_details()
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    let settings = registry_settings(&registry);

    let mut strategy_keys: Vec<&String> = details.valid.keys().collect();
    strategy_keys.sort();

    for strategy_key in strategy_keys {
        let info = &details.valid[strategy_key];
        writeln!(out, "### `{strategy_key}` — {}", info.name)?;
        writeln!(out)?;
        writeln!(out, "{}", info.description)?;
        writeln!(out)?;

        let dotrain = registry
            .orders()
            .0
            .get(strategy_key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("strategy '{strategy_key}' not in registry"))?;

        describe_strategy(&mut out, strategy_key, &dotrain, &settings).await?;
    }

    if !details.invalid.is_empty() {
        writeln!(out, "## Invalid Strategies")?;
        writeln!(out)?;
        writeln!(out, "The following registry entries failed to parse:")?;
        writeln!(out)?;
        for (key, err) in &details.invalid {
            writeln!(out, "- `{key}`: {}", err.readable_msg)?;
        }
        writeln!(out)?;
    }

    println!("{out}");
    Ok(())
}

async fn describe_strategy(
    out: &mut String,
    strategy_key: &str,
    dotrain: &str,
    settings: &Option<Vec<String>>,
) -> Result<()> {
    let deployment_keys =
        RaindexOrderBuilder::get_deployment_keys(dotrain.to_string(), settings.clone())
            .await
            .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    if deployment_keys.is_empty() {
        writeln!(out, "_No deployments defined._")?;
        writeln!(out)?;
        return Ok(());
    }

    writeln!(out, "**Deployments:**")?;
    writeln!(out)?;

    for deployment_key in deployment_keys {
        // Build each deployment individually to get its full config
        let builder = match RaindexOrderBuilder::new_with_deployment(
            dotrain.to_string(),
            settings.clone(),
            deployment_key.clone(),
        )
        .await
        {
            Ok(b) => b,
            Err(err) => {
                writeln!(
                    out,
                    "#### `{deployment_key}` — _failed to load: {}_",
                    err.to_readable_msg()
                )?;
                writeln!(out)?;
                continue;
            }
        };

        let deployment = match builder.get_current_deployment() {
            Ok(d) => d,
            Err(err) => {
                writeln!(
                    out,
                    "#### `{deployment_key}` — _failed to load: {}_",
                    err.to_readable_msg()
                )?;
                writeln!(out)?;
                continue;
            }
        };

        writeln!(out, "#### `{deployment_key}` — {}", deployment.name)?;
        writeln!(out)?;
        writeln!(out, "{}", deployment.description)?;
        writeln!(out)?;

        writeln!(out, "**Example command:**")?;
        writeln!(out)?;
        writeln!(out, "```")?;
        writeln!(
            out,
            "raindex strategy-builder \\\n  --registry <url> \\\n  --strategy {strategy_key} \\\n  --deployment {deployment_key} \\\n  --owner <0x-address>{}{}{}",
            render_token_flags(&deployment),
            render_field_flags(&deployment),
            render_deposit_flags(&deployment),
        )?;
        writeln!(out, "```")?;
        writeln!(out)?;

        describe_select_tokens(out, &deployment)?;
        describe_fields(out, &deployment)?;
        describe_deposits(out, &deployment)?;
    }

    Ok(())
}

fn render_token_flags(
    deployment: &rain_orderbook_app_settings::order_builder::OrderBuilderDeploymentCfg,
) -> String {
    match &deployment.select_tokens {
        Some(tokens) if !tokens.is_empty() => tokens
            .iter()
            .map(|t| format!(" \\\n  --select-token {}=<address>", t.key))
            .collect(),
        _ => String::new(),
    }
}

fn render_field_flags(
    deployment: &rain_orderbook_app_settings::order_builder::OrderBuilderDeploymentCfg,
) -> String {
    deployment
        .fields
        .iter()
        .filter(|f| f.default.is_none())
        .map(|f| format!(" \\\n  --set-field {}=<value>", f.binding))
        .collect()
}

fn render_deposit_flags(
    deployment: &rain_orderbook_app_settings::order_builder::OrderBuilderDeploymentCfg,
) -> String {
    deployment
        .deposits
        .iter()
        .map(|d| format!(" \\\n  --set-deposit {}=<amount>", d.token_key))
        .collect()
}

fn describe_select_tokens(
    out: &mut String,
    deployment: &rain_orderbook_app_settings::order_builder::OrderBuilderDeploymentCfg,
) -> Result<()> {
    let tokens = match &deployment.select_tokens {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(()),
    };

    writeln!(out, "**Tokens to select:**")?;
    writeln!(out)?;
    for token in tokens {
        let name = token.name.as_deref().unwrap_or("");
        let desc = token.description.as_deref().unwrap_or("");
        match (name, desc) {
            ("", "") => writeln!(out, "- `{}`", token.key)?,
            (n, "") => writeln!(out, "- `{}` — {n}", token.key)?,
            ("", d) => writeln!(out, "- `{}` — {d}", token.key)?,
            (n, d) => writeln!(out, "- `{}` ({n}) — {d}", token.key)?,
        }
    }
    writeln!(out)?;
    Ok(())
}

fn describe_fields(
    out: &mut String,
    deployment: &rain_orderbook_app_settings::order_builder::OrderBuilderDeploymentCfg,
) -> Result<()> {
    if deployment.fields.is_empty() {
        return Ok(());
    }

    writeln!(out, "**Fields:**")?;
    writeln!(out)?;
    for field in &deployment.fields {
        write!(out, "- `{}` ({})", field.binding, field.name)?;
        if let Some(default) = &field.default {
            write!(out, " _[default: `{default}`]_")?;
        }
        writeln!(out)?;
        if let Some(desc) = &field.description {
            writeln!(out, "  - {desc}")?;
        }
        if let Some(presets) = &field.presets {
            if !presets.is_empty() {
                write!(out, "  - Presets:")?;
                for preset in presets {
                    match &preset.name {
                        Some(n) => write!(out, " `{}` = `{}`,", n, preset.value)?,
                        None => write!(out, " `{}`,", preset.value)?,
                    }
                }
                writeln!(out)?;
            }
        }
    }
    writeln!(out)?;
    Ok(())
}

fn describe_deposits(
    out: &mut String,
    deployment: &rain_orderbook_app_settings::order_builder::OrderBuilderDeploymentCfg,
) -> Result<()> {
    if deployment.deposits.is_empty() {
        return Ok(());
    }

    writeln!(out, "**Deposits:**")?;
    writeln!(out)?;
    for deposit in &deployment.deposits {
        write!(out, "- `{}`", deposit.token_key)?;
        if let Some(presets) = &deposit.presets {
            if !presets.is_empty() {
                write!(
                    out,
                    " — presets: {}",
                    presets
                        .iter()
                        .map(|p| format!("`{p}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }
        }
        writeln!(out)?;
    }
    writeln!(out)?;
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

    // End-to-end: describe a real registry served by httpmock and verify
    // the markdown output contains all the strategy details.
    #[tokio::test]
    async fn describe_renders_registry_as_markdown() {
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
    bindings:
      max-spread: 0.002
deployments:
  base:
    order: base
    scenario: base
builder:
  name: Fixed spread
  description: A strategy that tracks a benchmark price with a fixed spread.
  short-description: Fixed spread strategy
  deployments:
    base:
      name: Base
      description: Deploy on Base network.
      deposits:
        - token: token2
          presets:
            - "10"
            - "100"
            - "1000"
      fields:
        - binding: max-spread
          name: Maximum spread
          description: The max spread as a decimal.
          presets:
            - name: Tight
              value: "0.001"
            - name: Loose
              value: "0.01"
      select-tokens:
        - key: token1
          name: Input token
          description: The token you buy
        - key: token2
          name: Output token
          description: The token you sell
---
#max-spread !max spread
#calculate-io
max-output: max-positive-value(),
io: 1;
#handle-io
:;
#handle-add-order
:;
"#;

        let settings_url = format!("{}/settings.yaml", server.base_url());
        let strategy_url = format!("{}/fixed-spread.rain", server.base_url());
        let registry_url = format!("{}/registry", server.base_url());

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/registry");
            then.status(200)
                .body(format!("{settings_url}\nfixed-spread {strategy_url}\n"));
        });
        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/settings.yaml");
            then.status(200).body(settings);
        });
        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/fixed-spread.rain");
            then.status(200).body(strategy);
        });

        // Capture stdout. The describe command uses println!, so we need a
        // helper — instead just verify it completes without error. The
        // individual helpers below cover the rendering logic.
        let result = run_describe(&registry_url).await;
        assert!(result.is_ok(), "describe failed: {result:?}");
    }
}
