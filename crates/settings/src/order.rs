use crate::{yaml::FieldErrorKind, *};
use alloy::primitives::U256;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    str::FromStr,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;

const ALLOWED_ORDER_KEYS: [&str; 5] = ["inputs", "oracle-url", "orderbook", "outputs", "rainlang"];
const ALLOWED_ORDER_IO_KEYS: [&str; 2] = ["token", "vault-id"];
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::{
    context::{Context, GuiContextTrait, SelectTokensContext},
    default_document, optional_string, require_hash, require_string, require_vec, YamlError,
    YamlParsableHash,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Tsify)]
#[serde(rename_all = "camelCase")]
pub enum VaultType {
    Input,
    Output,
}
impl_wasm_traits!(VaultType);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct OrderIOCfg {
    pub token_key: String,
    pub token: Option<Arc<TokenCfg>>,
    #[cfg_attr(
        target_family = "wasm",
        serde(rename = "vaultId"),
        tsify(optional, type = "string")
    )]
    pub vault_id: Option<U256>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderIOCfg);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "kebab-case")]
pub struct OrderCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub inputs: Vec<OrderIOCfg>,
    pub outputs: Vec<OrderIOCfg>,
    pub network: Arc<NetworkCfg>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub rainlang: Option<Arc<RainlangCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub orderbook: Option<Arc<OrderbookCfg>>,
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    #[serde(rename = "oracle-url")]
    pub oracle_url: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderCfg);

impl OrderCfg {
    pub fn validate_vault_id(value: &str) -> Result<U256, ParseOrderConfigSourceError> {
        U256::from_str(value).map_err(ParseOrderConfigSourceError::VaultParseError)
    }

