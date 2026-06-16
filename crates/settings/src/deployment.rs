use crate::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{strict_yaml::Hash, StrictYaml};
use thiserror::Error;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};
use yaml::{
    context::{Context, GuiContextTrait},
    default_document, require_hash, require_string, FieldErrorKind, YamlError, YamlParsableHash,
};

const ALLOWED_DEPLOYMENT_KEYS: [&str; 2] = ["scenario", "order"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DeploymentCfg {
    #[serde(skip, default = "default_document")]
    pub document: Arc<RwLock<StrictYaml>>,
    pub key: String,
    pub scenario: Arc<ScenarioCfg>,
    pub order: Arc<OrderCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentCfg);

impl DeploymentCfg {
    pub fn parse_order_key(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        deployment_key: &str,
    ) -> Result<String, YamlError> {
        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(deployments_hash) = require_hash(&document_read, Some("deployments"), None) {
                if let Some(deployment_yaml) =
                    deployments_hash.get(&StrictYaml::String(deployment_key.to_string()))
                {
                    return require_string(deployment_yaml, Some("order"), None);
                }
            }
        }
        Err(YamlError::Field {
            kind: FieldErrorKind::Missing("order".to_string()),
            location: format!("deployment '{deployment_key}'"),
        })
    }
}

impl YamlParsableHash for DeploymentCfg {
    fn parse_all_from_yaml(
        documents: Vec<Arc<RwLock<StrictYaml>>>,
        context: Option<&Context>,
    ) -> Result<HashMap<String, Self>, YamlError> {
        let mut deployments = HashMap::new();

        for document in &documents {
            let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;

            if let Ok(deployments_hash) = require_hash(&document_read, Some("deployments"), None) {
                for (key_yaml, deployment_yaml) in deployments_hash {
                    let deployment_key = key_yaml.as_str().unwrap_or_default().to_string();
                    let location = format!("deployment '{deployment_key}'");

                    if let Some(context) = context {
                        if let Some(current_deployment) = context.get_current_deployment() {
                            if current_deployment != &deployment_key {
                                continue;
                            }
                        }
                    }

                    let mut context = Context::from_context(context);

                    let order_key =
                        require_string(deployment_yaml, Some("order"), Some(location.clone()))?;
                    context.add_current_order(order_key.clone());

                    let order =
                        OrderCfg::parse_from_yaml(documents.clone(), &order_key, Some(&context))?;
                    context.add_order(Arc::new(order.clone()));

                    let scenario_key =
                        require_string(deployment_yaml, Some("scenario"), Some(location.clone()))?;
                    let scenario = ScenarioCfg::parse_from_yaml(
                        documents.clone(),
                        &scenario_key,
                        Some(&context),
                    )?;

                    if let Some(rainlang) = &order.rainlang {
                        if rainlang != &scenario.rainlang {
                            return Err(YamlError::ParseDeploymentConfigSourceError(
                                ParseDeploymentConfigSourceError::NoMatch,
                            ));
                        }
                    }

                    let deployment = DeploymentCfg {
                        document: document.clone(),
                        key: deployment_key.clone(),
                        scenario: Arc::new(scenario),
                        order: Arc::new(order),
                    };

                    if deployments.contains_key(&deployment_key) {
                        return Err(YamlError::KeyShadowing(
                            deployment_key.clone(),
                            "deployments".to_string(),
                        ));
                    }
                    deployments.insert(deployment_key, deployment);
                }
            }
        }

        if let Some(context) = context {
            if let Some(current_deployment) = context.get_current_deployment() {
                if !deployments.contains_key(current_deployment) {
                    return Err(YamlError::Field {
                        kind: FieldErrorKind::Missing(current_deployment.to_string()),
                        location: "deployments".to_string(),
                    });
                }
            }
        }
        if deployments.is_empty() {
            return Err(YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "root".to_string(),
            });
        }

