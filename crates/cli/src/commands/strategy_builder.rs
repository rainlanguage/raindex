use crate::execute::Execute;
use alloy::primitives::hex;
use anyhow::Result;
use clap::Parser;
use rain_orderbook_common::raindex_order_builder::RaindexOrderBuilder;
use rain_orderbook_js_api::registry::DotrainRegistry;
use std::collections::HashMap;

#[derive(Parser, Clone)]
pub struct StrategyBuilder {
    #[arg(
        long,
        help = "Registry URL (text file: settings URL on line 1, then 'key url' per order)"
    )]
    registry: String,

    #[arg(long, help = "Order/strategy key from the registry")]
    strategy: String,

    #[arg(long, help = "Deployment key within the strategy")]
    deployment: String,

    #[arg(long, help = "Order owner address")]
    owner: String,

    #[arg(
        long = "set-field",
        value_name = "BINDING=VALUE",
        help = "Set a field binding value (repeatable)"
    )]
    set_fields: Vec<String>,

    #[arg(
        long = "select-token",
        value_name = "KEY=ADDRESS",
        help = "Select a token for a slot (repeatable)"
    )]
    select_tokens: Vec<String>,

    #[arg(
        long = "set-deposit",
        value_name = "TOKEN=AMOUNT",
        help = "Set a deposit amount (repeatable)"
    )]
    set_deposits: Vec<String>,
}

fn parse_key_value_pairs(args: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for arg in args {
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("expected KEY=VALUE, got: {arg}"))?;
        let key = key.trim();
        if key.is_empty() {
            anyhow::bail!("expected non-empty KEY in KEY=VALUE, got: {arg}");
        }
        if map.contains_key(key) {
            anyhow::bail!("duplicate key: {key}");
        }
        map.insert(key.to_string(), value.to_string());
    }
    Ok(map)
}

impl Execute for StrategyBuilder {
    async fn execute(&self) -> Result<()> {
        let registry = DotrainRegistry::new(self.registry.clone())
            .await
            .map_err(|err| anyhow::anyhow!("{}", err.to_readable_msg()))?;

        let dotrain = registry
            .orders()
            .0
            .get(&self.strategy)
            .ok_or_else(|| {
                let mut available = registry.get_order_keys().unwrap_or_default();
                available.sort();
                anyhow::anyhow!(
                    "strategy '{}' not found in registry. Available: {:?}",
                    self.strategy,
                    available
                )
            })?
            .clone();

        let settings = {
            let content = registry.settings();
            if content.is_empty() {
                None
            } else {
                Some(vec![content])
            }
        };

        let mut builder =
            RaindexOrderBuilder::new_with_deployment(dotrain, settings, self.deployment.clone())
                .await
                .map_err(|err| {
                    anyhow::anyhow!("failed to create order builder: {}", err.to_readable_msg())
                })?;

        let fields = parse_key_value_pairs(&self.set_fields)?;
        for (binding, value) in &fields {
            builder
                .set_field_value(binding.clone(), value.clone())
                .map_err(|err| {
                    anyhow::anyhow!("failed to set field '{binding}': {}", err.to_readable_msg())
                })?;
        }

        let tokens = parse_key_value_pairs(&self.select_tokens)?;
        for (key, address) in &tokens {
            builder
                .set_select_token(key.clone(), address.clone())
                .await
                .map_err(|err| {
                    anyhow::anyhow!("failed to select token '{key}': {}", err.to_readable_msg())
                })?;
        }

        let deposits = parse_key_value_pairs(&self.set_deposits)?;
        for (token, amount) in &deposits {
            builder
                .set_deposit(token.clone(), amount.clone())
                .await
                .map_err(|err| {
                    anyhow::anyhow!("failed to set deposit '{token}': {}", err.to_readable_msg())
                })?;
        }

        let args = builder
            .get_deployment_transaction_args(self.owner.clone())
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to generate deployment calldata: {}",
                    err.to_readable_msg()
                )
            })?;

        for approval in &args.approvals {
            println!("{}:0x{}", approval.token, hex::encode(&approval.calldata));
        }

        println!(
            "{}:0x{}",
            args.orderbook_address,
            hex::encode(&args.deployment_calldata)
        );

        if let Some(meta_call) = &args.emit_meta_call {
            println!("{}:0x{}", meta_call.to, hex::encode(&meta_call.calldata));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        StrategyBuilder::command().debug_assert();
    }

    #[test]
    fn parse_key_value_pairs_valid() {
        let args = vec![
            "max-spread=0.002".to_string(),
            "oracle-key=ETH/USD".to_string(),
        ];
        let map = parse_key_value_pairs(&args).unwrap();
        assert_eq!(map.get("max-spread").unwrap(), "0.002");
        assert_eq!(map.get("oracle-key").unwrap(), "ETH/USD");
    }

    #[test]
    fn parse_key_value_pairs_missing_equals() {
        let args = vec!["no-equals".to_string()];
        let result = parse_key_value_pairs(&args);
        assert!(result.is_err());
    }

    #[test]
    fn parse_key_value_pairs_empty() {
        let args: Vec<String> = vec![];
        let map = parse_key_value_pairs(&args).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn parse_key_value_pairs_value_with_equals() {
        let args = vec!["key=value=with=equals".to_string()];
        let map = parse_key_value_pairs(&args).unwrap();
        assert_eq!(map.get("key").unwrap(), "value=with=equals");
    }

    #[test]
    fn parse_key_value_pairs_empty_key_fails() {
        let args = vec!["=value".to_string()];
        let err = parse_key_value_pairs(&args).unwrap_err().to_string();
        assert!(err.contains("expected non-empty KEY"), "got: {err}");
    }

    #[test]
    fn parse_key_value_pairs_whitespace_key_fails() {
        let args = vec!["  =value".to_string()];
        let err = parse_key_value_pairs(&args).unwrap_err().to_string();
        assert!(err.contains("expected non-empty KEY"), "got: {err}");
    }

    #[test]
    fn parse_key_value_pairs_duplicate_key_fails() {
        let args = vec!["key=first".to_string(), "key=second".to_string()];
        let err = parse_key_value_pairs(&args).unwrap_err().to_string();
        assert!(err.contains("duplicate key: key"), "got: {err}");
    }
}
