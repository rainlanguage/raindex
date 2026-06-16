use super::{cache::Cache, orderbook::OrderbookYaml, sanitize_all_documents, ValidationConfig, *};
use crate::{spec_version::SpecVersion, ChartCfg, DeploymentCfg, GuiCfg, OrderCfg, ScenarioCfg};
use serde::{
    de::{self, IgnoredAny, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    fmt,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::StrictYaml;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DotrainYaml {
    #[cfg_attr(target_family = "wasm", tsify(type = "string[]"))]
    pub documents: Vec<Arc<RwLock<StrictYaml>>>,
    pub cache: Cache,
    pub profile: ContextProfile,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DotrainYaml);

#[derive(Debug, Clone, Default)]
pub struct DotrainYamlValidation {
    pub orders: bool,
    pub scenarios: bool,
    pub deployments: bool,
}
impl DotrainYamlValidation {
    pub fn full() -> Self {
        DotrainYamlValidation {
            orders: true,
            scenarios: true,
            deployments: true,
        }
    }
}
impl ValidationConfig for DotrainYamlValidation {
    fn should_validate_networks(&self) -> bool {
        false
    }
    fn should_validate_remote_networks(&self) -> bool {
        false
    }
    fn should_validate_tokens(&self) -> bool {
        false
    }
    fn should_validate_remote_tokens(&self) -> bool {
        false
    }
    fn should_validate_subgraphs(&self) -> bool {
        false
    }
    fn should_validate_local_db_remotes(&self) -> bool {
        false
    }
    fn should_validate_local_db_sync(&self) -> bool {
        false
    }
    fn should_validate_orderbooks(&self) -> bool {
        false
    }
    fn should_validate_metaboards(&self) -> bool {
        false
    }
    fn should_validate_rainlangs(&self) -> bool {
        false
    }
    fn should_validate_orders(&self) -> bool {
        self.orders
    }
    fn should_validate_scenarios(&self) -> bool {
        self.scenarios
    }
    fn should_validate_deployments(&self) -> bool {
        self.deployments
    }
}

impl YamlParsable for DotrainYaml {
    type ValidationConfig = DotrainYamlValidation;

    fn new(sources: Vec<String>, validate: DotrainYamlValidation) -> Result<Self, YamlError> {
        let mut documents = Vec::new();

        for source in sources {
            let docs = StrictYamlLoader::load_from_str(&source)?;
            if docs.is_empty() {
                return Err(YamlError::EmptyFile);
            }
            let doc = docs[0].clone();
            let document = Arc::new(RwLock::new(doc));

            documents.push(document);
        }

        SpecVersion::validate(documents.clone())?;
        sanitize_all_documents(&documents)?;

        if validate.should_validate_orders() {
            OrderCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_scenarios() {
            ScenarioCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_deployments() {
            DeploymentCfg::parse_all_from_yaml(documents.clone(), None)?;
        }

        Ok(DotrainYaml {
            documents,
            cache: Cache::default(),
            profile: ContextProfile::Strict,
        })
    }

    fn from_documents(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Self {
        DotrainYaml {
            documents,
            cache: Cache::default(),
            profile: ContextProfile::Strict,
        }
    }

    fn from_dotrain_yaml(dotrain_yaml: DotrainYaml) -> Self {
        DotrainYaml {
            documents: dotrain_yaml.documents,
            cache: dotrain_yaml.cache,
            profile: dotrain_yaml.profile,
        }
    }

    fn from_orderbook_yaml(orderbook_yaml: OrderbookYaml) -> Self {
        DotrainYaml {
            documents: orderbook_yaml.documents,
            cache: orderbook_yaml.cache,
            profile: ContextProfile::Strict,
        }
    }
}

impl ContextProvider for DotrainYaml {
    fn get_remote_networks_from_cache(&self) -> HashMap<String, NetworkCfg> {
        self.cache.get_remote_networks()
    }

    fn get_remote_tokens_from_cache(&self) -> HashMap<String, TokenCfg> {
        self.cache.get_remote_tokens()
    }
}

impl DotrainYaml {
    pub fn new_with_profile(
        sources: Vec<String>,
        validate: DotrainYamlValidation,
        profile: ContextProfile,
    ) -> Result<Self, YamlError> {
        let mut instance = Self::new(sources, validate)?;
        instance.profile = profile;
        Ok(instance)
    }

    pub fn with_profile(mut self, profile: ContextProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn build_context(&self, profile: &ContextProfile) -> Result<Context, YamlError> {
        let mut context = self.create_context();

        match profile {
            ContextProfile::Strict => {
                self.expand_context_with_remote_networks(&mut context);
                self.expand_context_with_remote_tokens(&mut context);
            }
            ContextProfile::Gui { current_deployment } => {
                self.expand_context_with_current_deployment(&mut context, current_deployment);
                self.expand_context_with_remote_networks(&mut context);
                self.expand_context_with_remote_tokens(&mut context);

                if let Some(select_tokens) =
                    GuiCfg::parse_select_tokens(self.documents.clone(), current_deployment)?
                {
                    context
                        .add_select_tokens(select_tokens.iter().map(|st| st.key.clone()).collect());
                }

                let order_key =
                    DeploymentCfg::parse_order_key(self.documents.clone(), current_deployment)?;
                context.add_current_order(order_key.clone());
                let order =
                    OrderCfg::parse_from_yaml(self.documents.clone(), &order_key, Some(&context))?;
                context.add_order(Arc::new(order));
            }
        }

        Ok(context)
    }

    pub fn get_order_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_orders()?.keys().cloned().collect())
    }
    pub fn get_orders(&self) -> Result<HashMap<String, OrderCfg>, YamlError> {
        let context = self.build_context(&self.profile)?;
        let orders = OrderCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))?;
        Ok(orders)
    }
    pub fn get_order(&self, key: &str) -> Result<OrderCfg, YamlError> {
        let context = self.build_context(&self.profile)?;
        OrderCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }
    pub fn get_order_for_gui_deployment(
        &self,
        order_key: &str,
        deployment_key: &str,
    ) -> Result<OrderCfg, YamlError> {
        let context = self.build_context(&ContextProfile::gui(deployment_key.to_string()))?;
        OrderCfg::parse_from_yaml(self.documents.clone(), order_key, Some(&context))
    }

    pub fn get_scenario_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_scenarios()?.keys().cloned().collect())
    }
    pub fn get_scenarios(&self) -> Result<HashMap<String, ScenarioCfg>, YamlError> {
        let context = self.build_context(&self.profile)?;
        ScenarioCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_scenario(&self, key: &str) -> Result<ScenarioCfg, YamlError> {
        let context = self.build_context(&self.profile)?;
        ScenarioCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }

    pub fn get_deployment_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_deployments()?.keys().cloned().collect())
    }
    pub fn get_deployments(&self) -> Result<HashMap<String, DeploymentCfg>, YamlError> {
        let context = self.build_context(&self.profile)?;
        DeploymentCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_deployment(&self, key: &str) -> Result<DeploymentCfg, YamlError> {
        let context = self.build_context(&self.profile)?;
        DeploymentCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }

    pub fn get_gui(&self, current_deployment: &str) -> Result<Option<GuiCfg>, YamlError> {
        let context = self.build_context(&ContextProfile::gui(current_deployment.to_string()))?;
        GuiCfg::parse_from_yaml_optional(self.documents.clone(), Some(&context))
    }

    pub fn get_chart_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_charts()?.keys().cloned().collect())
    }
    pub fn get_charts(&self) -> Result<HashMap<String, ChartCfg>, YamlError> {
        let context = self.build_context(&self.profile)?;
        ChartCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_chart(&self, key: &str) -> Result<ChartCfg, YamlError> {
        let context = self.build_context(&self.profile)?;
        ChartCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }
}