    pub fn update_vault_id(
        &mut self,
        vault_type: VaultType,
        token: String,
        vault_id: Option<String>,
    ) -> Result<Self, YamlError> {
        let new_vault_id = if let Some(ref v) = vault_id {
            if v.is_empty() {
                None
            } else {
                match OrderCfg::validate_vault_id(v) {
                    Ok(id) => Some(id),
                    Err(e) => {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "vault-id".to_string(),
                                reason: e.to_string(),
                            },
                            location: format!(
                                "token '{}' in {} of order '{}'",
                                token,
                                match vault_type {
                                    VaultType::Input => "inputs",
                                    VaultType::Output => "outputs",
                                },
                                self.key
                            ),
                        });
                    }
                }
            }
        } else {
            None
        };

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut orders)) =
                document_hash.get_mut(&StrictYaml::String("orders".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut order)) =
                    orders.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    let vec_key = match vault_type {
                        VaultType::Input => "inputs",
                        VaultType::Output => "outputs",
                    };
                    if let Some(StrictYaml::Array(ref mut vec)) =
                        order.get_mut(&StrictYaml::String(vec_key.to_string()))
                    {
                        // Find the item with matching token key
                        let item_index = vec.iter().position(|item| {
                            if let StrictYaml::Hash(ref item_map) = item {
                                if let Some(StrictYaml::String(item_token)) =
                                    item_map.get(&StrictYaml::String("token".to_string()))
                                {
                                    return item_token == &token;
                                }
                            }
                            false
                        });

                        if let Some(idx) = item_index {
                            if let Some(item) = vec.get_mut(idx) {
                                if let StrictYaml::Hash(ref mut item_map) = item {
                                    if let Some(vault_id) = new_vault_id {
                                        item_map.insert(
                                            StrictYaml::String("vault-id".to_string()),
                                            StrictYaml::String(vault_id.to_string()),
                                        );
                                        match vault_type {
                                            VaultType::Input => {
                                                self.inputs[idx].vault_id = Some(vault_id);
                                            }
                                            VaultType::Output => {
                                                self.outputs[idx].vault_id = Some(vault_id);
                                            }
                                        }
                                    } else {
                                        item_map
                                            .remove(&StrictYaml::String("vault-id".to_string()));
                                        match vault_type {
                                            VaultType::Input => {
                                                self.inputs[idx].vault_id = None;
                                            }
                                            VaultType::Output => {
                                                self.outputs[idx].vault_id = None;
                                            }
                                        }
                                    }
                                } else {
                                    return Err(YamlError::Field {
                                        kind: FieldErrorKind::InvalidType {
                                            field: vec_key.to_string(),
                                            expected: "a hash".to_string(),
                                        },
                                        location: format!("order '{0}'", self.key),
                                    });
                                }
                            }
                        } else {
                            return Err(YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: vec_key.to_string(),
                                    reason: format!("token '{}' not found", token),
                                },
                                location: format!("order '{0}'", self.key),
                            });
                        }
                    } else {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::Missing(vec_key.to_string()),
                            location: format!("order '{0}'", self.key),
                        });
                    }
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "orders".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("orders".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(self.clone())
    }

    pub fn populate_vault_ids(&mut self) -> Result<Self, YamlError> {
        let vault_id = U256::random();

        let mut document = self
            .document
            .write()
            .map_err(|_| YamlError::WriteLockError)?;

        if let StrictYaml::Hash(ref mut document_hash) = *document {
            if let Some(StrictYaml::Hash(ref mut orders)) =
                document_hash.get_mut(&StrictYaml::String("orders".to_string()))
            {
                if let Some(StrictYaml::Hash(ref mut order)) =
                    orders.get_mut(&StrictYaml::String(self.key.to_string()))
                {
                    if let Some(StrictYaml::Array(ref mut inputs)) =
                        order.get_mut(&StrictYaml::String("inputs".to_string()))
                    {
                        for (index, input) in inputs.iter_mut().enumerate() {
                            if let StrictYaml::Hash(ref mut input_hash) = input {
                                if !input_hash
                                    .contains_key(&StrictYaml::String("vault-id".to_string()))
                                {
                                    input_hash.insert(
                                        StrictYaml::String("vault-id".to_string()),
                                        StrictYaml::String(vault_id.to_string()),
                                    );
                                }
                            } else {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidType {
                                        field: format!("input index: {index}"),
                                        expected: "a hash".to_string(),
                                    },
                                    location: format!("order '{0}'", self.key),
                                });
                            }
                        }
                    } else {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::Missing("inputs".to_string()),
                            location: format!("order '{0}'", self.key),
                        });
                    }
                    if let Some(StrictYaml::Array(ref mut outputs)) =
                        order.get_mut(&StrictYaml::String("outputs".to_string()))
                    {
                        for (index, output) in outputs.iter_mut().enumerate() {
                            if let StrictYaml::Hash(ref mut output_hash) = output {
                                if !output_hash
                                    .contains_key(&StrictYaml::String("vault-id".to_string()))
                                {
                                    output_hash.insert(
                                        StrictYaml::String("vault-id".to_string()),
                                        StrictYaml::String(vault_id.to_string()),
                                    );
                                }
                            } else {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidType {
                                        field: format!("output index: {index}"),
                                        expected: "a hash".to_string(),
                                    },
                                    location: format!("order '{0}'", self.key),
                                });
                            }
                        }
                    } else {
                        return Err(YamlError::Field {
                            kind: FieldErrorKind::Missing("outputs".to_string()),
                            location: format!("order '{0}'", self.key),
                        });
                    }

                    self.inputs.iter_mut().for_each(|input| {
                        input.vault_id = Some(input.vault_id.unwrap_or(vault_id));
                    });
                    self.outputs.iter_mut().for_each(|output| {
                        output.vault_id = Some(output.vault_id.unwrap_or(vault_id));
                    });
                } else {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(self.key.clone()),
                        location: "orders".to_string(),
                    });
                }
            } else {
                return Err(YamlError::Field {
                    kind: FieldErrorKind::Missing("orders".to_string()),
                    location: "root".to_string(),
                });
            }
        } else {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "document".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(self.clone())
    }

    pub fn parse_network_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        order_key: &str,
    ) -> Result<String, YamlError> {
        let mut network_key: Option<String> = None;

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orders_hash) = require_hash(&document_read, Some("orders"), None) {
                if let Some(order_yaml) =
                    orders_hash.get(&StrictYaml::String(order_key.to_string()))
                {
                    let location = format!("order '{}'", order_key);

                    if let Some(rainlang_key) = optional_string(order_yaml, "rainlang") {
                        let key = RainlangCfg::parse_network_key(documents.clone(), &rainlang_key)?;

                        if let Some(ref existing_key) = network_key {
                            if *existing_key != key {
                                return Err(YamlError::ParseOrderConfigSourceError(
                                    ParseOrderConfigSourceError::RainlangNetworkDoesNotMatch {
                                        expected: existing_key.clone(),
                                        found: key.clone(),
                                    },
                                ));
                            }
                        } else {
                            network_key = Some(key);
                        }
                    }

                    if let Some(orderbook_key) = optional_string(order_yaml, "orderbook") {
                        let key =
                            OrderbookCfg::parse_network_key(documents.clone(), &orderbook_key)?;

                        if let Some(ref existing_key) = network_key {
                            if *existing_key != key {
                                return Err(YamlError::ParseOrderConfigSourceError(
                                    ParseOrderConfigSourceError::OrderbookNetworkDoesNotMatch {
                                        expected: existing_key.clone(),
                                        found: key.clone(),
                                    },
                                ));
                            }
                        } else {
                            network_key = Some(key);
                        }
                    }

                    for (index, input) in require_vec(order_yaml, "inputs", Some(location.clone()))?
                        .iter()
                        .enumerate()
                    {
                        let location = format!("input index '{index}' in order '{order_key}'");

                        let token_key =
                            require_string(input, Some("token"), Some(location.clone()))?;
                        let res = TokenCfg::parse_network_key(documents.clone(), &token_key);
                        if let Ok(key) = res {
                            if let Some(ref existing_key) = network_key {
                                if *existing_key != key {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::InputTokenNetworkDoesNotMatch {
                                            key: token_key,
                                            expected: existing_key.clone(),
                                            found: key.clone(),
                                        },
                                    ));
                                }
                            } else {
                                network_key = Some(key);
                            }
                        }
                    }

                    for (index, output) in
                        require_vec(order_yaml, "outputs", Some(location.clone()))?
                            .iter()
                            .enumerate()
                    {
                        let location = format!("output index '{index}' in order '{order_key}'");

                        let token_key =
                            require_string(output, Some("token"), Some(location.clone()))?;
                        let res = TokenCfg::parse_network_key(documents.clone(), &token_key);
                        if let Ok(key) = res {
                            if let Some(ref existing_key) = network_key {
                                if *existing_key != key {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::OutputTokenNetworkDoesNotMatch {
                                            key: token_key,
                                            expected: existing_key.clone(),
                                            found: key.clone(),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(
            network_key.ok_or(ParseOrderConfigSourceError::NetworkNotFoundError(
                String::new(),
            ))?,
        )
    }

    pub fn parse_vault_ids(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        order_key: &str,
        r#type: VaultType,
    ) -> Result<HashMap<String, Option<String>>, YamlError> {
        let mut vault_ids = HashMap::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orders_hash) = require_hash(&document_read, Some("orders"), None) {
                if let Some(order_yaml) =
                    orders_hash.get(&StrictYaml::String(order_key.to_string()))
                {
                    let location = format!("order '{}'", order_key);

                    let items = match r#type {
                        VaultType::Input => {
                            require_vec(order_yaml, "inputs", Some(location.clone()))?
                        }
                        VaultType::Output => {
                            require_vec(order_yaml, "outputs", Some(location.clone()))?
                        }
                    };

                    for (idx, item) in items.iter().enumerate() {
                        let token = require_string(
                            item,
                            Some("token"),
                            Some(format!(
                                "{} index '{}' in order '{}'",
                                if r#type == VaultType::Input {
                                    "input"
                                } else {
                                    "output"
                                },
                                idx,
                                order_key
                            )),
                        )?;
                        let vault_id = optional_string(item, "vault-id");
                        vault_ids.insert(token, vault_id);
                    }
                }
            }
        }

        if vault_ids.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidType {
                    field: "orders".to_string(),
                    expected: "a map".to_string(),
                },
                location: "root".to_string(),
            });
        }

        Ok(vault_ids)
    }

    pub fn parse_io_token_keys(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        order_key: &str,
    ) -> Result<Vec<String>, YamlError> {
        let mut token_keys = BTreeSet::new();

        for document in documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orders_hash) = require_hash(&document_read, Some("orders"), None) {
                if let Some(order_yaml) =
                    orders_hash.get(&StrictYaml::String(order_key.to_string()))
                {
                    let location = format!("order '{}'", order_key);

                    let inputs = require_vec(order_yaml, "inputs", Some(location.clone()))?;
                    let outputs = require_vec(order_yaml, "outputs", Some(location.clone()))?;

                    for input in inputs {
                        let token_key =
                            require_string(input, Some("token"), Some(location.clone()))?;
                        token_keys.insert(token_key);
                    }
                    for output in outputs {
                        let token_key =
                            require_string(output, Some("token"), Some(location.clone()))?;
                        token_keys.insert(token_key);
                    }
                }
            }
        }

        Ok(token_keys.into_iter().collect())
    }
}