        Ok(deployments)
    }

    fn sanitize_documents(documents: &[Arc<RwLock<StrictYaml>>]) -> Result<(), YamlError> {
        for document in documents {
            let mut document_write = document.write().map_err(|_| YamlError::WriteLockError)?;
            let StrictYaml::Hash(ref mut root_hash) = *document_write else {
                continue;
            };

            let deployments_key = StrictYaml::String("deployments".to_string());
            let Some(deployments_value) = root_hash.get(&deployments_key) else {
                continue;
            };
            let StrictYaml::Hash(ref deployments_hash) = deployments_value.clone() else {
                continue;
            };

            let mut sanitized_deployments: Vec<(String, StrictYaml)> = Vec::new();

            for (key, value) in deployments_hash {
                let Some(key_str) = key.as_str() else {
                    continue;
                };

                let StrictYaml::Hash(ref deployment_hash) = *value else {
                    continue;
                };

                let mut sanitized = Hash::new();
                for allowed_key in ALLOWED_DEPLOYMENT_KEYS.iter() {
                    let key_yaml = StrictYaml::String(allowed_key.to_string());
                    if let Some(v) = deployment_hash.get(&key_yaml) {
                        sanitized.insert(key_yaml, v.clone());
                    }
                }
                sanitized_deployments.push((key_str.to_string(), StrictYaml::Hash(sanitized)));
            }

            sanitized_deployments.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut new_deployments_hash = Hash::new();
            for (key, value) in sanitized_deployments {
                new_deployments_hash.insert(StrictYaml::String(key), value);
            }

            root_hash.insert(deployments_key, StrictYaml::Hash(new_deployments_hash));
        }

        Ok(())
    }
}

impl Default for DeploymentCfg {
    fn default() -> Self {
        Self {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: String::new(),
            scenario: Arc::new(ScenarioCfg::default()),
            order: Arc::new(OrderCfg::default()),
        }
    }
}

