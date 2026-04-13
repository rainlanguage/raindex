use alloy::primitives::hex;
use anyhow::{Context, Result};
use dialoguer::{FuzzySelect, Input, Select};
use rain_orderbook_common::raindex_order_builder::RaindexOrderBuilder;
use rain_orderbook_js_api::registry::DotrainRegistry;
use rain_orderbook_app_settings::order_builder::{
    OrderBuilderFieldDefinitionCfg, OrderBuilderSelectTokensCfg,
};
use std::io::Write;

pub async fn run_interactive(registry_url: &str) -> Result<()> {
    let registry = DotrainRegistry::new(registry_url.to_string())
        .await
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    let (strategy_key, dotrain) = pick_strategy(&registry)?;
    let settings = registry_settings(&registry);
    let deployment_key = pick_deployment(&dotrain, &settings)?;

    eprintln!();
    eprintln!("Initializing builder...");
    let mut builder =
        RaindexOrderBuilder::new_with_deployment(dotrain, settings.clone(), deployment_key)
            .await
            .map_err(|err| {
                anyhow::anyhow!("failed to create order builder: {}", err.to_readable_msg())
            })?;

    if let Ok(tokens) = builder.get_select_tokens() {
        if !tokens.is_empty() {
            select_tokens(&mut builder, &tokens).await?;
        }
    }

    fill_fields(&mut builder)?;
    fill_deposits(&mut builder).await?;

    let owner: String = Input::new()
        .with_prompt("Owner address (0x...)")
        .interact_text()?;

    eprintln!();
    eprintln!("Generating calldata...");
    let args = builder
        .get_deployment_transaction_args(owner)
        .await
        .map_err(|err| {
            anyhow::anyhow!(
                "failed to generate deployment calldata: {}",
                err.to_readable_msg()
            )
        })?;

    eprintln!();
    eprintln!("Strategy: {strategy_key}");
    eprintln!("Chain ID: {}", args.chain_id);
    eprintln!("Orderbook: {}", args.orderbook_address);
    eprintln!(
        "Transactions: {}",
        args.approvals.len() + 1 + args.emit_meta_call.as_ref().map_or(0, |_| 1)
    );
    eprintln!();

    for approval in &args.approvals {
        eprintln!(
            "  Approve {} — {} bytes",
            approval.symbol,
            approval.calldata.len()
        );
    }
    eprintln!(
        "  Deploy order — {} bytes",
        args.deployment_calldata.len()
    );
    if args.emit_meta_call.is_some() {
        eprintln!("  Emit metadata");
    }
    eprintln!();

    let output_choice = Select::new()
        .with_prompt("Output")
        .items(&["Print to stdout (pipe to stox submit)", "Save to file"])
        .default(0)
        .interact()?;

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
            eprintln!("Wrote {} transactions to {path}", lines.len());
            eprintln!();
            eprintln!("Deploy with:");
            eprintln!("  cat {path} | stox submit");
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn pick_strategy(registry: &DotrainRegistry) -> Result<(String, String)> {
    let details = registry
        .get_all_order_details()
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    if details.valid.is_empty() {
        anyhow::bail!("no valid strategies found in registry");
    }

    let keys: Vec<&String> = details.valid.keys().collect();
    let display: Vec<String> = keys
        .iter()
        .map(|key| {
            let info = &details.valid[*key];
            let desc = info
                .short_description
                .as_deref()
                .unwrap_or(&info.description);
            format!("{key}  —  {desc}")
        })
        .collect();

    eprintln!();
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

    let info = &details.valid[&key];
    eprintln!("  {}", info.name);
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
        eprintln!("Deployment: {} — {}", info.name, info.description);
        return Ok(key);
    }

    let keys: Vec<&String> = deployments.keys().collect();
    let display: Vec<String> = keys
        .iter()
        .map(|key| {
            let info = &deployments[*key];
            let desc = info
                .short_description
                .as_deref()
                .unwrap_or(&info.description);
            format!("{key}  —  {desc}")
        })
        .collect();

    eprintln!();
    let idx = Select::new()
        .with_prompt("Deployment")
        .items(&display)
        .default(0)
        .interact()?;

    let key = keys[idx].clone();
    let info = &deployments[&key];
    eprintln!("  {}", info.name);
    eprintln!("  {}", info.description);

    Ok(key)
}