impl YamlParsableHash for OrderCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut orders = HashMap::new();

        let rainlangs = RainlangCfg::parse_all_from_yaml(documents.clone(), context);
        let orderbooks = OrderbookCfg::parse_all_from_yaml(documents.clone(), context);
        let tokens = TokenCfg::parse_all_from_yaml(documents.clone(), context);

        let tokens = if let Some(context) = context {
            if context.select_tokens.is_some() {
                match tokens {
                    Ok(tokens) => Ok(tokens),
                    Err(err) => match err {
                        YamlError::Field {
                            kind: FieldErrorKind::Missing(field),
                            location,
                        } if field == "tokens" && location == "root" => Err(YamlError::Field {
                            kind: FieldErrorKind::Missing(field),
                            location,
                        }),
                        other => return Err(other),
                    },
                }
            } else {
                tokens
            }
        } else {
            tokens
        };

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(orders_hash) = require_hash(&document_read, Some("orders"), None) {
                for (key_yaml, order_yaml) in orders_hash {
                    let order_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("order '{}'", order_key);

                    if let Some(context) = context {
                        if let Some(current_order) = context.get_current_order() {
                            if current_order != &order_key {
                                continue;
                            }
                        }
                    }

                    let mut network: Option<Arc<NetworkCfg>> = None;

                    let rainlang = match optional_string(order_yaml, "rainlang") {
                        Some(rainlang_name) => {
                            let rainlangs = rainlangs.as_ref().map_err(|e| YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "rainlangs".to_string(),
                                    reason: e.to_string(),
                                },
                                location: "root".to_string(),
                            })?;
                            let rainlang_cfg = Arc::new(
                                rainlangs
                                    .get(&rainlang_name)
                                    .ok_or_else(|| {
                                        YamlError::KeyNotFound(rainlang_name.to_string())
                                    })?
                                    .clone(),
                            );
                            if let Some(n) = &network {
                                if rainlang_cfg.network != *n {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::RainlangNetworkDoesNotMatch {
                                            expected: n.key.clone(),
                                            found: rainlang_cfg.network.key.clone(),
                                        },
                                    ));
                                }
                            } else {
                                network = Some(rainlang_cfg.network.clone());
                            }
                            Some(rainlang_cfg)
                        }
                        None => None,
                    };

                    let orderbook = match optional_string(order_yaml, "orderbook") {
                        Some(orderbook_name) => {
                            let orderbooks = orderbooks.as_ref().map_err(|e| YamlError::Field {
                                kind: FieldErrorKind::InvalidValue {
                                    field: "orderbooks".to_string(),
                                    reason: e.to_string(),
                                },
                                location: "root".to_string(),
                            })?;
                            let orderbook = Arc::new(
                                orderbooks
                                    .get(&orderbook_name)
                                    .ok_or_else(|| {
                                        YamlError::KeyNotFound(orderbook_name.to_string())
                                    })?
                                    .clone(),
                            );
                            if let Some(n) = &network {
                                if orderbook.network != *n {
                                    return Err(YamlError::ParseOrderConfigSourceError(
                                        ParseOrderConfigSourceError::OrderbookNetworkDoesNotMatch {
                                            expected: n.key.clone(),
                                            found: orderbook.network.key.clone(),
                                        },
                                    ));
                                }
                            } else {
                                network = Some(orderbook.network.clone());
                            }
                            Some(orderbook)
                        }
                        None => None,
                    };

                    let inputs = require_vec(
                        order_yaml,
                        "inputs",
                        Some(location.clone()),
                    )?
                    .iter()
                    .enumerate()
                    .map(|(i, input)| {
                        let location = format!("input index '{i}' in order '{order_key}'");

                        let token_name = require_string(
                            input,
                            Some("token"),
                            Some(location.clone()),
                        )?;

                        let mut order_token = None;

                        if let Ok(tokens) = &tokens {
                            let token = tokens.get(&token_name);

                            if let Some(token) = token {
                                if let Some(n) = &network {
                                    if token.network != *n {
                                        return Err(YamlError::ParseOrderConfigSourceError(
                                            ParseOrderConfigSourceError::InputTokenNetworkDoesNotMatch {
                                                key: token_name,
                                                expected: n.key.clone(),
                                                found: token.network.key.clone(),
                                            },
                                        ));
                                    }
                                } else {
                                    network = Some(token.network.clone());
                                }

                                order_token = Some(token.clone());
                            } else if let Some(context) = context {
                                if !context.is_select_token(&token_name) {
                                    return Err(YamlError::Field {
                                        kind: FieldErrorKind::InvalidValue {
                                            field: "token".to_string(),
                                            reason: format!(
                                                "missing yaml data for token '{token_name}'"
                                            ),
                                        },
                                        location: location.clone(),
                                    });
                                }
                            }
                        } else if let Some(context) = context {
                            if !context.is_select_token(&token_name) {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "token".to_string(),
                                        reason: format!(
                                            "missing yaml data for token '{token_name}'"
                                        ),
                                    },
                                    location: location.clone(),
                                });
                            }
                        }

                        let vault_id = match optional_string(input, "vault-id") {
                            Some(id) => Some(OrderCfg::validate_vault_id(&id).map_err(|e| {
                                YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "vault-id".to_string(),
                                        reason: e.to_string(),
                                    },
                                    location: location.clone(),
                                }
                            })?),
                            None => None,
                        };

                        Ok(OrderIOCfg {
                            token_key: token_name,
                            token: order_token.map(Arc::new),
                            vault_id,
                        })
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let outputs = require_vec(
                        order_yaml,
                        "outputs",
                        Some(location.clone()),
                    )?
                    .iter()
                    .enumerate()
                    .map(|(i, output)| {
                        let location = format!("output index '{i}' in order '{order_key}'");

                        let token_name = require_string(
                            output,
                            Some("token"),
                            Some(location.clone()),
                        )?;

                        let mut order_token = None;

                        if let Ok(tokens) = &tokens {
                            let token = tokens.get(&token_name);

                            if let Some(token) = token {
                                if let Some(n) = &network {
                                    if token.network != *n {
                                        return Err(YamlError::ParseOrderConfigSourceError(
                                            ParseOrderConfigSourceError::OutputTokenNetworkDoesNotMatch {
                                                key: token_name,
                                                expected: n.key.clone(),
                                                found: token.network.key.clone(),
                                            },
                                        ));
                                    }
                                } else {
                                    network = Some(token.network.clone());
                                }

                                order_token = Some(token.clone());
                            } else if let Some(context) = context {
                                if !context.is_select_token(&token_name) {
                                    return Err(YamlError::Field {
                                        kind: FieldErrorKind::InvalidValue {
                                            field: "token".to_string(),
                                            reason: format!(
                                                "missing yaml data for token '{token_name}'"
                                            ),
                                        },
                                        location: location.clone(),
                                    });
                                }
                            }
                        } else if let Some(context) = context {
                            if !context.is_select_token(&token_name) {
                                return Err(YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "token".to_string(),
                                        reason: format!(
                                            "missing yaml data for token '{token_name}'"
                                        ),
                                    },
                                    location: location.clone(),
                                });
                            }
                        }

                        let vault_id = match optional_string(output, "vault-id") {
                            Some(id) => Some(OrderCfg::validate_vault_id(&id).map_err(|e| {
                                YamlError::Field {
                                    kind: FieldErrorKind::InvalidValue {
                                        field: "vault-id".to_string(),
                                        reason: e.to_string(),
                                    },
                                    location: location.clone(),
                                }
                            })?),
                            None => None,
                        };

                        Ok(OrderIOCfg {
                            token_key: token_name,
                            token: order_token.map(Arc::new),
                            vault_id,
                        })
                    })
                    .collect::<Result<Vec<_>, YamlError>>()?;

                    let oracle_url = optional_string(order_yaml, "oracle-url");

                    let order = OrderCfg {
                        document: document.clone(),
                        key: order_key.clone(),
                        inputs,
                        outputs,
                        network: network.ok_or(
                            ParseOrderConfigSourceError::NetworkNotFoundError(String::new()),
                        )?,
                        rainlang,
                        orderbook,
                        oracle_url,
                    };

                    if orders.contains_key(&order_key) {
                        return Err(YamlError::KeyShadowing(order_key, "orders".to_string()));
                    }
                    orders.insert(order_key, order);
                }
            }
        }

        if orders.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("orders".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(orders)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let orders_key = StrictYaml::String("orders".to_string());
            let Some(orders_value) = root_hash.get(&orders_key) else {
                continue;
            };
            let StrictYaml::Hash(ref orders_hash) = *orders_value else {
                continue;
            };

            let mut sanitized_orders: Vec<(String, StrictYaml)> = Vec::new();

            for (order_key, order_value) in orders_hash {
                let Some(order_key_str) = order_key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref order_hash) = *order_value else {
                    continue;
                };

                let mut sanitized_order = Hash::new();

                for allowed_key in ALLOWED_ORDER_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = order_hash.get(&key_yaml) {
                        if *allowed_key == "inputs" || *allowed_key == "outputs" {
                            if let StrictYaml::Array(ref io_array) = *v {
                                let sanitized_io: Vec<StrictYaml> = io_array
                                    .iter()
                                    .filter_map(|io_item| {
                                        let StrictYaml::Hash(ref io_hash) = *io_item else {
                                            return None;
                                        };

                                        let mut sanitized_io_item = Hash::new();
                                        for allowed_io_key in ALLOWED_ORDER_IO_KEYS.iter() {
                                            let io_key_yaml =
                                                StrictYaml::String(allowed_io_key.to_string());
                                            if let Some(io_v) = io_hash.get(&io_key_yaml) {
                                                sanitized_io_item.insert(io_key_yaml, io_v.clone());
                                            }
                                        }
                                        Some(StrictYaml::Hash(sanitized_io_item))
                                    })
                                    .collect();
                                sanitized_order.insert(key_yaml, StrictYaml::Array(sanitized_io));
                            } else {
                                sanitized_order.insert(key_yaml, v.clone());
                            }
                        } else {
                            sanitized_order.insert(key_yaml, v.clone());
                        }
                    }
                }

                sanitized_orders
                    .push((order_key_str.to_string(), StrictYaml::Hash(sanitized_order)));
            }

            sanitized_orders.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_orders_hash = Hash::new();
            for (key, value) in sanitized_orders {
                new_orders_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(orders_key, StrictYaml::Hash(new_orders_hash));
        }

        Ok(())
    }
}

