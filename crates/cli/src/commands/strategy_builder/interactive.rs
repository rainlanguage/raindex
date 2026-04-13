use alloy::primitives::hex;
use anyhow::{Context, Result};
use console::Style;
use dialoguer::{Input, Select};
use rain_orderbook_app_settings::order_builder::{
    OrderBuilderFieldDefinitionCfg, OrderBuilderSelectTokensCfg,
};
use rain_orderbook_common::raindex_order_builder::RaindexOrderBuilder;
use rain_orderbook_js_api::registry::DotrainRegistry;
use std::io::Write;

fn heading(text: &str) {
    let style = Style::new().bold().underlined();
    eprintln!();
    eprintln!("{}", style.apply_to(text));
    eprintln!();
}

fn bold(text: &str) -> String {
    Style::new().bold().apply_to(text).to_string()
}

fn dim(text: &str) -> String {
    Style::new().dim().apply_to(text).to_string()
}

fn separator() {
    eprintln!("{}", dim("────────────────────────────────────────────────────────────"));
}

pub async fn run_interactive(registry_url: &str) -> Result<()> {
    heading("Raindex Strategy Builder");
    eprintln!("  Registry: {}", dim(registry_url));
    eprintln!("  Fetching strategies...");

    let registry = DotrainRegistry::new(registry_url.to_string())
        .await
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    // 1. Owner address first
    eprintln!();
    let owner: String = Input::new()
        .with_prompt("Owner address (0x...)")
        .interact_text()?;

    // 2. Pick strategy
    let (strategy_key, dotrain) = pick_strategy(&registry)?;
    let settings = registry_settings(&registry);

    // 3. Pick deployment
    let deployment_key = pick_deployment(&dotrain, &settings)?;

    eprintln!();
    eprintln!("  Initializing builder...");

    let mut builder =
        RaindexOrderBuilder::new_with_deployment(dotrain, settings.clone(), deployment_key)
            .await
            .map_err(|err| {
                anyhow::anyhow!("failed to create order builder: {}", err.to_readable_msg())
            })?;

    // 4. Token selection
    if let Ok(tokens) = builder.get_select_tokens() {
        if !tokens.is_empty() {
            select_tokens(&mut builder, &tokens).await?;
        }
    }

    // 5. Fields
    fill_fields(&mut builder)?;

    // 6. Deposits
    fill_deposits(&mut builder, &owner).await?;

    // 7. Generate calldata
    eprintln!();
    eprintln!("  Generating calldata...");

    let args = builder
        .get_deployment_transaction_args(owner.clone())
        .await
        .map_err(|err| {
            anyhow::anyhow!(
                "failed to generate deployment calldata: {}",
                err.to_readable_msg()
            )
        })?;

    heading("Deployment Summary");

    eprintln!("  {}: {strategy_key}", bold("Strategy"));
    eprintln!("  {}: {owner}", bold("Owner"));
    eprintln!("  {}: {}", bold("Chain ID"), args.chain_id);
    eprintln!("  {}: {}", bold("Orderbook"), args.orderbook_address);

    let tx_count = args.approvals.len() + 1 + args.emit_meta_call.as_ref().map_or(0, |_| 1);
    eprintln!("  {}: {tx_count}", bold("Transactions"));
    eprintln!();

    for approval in &args.approvals {
        eprintln!(
            "    Approve {} {} {} bytes",
            Style::new().cyan().apply_to(&approval.symbol),
            dim("—"),
            approval.calldata.len()
        );
    }
    eprintln!(
        "    Deploy order {} {} bytes",
        dim("—"),
        args.deployment_calldata.len()
    );
    if args.emit_meta_call.is_some() {
        eprintln!("    Emit metadata");
    }

    separator();

    let mut lines = Vec::new();

    for approval in &args.approvals {
        lines.push(format!(
            "{}:0x{}",
            approval.token,
            hex::encode(&approval.calldata)
        ));
    }
    lines.push(format!(
        "{}:0x{}",
        args.orderbook_address,
        hex::encode(&args.deployment_calldata)
    ));
    if let Some(meta_call) = &args.emit_meta_call {
        lines.push(format!(
            "{}:0x{}",
            meta_call.to,
            hex::encode(&meta_call.calldata)
        ));
    }

    // Short items — won't wrap
    let output_choice = Select::new()
        .with_prompt("Output")
        .items(&["Print to stdout (address:calldata lines)", "Save to file"])
        .default(0)
        .interact()?;

    match output_choice {
        0 => {
            for line in &lines {
                println!("{line}");
            }
        }
        1 => {
            let path: String = Input::new()
                .with_prompt("Output file path")
                .default("deploy.calldata".to_string())
                .interact_text()?;

            let mut file =
                std::fs::File::create(&path).with_context(|| format!("creating {path}"))?;
            for line in &lines {
                writeln!(file, "{line}")?;
            }

            eprintln!("  Wrote {} transactions to {path}", lines.len());
            eprintln!();
            eprintln!("  Deploy with:");
            eprintln!("    cat {path} | stox submit");
        }
        _ => unreachable!(),
    }

    eprintln!();
    Ok(())
}

