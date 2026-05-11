use super::select::{self, SelectContext, SelectItem};
use alloy::primitives::hex;
use anyhow::{Context, Result};
use console::Style;
use crossterm::{cursor, execute, terminal};
use rain_orderbook_app_settings::order_builder::{
    OrderBuilderFieldDefinitionCfg, OrderBuilderSelectTokensCfg,
};
use rain_orderbook_common::raindex_order_builder::RaindexOrderBuilder;
use rain_orderbook_js_api::registry::DotrainRegistry;
use std::io::{stderr, Write};

fn bold(text: &str) -> String {
    Style::new().bold().apply_to(text).to_string()
}

fn dim(text: &str) -> String {
    Style::new().dim().apply_to(text).to_string()
}

/// RAII guard that restores the terminal to a sane state (cooked mode, main
/// screen, visible cursor) on drop, even if a panic unwinds through us.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stderr(), terminal::LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

/// Enter alternate screen once, run the entire wizard there, leave at the end.
pub async fn run_interactive(registry_url: &str) -> Result<()> {
    eprintln!("  Fetching strategies from {}...", dim(registry_url));

    let registry = DotrainRegistry::new(registry_url.to_string())
        .await
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    let mut w = stderr();
    terminal::enable_raw_mode()?;
    execute!(w, terminal::EnterAlternateScreen, cursor::Hide)?;
    let _guard = TerminalGuard;

    let mut progress: Vec<String> = Vec::new();
    let result = run_wizard(&mut w, &registry, &mut progress).await;

    match result {
        Ok(output) => {
            eprintln!();
            for line in &progress {
                eprintln!("  {line}");
            }
            eprintln!();

            for line in &output {
                println!("{line}");
            }
            Ok(())
        }
        Err(err) => Err(err),
    }
}

async fn run_wizard(
    w: &mut impl Write,
    registry: &DotrainRegistry,
    progress: &mut Vec<String>,
) -> Result<Vec<String>> {
    // 1. Owner
    let owner = select::input(
        w,
        "Owner address",
        Some("The wallet that will own this order and sign the deploy transactions."),
        None,
        false,
        progress,
    )?;
    progress.push(format!("{}: {owner}", bold("Owner")));

    // 2. Strategy
    let (strategy_key, dotrain) = pick_strategy(w, registry, progress)?;
    progress.push(format!("{}: {strategy_key}", bold("Strategy")));

    let settings = registry_settings(registry);

    // 3. Deployment
    let deployment_key = pick_deployment(w, &dotrain, &settings, progress)?;
    progress.push(format!("{}: {deployment_key}", bold("Deployment")));

    render_progress(w, progress, Some("Initializing builder..."))?;

    let mut builder =
        RaindexOrderBuilder::new_with_deployment(dotrain, settings.clone(), deployment_key)
            .await
            .map_err(|err| {
                anyhow::anyhow!("failed to create order builder: {}", err.to_readable_msg())
            })?;

    // 4. Token selection
    if let Ok(tokens) = builder.get_select_tokens() {
        if !tokens.is_empty() {
            select_tokens(w, &mut builder, &tokens, progress).await?;
        }
    }

    // 5. Fields
    fill_fields(w, &mut builder, progress)?;

    // 6. Deposits
    fill_deposits(w, &mut builder, &owner, progress).await?;

    // 7. Generate calldata
    render_progress(w, progress, Some("Generating calldata..."))?;

    let args = builder
        .get_deployment_transaction_args(owner.clone())
        .await
        .map_err(|err| {
            anyhow::anyhow!(
                "failed to generate deployment calldata: {}",
                err.to_readable_msg()
            )
        })?;

    progress.push(format!("{}: {}", bold("Chain"), args.chain_id));
    progress.push(format!("{}: {}", bold("Orderbook"), args.orderbook_address));

    let mut calldata_lines = Vec::new();
    for approval in &args.approvals {
        calldata_lines.push(format!(
            "{}:0x{}",
            approval.token,
            hex::encode(&approval.calldata)
        ));
        progress.push(format!(
            "  Approve {} — {} bytes",
            Style::new().cyan().apply_to(&approval.symbol),
            approval.calldata.len()
        ));
    }
    calldata_lines.push(format!(
        "{}:0x{}",
        args.orderbook_address,
        hex::encode(&args.deployment_calldata)
    ));
    progress.push(format!(
        "  Deploy order — {} bytes",
        args.deployment_calldata.len()
    ));
    if let Some(meta_call) = &args.emit_meta_call {
        calldata_lines.push(format!(
            "{}:0x{}",
            meta_call.to,
            hex::encode(&meta_call.calldata)
        ));
        progress.push("  Emit metadata".to_string());
    }

    // 8. Output choice
    let output_items = vec![
        SelectItem {
            key: "Print to stdout".to_string(),
            description: "address:calldata lines".to_string(),
        },
        SelectItem {
            key: "Save to file".to_string(),
            description: String::new(),
        },
    ];
    let ctx = SelectContext::new(progress);
    let output_choice = select::select(w, "Output", &output_items, &ctx)?;

    match output_choice {
        0 => Ok(calldata_lines),
        1 => {
            let path = select::input(
                w,
                "Output file path",
                None,
                Some("deploy.calldata"),
                false,
                progress,
            )?;

            let mut file =
                std::fs::File::create(&path).with_context(|| format!("creating {path}"))?;
            for line in &calldata_lines {
                writeln!(file, "{line}")?;
            }
            progress.push(format!("  Wrote to {path}"));
            Ok(Vec::new())
        }
        other => unreachable!("unexpected output_choice index: {other}"),
    }
}