impl Default for OrderCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            inputs: vec![],
            outputs: vec![],
            network: Arc::new(NetworkCfg::default()),
            rainlang: None,
            orderbook: None,
            oracle_url: None,
        }
    }
}

impl PartialEq for OrderCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.inputs == other.inputs
            && self.outputs == other.outputs
            && self.network == other.network
            && self.rainlang == other.rainlang
            && self.orderbook == other.orderbook
            && self.oracle_url == other.oracle_url
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseOrderConfigSourceError {
    #[error("Failed to parse rainlang")]
    RainlangParseError(ParseRainlangConfigSourceError),
    #[error("Failed to parse orderbook")]
    OrderbookParseError(ParseOrderbookConfigSourceError),
    #[error("Failed to parse token")]
    TokenParseError(ParseTokenConfigSourceError),
    #[error("Network not found for Order: {0}")]
    NetworkNotFoundError(String),
    #[error("Network does not match")]
    NetworkNotMatch,
    #[error("Rainlang network does not match: expected {expected}, found {found}")]
    RainlangNetworkDoesNotMatch { expected: String, found: String },
    #[error("Orderbook network does not match: expected {expected}, found {found}")]
    OrderbookNetworkDoesNotMatch { expected: String, found: String },
    #[error(
        "Input token network with key: {key} does not match: expected {expected}, found {found}"
    )]
    InputTokenNetworkDoesNotMatch {
        key: String,
        expected: String,
        found: String,
    },
    #[error(
        "Output token network with key: {key} does not match: expected {expected}, found {found}"
    )]
    OutputTokenNetworkDoesNotMatch {
        key: String,
        expected: String,
        found: String,
    },
    #[error("Failed to parse vault id: {0}")]
    VaultParseError(#[from] alloy::primitives::ruint::ParseError),
}