fn pick_strategy(registry: &DotrainRegistry) -> Result<(String, String)> {
    let details = registry
        .get_all_order_details()
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    if details.valid.is_empty() {
        anyhow::bail!("no valid strategies found in registry");
    }

    // Print full descriptions as a reference table ABOVE the Select.
    // Select items are short (just the key) so they won't wrap.
    // dialoguer will only redraw the short items, not the table.
    heading("Strategies");

    let keys: Vec<&String> = details.valid.keys().collect();
    for key in &keys {
        let info = &details.valid[*key];
        let desc = info
            .short_description
            .as_deref()
            .unwrap_or(&info.description);
        eprintln!("  {}  {}", bold(key), dim("—"));
        eprintln!("  {desc}");
        eprintln!();
    }

    separator();

    // Short single-line items — dialoguer counts lines correctly
    let items: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();

    let idx = Select::new()
        .with_prompt("Strategy")
        .items(&items)
        .default(0)
        .interact()?;

    let key = keys[idx].clone();
    let dotrain = registry
        .orders()
        .0
        .get(&key)
        .ok_or_else(|| anyhow::anyhow!("strategy '{key}' not found"))?
        .clone();

    let info = &details.valid[&key];
    eprintln!("  {}", bold(&info.name));
    eprintln!("  {}", info.description);

    Ok((key, dotrain))
}

fn pick_deployment(dotrain: &str, settings: &Option<Vec<String>>) -> Result<String> {
    let deployments =
        RaindexOrderBuilder::get_deployment_details(dotrain.to_string(), settings.clone())
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to get deployment details: {}",
                    err.to_readable_msg()
                )
            })?;

    if deployments.is_empty() {
        anyhow::bail!("no deployments found for this strategy");
    }

    if deployments.len() == 1 {
        let (key, info) = deployments.into_iter().next().unwrap();
        eprintln!();
        eprintln!(
            "  Deployment: {} {} {}",
            bold(&info.name),
            dim("—"),
            info.description
        );
        return Ok(key);
    }

    heading("Deployments");

    let keys: Vec<&String> = deployments.keys().collect();
    for key in &keys {
        let info = &deployments[*key];
        let desc = info
            .short_description
            .as_deref()
            .unwrap_or(&info.description);
        eprintln!("  {} ({})  {}", bold(&info.name), key, dim("—"));
        eprintln!("  {desc}");
        eprintln!();
    }

    separator();

    let names: Vec<String> = keys
        .iter()
        .map(|key| {
            let info = &deployments[*key];
            format!("{} ({})", info.name, key)
        })
        .collect();
    let items: Vec<&str> = names.iter().map(|n| n.as_str()).collect();

    let idx = Select::new()
        .with_prompt("Deployment")
        .items(&items)
        .default(0)
        .interact()?;

    let key = keys[idx].clone();
    let info = &deployments[&key];
    eprintln!("  {}", info.description);

    Ok(key)
}

async fn select_tokens(
    builder: &mut RaindexOrderBuilder,
    tokens: &[OrderBuilderSelectTokensCfg],
) -> Result<()> {
    heading("Token Selection");

    for token_cfg in tokens {
        let prompt_label = token_cfg.name.as_deref().unwrap_or(&token_cfg.key);

        if let Some(desc) = &token_cfg.description {
            eprintln!("  {}", desc);
        }

        let available = builder.get_all_tokens(None).await.unwrap_or_default();

        let address = if available.is_empty() {
            Input::new()
                .with_prompt(format!("{prompt_label} (address)"))
                .interact_text()?
        } else {
            // Print token table above, short items in Select
            for token in &available {
                eprintln!(
                    "    {}  ({})  {}",
                    bold(&token.symbol),
                    token.name,
                    dim(&format!("{}", token.address))
                );
            }
            eprintln!();

            separator();

            // Short items — just symbol
            let mut items: Vec<String> = available
                .iter()
                .map(|t| t.symbol.clone())
                .collect();
            items.push("Enter address manually".to_string());
            let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();

            let idx = Select::new()
                .with_prompt(prompt_label)
                .items(&item_refs)
                .default(0)
                .interact()?;

            if idx < available.len() {
                let token = &available[idx];
                eprintln!(
                    "  Selected: {} ({}) {}",
                    bold(&token.symbol),
                    token.name,
                    dim(&format!("{}", token.address))
                );
                format!("{}", token.address)
            } else {
                Input::new()
                    .with_prompt(format!("{prompt_label} address"))
                    .interact_text()?
            }
        };

        builder
            .set_select_token(token_cfg.key.clone(), address)
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to select token '{}': {}",
                    token_cfg.key,
                    err.to_readable_msg()
                )
            })?;
    }

    Ok(())
}