impl PartialEq for DeploymentCfg {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.scenario == other.scenario && self.order == other.order
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum ParseDeploymentConfigSourceError {
    #[error("Scenario not found: {0}")]
    ScenarioNotFoundError(String),
    #[error("Order not found: {0}")]
    OrderNotFoundError(String),
    #[error("Scenario and Order do not match")]
    NoMatch,
}

impl ParseDeploymentConfigSourceError {
    pub fn to_readable_msg(&self) -> String {
        match self {
            ParseDeploymentConfigSourceError::ScenarioNotFoundError(scenario) =>
                format!("The scenario '{}' referenced in your deployment configuration was not found in your YAML configuration. Please check that this scenario is defined correctly.", scenario),
            ParseDeploymentConfigSourceError::OrderNotFoundError(order) =>
                format!("The order '{}' referenced in your deployment configuration was not found in your YAML configuration. Please check that this order is defined correctly.", order),
            ParseDeploymentConfigSourceError::NoMatch =>
                "The scenario and order in your deployment configuration do not match. The rainlang specified in the order must match the rainlang specified in the scenario.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml::tests::get_document;

    #[test]
    fn test_parse_deployments_from_yaml() {
        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
test: test
"#;
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "root".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'deployments' in root"
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
deployments:
    deployment1:
        test: test
        "#;
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'order' in deployment 'deployment1'"
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
deployments:
    deployment1:
        order: order1
        test: test
        "#;
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("scenario".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Missing required field 'scenario' in deployment 'deployment1'"
        );

        let yaml = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
    network2:
        rpcs:
            - https://test.com
        chain-id: 2
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
    registry2:
        address: 0x0000000000000000000000000000000000000000
        network: network2
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network2
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry2
deployments:
    deployment1:
        scenario: scenario1
        order: order1
        "#;
        let error = DeploymentCfg::parse_all_from_yaml(vec![get_document(yaml)], None).unwrap_err();
        assert_eq!(
            error.to_string(),
            YamlError::ParseDeploymentConfigSourceError(ParseDeploymentConfigSourceError::NoMatch)
                .to_string()
        );
        assert_eq!(
            error.to_readable_msg(),
            "Deployment configuration error in your YAML: The scenario and order in your deployment configuration do not match. The rainlang specified in the order must match the rainlang specified in the scenario."
        );
    }

    const PREFIX: &str = r#"
networks:
    network1:
        rpcs:
            - https://eth.llamarpc.com
        chain-id: 1
    network2:
        rpcs:
            - https://test.com
        chain-id: 2
rainlangs:
    registry1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
    registry2:
        address: 0x0000000000000000000000000000000000000000
        network: network2
scenarios:
    scenario1:
        bindings:
            test: test
        rainlang: registry1
    scenario2:
        bindings:
            test: test
        rainlang: registry2
tokens:
    token1:
        address: 0x0000000000000000000000000000000000000000
        network: network1
    token2:
        address: 0x0000000000000000000000000000000000000000
        network: network2
orders:
    order1:
        inputs:
            - token: token1
        outputs:
            - token: token1
        rainlang: registry1
    order2:
        inputs:
            - token: token2
        outputs:
            - token: token2
        rainlang: registry2
"#;

    #[test]
    fn test_parse_deployments_from_yaml_multiple_files() {
        let yaml_one = r#"
deployments:
    DeploymentOne:
        scenario: scenario1
        order: order1
"#;
        let yaml_two = r#"
deployments:
    DeploymentTwo:
        scenario: scenario2
        order: order2
"#;

        let deployments = DeploymentCfg::parse_all_from_yaml(
            vec![
                get_document(&format!("{PREFIX}{yaml_one}")),
                get_document(yaml_two),
            ],
            None,
        )
        .unwrap();

        assert_eq!(deployments.len(), 2);
        assert!(deployments.contains_key("DeploymentOne"));
        assert!(deployments.contains_key("DeploymentTwo"));

        assert_eq!(
            deployments.get("DeploymentOne").unwrap().key,
            "DeploymentOne"
        );
        assert_eq!(
            deployments.get("DeploymentTwo").unwrap().key,
            "DeploymentTwo"
        );
    }

    #[test]
    fn test_parse_deployments_from_yaml_duplicate_key() {
        let yaml_one = r#"
deployments:
    DuplicateDeployment:
        scenario: scenario1
        order: order1
"#;
        let yaml_two = r#"
deployments:
    DuplicateDeployment:
        scenario: scenario2
        order: order2
"#;

        let error = DeploymentCfg::parse_all_from_yaml(
            vec![
                get_document(&format!("{PREFIX}{yaml_one}")),
                get_document(yaml_two),
            ],
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            YamlError::KeyShadowing("DuplicateDeployment".to_string(), "deployments".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The key 'DuplicateDeployment' is defined multiple times in your YAML configuration at deployments"
        );
    }

    #[test]
    fn test_parse_order_key() {
        let yaml = r#"
deployments: test
"#;
        let error =
            DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
deployments:
  - test
"#;
        let error =
            DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
deployments:
  - test: test
"#;
        let error =
            DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("order".to_string()),
                location: "deployment 'deployment1'".to_string(),
            }
        );

        let yaml = r#"
deployments:
  deployment1:
    order: order1
    scenario: scenario1
"#;
        let res = DeploymentCfg::parse_order_key(vec![get_document(yaml)], "deployment1").unwrap();
        assert_eq!(res, "order1");
    }

    #[test]
    fn test_sanitize_documents_drops_unknown_keys() {
        let yaml = r#"
deployments:
    deployment1:
        scenario: scenario1
        order: order1
        unknown-key: should-be-dropped
        another-unknown: also-dropped
"#;
        let document = get_document(yaml);
        DeploymentCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployments = root
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };
        let deployment1 = deployments_hash
            .get(&StrictYaml::String("deployment1".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployment1_hash) = *deployment1 else {
            panic!("expected deployment1 hash");
        };

        assert!(deployment1_hash.contains_key(&StrictYaml::String("scenario".to_string())));
        assert!(deployment1_hash.contains_key(&StrictYaml::String("order".to_string())));
        assert!(!deployment1_hash.contains_key(&StrictYaml::String("unknown-key".to_string())));
        assert!(!deployment1_hash.contains_key(&StrictYaml::String("another-unknown".to_string())));
        assert_eq!(deployment1_hash.len(), 2);
    }

    #[test]
    fn test_sanitize_documents_preserves_allowed_key_order() {
        let yaml = r#"
deployments:
    deployment1:
        order: order1
        scenario: scenario1
        extra: dropped
"#;
        let document = get_document(yaml);
        DeploymentCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployments = root
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };
        let deployment1 = deployments_hash
            .get(&StrictYaml::String("deployment1".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployment1_hash) = *deployment1 else {
            panic!("expected deployment1 hash");
        };

        let keys: Vec<String> = deployment1_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["scenario", "order"]);
    }