impl ParseOrderConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseOrderConfigSourceError::RainlangParseError(err) =>
                err.to_readable_msg(),
            ParseOrderConfigSourceError::OrderbookParseError(err) =>
                err.to_readable_msg(),
            ParseOrderConfigSourceError::TokenParseError(err) =>
                err.to_readable_msg(),
            ParseOrderConfigSourceError::NetworkNotFoundError(_) =>
                "No network could be determined for this order. Please specify a network or ensure that tokens, rainlangs, or orderbooks have valid networks.".to_string(),
            ParseOrderConfigSourceError::NetworkNotMatch =>
                "The networks specified in your order configuration do not match. All components (tokens, rainlangs, orderbooks) must use the same network.".to_string(),
            ParseOrderConfigSourceError::RainlangNetworkDoesNotMatch { expected, found } =>
                format!("Network mismatch in your YAML configuration: The rainlang is using network '{}' but the order is using network '{}'. Please ensure all components use the same network.", found, expected),
            ParseOrderConfigSourceError::OrderbookNetworkDoesNotMatch { expected, found } =>
                format!("Network mismatch in your YAML configuration: The orderbook is using network '{}' but the order is using network '{}'. Please ensure all components use the same network.", found, expected),
            ParseOrderConfigSourceError::InputTokenNetworkDoesNotMatch { key, expected, found } =>
                format!("Network mismatch in your YAML configuration: The input token '{}' is using network '{}' but the order is using network '{}'. Please ensure all components use the same network.", key, found, expected),
            ParseOrderConfigSourceError::OutputTokenNetworkDoesNotMatch { key, expected, found } =>
                format!("Network mismatch in your YAML configuration: The output token '{}' is using network '{}' but the order is using network '{}'. Please ensure all components use the same network.", key, found, expected),
            ParseOrderConfigSourceError::VaultParseError(err) =>
                format!("The vault ID in your YAML configuration is invalid. Please provide a valid number: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml::tests::get_document;

    #[test]
    fn test_parse_orders_from_yaml() {
        let yaml = r#"
test: test
"#;
        let error = OrderCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("orders".to_string()),
                location: "root".to_string()
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'orders' in root"
        );

        let yaml = r#"
orders:
    order1:
"#;
        let error = OrderCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("inputs".to_string()),
                location: "order 'order1'".to_string()
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'inputs' in order 'order1'"
        );

        let yaml = r#"