impl Serialize for DotrainYaml {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut documents = Vec::with_capacity(self.documents.len());
        for doc in &self.documents {
            let yaml_str = Self::get_yaml_string(doc.clone()).map_err(serde::ser::Error::custom)?;
            documents.push(yaml_str);
        }

        let mut state = serializer.serialize_struct("DotrainYaml", 2)?;
        state.serialize_field("documents", &documents)?;
        state.serialize_field("profile", &self.profile)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for DotrainYaml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DotrainYamlVisitor;

        impl<'de> Visitor<'de> for DotrainYamlVisitor {
            type Value = DotrainYaml;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of YAML documents as strings")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut documents: Option<Vec<String>> = None;
                let mut profile = ContextProfile::Strict;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "documents" => {
                            if documents.is_some() {
                                return Err(de::Error::duplicate_field("documents"));
                            }
                            documents = Some(map.next_value()?);
                        }
                        "profile" => {
                            profile = map.next_value()?;
                        }
                        _ => {
                            let _ = map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let documents = documents.ok_or_else(|| de::Error::missing_field("documents"))?;
                let documents = documents
                    .into_iter()
                    .map(|doc_str| {
                        let docs =
                            StrictYamlLoader::load_from_str(&doc_str).map_err(de::Error::custom)?;
                        if docs.is_empty() {
                            return Err(de::Error::custom("Empty YAML document"));
                        }
                        Ok(Arc::new(RwLock::new(docs[0].clone())))
                    })
                    .collect::<Result<Vec<_>, M::Error>>()?;

                Ok(DotrainYaml {
                    documents,
                    cache: Cache::default(),
                    profile,
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut documents = Vec::new();

                while let Some(doc_str) = seq.next_element::<String>()? {
                    let docs =
                        StrictYamlLoader::load_from_str(&doc_str).map_err(de::Error::custom)?;
                    if docs.is_empty() {
                        return Err(de::Error::custom("Empty YAML document"));
                    }
                    let doc = docs[0].clone();
                    documents.push(Arc::new(RwLock::new(doc)));
                }

                Ok(DotrainYaml {
                    documents,
                    cache: Cache::default(),
                    profile: ContextProfile::Strict,
                })
            }
        }

        deserializer.deserialize_any(DotrainYamlVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::yaml::FieldErrorKind;
    use crate::{
        spec_version::SpecVersion,
        test::*,
        yaml::{
            context::{ContextProfile, YamlCacheTrait},
            orderbook::OrderbookYamlValidation,
        },
        BinXOptionsCfg, BinXTransformCfg, DotOptionsCfg, GuiSelectTokensCfg, HexBinOptionsCfg,
        HexBinTransformCfg, LineOptionsCfg, MarkCfg, RectYOptionsCfg, TransformCfg,
        TransformOutputsCfg, VaultType,
    };
    use alloy::primitives::U256;
    use orderbook::OrderbookYaml;

    use super::*;

    fn full_yaml() -> String {
        format!(
            r#"
    version: {version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
        testnet:
            rpcs:
                - https://testnet.infura.io
            chain-id: 1337
    tokens:
        token1:
            network: mainnet
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
        token2:
            network: mainnet
            address: 0x0000000000000000000000000000000000000002
            decimals: 6
            label: USD Coin
            symbol: USDC
    rainlangs:
        scenario1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
        registry2:
            address: 0x0000000000000000000000000000000000000003
            network: testnet
    orders:
        order1:
            inputs:
                - token: token1
                  vault-id: 1
            outputs:
                - token: token2
                  vault-id: 2
    scenarios:
        scenario1:
            bindings:
                key1: value1
            scenarios:
                scenario2:
                    bindings:
                        key2: value2
                    scenarios:
                        runs: 10
    deployments:
        deployment1:
            order: order1
            scenario: scenario1.scenario2
        deployment2:
            order: order1
            scenario: scenario1
    gui:
        name: Test gui
        description: Test description
        short-description: Test short description
        deployments:
            deployment1:
                name: Test deployment
                description: Test description
                deposits:
                    - token: token1
                      presets:
                        - 100
                        - 2000
                fields:
                    - binding: key1
                      name: Binding test
                      presets:
                        - value: value2
                select-tokens:
                    - key: token2
                      name: Test token
                      description: Test description
    charts:
        chart1:
            scenario: scenario1.scenario2
            plots:
                plot1:
                    title: Test title
                    subtitle: Test subtitle
                    marks:
                        - type: dot
                          options:
                            x: 1
                            y: 2
                            r: 3
                            fill: red
                            stroke: blue
                            transform:
                                type: hexbin
                                content:
                                    outputs:
                                        x: 1
                                        y: 2
                                        r: 3
                                        z: 4
                                        stroke: green
                                        fill: blue
                                    options:
                                        x: 1
                                        y: 2
                                        bin-width: 10
                        - type: line
                          options:
                            transform:
                                type: binx
                                content:
                                    outputs:
                                        x: 1
                                    options:
                                        thresholds: 10
                        - type: recty
                          options:
                            x0: 1
                            x1: 2
                            y0: 3
                            y1: 4
                    x:
                       label: Test x label
                       anchor: start
                       label-anchor: start
                       label-arrow: none
                    y:
                       label: Test y label
                       anchor: start
                       label-anchor: start
                       label-arrow: none
                    margin: 10
                    margin-left: 20
                    margin-right: 30
                    margin-top: 40
                    margin-bottom: 50
                    inset: 60
    "#,
            version = SpecVersion::current()
        )
    }

    fn handlebars_yaml() -> String {
        format!(
            r#"
    version: {version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    tokens:
        token1:
            network: mainnet
            address: 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
        token2:
            network: mainnet
            address: 0x0000000000000000000000000000000000000002
            decimals: 6
            label: USD Coin
            symbol: USDC
    rainlangs:
        registry1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
    orders:
        order1:
            inputs:
                - token: token1
                  vault-id: 1
            outputs:
                - token: token2
                  vault-id: 2
    scenarios:
        scenario1:
            bindings:
                key1: ${{order.inputs.0.token.address}}
            rainlang: registry1
            scenarios:
                scenario2:
                    bindings:
                        key2: ${{order.outputs.0.token.address}}
    deployments:
        deployment1:
            order: order1
            scenario: scenario1.scenario2
    gui:
        name: Test gui
        description: Test description
        deployments:
            deployment1:
                name: Test deployment
                description: Test description
                deposits:
                    - token: token1
                      presets:
                        - 100
                        - 2000
                fields:
                    - binding: key1
                      name: Binding for ${{order.inputs.0.token.label}}
                      description: With token symbol ${{order.inputs.0.token.symbol}}
                      presets:
                        - value: value2
    "#,
            version = SpecVersion::current()
        )
    }

    #[test]
    fn test_full_yaml() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();
        let dotrain_yaml =
            DotrainYaml::new(vec![full_yaml()], DotrainYamlValidation::default()).unwrap();

        assert_eq!(dotrain_yaml.get_order_keys().unwrap().len(), 1);
        let order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs.len(), 1);
        let input = order.inputs.first().unwrap();
        assert_eq!(
            *input.token.clone().as_ref().unwrap(),
            ob_yaml.get_token("token1").unwrap().into()
        );
        assert_eq!(input.vault_id, Some(U256::from(1)));
        let output = order.outputs.first().unwrap();
        assert_eq!(
            *output.token.as_ref().unwrap(),
            ob_yaml.get_token("token2").unwrap().into()
        );
        assert_eq!(output.vault_id, Some(U256::from(2)));
        assert_eq!(
            *order.network.as_ref(),
            ob_yaml.get_network("mainnet").unwrap()
        );
        let input_vault_ids =
            OrderCfg::parse_vault_ids(dotrain_yaml.documents.clone(), &order.key, VaultType::Input)
                .unwrap();
        assert_eq!(input_vault_ids.len(), 1);
        assert_eq!(input_vault_ids.get("token1"), Some(&Some("1".to_string())));
        let output_vault_ids = OrderCfg::parse_vault_ids(
            dotrain_yaml.documents.clone(),
            &order.key,
            VaultType::Output,
        )
        .unwrap();
        assert_eq!(output_vault_ids.len(), 1);
        assert_eq!(output_vault_ids.get("token2"), Some(&Some("2".to_string())));
        let io_token_keys =
            OrderCfg::parse_io_token_keys(dotrain_yaml.documents.clone(), &order.key).unwrap();
        assert_eq!(io_token_keys.len(), 2);
        assert_eq!(io_token_keys[0], "token1");
        assert_eq!(io_token_keys[1], "token2");

        let scenario_keys = dotrain_yaml.get_scenario_keys().unwrap();
        assert_eq!(scenario_keys.len(), 2);
        let scenario1 = dotrain_yaml.get_scenario("scenario1").unwrap();
        assert_eq!(scenario1.bindings.len(), 1);
        assert_eq!(scenario1.bindings.get("key1").unwrap(), "value1");
        assert_eq!(
            *scenario1.rainlang.as_ref(),
            ob_yaml.get_rainlang("scenario1").unwrap()
        );
        let scenario2 = dotrain_yaml.get_scenario("scenario1.scenario2").unwrap();
        assert_eq!(scenario2.bindings.len(), 2);
        assert_eq!(scenario2.bindings.get("key1").unwrap(), "value1");
        assert_eq!(scenario2.bindings.get("key2").unwrap(), "value2");
        assert_eq!(
            *scenario2.rainlang.as_ref(),
            ob_yaml.get_rainlang("scenario1").unwrap()
        );

        let deployment_keys = dotrain_yaml.get_deployment_keys().unwrap();
        assert_eq!(deployment_keys.len(), 2);
        let deployment = dotrain_yaml.get_deployment("deployment1").unwrap();
        assert_eq!(
            deployment.order,
            dotrain_yaml.get_order("order1").unwrap().into()
        );
        assert_eq!(
            deployment.scenario,
            dotrain_yaml
                .get_scenario("scenario1.scenario2")
                .unwrap()
                .into()
        );
        let deployment = dotrain_yaml.get_deployment("deployment2").unwrap();
        assert_eq!(
            deployment.order,
            dotrain_yaml.get_order("order1").unwrap().into()
        );
        assert_eq!(
            deployment.scenario,
            dotrain_yaml.get_scenario("scenario1").unwrap().into()
        );
        assert_eq!(
            DeploymentCfg::parse_order_key(dotrain_yaml.documents.clone(), "deployment1").unwrap(),
            "order1"
        );
        assert_eq!(
            DeploymentCfg::parse_order_key(dotrain_yaml.documents.clone(), "deployment2").unwrap(),
            "order1"
        );

        let gui = dotrain_yaml.get_gui("deployment1").unwrap().unwrap();
        assert_eq!(gui.name, "Test gui");
        assert_eq!(gui.description, "Test description");
        assert_eq!(gui.deployments.len(), 1);
        let deployment = gui.deployments.get("deployment1").unwrap();
        assert_eq!(deployment.name, "Test deployment");
        assert_eq!(deployment.description, "Test description");
        assert_eq!(deployment.deposits.len(), 1);
        let deposit = &deployment.deposits[0];
        assert_eq!(
            *deposit.token.as_ref().unwrap(),
            ob_yaml.get_token("token1").unwrap().into()
        );
        let presets = deposit.presets.as_ref().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0], "100".to_string());
        assert_eq!(presets[1], "2000".to_string());
        assert_eq!(deployment.fields.len(), 1);
        let field = &deployment.fields[0];
        assert_eq!(field.binding, "key1");
        assert_eq!(field.name, "Binding test");
        let presets = field.presets.as_ref().unwrap();
        assert_eq!(presets[0].value, "value2");
        let select_tokens = deployment.select_tokens.as_ref().unwrap();
        assert_eq!(select_tokens.len(), 1);
        assert_eq!(select_tokens[0].key, "token2".to_string());
        assert_eq!(select_tokens[0].name, Some("Test token".to_string()));
        assert_eq!(
            select_tokens[0].description,
            Some("Test description".to_string())
        );

        let details = GuiCfg::parse_order_details(dotrain_yaml.documents.clone()).unwrap();
        assert_eq!(details.name, "Test gui");
        assert_eq!(details.description, "Test description");

        let deployment_details =
            GuiCfg::parse_deployment_details(dotrain_yaml.documents.clone()).unwrap();
        assert_eq!(
            deployment_details.get("deployment1").unwrap().name,
            "Test deployment"
        );
        assert_eq!(
            deployment_details.get("deployment1").unwrap().description,
            "Test description"
        );

        let deployment_keys =
            GuiCfg::parse_deployment_keys(dotrain_yaml.documents.clone()).unwrap();
        assert_eq!(deployment_keys.len(), 1);
        assert_eq!(deployment_keys[0], "deployment1");

        let select_tokens =
            GuiCfg::parse_select_tokens(dotrain_yaml.documents.clone(), "deployment1").unwrap();
        assert!(select_tokens.is_some());
        assert_eq!(
            select_tokens.unwrap()[0],
            GuiSelectTokensCfg {
                key: "token2".to_string(),
                name: Some("Test token".to_string()),
                description: Some("Test description".to_string())
            }
        );
        let select_tokens =
            GuiCfg::parse_select_tokens(dotrain_yaml.documents.clone(), "deployment2").unwrap();
        assert!(select_tokens.is_none());

        let gui_context = dotrain_yaml
            .build_context(&ContextProfile::gui("deployment1".to_string()))
            .unwrap();
        assert_eq!(
            gui_context
                .gui_context
                .as_ref()
                .and_then(|gc| gc.current_deployment.clone()),
            Some("deployment1".to_string())
        );
        assert_eq!(gui_context.select_tokens, Some(vec!["token2".to_string()]));

        let field_presets =
            GuiCfg::parse_field_presets(dotrain_yaml.documents.clone(), "deployment1", "key1")
                .unwrap()
                .unwrap();
        assert_eq!(field_presets[0].id, "0");
        assert_eq!(field_presets[0].name, None);
        assert_eq!(field_presets[0].value, "value2");

        let chart_keys = dotrain_yaml.get_chart_keys().unwrap();
        assert_eq!(chart_keys.len(), 1);
        let chart = dotrain_yaml.get_chart(&chart_keys[0]).unwrap();
        assert_eq!(chart.key, "chart1");
        assert_eq!(chart.scenario.key, "scenario1.scenario2");
        let plot = chart.plots.unwrap()[0].clone();
        assert_eq!(plot.title, Some("Test title".to_string()));
        assert_eq!(plot.subtitle, Some("Test subtitle".to_string()));
        assert_eq!(plot.x.unwrap().label, Some("Test x label".to_string()));
        assert_eq!(plot.y.unwrap().label, Some("Test y label".to_string()));
        assert_eq!(plot.margin, Some(10));
        assert_eq!(plot.margin_left, Some(20));
        assert_eq!(plot.margin_right, Some(30));
        assert_eq!(plot.margin_top, Some(40));
        assert_eq!(plot.margin_bottom, Some(50));
        assert_eq!(plot.marks.len(), 3);
        assert_eq!(
            plot.marks[0],
            MarkCfg::Dot(DotOptionsCfg {
                x: Some("1".to_string()),
                y: Some("2".to_string()),
                r: Some(3),
                fill: Some("red".to_string()),
                stroke: Some("blue".to_string()),
                transform: Some(TransformCfg::HexBin(HexBinTransformCfg {
                    outputs: TransformOutputsCfg {
                        x: Some("1".to_string()),
                        y: Some("2".to_string()),
                        r: Some(3),
                        z: Some("4".to_string()),
                        stroke: Some("green".to_string()),
                        fill: Some("blue".to_string()),
                    },
                    options: HexBinOptionsCfg {
                        x: Some("1".to_string()),
                        y: Some("2".to_string()),
                        bin_width: Some(10),
                    },
                })),
            })
        );
        assert_eq!(
            plot.marks[1],
            MarkCfg::Line(LineOptionsCfg {
                x: None,
                y: None,
                r: None,
                fill: None,
                stroke: None,
                transform: Some(TransformCfg::BinX(BinXTransformCfg {
                    outputs: TransformOutputsCfg {
                        x: Some("1".to_string()),
                        y: None,
                        r: None,
                        z: None,
                        stroke: None,
                        fill: None,
                    },
                    options: BinXOptionsCfg {
                        x: None,
                        thresholds: Some(10),
                    },
                })),
            })
        );
        assert_eq!(
            plot.marks[2],
            MarkCfg::RectY(RectYOptionsCfg {
                x0: Some("1".to_string()),
                x1: Some("2".to_string()),
                y0: Some("3".to_string()),
                y1: Some("4".to_string()),
                transform: None,
            })
        );
    }