    #[test]
    fn test_sanitize_documents_drops_non_hash_entries() {
        let yaml = r#"
deployments:
    deployment1: not-a-hash
"#;
        let document = get_document(yaml);
        DeploymentCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployments = root
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };

        assert!(!deployments_hash.contains_key(&StrictYaml::String("deployment1".to_string())));
        assert!(deployments_hash.is_empty());
    }

    #[test]
    fn test_sanitize_documents_lexicographic_order() {
        let yaml = r#"
deployments:
    zebra:
        scenario: scenario3
        order: order3
    alpha:
        scenario: scenario1
        order: order1
    beta:
        scenario: scenario2
        order: order2
"#;
        let document = get_document(yaml);
        DeploymentCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployments = root
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        let StrictYaml::Hash(ref deployments_hash) = *deployments else {
            panic!("expected deployments hash");
        };

        let keys: Vec<String> = deployments_hash
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect();
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_sanitize_documents_handles_missing_deployments_section() {
        let yaml = r#"
other: value
"#;
        let document = get_document(yaml);
        DeploymentCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        assert!(!root.contains_key(&StrictYaml::String("deployments".to_string())));
    }

    #[test]
    fn test_sanitize_documents_handles_non_hash_root() {
        let yaml = r#"just a string"#;
        let document = get_document(yaml);
        DeploymentCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();
    }

    #[test]
    fn test_sanitize_documents_skips_non_hash_deployments() {
        let yaml = r#"
deployments: not-a-hash
"#;
        let document = get_document(yaml);
        DeploymentCfg::sanitize_documents(std::slice::from_ref(&document)).unwrap();

        let doc_read = document.read().unwrap();
        let StrictYaml::Hash(ref root) = *doc_read else {
            panic!("expected root hash");
        };
        let deployments = root
            .get(&StrictYaml::String("deployments".to_string()))
            .unwrap();
        assert_eq!(deployments.as_str(), Some("not-a-hash"));
    }

    #[test]
    fn test_sanitize_documents_per_doc_no_cross_merge() {
        let yaml_one = r#"
deployments:
    deployment-one:
        scenario: scenario1
        order: order1
        extra-key: dropped
"#;
        let yaml_two = r#"
deployments:
    deployment-two:
        scenario: scenario2
        order: order2
        another-extra: also-dropped
"#;
        let doc_one = get_document(yaml_one);
        let doc_two = get_document(yaml_two);
        let documents = vec![doc_one.clone(), doc_two.clone()];
        DeploymentCfg::sanitize_documents(&documents).unwrap();

        {
            let doc_read = doc_one.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let deployments = root
                .get(&StrictYaml::String("deployments".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployments_hash) = *deployments else {
                panic!("expected deployments hash");
            };

            let keys: Vec<String> = deployments_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["deployment-one"]);

            let deployment = deployments_hash
                .get(&StrictYaml::String("deployment-one".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployment_hash) = *deployment else {
                panic!("expected deployment hash");
            };
            assert!(!deployment_hash.contains_key(&StrictYaml::String("extra-key".to_string())));
        }

        {
            let doc_read = doc_two.read().unwrap();
            let StrictYaml::Hash(ref root) = *doc_read else {
                panic!("expected root hash");
            };
            let deployments = root
                .get(&StrictYaml::String("deployments".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployments_hash) = *deployments else {
                panic!("expected deployments hash");
            };

            let keys: Vec<String> = deployments_hash
                .keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect();
            assert_eq!(keys, vec!["deployment-two"]);

            let deployment = deployments_hash
                .get(&StrictYaml::String("deployment-two".to_string()))
                .unwrap();
            let StrictYaml::Hash(ref deployment_hash) = *deployment else {
                panic!("expected deployment hash");
            };
            assert!(!deployment_hash.contains_key(&StrictYaml::String("another-extra".to_string())));
        }
    }
}