fn render_progress(w: &mut impl Write, progress: &[String], status: Option<&str>) -> Result<()> {
    execute!(
        w,
        cursor::MoveTo(0, 0),
        terminal::Clear(terminal::ClearType::All)
    )?;
    for line in progress {
        write!(w, "  {line}\r\n")?;
    }
    if let Some(msg) = status {
        write!(w, "\r\n  {msg}\r\n")?;
    }
    w.flush()?;
    Ok(())
}

fn pick_strategy(
    w: &mut impl Write,
    registry: &DotrainRegistry,
    progress: &[String],
) -> Result<(String, String)> {
    let details = registry
        .get_all_order_details()
        .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

    if details.valid.is_empty() {
        anyhow::bail!("no valid strategies found in registry");
    }

    let keys: Vec<&String> = details.valid.keys().collect();
    let select_items: Vec<SelectItem> = keys
        .iter()
        .map(|key| {
            let info = &details.valid[*key];
            SelectItem {
                key: key.to_string(),
                description: info
                    .short_description
                    .as_deref()
                    .unwrap_or(&info.description)
                    .to_string(),
            }
        })
        .collect();

    let ctx = SelectContext::new(progress);
    let idx = select::select(w, "Strategy", &select_items, &ctx)?;

    let key = keys[idx].clone();
    let dotrain = registry
        .orders()
        .0
        .get(&key)
        .ok_or_else(|| anyhow::anyhow!("strategy '{key}' not found"))?
        .clone();

    Ok((key, dotrain))
}

fn pick_deployment(
    w: &mut impl Write,
    dotrain: &str,
    settings: &Option<Vec<String>>,
    progress: &[String],
) -> Result<String> {
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
        let (key, _) = deployments.into_iter().next().unwrap();
        return Ok(key);
    }

    let keys: Vec<&String> = deployments.keys().collect();
    let select_items: Vec<SelectItem> = keys
        .iter()
        .map(|key| {
            let info = &deployments[*key];
            SelectItem {
                key: format!("{} ({})", info.name, key),
                description: info
                    .short_description
                    .as_deref()
                    .unwrap_or(&info.description)
                    .to_string(),
            }
        })
        .collect();

    let ctx = SelectContext::new(progress);
    let idx = select::select(w, "Deployment", &select_items, &ctx)?;
    let key = keys[idx].clone();
    Ok(key)
}