    #[test]
    fn test_build_context_profiles() {
        let yaml = format!(
            r#"
        version: {}
        networks:
            mainnet:
                rpcs:
                    - https://mainnet.infura.io
                chain-id: 1
        rainlangs:
            registry1:
                address: 0x0000000000000000000000000000000000000002
                network: mainnet
        tokens:
            token1:
                network: mainnet
                address: 0x0000000000000000000000000000000000000003
                decimals: 18
            token2:
                network: mainnet
                address: 0x0000000000000000000000000000000000000004
                decimals: 18
        orders:
            order1:
                rainlang: registry1
                inputs:
                    - token: token1
                outputs:
                    - token: token2
        scenarios:
            scenario1:
                rainlang: registry1
        deployments:
            deployment1:
                order: order1
                scenario: scenario1
        "#,
            SpecVersion::current()
        );

        let mut dotrain_yaml =
            DotrainYaml::new(vec![yaml.to_string()], DotrainYamlValidation::default()).unwrap();

        let remote_network = mock_network();
        let remote_token = mock_token("remote-token");
        dotrain_yaml
            .cache
            .update_remote_network("remote-net".to_string(), remote_network.as_ref().clone());
        dotrain_yaml
            .cache
            .update_remote_token("remote-token".to_string(), remote_token.as_ref().clone());

        let strict_ctx = dotrain_yaml.build_context(&ContextProfile::Strict).unwrap();
        assert!(strict_ctx.get_remote_network("remote-net").is_some());
        assert!(strict_ctx.get_remote_token("remote-token").is_some());
        assert!(strict_ctx.gui_context.is_none());
        assert!(strict_ctx.order.is_none());

        let gui_ctx = dotrain_yaml
            .build_context(&ContextProfile::gui("deployment1".to_string()))
            .unwrap();
        assert!(gui_ctx.get_remote_network("remote-net").is_some());
        assert!(gui_ctx.get_remote_token("remote-token").is_some());
        assert_eq!(
            gui_ctx
                .gui_context
                .as_ref()
                .and_then(|gc| gc.current_order.clone()),
            Some("order1".to_string())
        );
        assert_eq!(
            gui_ctx
                .gui_context
                .as_ref()
                .and_then(|gc| gc.current_deployment.clone()),
            Some("deployment1".to_string())
        );
        assert_eq!(
            gui_ctx.order.as_ref().map(|order| order.key.clone()),
            Some("order1".to_string())
        );

        let propagated = DotrainYaml::from_dotrain_yaml(dotrain_yaml);
        let propagated_ctx = propagated
            .build_context(&ContextProfile::gui("deployment1".to_string()))
            .unwrap();
        assert_eq!(
            propagated_ctx
                .gui_context
                .as_ref()
                .and_then(|gc| gc.current_order.clone()),
            Some("order1".to_string())
        );
        assert_eq!(
            propagated_ctx.order.as_ref().map(|order| order.key.clone()),
            Some("order1".to_string())
        );
    }