fn fill_fields(builder: &mut RaindexOrderBuilder) -> Result<()> {
    let missing = builder
        .get_missing_field_values()
        .map_err(|err| anyhow::anyhow!("failed to get fields: {}", err.to_readable_msg()))?;

    if missing.is_empty() {
        return Ok(());
    }

    heading("Configuration");

    for field in &missing {
        fill_single_field(builder, field)?;
    }

    Ok(())
}

fn fill_single_field(
    builder: &mut RaindexOrderBuilder,
    field: &OrderBuilderFieldDefinitionCfg,
) -> Result<()> {
    if let Some(desc) = &field.description {
        eprintln!("  {}", desc);
    }

    let value = match &field.presets {
        Some(presets) if !presets.is_empty() => {
            let show_custom = field.show_custom_field.unwrap_or(true);

            // Short preset items — won't wrap
            let mut display: Vec<String> = presets
                .iter()
                .map(|p| {
                    let label = p.name.as_deref().unwrap_or(&p.value);
                    format!("{label} = {}", p.value)
                })
                .collect();

            if show_custom {
                display.push("Custom value".to_string());
            }

            let items: Vec<&str> = display.iter().map(|s| s.as_str()).collect();

            let idx = Select::new()
                .with_prompt(&field.name)
                .items(&items)
                .default(0)
                .interact()?;

            if idx < presets.len() {
                presets[idx].value.clone()
            } else {
                Input::new().with_prompt(&field.name).interact_text()?
            }
        }
        _ => Input::new().with_prompt(&field.name).interact_text()?,
    };

    builder
        .set_field_value(field.binding.clone(), value)
        .map_err(|err| {
            anyhow::anyhow!(
                "failed to set field '{}': {}",
                field.binding,
                err.to_readable_msg()
            )
        })?;

    Ok(())
}

async fn fill_deposits(builder: &mut RaindexOrderBuilder, owner: &str) -> Result<()> {
    let deployment = builder
        .get_current_deployment()
        .map_err(|err| anyhow::anyhow!("failed to get deployment: {}", err.to_readable_msg()))?;

    if deployment.deposits.is_empty() {
        return Ok(());
    }

    heading("Deposits");

    for deposit_cfg in &deployment.deposits {
        let token_display = match builder.get_token_info(deposit_cfg.token_key.clone()).await {
            Ok(info) => {
                if let Ok(bal) = builder
                    .get_account_balance(format!("{}", info.address), owner.to_string())
                    .await
                {
                    eprintln!(
                        "  {} ({})  Balance: {}",
                        bold(&info.symbol),
                        info.name,
                        bal.formatted_balance()
                    );
                }
                info.symbol.clone()
            }
            Err(_) => deposit_cfg.token_key.clone(),
        };

        let presets = builder
            .get_deposit_presets(deposit_cfg.token_key.clone())
            .unwrap_or_default();

        let amount = if presets.is_empty() {
            Input::new()
                .with_prompt(format!("Deposit {token_display} (blank to skip)"))
                .default(String::new())
                .show_default(false)
                .interact_text()?
        } else {
            let mut display: Vec<String> = presets
                .iter()
                .map(|p| format!("{p} {token_display}"))
                .collect();
            display.push("Custom amount".to_string());
            display.push("Skip".to_string());

            let items: Vec<&str> = display.iter().map(|s| s.as_str()).collect();

            let idx = Select::new()
                .with_prompt(format!("Deposit {token_display}"))
                .items(&items)
                .default(0)
                .interact()?;

            if idx < presets.len() {
                presets[idx].clone()
            } else if idx == presets.len() {
                Input::new()
                    .with_prompt(format!("Amount ({token_display})"))
                    .interact_text()?
            } else {
                continue;
            }
        };

        if amount.is_empty() {
            continue;
        }

        builder
            .set_deposit(deposit_cfg.token_key.clone(), amount)
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to set deposit '{}': {}",
                    deposit_cfg.token_key,
                    err.to_readable_msg()
                )
            })?;
    }

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