orders:
    order1:
        inputs:
            - test: test
"#;
        let error = OrderCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("token".to_string()),
                location: "input index '0' in order 'order1'".to_string()
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'token' in input index '0' in order 'order1'"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    eth:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
orders:
    order1:
        inputs:
            - token: eth
"#;
        let error = OrderCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("outputs".to_string()),
                location: "order 'order1'".to_string()
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'outputs' in order 'order1'"
        );

        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
tokens:
    eth:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - test: test
"#;
        let error = OrderCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("token".to_string()),
                location: "output index '0' in order 'order1'".to_string()
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'token' in output index '0' in order 'order1'"
        );
    }

    #[test]
    fn test_parse_orders_from_yaml_multiple_files() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
rainlangs:
    mainnet:
        address: 0x0000000000000000000000000000000000000001
        network: mainnet
tokens:
    token-one:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
    token-two:
        network: mainnet
        address: 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
orders:
    OrderOne:
        rainlang: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;
        let yaml_two = r#"
orders:
    OrderTwo:
        rainlang: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let orders = OrderCfg::parse_all_from_yaml(documents, None).unwrap();

        assert_eq!(orders.len(), 2);
        assert!(orders.contains_key("OrderOne"));
        assert!(orders.contains_key("OrderTwo"));

        assert_eq!(orders.get("OrderOne").unwrap().key, "OrderOne");
        assert_eq!(orders.get("OrderTwo").unwrap().key, "OrderTwo");
    }

    #[test]
    fn test_parse_orders_from_yaml_duplicate_key() {
        let yaml_one = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
rainlangs:
    mainnet:
        address: 0x0000000000000000000000000000000000000001
        network: mainnet
tokens:
    token-one:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
    token-two:
        network: mainnet
        address: 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
orders:
    DuplicateOrder:
        rainlang: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;
        let yaml_two = r#"
orders:
    DuplicateOrder:
        rainlang: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;

        let documents = vec![get_document(yaml_one), get_document(yaml_two)];
        let error = OrderCfg::parse_all_from_yaml(documents, None).unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateOrder".to_string(), "orders".to_string())
        );
        assert_eq!(error.to_readable_msg(), "The key 'DuplicateOrder' is defined multiple times in your YAML configuration at orders");
    }

    #[test]
    fn parse_network_key() {
        let yaml = r#"
orders: test
"#;
        let error = OrderCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseOrderConfigSourceError(
                ParseOrderConfigSourceError::NetworkNotFoundError("".to_string())
            )
        );
        assert_eq!(
            error.to_readable_msg(),
            "Order configuration error in your YAML: No network could be determined for this order. Please specify a network or ensure that tokens, rainlangs, or orderbooks have valid networks."
        );

        let yaml = r#"