    #[test]
    fn test_get_order_uses_profile() {
        let yaml = format!(
            r#"
        version: {}
        networks:
            mainnet:
                rpcs:
                    - https://mainnet.infura.io
                chain-id: 1
        rainlangs:
            registry1:
                address: 0x0000000000000000000000000000000000000002
                network: mainnet
        orders:
            order1:
                rainlang: registry1
                inputs:
                    - token: token1
                outputs:
                    - token: token2
        scenarios:
            scenario1:
                rainlang: registry1
        deployments:
            deployment1:
                order: order1
                scenario: scenario1
        gui:
            deployments:
                deployment1:
                    select-tokens:
                        - key: token1
                        - key: token2
        "#,
            SpecVersion::current()
        );

        let strict_yaml =
            DotrainYaml::new(vec![yaml.to_string()], DotrainYamlValidation::default()).unwrap();
        let err = strict_yaml.get_order("order1").unwrap_err();
        assert_eq!(
            err,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "token".to_string(),
                    reason: "missing yaml data for token 'token1'".to_string()
                },
                location: "input index '0' in order 'order1'".to_string(),
            }
        );

        let gui_yaml = DotrainYaml::new_with_profile(
            vec![yaml.to_string()],
            DotrainYamlValidation::default(),
            ContextProfile::gui("deployment1".to_string()),
        )
        .unwrap();
        let order = gui_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs[0].token, None);
        assert_eq!(order.outputs[0].token, None);
    }

    #[test]
    fn test_dotrain_yaml_serialization_preserves_profile() {
        let yaml = format!("version: {}\norders: {{}}", SpecVersion::current());
        let dotrain_yaml = DotrainYaml::new_with_profile(
            vec![yaml.clone()],
            DotrainYamlValidation::default(),
            ContextProfile::gui("deployment1".to_string()),
        )
        .unwrap();

        let serialized = serde_json::to_string(&dotrain_yaml).unwrap();
        let round_tripped: DotrainYaml = serde_json::from_str(&serialized).unwrap();
        match round_tripped.profile {
            ContextProfile::Gui { current_deployment } => {
                assert_eq!(current_deployment, "deployment1");
            }
            _ => panic!("expected gui profile"),
        }
    }

    #[test]
    fn test_dotrain_yaml_legacy_sequence_deserialization_defaults_profile() {
        let yaml = "orders: {}".to_string();
        let legacy_serialized = serde_json::to_string(&vec![yaml.clone()]).unwrap();

        let deserialized: DotrainYaml = serde_json::from_str(&legacy_serialized).unwrap();
        assert!(matches!(deserialized.profile, ContextProfile::Strict));
        assert_eq!(deserialized.documents.len(), 1);
    }

    #[test]
    fn test_update_vault_ids() {
        let yaml = format!(
            r#"
        version: {version}
        networks:
            mainnet:
                rpcs:
                    - https://mainnet.infura.io
                chain-id: 1
            testnet:
                rpcs:
                    - https://testnet.infura.io
                chain-id: 1337
        tokens:
            token1:
                network: mainnet
                address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
                decimals: 18
                label: Wrapped Ether
                symbol: WETH
            token2:
                network: mainnet
                address: 0x0000000000000000000000000000000000000002
                decimals: 6
                label: USD Coin
                symbol: USDC
        orders:
            order1:
                inputs:
                    - token: token1
                outputs:
                    - token: token2
        "#,
            version = SpecVersion::current()
        );
        let dotrain_yaml = DotrainYaml::new(vec![yaml], DotrainYamlValidation::default()).unwrap();

        let mut order = dotrain_yaml.get_order("order1").unwrap();

        assert!(order.inputs[0].vault_id.is_none());
        assert!(order.outputs[0].vault_id.is_none());

        let updated_order = order.populate_vault_ids().unwrap();

        // After population, all vault IDs should be set and equal
        assert!(updated_order.inputs[0].vault_id.is_some());
        assert!(updated_order.outputs[0].vault_id.is_some());
        assert_eq!(
            updated_order.inputs[0].vault_id,
            updated_order.outputs[0].vault_id
        );

        let order_after = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(
            order_after.inputs[0].vault_id,
            updated_order.inputs[0].vault_id
        );
        assert_eq!(
            order_after.outputs[0].vault_id,
            updated_order.outputs[0].vault_id
        );

        // Populate vault IDs should not change if the vault IDs are already set
        let dotrain_yaml =
            DotrainYaml::new(vec![full_yaml()], DotrainYamlValidation::default()).unwrap();
        let mut order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(2)));
        order.populate_vault_ids().unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(2)));
    }

    #[test]
    fn test_update_vault_id() {
        let yaml = format!(
            r#"
        version: {version}
        networks:
            mainnet:
                rpcs:
                    - https://mainnet.infura.io
                chain-id: 1
            testnet:
                rpcs:
                    - https://testnet.infura.io
                chain-id: 1337
        tokens:
            token1:
                network: mainnet
                address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
                decimals: 18
                label: Wrapped Ether
                symbol: WETH
            token2:
                network: mainnet
                address: 0x0000000000000000000000000000000000000002
                decimals: 6
                label: USD Coin
                symbol: USDC
        orders:
            order1:
                inputs:
                    - token: token1
                outputs:
                    - token: token2
        "#,
            version = SpecVersion::current()
        );
        let dotrain_yaml = DotrainYaml::new(vec![yaml], DotrainYamlValidation::default()).unwrap();
        let mut order = dotrain_yaml.get_order("order1").unwrap();

        assert!(order.inputs[0].vault_id.is_none());
        assert!(order.outputs[0].vault_id.is_none());

        let mut updated_order = order
            .update_vault_id(
                VaultType::Input,
                "token1".to_string(),
                Some("1".to_string()),
            )
            .unwrap();
        let updated_order = updated_order
            .update_vault_id(
                VaultType::Output,
                "token2".to_string(),
                Some("11".to_string()),
            )
            .unwrap();

        assert_eq!(updated_order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(updated_order.outputs[0].vault_id, Some(U256::from(11)));

        let mut order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(1)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(11)));

        let mut updated_order = order
            .update_vault_id(
                VaultType::Input,
                "token1".to_string(),
                Some("3".to_string()),
            )
            .unwrap();
        let updated_order = updated_order
            .update_vault_id(
                VaultType::Output,
                "token2".to_string(),
                Some("33".to_string()),
            )
            .unwrap();
        assert_eq!(updated_order.inputs[0].vault_id, Some(U256::from(3)));
        assert_eq!(updated_order.outputs[0].vault_id, Some(U256::from(33)));

        let order = dotrain_yaml.get_order("order1").unwrap();
        assert_eq!(order.inputs[0].vault_id, Some(U256::from(3)));
        assert_eq!(order.outputs[0].vault_id, Some(U256::from(33)));
    }

    #[test]
    fn test_update_bindings() {
        // Parent scenario
        {
            let dotrain_yaml =
                DotrainYaml::new(vec![full_yaml()], DotrainYamlValidation::default()).unwrap();

            let mut scenario = dotrain_yaml.get_scenario("scenario1").unwrap();

            assert_eq!(scenario.bindings.len(), 1);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value1");

            let updated_scenario = scenario
                .update_bindings(HashMap::from([("key1".to_string(), "value2".to_string())]))
                .unwrap();

            assert_eq!(updated_scenario.bindings.len(), 1);
            assert_eq!(updated_scenario.bindings.get("key1").unwrap(), "value2");

            let scenario = dotrain_yaml.get_scenario("scenario1").unwrap();
            assert_eq!(scenario.bindings.len(), 1);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value2");
        }

        // Child scenario
        {
            let dotrain_yaml =
                DotrainYaml::new(vec![full_yaml()], DotrainYamlValidation::default()).unwrap();

            let mut scenario = dotrain_yaml.get_scenario("scenario1.scenario2").unwrap();

            assert_eq!(scenario.bindings.len(), 2);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value1");
            assert_eq!(scenario.bindings.get("key2").unwrap(), "value2");

            let updated_scenario = scenario
                .update_bindings(HashMap::from([
                    ("key1".to_string(), "value3".to_string()),
                    ("key2".to_string(), "value4".to_string()),
                ]))
                .unwrap();

            assert_eq!(updated_scenario.bindings.len(), 2);
            assert_eq!(updated_scenario.bindings.get("key1").unwrap(), "value3");
            assert_eq!(updated_scenario.bindings.get("key2").unwrap(), "value4");

            let scenario = dotrain_yaml.get_scenario("scenario1.scenario2").unwrap();
            assert_eq!(scenario.bindings.len(), 2);
            assert_eq!(scenario.bindings.get("key1").unwrap(), "value3");
            assert_eq!(scenario.bindings.get("key2").unwrap(), "value4");
        }

        // Adding additional bindings
        {
            let dotrain_yaml =
                DotrainYaml::new(vec![full_yaml()], DotrainYamlValidation::default()).unwrap();

            let mut scenario = dotrain_yaml.get_scenario("scenario1.scenario2").unwrap();
            let updated_scenario = scenario
                .update_bindings(HashMap::from([
                    ("key3".to_string(), "value3".to_string()),
                    ("key4".to_string(), "value4".to_string()),
                ]))
                .unwrap();

            assert_eq!(updated_scenario.bindings.len(), 4);
            assert_eq!(updated_scenario.bindings.get("key3").unwrap(), "value3");
            assert_eq!(updated_scenario.bindings.get("key4").unwrap(), "value4");
        }
    }

    #[test]
    fn test_handlebars() {
        let dotrain_yaml =
            DotrainYaml::new(vec![handlebars_yaml()], DotrainYamlValidation::default()).unwrap();

        let gui = dotrain_yaml.get_gui("deployment1").unwrap().unwrap();
        let deployment = gui.deployments.get("deployment1").unwrap();

        assert_eq!(
            deployment.deployment.scenario.bindings.get("key1").unwrap(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
        assert_eq!(
            deployment.deployment.scenario.bindings.get("key2").unwrap(),
            "0x0000000000000000000000000000000000000002"
        );

        assert_eq!(deployment.fields[0].name, "Binding for Wrapped Ether");
        assert_eq!(
            deployment.fields[0].description,
            Some("With token symbol WETH".to_string())
        );
    }

    #[test]
    fn test_parse_orders_missing_token() {
        let yaml_prefix = format!(
            r#"
version: {version}
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
rainlangs:
    mainnet:
        address: 0x0000000000000000000000000000000000000001
        network: mainnet
scenarios:
    scenario1:
        rainlang: mainnet
        bindings:
            key1: value1
deployments:
    deployment1:
        order: order1
        scenario: scenario1
gui:
    name: test
    description: test
    deployments:
        deployment1:
            name: test
            description: test
            deposits:
                - token: token-one
                  presets:
                    - 1
                - token: token-two
                  presets:
                    - 1
                - token: token-three
                  presets:
                    - 1
            fields:
                - binding: key1
                  name: test
                  presets:
                    - value: 1
            select-tokens:
                - key: token-one
                - key: token-two
"#,
            version = SpecVersion::current()
        );
        let missing_input_token_yaml = format!(
            "{yaml_prefix}
orders:
    order1:
        rainlang: mainnet
        inputs:
            - token: token-three
        outputs:
            - token: token-two
            - token: token-three
        "
        );
        let missing_output_token_yaml = format!(
            "{yaml_prefix}
orders:
    order1:
        rainlang: mainnet
        inputs:
            - token: token-one
            - token: token-two
        outputs:
            - token: token-three
        "
        );

        let dotrain_yaml = DotrainYaml::new(
            vec![missing_input_token_yaml],
            DotrainYamlValidation::default(),
        )
        .unwrap();
        let error = dotrain_yaml.get_gui("deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "token".to_string(),
                    reason: "missing yaml data for token 'token-three'".to_string(),
                },
                location: "input index '0' in order 'order1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Invalid value for field 'token' in input index '0' in order 'order1': missing yaml data for token 'token-three'"
        );

        let dotrain_yaml = DotrainYaml::new(
            vec![missing_output_token_yaml],
            DotrainYamlValidation::default(),
        )
        .unwrap();
        let error = dotrain_yaml.get_gui("deployment1").unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "token".to_string(),
                    reason: "missing yaml data for token 'token-three'".to_string(),
                },
                location: "output index '0' in order 'order1'".to_string(),
            }
        );
        assert_eq!(
            error.to_readable_msg(),
            "Invalid value for field 'token' in output index '0' in order 'order1': missing yaml data for token 'token-three'"
        );
    }
}
