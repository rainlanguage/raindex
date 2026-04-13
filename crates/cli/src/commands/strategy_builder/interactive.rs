use alloy::primitives::hex;
use anyhow::{Context, Result};
use console::Style;
use dialoguer::{FuzzySelect, Input, Select};
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

fn label(text: &str) -> String {
    Style::new().bold().apply_to(text).to_string()
}

fn dim(text: &str) -> String {
    Style::new().dim().apply_to(text).to_string()
}

fn separator() {
    eprintln!("{}", dim("────────────────────────────────────────"));
}

pub async fn run_interactive(registry_url: &str) -> Result<()> {
    heading("Raindex Strategy Builder");

    eprintln!("  Registry: {}", dim(registry_url));
    eprintln!("  Fetching strategies...");

    let registry = DotrainRegistry::new(registry_url.to_string())
        .await
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    // 1. Owner address (ask first so we can show balances later)
    heading("Owner");
    let owner: String = Input::new()
        .with_prompt("Wallet address that will own this order")
        .interact_text()?;

    // 2. Pick strategy
    let (strategy_key, dotrain) = pick_strategy(&registry)?;
    let settings = registry_settings(&registry);

    // 3. Pick deployment
    let deployment_key = pick_deployment(&dotrain, &settings)?;

    separator();
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

    // 6. Deposits (with token names and balances)
    fill_deposits(&mut builder, &owner).await?;

    // 7. Generate calldata
    separator();
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

    eprintln!("  {}  {strategy_key}", label("Strategy"));
    eprintln!("  {}  {owner}", label("Owner   "));
    eprintln!(
        "  {}  {} ({})",
        label("Chain   "),
        args.chain_id,
        args.orderbook_address
    );
    eprintln!();

    let tx_count = args.approvals.len() + 1 + args.emit_meta_call.as_ref().map_or(0, |_| 1);
    eprintln!(
        "  {} {tx_count} transaction{}",
        label("Transactions"),
        if tx_count == 1 { "" } else { "s" }
    );
    eprintln!();

    for (idx, approval) in args.approvals.iter().enumerate() {
        eprintln!(
            "    {}  Approve {} {} {} bytes",
            dim(&format!("{}.", idx + 1)),
            Style::new().cyan().apply_to(&approval.symbol),
            dim("->"),
            approval.calldata.len()
        );
    }
    let deploy_idx = args.approvals.len() + 1;
    eprintln!(
        "    {}  Deploy order {} {} bytes",
        dim(&format!("{deploy_idx}.")),
        dim("->"),
        args.deployment_calldata.len()
    );
    if args.emit_meta_call.is_some() {
        eprintln!("    {}  Emit metadata", dim(&format!("{}.", deploy_idx + 1)));
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

    // Output choice — only 2 items, short labels, no wipe issue
    let output_choice = Select::new()
        .with_prompt("Output")
        .items(&[
            "Print to stdout (address:calldata lines)",
            "Save to file",
        ])
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

            eprintln!();
            eprintln!(
                "  Wrote {} transaction{} to {}",
                lines.len(),
                if lines.len() == 1 { "" } else { "s" },
                Style::new().green().apply_to(&path)
            );
            eprintln!();
            eprintln!(
                "  Format: one {} line per transaction",
                dim("address:0xcalldata")
            );
            eprintln!("  Pipe or read into any calldata submitter to deploy.");
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

    // Show the list FIRST with just the keys — no content above to get wiped
    let keys: Vec<&String> = details.valid.keys().collect();
    let display: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();

    let idx = FuzzySelect::new()
        .with_prompt("Strategy")
        .items(&display)
        .default(0)
        .interact()?;

    let key = keys[idx].clone();
    let dotrain = registry
        .orders()
        .0
        .get(&key)
        .ok_or_else(|| anyhow::anyhow!("strategy '{key}' not found"))?
        .clone();

    // Show description AFTER selection
    let info = &details.valid[&key];
    heading(&info.name);
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
        heading(&format!("Deployment: {}", info.name));
        eprintln!("  {}", info.description);
        return Ok(key);
    }

    // Show just names in the selector — description shown after
    let keys: Vec<&String> = deployments.keys().collect();
    let names: Vec<String> = keys
        .iter()
        .map(|key| {
            let info = &deployments[*key];
            format!("{} ({})", info.name, key)
        })
        .collect();
    let display: Vec<&str> = names.iter().map(|n| n.as_str()).collect();

    let idx = Select::new()
        .with_prompt("Deployment")
        .items(&display)
        .default(0)
        .interact()?;

    let key = keys[idx].clone();
    let info = &deployments[&key];

    // Show description AFTER selection
    heading(&format!("Deployment: {}", info.name));
    eprintln!("  {}", info.description);

    Ok(key)
}

async fn select_tokens(
    builder: &mut RaindexOrderBuilder,
    tokens: &[OrderBuilderSelectTokensCfg],
) -> Result<()> {
    for token_cfg in tokens {
        let prompt_label = token_cfg.name.as_deref().unwrap_or(&token_cfg.key);

        let available = builder.get_all_tokens(None).await.unwrap_or_default();

        let address = if available.is_empty() {
            if let Some(desc) = &token_cfg.description {
                eprintln!("  {}", dim(desc));
            }
            Input::new()
                .with_prompt(format!("{prompt_label} (address)"))
                .interact_text()?
        } else {
            // FuzzySelect with just symbol + address — description shown after
            let display: Vec<String> = available
                .iter()
                .map(|t| format!("{} ({})  {}", t.symbol, t.name, t.address))
                .collect();

            let mut items: Vec<&str> = display.iter().map(|s| s.as_str()).collect();
            items.push("Enter address manually");

            let idx = FuzzySelect::new()
                .with_prompt(format!("{prompt_label} — type to search"))
                .items(&items)
                .default(0)
                .max_length(12)
                .interact()?;

            if idx < available.len() {
                let token = &available[idx];
                eprintln!(
                    "  {} {} {}",
                    label(&token.symbol),
                    dim(&format!("({})", token.name)),
                    dim(&format!("{}", token.address))
                );
                format!("{}", token.address)
            } else {
                Input::new()
                    .with_prompt(format!("{prompt_label} (address)"))
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
    // Show description before the input (Input doesn't redraw/wipe)
    if let Some(desc) = &field.description {
        eprintln!("  {}", dim(desc));
    }

    let value = match &field.presets {
        Some(presets) if !presets.is_empty() => {
            let show_custom = field.show_custom_field.unwrap_or(true);

            let mut display: Vec<String> = presets
                .iter()
                .map(|p| {
                    let preset_label = p.name.as_deref().unwrap_or(&p.value);
                    format!("{preset_label} = {}", p.value)
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
                let balance_str = match builder
                    .get_account_balance(format!("{}", info.address), owner.to_string())
                    .await
                {
                    Ok(bal) => format!("  Balance: {}", bal.formatted_balance()),
                    Err(_) => String::new(),
                };

                eprintln!(
                    "  {} {} {}{}",
                    label(&info.symbol),
                    dim(&format!("({})", info.name)),
                    dim(&format!("{}", info.address)),
                    if balance_str.is_empty() {
                        String::new()
                    } else {
                        format!("\n  {}", dim(&balance_str))
                    }
                );

                info.symbol.clone()
            }
            Err(_) => deposit_cfg.token_key.clone(),
        };

        let presets = builder
            .get_deposit_presets(deposit_cfg.token_key.clone())
            .unwrap_or_default();

        let amount = if presets.is_empty() {
            Input::new()
                .with_prompt(format!("Deposit amount ({token_display})"))
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