async fn select_tokens(
    builder: &mut RaindexOrderBuilder,
    tokens: &[OrderBuilderSelectTokensCfg],
) -> Result<()> {
    eprintln!();
    eprintln!("Token selection");

    for token_cfg in tokens {
        let label = token_cfg
            .name
            .as_deref()
            .unwrap_or(&token_cfg.key);

        if let Some(desc) = &token_cfg.description {
            eprintln!("  {desc}");
        }

        // Try to fetch available tokens for search
        let available = builder.get_all_tokens(None).await.unwrap_or_default();

        let address = if available.is_empty() {
            Input::new()
                .with_prompt(format!("{label} address"))
                .interact_text()?
        } else {
            let display: Vec<String> = available
                .iter()
                .map(|t| format!("{} ({})  {}", t.symbol, t.name, t.address))
                .collect();

            let mut items = display.clone();
            items.push("Enter address manually".to_string());

            let idx = FuzzySelect::new()
                .with_prompt(label)
                .items(&items)
                .default(0)
                .interact()?;

            if idx < available.len() {
                format!("{}", available[idx].address)
            } else {
                Input::new()
                    .with_prompt(format!("{label} address"))
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

    eprintln!();
    eprintln!("Fields");

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
        eprintln!("  {desc}");
    }

    let value = match &field.presets {
        Some(presets) if !presets.is_empty() => {
            let show_custom = field.show_custom_field.unwrap_or(true);

            let mut display: Vec<String> = presets
                .iter()
                .map(|p| {
                    let label = p.name.as_deref().unwrap_or(&p.value);
                    format!("{label}  =  {}", p.value)
                })
                .collect();

            if show_custom {
                display.push("Custom value".to_string());
            }

            let idx = Select::new()
                .with_prompt(&field.name)
                .items(&display)
                .default(0)
                .interact()?;

            if idx < presets.len() {
                presets[idx].value.clone()
            } else {
                Input::new()
                    .with_prompt(&field.name)
                    .interact_text()?
            }
        }
        _ => Input::new()
            .with_prompt(&field.name)
            .interact_text()?,
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

async fn fill_deposits(builder: &mut RaindexOrderBuilder) -> Result<()> {
    let deployment = builder
        .get_current_deployment()
        .map_err(|err| anyhow::anyhow!("failed to get deployment: {}", err.to_readable_msg()))?;

    if deployment.deposits.is_empty() {
        return Ok(());
    }

    eprintln!();
    eprintln!("Deposits (leave blank to skip)");

    for deposit_cfg in &deployment.deposits {
        let label = &deposit_cfg.token_key;

        let presets = builder
            .get_deposit_presets(deposit_cfg.token_key.clone())
            .unwrap_or_default();

        let amount = if presets.is_empty() {
            Input::new()
                .with_prompt(format!("Deposit {label}"))
                .default(String::new())
                .show_default(false)
                .interact_text()?
        } else {
            let mut display: Vec<String> = presets.iter().map(|p| p.to_string()).collect();
            display.push("Custom amount".to_string());
            display.push("Skip".to_string());

            let idx = Select::new()
                .with_prompt(format!("Deposit {label}"))
                .items(&display)
                .default(0)
                .interact()?;

            if idx < presets.len() {
                presets[idx].clone()
            } else if idx == presets.len() {
                Input::new()
                    .with_prompt(format!("Deposit {label}"))
                    .interact_text()?
            } else {
                continue;
            }
        };

        if amount.is_empty() {
            continue;
        }

        builder
            .set_deposit(deposit_cfg.token_key.clone(), amount.clone())
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