async fn select_tokens(
    w: &mut impl Write,
    builder: &mut RaindexOrderBuilder,
    tokens: &[OrderBuilderSelectTokensCfg],
    progress: &mut Vec<String>,
) -> Result<()> {
    for token_cfg in tokens {
        let prompt_label = token_cfg.name.as_deref().unwrap_or(&token_cfg.key);

        let available = builder.get_all_tokens(None).await.unwrap_or_default();

        let address = if available.is_empty() {
            select::input(
                w,
                &format!("{prompt_label} (address)"),
                token_cfg.description.as_deref(),
                None,
                false,
                progress,
            )?
        } else {
            let mut select_items: Vec<SelectItem> = available
                .iter()
                .map(|t| SelectItem {
                    key: format!("{} ({})", t.symbol, t.name),
                    description: format!("{}", t.address),
                })
                .collect();
            select_items.push(SelectItem {
                key: "Enter address manually".to_string(),
                description: String::new(),
            });

            let mut ctx = SelectContext::new(progress);
            if let Some(desc) = &token_cfg.description {
                ctx = ctx.with_description(desc);
            }
            let idx = select::select(w, prompt_label, &select_items, &ctx)?;

            if idx < available.len() {
                let token = &available[idx];
                progress.push(format!(
                    "{}: {} ({})",
                    bold(prompt_label),
                    token.symbol,
                    token.address
                ));
                format!("{}", token.address)
            } else {
                let addr = select::input(
                    w,
                    &format!("{prompt_label} address"),
                    None,
                    None,
                    false,
                    progress,
                )?;
                progress.push(format!("{}: {addr}", bold(prompt_label)));
                addr
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

fn fill_fields(
    w: &mut impl Write,
    builder: &mut RaindexOrderBuilder,
    progress: &mut Vec<String>,
) -> Result<()> {
    let missing = builder
        .get_missing_field_values()
        .map_err(|err| anyhow::anyhow!("failed to get fields: {}", err.to_readable_msg()))?;

    if missing.is_empty() {
        return Ok(());
    }

    for field in &missing {
        fill_single_field(w, builder, field, progress)?;
    }

    Ok(())
}

fn fill_single_field(
    w: &mut impl Write,
    builder: &mut RaindexOrderBuilder,
    field: &OrderBuilderFieldDefinitionCfg,
    progress: &mut Vec<String>,
) -> Result<()> {
    let value = match &field.presets {
        Some(presets) if !presets.is_empty() => {
            let show_custom = field.show_custom_field.unwrap_or(true);

            let mut select_items: Vec<SelectItem> = presets
                .iter()
                .map(|p| {
                    let label = p.name.as_deref().unwrap_or(&p.value);
                    SelectItem {
                        key: label.to_string(),
                        description: format!("= {}", p.value),
                    }
                })
                .collect();

            if show_custom {
                select_items.push(SelectItem {
                    key: "Custom value".to_string(),
                    description: String::new(),
                });
            }

            let mut ctx = SelectContext::new(progress);
            if let Some(desc) = &field.description {
                ctx = ctx.with_description(desc);
            }
            let idx = select::select(w, &field.name, &select_items, &ctx)?;

            if idx < presets.len() {
                presets[idx].value.clone()
            } else {
                select::input(
                    w,
                    &field.name,
                    field.description.as_deref(),
                    None,
                    false,
                    progress,
                )?
            }
        }
        _ => select::input(
            w,
            &field.name,
            field.description.as_deref(),
            None,
            false,
            progress,
        )?,
    };

    progress.push(format!("{}: {value}", bold(&field.name)));

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

async fn fill_deposits(
    w: &mut impl Write,
    builder: &mut RaindexOrderBuilder,
    owner: &str,
    progress: &mut Vec<String>,
) -> Result<()> {
    let deployment = builder
        .get_current_deployment()
        .map_err(|err| anyhow::anyhow!("failed to get deployment: {}", err.to_readable_msg()))?;

    if deployment.deposits.is_empty() {
        return Ok(());
    }

    for deposit_cfg in &deployment.deposits {
        let (token_display, balance_desc) =
            match builder.get_token_info(deposit_cfg.token_key.clone()).await {
                Ok(info) => {
                    let balance = builder
                        .get_account_balance(format!("{}", info.address), owner.to_string())
                        .await
                        .ok()
                        .map(|b| b.formatted_balance().to_string());
                    let desc = balance
                        .map(|b| format!("Your balance: {b} {}", info.symbol))
                        .unwrap_or_default();
                    (info.symbol.clone(), desc)
                }
                Err(_) => (deposit_cfg.token_key.clone(), String::new()),
            };

        let presets = builder
            .get_deposit_presets(deposit_cfg.token_key.clone())
            .unwrap_or_default();

        let desc_opt = if balance_desc.is_empty() {
            None
        } else {
            Some(balance_desc.as_str())
        };

        let amount = if presets.is_empty() {
            select::input(
                w,
                &format!("Deposit amount ({token_display}) — blank to skip"),
                desc_opt,
                None,
                true,
                progress,
            )?
        } else {
            let mut select_items: Vec<SelectItem> = presets
                .iter()
                .map(|p| SelectItem {
                    key: format!("{p} {token_display}"),
                    description: String::new(),
                })
                .collect();
            select_items.push(SelectItem {
                key: "Custom amount".to_string(),
                description: String::new(),
            });
            select_items.push(SelectItem {
                key: "Skip".to_string(),
                description: String::new(),
            });

            let title = format!("Deposit {token_display}");
            let mut ctx = SelectContext::new(progress);
            if !balance_desc.is_empty() {
                ctx = ctx.with_description(&balance_desc);
            }
            let idx = select::select(w, &title, &select_items, &ctx)?;

            if idx < presets.len() {
                presets[idx].clone()
            } else if idx == presets.len() {
                select::input(
                    w,
                    &format!("Amount ({token_display})"),
                    None,
                    None,
                    false,
                    progress,
                )?
            } else {
                continue;
            }
        };

        if amount.is_empty() {
            continue;
        }

        progress.push(format!("{}: {amount} {token_display}", bold("Deposit")));

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