orders:
  - test
"#;
        let error = OrderCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseOrderConfigSourceError(
                ParseOrderConfigSourceError::NetworkNotFoundError("".to_string())
            )
        );
        assert_eq!(
            error.to_readable_msg(),
            "Order configuration error in your YAML: No network could be determined for this order. Please specify a network or ensure that tokens, rainlangs, or orderbooks have valid networks."
        );

        let yaml = r#"
orders:
  - test: test
"#;
        let error = OrderCfg::parse_network_key(vec![get_document(yaml)], "order1").unwrap_err();
        assert_eq!(
            error,
            YamlError::ParseOrderConfigSourceError(
                ParseOrderConfigSourceError::NetworkNotFoundError("".to_string())
            )
        );
        assert_eq!(
            error.to_readable_msg(),
            "Order configuration error in your YAML: No network could be determined for this order. Please specify a network or ensure that tokens, rainlangs, or orderbooks have valid networks."
        );
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_order_keys() {
        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
        outputs:
            - token: usdc
        rainlang: mainnet
        orderbook: mainnet
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let orders_hash = root_hash
            .get(&StrictYaml::String("orders".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let order_hash = orders_hash
            .get(&StrictYaml::String("order1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(order_hash.len(), 4);
        assert!(order_hash.contains_key(&StrictYaml::String("inputs".to_string())));
        assert!(order_hash.contains_key(&StrictYaml::String("outputs".to_string())));
        assert!(order_hash.contains_key(&StrictYaml::String("rainlang".to_string())));
        assert!(order_hash.contains_key(&StrictYaml::String("orderbook".to_string())));
        assert!(!order_hash.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!order_hash.contains_key(&StrictYaml::String("another-unknown".to_string())));
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_io_keys() {
        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
              vault-id: 123
              unknown: dropped
        outputs:
            - token: usdc
              extra-field: removed
"#;
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let orders_hash = root_hash
            .get(&StrictYaml::String("orders".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let order_hash = orders_hash
            .get(&StrictYaml::String("order1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let inputs = order_hash
            .get(&StrictYaml::String("inputs".to_string()))
            .unwrap()
            .as_vec()
            .unwrap();
        let input_hash = inputs[0].as_hash().unwrap();
        assert_eq!(input_hash.len(), 2);
        assert!(input_hash.contains_key(&StrictYaml::String("token".to_string())));
        assert!(input_hash.contains_key(&StrictYaml::String("vault-id".to_string())));
        assert!(!input_hash.contains_key(&StrictYaml::String("unknown".to_string())));

        let outputs = order_hash
            .get(&StrictYaml::String("outputs".to_string()))
            .unwrap()
            .as_vec()
            .unwrap();
        let output_hash = outputs[0].as_hash().unwrap();
        assert_eq!(output_hash.len(), 1);
        assert!(output_hash.contains_key(&StrictYaml::String("token".to_string())));
        assert!(!output_hash.contains_key(&StrictYaml::String("extra-field".to_string())));
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
orders:
    zebra:
        inputs:
            - token: a
        outputs:
            - token: b
    alpha:
        inputs:
            - token: c
        outputs:
            - token: d
"#;
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let orders_hash = root_hash
            .get(&StrictYaml::String("orders".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let keys: Vec<&str> = orders_hash.iter().filter_map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["alpha", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_orders() {
        let yaml = r#"
orders:
    valid-order:
        inputs:
            - token: eth
        outputs:
            - token: usdc
    invalid-order: not_a_hash
"#;
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let orders_hash = root_hash
            .get(&StrictYaml::String("orders".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        assert_eq!(orders_hash.len(), 1);
        assert!(orders_hash.contains_key(&StrictYaml::String("valid-order".to_string())));
        assert!(!orders_hash.contains_key(&StrictYaml::String("invalid-order".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_missing_section() {
        let yaml = r#"
other-section: value
"#;
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = "just a string";
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_section() {
        let yaml = r#"
orders: not_a_hash
"#;
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_io_items() {
        let yaml = r#"
orders:
    order1:
        inputs:
            - token: eth
            - not_a_hash
        outputs:
            - token: usdc
"#;
        let document = get_document(yaml);
        OrderCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let root_hash = doc_read.as_hash().unwrap();
        let orders_hash = root_hash
            .get(&StrictYaml::String("orders".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();
        let order_hash = orders_hash
            .get(&StrictYaml::String("order1".to_string()))
            .unwrap()
            .as_hash()
            .unwrap();

        let inputs = order_hash
            .get(&StrictYaml::String("inputs".to_string()))
            .unwrap()
            .as_vec()
            .unwrap();
        assert_eq!(inputs.len(), 1);
    }

    #[test]
    fn test_parse_order_with_oracle_url() {
        let yaml = r#"
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
deployers:
    mainnet:
        address: 0x0000000000000000000000000000000000000001
        network: mainnet
tokens:
    token-one:
        network: mainnet
        address: 0x1234567890123456789012345678901234567890
    token-two:
        network: mainnet
        address: 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
orders:
    oracle-order:
        deployer: mainnet
        oracle-url: https://my-oracle.example.com/context
        inputs:
            - token: token-one
        outputs:
            - token: token-two
    plain-order:
        deployer: mainnet
        inputs:
            - token: token-one
        outputs:
            - token: token-two
"#;
        let orders = OrderCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap();

        assert_eq!(orders.len(), 2);

        let oracle_order = orders.get("oracle-order").unwrap();
        assert_eq!(
            oracle_order.oracle_url.as_deref(),
            Some("https://my-oracle.example.com/context")
        );

        let plain_order = orders.get("plain-order").unwrap();
        assert!(plain_order.oracle_url.is_none());
    }
}
