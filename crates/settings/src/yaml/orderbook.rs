use super::{cache::Cache, sanitize_all_documents, ValidationConfig, *};
use crate::{
    accounts::AccountCfg, local_db_remotes::LocalDbRemoteCfg, local_db_sync::LocalDbSyncCfg,
    metaboard::MetaboardCfg, remote_networks::RemoteNetworksCfg, remote_tokens::RemoteTokensCfg,
    sentry::Sentry, spec_version::SpecVersion, subgraph::SubgraphCfg, NetworkCfg, OrderbookCfg,
    RainlangCfg, TokenCfg,
};
use alloy::primitives::Address;
use serde::{
    de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
    Deserialize,
};
use std::{
    fmt,
    sync::{Arc, RwLock},
};
use strict_yaml_rust::{StrictYaml, StrictYamlLoader};
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*};

#[derive(Debug, Clone, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct OrderbookYaml {
    #[cfg_attr(target_family = "wasm", tsify(type = "string[]"))]
    pub documents: Vec<Arc<RwLock<StrictYaml>>>,
    pub cache: Cache,
    pub profile: ContextProfile,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(OrderbookYaml);

#[derive(Debug, Clone, Default)]
pub struct OrderbookYamlValidation {
    pub networks: bool,
    pub remote_networks: bool,
    pub tokens: bool,
    pub remote_tokens: bool,
    pub subgraphs: bool,
    pub local_db_remotes: bool,
    pub local_db_sync: bool,
    pub orderbooks: bool,
    pub metaboards: bool,
    pub rainlangs: bool,
}
impl OrderbookYamlValidation {
    pub fn full() -> Self {
        OrderbookYamlValidation {
            networks: true,
            remote_networks: true,
            tokens: true,
            remote_tokens: true,
            subgraphs: true,
            local_db_remotes: true,
            local_db_sync: true,
            orderbooks: true,
            metaboards: true,
            rainlangs: true,
        }
    }
}
impl ValidationConfig for OrderbookYamlValidation {
    fn should_validate_networks(&self) -> bool {
        self.networks
    }
    fn should_validate_remote_networks(&self) -> bool {
        self.remote_networks
    }
    fn should_validate_tokens(&self) -> bool {
        self.tokens
    }
    fn should_validate_remote_tokens(&self) -> bool {
        self.remote_tokens
    }
    fn should_validate_subgraphs(&self) -> bool {
        self.subgraphs
    }
    fn should_validate_local_db_remotes(&self) -> bool {
        self.local_db_remotes
    }
    fn should_validate_local_db_sync(&self) -> bool {
        self.local_db_sync
    }
    fn should_validate_orderbooks(&self) -> bool {
        self.orderbooks
    }
    fn should_validate_metaboards(&self) -> bool {
        self.metaboards
    }
    fn should_validate_rainlangs(&self) -> bool {
        self.rainlangs
    }
    fn should_validate_orders(&self) -> bool {
        false
    }
    fn should_validate_scenarios(&self) -> bool {
        false
    }
    fn should_validate_deployments(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Default)]
pub struct FetchedRemoteData {
    pub remote_networks: HashMap<String, NetworkCfg>,
    pub remote_tokens: HashMap<String, TokenCfg>,
}

impl YamlParsable for OrderbookYaml {
    type ValidationConfig = OrderbookYamlValidation;

    fn new(sources: Vec<String>, validate: OrderbookYamlValidation) -> Result<Self, YamlError> {
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

        if validate.should_validate_networks() {
            NetworkCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_remote_networks() {
            RemoteNetworksCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_tokens() {
            TokenCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_remote_tokens() {
            RemoteTokensCfg::parse_from_yaml_optional(documents.clone(), None)?;
        }
        if validate.should_validate_subgraphs() {
            SubgraphCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_local_db_remotes() {
            LocalDbRemoteCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_local_db_sync() {
            LocalDbSyncCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_orderbooks() {
            OrderbookCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_metaboards() {
            MetaboardCfg::parse_all_from_yaml(documents.clone(), None)?;
        }
        if validate.should_validate_rainlangs() {
            RainlangCfg::parse_all_from_yaml(documents.clone(), None)?;
        }

        Ok(OrderbookYaml {
            documents,
            cache: Cache::default(),
            profile: ContextProfile::Strict,
        })
    }

    fn from_documents(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Self {
        OrderbookYaml {
            documents,
            cache: Cache::default(),
            profile: ContextProfile::Strict,
        }
    }

    fn from_orderbook_yaml(orderbook_yaml: OrderbookYaml) -> Self {
        OrderbookYaml {
            documents: orderbook_yaml.documents,
            cache: orderbook_yaml.cache,
            profile: orderbook_yaml.profile,
        }
    }

    fn from_dotrain_yaml(dotrain_yaml: DotrainYaml) -> Self {
        OrderbookYaml {
            documents: dotrain_yaml.documents,
            cache: dotrain_yaml.cache,
            profile: dotrain_yaml.profile,
        }
    }
}

impl ContextProvider for OrderbookYaml {
    fn get_remote_networks_from_cache(&self) -> HashMap<String, NetworkCfg> {
        self.cache.get_remote_networks()
    }

    fn get_remote_tokens_from_cache(&self) -> HashMap<String, TokenCfg> {
        self.cache.get_remote_tokens()
    }
}

impl OrderbookYaml {
    pub fn new_with_profile(
        sources: Vec<String>,
        validate: OrderbookYamlValidation,
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

    pub fn build_context(&self) -> Context {
        let mut context = self.create_context();
        self.expand_context_with_remote_networks(&mut context);
        self.expand_context_with_remote_tokens(&mut context);
        context
    }

    pub fn get_network_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_networks()?.keys().cloned().collect())
    }
    pub fn get_networks(&self) -> Result<HashMap<String, NetworkCfg>, YamlError> {
        let context = self.build_context();
        NetworkCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_network(&self, key: &str) -> Result<NetworkCfg, YamlError> {
        let context = self.build_context();
        NetworkCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }
    pub fn get_network_by_chain_id(&self, chain_id: u32) -> Result<NetworkCfg, YamlError> {
        let networks = self.get_networks()?;
        for network in networks.values() {
            if network.chain_id == chain_id {
                return Ok(network.clone());
            }
        }
        Err(YamlError::NotFound(format!(
            "network with chain-id: {}",
            chain_id
        )))
    }

    pub fn get_remote_networks(&self) -> Result<HashMap<String, RemoteNetworksCfg>, YamlError> {
        let context = self.build_context();
        RemoteNetworksCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }

    pub fn get_token_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_tokens()?.keys().cloned().collect())
    }
    pub fn get_tokens(&self) -> Result<HashMap<String, TokenCfg>, YamlError> {
        let context = self.build_context();
        TokenCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_token(&self, key: &str) -> Result<TokenCfg, YamlError> {
        let context = self.build_context();
        TokenCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }

    pub fn get_remote_tokens(&self) -> Result<Option<RemoteTokensCfg>, YamlError> {
        let context = self.build_context();
        RemoteTokensCfg::parse_from_yaml_optional(self.documents.clone(), Some(&context))
    }

    pub async fn fetch_remote_data(&mut self) -> Result<FetchedRemoteData, YamlError> {
        let remote_networks =
            RemoteNetworksCfg::fetch_networks(self.get_remote_networks()?).await?;
        if !remote_networks.is_empty() {
            self.cache.update_remote_networks(remote_networks.clone());
        }
        let remote_tokens = if let Some(remote_tokens_cfg) = self.get_remote_tokens()? {
            let networks = self.get_networks()?;
            let tokens = RemoteTokensCfg::fetch_tokens(&networks, remote_tokens_cfg).await?;
            self.cache.update_remote_tokens(tokens.clone());
            tokens
        } else {
            HashMap::new()
        };
        Ok(FetchedRemoteData {
            remote_networks,
            remote_tokens,
        })
    }

    pub fn get_subgraph_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_subgraphs()?.keys().cloned().collect())
    }
    pub fn get_subgraphs(&self) -> Result<HashMap<String, SubgraphCfg>, YamlError> {
        let context = self.build_context();
        SubgraphCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_subgraph(&self, key: &str) -> Result<SubgraphCfg, YamlError> {
        let context = self.build_context();
        SubgraphCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }

    pub fn get_local_db_remote_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_local_db_remotes()?.keys().cloned().collect())
    }
    pub fn get_local_db_remotes(&self) -> Result<HashMap<String, LocalDbRemoteCfg>, YamlError> {
        let context = self.build_context();
        LocalDbRemoteCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_local_db_remote(&self, key: &str) -> Result<LocalDbRemoteCfg, YamlError> {
        let context = self.build_context();
        LocalDbRemoteCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }

    pub fn get_local_db_sync_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_local_db_syncs()?.keys().cloned().collect())
    }
    pub fn get_local_db_syncs(&self) -> Result<HashMap<String, LocalDbSyncCfg>, YamlError> {
        let context = self.build_context();
        LocalDbSyncCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_local_db_sync(&self, key: &str) -> Result<LocalDbSyncCfg, YamlError> {
        let context = self.build_context();
        LocalDbSyncCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }

    pub fn get_orderbook_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_orderbooks()?.keys().cloned().collect())
    }
    pub fn get_orderbooks(&self) -> Result<HashMap<String, OrderbookCfg>, YamlError> {
        let context = self.build_context();
        OrderbookCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_orderbook(&self, key: &str) -> Result<OrderbookCfg, YamlError> {
        let context = self.build_context();
        OrderbookCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }
    pub fn get_orderbook_by_address(&self, address: Address) -> Result<OrderbookCfg, YamlError> {
        let context = self.build_context();
        let orderbooks = OrderbookCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))?;
        for (_, orderbook) in orderbooks {
            if orderbook.address == address {
                return Ok(orderbook);
            }
        }
        Err(YamlError::NotFound(format!(
            "orderbook with address: {}",
            address
        )))
    }
    pub fn get_orderbooks_by_network_key(
        &self,
        network_key: &str,
    ) -> Result<Vec<OrderbookCfg>, YamlError> {
        let mut orderbooks: Vec<_> = self
            .get_orderbooks()?
            .into_iter()
            .filter(|(_, ob)| ob.network.key == network_key)
            .map(|(_, ob)| ob)
            .collect();
        orderbooks.sort_by(|a, b| a.key.cmp(&b.key));

        if orderbooks.is_empty() {
            return Err(YamlError::NotFound(format!(
                "orderbook with network key: {}",
                network_key
            )));
        }
        Ok(orderbooks)
    }

    pub fn get_orderbooks_by_chain_id(
        &self,
        chain_id: u32,
    ) -> Result<Vec<OrderbookCfg>, YamlError> {
        let network = self.get_network_by_chain_id(chain_id)?;
        let mut orderbooks: Vec<_> = self
            .get_orderbooks()?
            .into_iter()
            .filter(|(_, ob)| ob.network.key == network.key)
            .map(|(_, ob)| ob)
            .collect();
        orderbooks.sort_by(|a, b| a.key.cmp(&b.key));

        if orderbooks.is_empty() {
            return Err(YamlError::NotFound(format!(
                "orderbook with chain-id: {}",
                chain_id
            )));
        }
        Ok(orderbooks)
    }

    pub fn get_metaboard_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_metaboards()?.keys().cloned().collect())
    }
    pub fn get_metaboards(&self) -> Result<HashMap<String, MetaboardCfg>, YamlError> {
        let context = self.build_context();
        MetaboardCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_metaboard(&self, key: &str) -> Result<MetaboardCfg, YamlError> {
        let context = self.build_context();
        MetaboardCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }
    pub fn add_metaboard(&self, key: &str, value: &str) -> Result<(), YamlError> {
        MetaboardCfg::add_record_to_yaml(self.documents[0].clone(), key, value)
    }

    pub fn get_rainlang_keys(&self) -> Result<Vec<String>, YamlError> {
        Ok(self.get_rainlangs()?.keys().cloned().collect())
    }
    pub fn get_rainlangs(&self) -> Result<HashMap<String, RainlangCfg>, YamlError> {
        let context = self.build_context();
        RainlangCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_rainlang(&self, key: &str) -> Result<RainlangCfg, YamlError> {
        let context = self.build_context();
        RainlangCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }

    pub fn get_sentry(&self) -> Result<Option<bool>, YamlError> {
        let value_opt_str = Sentry::parse_from_yaml_optional(self.documents[0].clone())?;

        let res = value_opt_str
            .map(|v| v.to_ascii_lowercase())
            .map(|v| match v.as_str() {
                "true" | "1" => Ok(true),
                "false" | "0" => Ok(false),
                _ => Err(YamlError::Field {
                    kind: FieldErrorKind::InvalidType {
                        field: "sentry".to_string(),
                        expected: "a boolean".to_string(),
                    },
                    location: "root".to_string(),
                }),
            });

        res.transpose()
    }

    pub fn get_spec_version(&self) -> Result<String, YamlError> {
        SpecVersion::parse_from_yaml(self.documents.clone())
    }

    pub fn get_account_keys(&self) -> Result<Vec<String>, YamlError> {
        let accounts = self.get_accounts()?;
        Ok(accounts.keys().cloned().collect())
    }
    pub fn get_accounts(&self) -> Result<HashMap<String, AccountCfg>, YamlError> {
        let context = self.build_context();
        AccountCfg::parse_all_from_yaml(self.documents.clone(), Some(&context))
    }
    pub fn get_account(&self, key: &str) -> Result<AccountCfg, YamlError> {
        let context = self.build_context();
        AccountCfg::parse_from_yaml(self.documents.clone(), key, Some(&context))
    }
}

impl Serialize for OrderbookYaml {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut documents = Vec::with_capacity(self.documents.len());
        for doc in &self.documents {
            let yaml_str = Self::get_yaml_string(doc.clone()).map_err(serde::ser::Error::custom)?;
            documents.push(yaml_str);
        }

        let mut state = serializer.serialize_struct("OrderbookYaml", 2)?;
        state.serialize_field("documents", &documents)?;
        state.serialize_field("profile", &self.profile)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for OrderbookYaml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderbookYamlVisitor;

        impl<'de> Visitor<'de> for OrderbookYamlVisitor {
            type Value = OrderbookYaml;

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

                Ok(OrderbookYaml {
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

                Ok(OrderbookYaml {
                    documents,
                    cache: Cache::default(),
                    profile: ContextProfile::Strict,
                })
            }
        }

        deserializer.deserialize_any(OrderbookYamlVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_version::SpecVersion;
    use alloy::primitives::Address;

    use std::str::FromStr;
    use url::Url;

    #[test]
    fn test_orderbook_yaml_profile_helpers() {
        let sources = vec![full_yaml()];
        let ob = OrderbookYaml::new_with_profile(
            sources.clone(),
            OrderbookYamlValidation::default(),
            ContextProfile::Gui {
                current_deployment: "deployment1".to_string(),
            },
        )
        .unwrap();
        assert!(matches!(ob.profile, ContextProfile::Gui { .. }));

        let ob_default = OrderbookYaml::new(sources, OrderbookYamlValidation::default()).unwrap();
        assert!(matches!(ob_default.profile, ContextProfile::Strict));

        let ob_strict = ob.with_profile(ContextProfile::Strict);
        assert!(matches!(ob_strict.profile, ContextProfile::Strict));
    }

    #[test]
    fn test_orderbook_yaml_serialization_preserves_profile() {
        let ob = OrderbookYaml::new_with_profile(
            vec![full_yaml()],
            OrderbookYamlValidation::default(),
            ContextProfile::Gui {
                current_deployment: "deployment1".to_string(),
            },
        )
        .unwrap();

        let serialized = serde_json::to_string(&ob).unwrap();
        let round_tripped: OrderbookYaml = serde_json::from_str(&serialized).unwrap();
        match round_tripped.profile {
            ContextProfile::Gui { current_deployment } => {
                assert_eq!(current_deployment, "deployment1");
            }
            _ => panic!("expected gui profile"),
        }
    }

    #[test]
    fn test_orderbook_yaml_legacy_sequence_deserialization_defaults_profile() {
        let legacy_serialized = serde_json::to_string(&vec![full_yaml()]).unwrap();
        let deserialized: OrderbookYaml = serde_json::from_str(&legacy_serialized).unwrap();

        assert!(matches!(deserialized.profile, ContextProfile::Strict));
        assert_eq!(deserialized.documents.len(), 1);
    }

    fn full_yaml() -> String {
        format!(
            r#"
    version: {version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io/1
                - https://mainnet.infura.io/2
                - https://mainnet.infura.io/3
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
    using-networks-from:
        chainid:
            url: https://chainid.network/v2/chains.json
            format: chainid
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
        secondary: https://api.thegraph.com/subgraphs/name/abc
    local-db-remotes:
        mainnet: https://example.com/localdb/mainnet
    metaboards:
        board1: https://meta.example.com/board1
        board2: https://meta.example.com/board2
    orderbooks:
        orderbook1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
            subgraph: mainnet
            local-db-remote: mainnet
            label: Primary Orderbook
            deployment-block: 12345
    tokens:
        token1:
            network: mainnet
            address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
            decimals: 18
            label: Wrapped Ether
            symbol: WETH
    rainlangs:
        registry1:
            address: 0x0000000000000000000000000000000000000002
            network: mainnet
    accounts:
        admin: 0x0000000000000000000000000000000000000001
        user: 0x0000000000000000000000000000000000000002
    sentry: true
    "#,
            version = SpecVersion::current()
        )
    }

    const _YAML_WITHOUT_OPTIONAL_FIELDS: &str = r#"
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
    metaboards:
        board1: https://meta.example.com/board1
    orderbooks:
        orderbook1:
            address: 0x1234567890abcdef
            deployment-block: 12345
    tokens:
        token1:
            network: mainnet
            address: 0x2345678901abcdef
    rainlangs:
        registry1:
            address: 0x3456789012abcdef
    "#;

    #[test]
    fn test_full_yaml() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        assert_eq!(ob_yaml.get_network_keys().unwrap().len(), 1);
        let network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpcs,
            vec![
                Url::parse("https://mainnet.infura.io/1").unwrap(),
                Url::parse("https://mainnet.infura.io/2").unwrap(),
                Url::parse("https://mainnet.infura.io/3").unwrap(),
            ]
        );
        assert_eq!(network.chain_id, 1);
        assert_eq!(network.label, Some("Ethereum Mainnet".to_string()));
        assert_eq!(network.network_id, Some(1));
        assert_eq!(network.currency, Some("ETH".to_string()));
        assert_eq!(
            NetworkCfg::parse_rpcs(ob_yaml.documents.clone(), "mainnet").unwrap(),
            vec![
                Url::parse("https://mainnet.infura.io/1").unwrap(),
                Url::parse("https://mainnet.infura.io/2").unwrap(),
                Url::parse("https://mainnet.infura.io/3").unwrap(),
            ]
        );

        let remote_networks = ob_yaml.get_remote_networks().unwrap();
        assert_eq!(remote_networks.len(), 1);
        assert_eq!(
            remote_networks.get("chainid").unwrap().url,
            Url::parse("https://chainid.network/v2/chains.json").unwrap()
        );

        assert_eq!(ob_yaml.get_tokens().unwrap().len(), 1);
        assert_eq!(ob_yaml.get_token_keys().unwrap().len(), 1);
        let token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );
        assert_eq!(token.decimals, Some(18));
        assert_eq!(token.label, Some("Wrapped Ether".to_string()));
        assert_eq!(token.symbol, Some("WETH".to_string()));
        assert_eq!(
            TokenCfg::parse_network_key(ob_yaml.documents.clone(), "token1").unwrap(),
            "mainnet"
        );

        assert_eq!(ob_yaml.get_subgraph_keys().unwrap().len(), 2);
        let subgraph = ob_yaml.get_subgraph("mainnet").unwrap();
        assert_eq!(
            subgraph.url,
            Url::parse("https://api.thegraph.com/subgraphs/name/xyz").unwrap()
        );

        assert_eq!(ob_yaml.get_orderbook_keys().unwrap().len(), 1);
        let orderbook = ob_yaml.get_orderbook("orderbook1").unwrap();
        assert_eq!(
            orderbook.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(orderbook.network, network.clone().into());
        assert_eq!(orderbook.subgraph, subgraph.into());
        assert_eq!(orderbook.label, Some("Primary Orderbook".to_string()));
        assert_eq!(
            OrderbookCfg::parse_network_key(ob_yaml.documents.clone(), "orderbook1").unwrap(),
            "mainnet"
        );
        let orderbook_by_address = ob_yaml
            .get_orderbook_by_address(
                Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
            )
            .unwrap();
        assert_eq!(orderbook_by_address, orderbook);

        assert_eq!(ob_yaml.get_metaboard_keys().unwrap().len(), 2);
        assert_eq!(
            ob_yaml.get_metaboard("board1").unwrap().url,
            Url::parse("https://meta.example.com/board1").unwrap()
        );
        assert_eq!(
            ob_yaml.get_metaboard("board2").unwrap().url,
            Url::parse("https://meta.example.com/board2").unwrap()
        );

        assert_eq!(ob_yaml.get_rainlang_keys().unwrap().len(), 1);
        let rainlang_cfg = ob_yaml.get_rainlang("registry1").unwrap();
        assert_eq!(
            rainlang_cfg.address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
        assert_eq!(rainlang_cfg.network, network.into());
        assert_eq!(
            RainlangCfg::parse_network_key(ob_yaml.documents.clone(), "registry1").unwrap(),
            "mainnet"
        );

        assert_eq!(ob_yaml.get_sentry().unwrap(), Some(true));

        assert_eq!(ob_yaml.get_spec_version().unwrap(), SpecVersion::current());

        assert_eq!(ob_yaml.get_account_keys().unwrap().len(), 2);
        assert_eq!(
            ob_yaml.get_account("admin").unwrap().address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        assert_eq!(
            ob_yaml.get_account("user").unwrap().address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );
    }

    #[test]
    fn test_update_network_rpc() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        let mut network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpcs,
            vec![
                Url::parse("https://mainnet.infura.io/1").unwrap(),
                Url::parse("https://mainnet.infura.io/2").unwrap(),
                Url::parse("https://mainnet.infura.io/3").unwrap(),
            ]
        );

        let network = network
            .update_rpcs(vec![
                "https://some-random-rpc-address.com".to_string(),
                "https://some-other-random-rpc-address.com".to_string(),
            ])
            .unwrap();
        assert_eq!(
            network.rpcs,
            vec![
                Url::parse("https://some-random-rpc-address.com").unwrap(),
                Url::parse("https://some-other-random-rpc-address.com").unwrap(),
            ]
        );

        let network = ob_yaml.get_network("mainnet").unwrap();
        assert_eq!(
            network.rpcs,
            vec![
                Url::parse("https://some-random-rpc-address.com").unwrap(),
                Url::parse("https://some-other-random-rpc-address.com").unwrap(),
            ]
        );
    }

    #[test]
    fn test_update_token_address() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        let mut token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
        );

        let token = token
            .update_address("0x0000000000000000000000000000000000000001")
            .unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );

        let token = ob_yaml.get_token("token1").unwrap();
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
    }

    #[test]
    fn test_add_token_to_yaml() {
        let yaml = format!(
            r#"
version: {version}
networks:
    mainnet:
        rpcs:
            - "https://mainnet.infura.io"
        chain-id: "1"
"#,
            version = SpecVersion::current()
        );
        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        TokenCfg::add_record_to_yaml(
            ob_yaml.documents.clone(),
            "test-token",
            "mainnet",
            "0x0000000000000000000000000000000000000001",
            Some("18"),
            Some("Test Token"),
            Some("TTK"),
        )
        .unwrap();

        let token = ob_yaml.get_token("test-token").unwrap();
        assert_eq!(token.key, "test-token");
        assert_eq!(token.network.key, "mainnet");
        assert_eq!(
            token.address,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        assert_eq!(token.decimals, Some(18));
        assert_eq!(token.label, Some("Test Token".to_string()));
        assert_eq!(token.symbol, Some("TTK".to_string()));
    }

    #[test]
    fn test_remove_token_from_yaml() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        assert!(ob_yaml.get_token("token1").is_ok());
        TokenCfg::remove_record_from_yaml(ob_yaml.documents.clone(), "token1").unwrap();
        assert!(ob_yaml.get_token("token1").is_err());
    }

    #[test]
    fn test_add_metaboard_to_yaml() {
        let yaml = format!(
            r#"
version: {version}
test: test
"#,
            version = SpecVersion::current()
        );
        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        ob_yaml
            .add_metaboard("test-metaboard", "https://test-metaboard.com")
            .unwrap();

        assert_eq!(
            ob_yaml.get_metaboard_keys().unwrap(),
            vec!["test-metaboard".to_string()]
        );
        assert_eq!(
            ob_yaml.get_metaboard("test-metaboard").unwrap().url,
            Url::parse("https://test-metaboard.com").unwrap()
        );
    }

    #[test]
    fn test_get_network_by_chain_id() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        // Test successful lookup
        let network = ob_yaml.get_network_by_chain_id(1).unwrap();
        assert_eq!(network.key, "mainnet");
        assert_eq!(network.chain_id, 1);
        assert_eq!(
            network.rpcs,
            vec![
                Url::parse("https://mainnet.infura.io/1").unwrap(),
                Url::parse("https://mainnet.infura.io/2").unwrap(),
                Url::parse("https://mainnet.infura.io/3").unwrap(),
            ]
        );

        // Test error case - chain ID not found
        let error = ob_yaml.get_network_by_chain_id(999).unwrap_err();
        assert_eq!(
            error,
            YamlError::NotFound("network with chain-id: 999".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The requested item \"network with chain-id: 999\" could not be found in the YAML configuration."
        );
    }

    #[test]
    fn test_get_orderbook_by_network_key() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        // Test successful lookup
        let orderbooks = ob_yaml.get_orderbooks_by_network_key("mainnet").unwrap();
        assert_eq!(orderbooks.len(), 1);
        assert_eq!(orderbooks[0].key, "orderbook1");
        assert_eq!(orderbooks[0].network.key, "mainnet");
        assert_eq!(
            orderbooks[0].address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );

        // Test error case - network key not found
        let error = ob_yaml
            .get_orderbooks_by_network_key("nonexistent")
            .unwrap_err();
        assert_eq!(
            error,
            YamlError::NotFound("orderbook with network key: nonexistent".to_string())
        );
        assert_eq!(
            error.to_readable_msg(),
            "The requested item \"orderbook with network key: nonexistent\" could not be found in the YAML configuration."
        );
    }

    #[test]
    fn test_get_network_by_chain_id_with_multiple_networks() {
        let yaml = format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
            label: Ethereum Mainnet
            network-id: 1
            currency: ETH
        polygon:
            rpcs:
                - https://polygon-rpc.com
            chain-id: 137
            label: Polygon Mainnet
            network-id: 137
            currency: MATIC
        arbitrum:
            rpcs:
                - https://arb1.arbitrum.io
            chain-id: 42161
            label: Arbitrum One
            network-id: 42161
            currency: ETH
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
    orderbooks:
        mainnet-orderbook:
            address: 0x1234567890123456789012345678901234567890
            network: mainnet
            subgraph: mainnet
            local-db-remote: mainnet
            deployment-block: 12345
        other-orderbook:
            address: 0x1234567890123456789012345678901234567891
            network: mainnet
            subgraph: mainnet
            local-db-remote: mainnet
            deployment-block: 12345
        polygon-orderbook:
            address: 0x0987654321098765432109876543210987654321
            network: polygon
            deployment-block: 12345
            subgraph: mainnet
            local-db-remote: mainnet
    "#,
            spec_version = SpecVersion::current()
        );

        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        // Test each network
        let mainnet = ob_yaml.get_network_by_chain_id(1).unwrap();
        assert_eq!(mainnet.key, "mainnet");
        assert_eq!(mainnet.chain_id, 1);

        let polygon = ob_yaml.get_network_by_chain_id(137).unwrap();
        assert_eq!(polygon.key, "polygon");
        assert_eq!(polygon.chain_id, 137);

        let arbitrum = ob_yaml.get_network_by_chain_id(42161).unwrap();
        assert_eq!(arbitrum.key, "arbitrum");
        assert_eq!(arbitrum.chain_id, 42161);

        // Test orderbook lookup by network key
        let orderbooks = ob_yaml.get_orderbooks_by_network_key("mainnet").unwrap();
        assert_eq!(orderbooks.len(), 2);
        assert_eq!(orderbooks[0].key, "mainnet-orderbook");
        assert_eq!(orderbooks[0].network.key, "mainnet");
        assert_eq!(orderbooks[1].key, "other-orderbook");
        assert_eq!(orderbooks[1].network.key, "mainnet");

        let orderbooks = ob_yaml.get_orderbooks_by_network_key("polygon").unwrap();
        assert_eq!(orderbooks.len(), 1);
        assert_eq!(orderbooks[0].key, "polygon-orderbook");
        assert_eq!(orderbooks[0].network.key, "polygon");

        // Test error for network without orderbook
        let error = ob_yaml
            .get_orderbooks_by_network_key("arbitrum")
            .unwrap_err();
        assert_eq!(
            error,
            YamlError::NotFound("orderbook with network key: arbitrum".to_string())
        );
    }

    #[test]
    fn test_get_orderbooks_by_chain_id_single_network() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        let orderbooks = ob_yaml.get_orderbooks_by_chain_id(1).unwrap();
        assert_eq!(orderbooks.len(), 1);
        assert_eq!(orderbooks[0].key, "orderbook1");
        assert_eq!(orderbooks[0].network.key, "mainnet");
        assert_eq!(
            orderbooks[0].address,
            Address::from_str("0x0000000000000000000000000000000000000002").unwrap()
        );

        let err = ob_yaml.get_orderbooks_by_chain_id(999).unwrap_err();
        assert_eq!(
            err,
            YamlError::NotFound("network with chain-id: 999".to_string())
        );
        assert_eq!(
            err.to_readable_msg(),
            "The requested item \"network with chain-id: 999\" could not be found in the YAML configuration."
        );
    }

    #[test]
    fn test_get_orderbooks_by_chain_id_multiple_networks() {
        let yaml = format!(
            r#"
    version: {spec_version}
    networks:
        mainnet:
            rpcs:
                - https://mainnet.infura.io
            chain-id: 1
        polygon:
            rpcs:
                - https://polygon-rpc.com
            chain-id: 137
        arbitrum:
            rpcs:
                - https://arb1.arbitrum.io
            chain-id: 42161
    subgraphs:
        mainnet: https://api.thegraph.com/subgraphs/name/xyz
    orderbooks:
        mainnet-orderbook:
            address: 0x1234567890123456789012345678901234567890
            network: mainnet
            subgraph: mainnet
            local-db-remote: mainnet
            deployment-block: 12345
        other-orderbook:
            address: 0x1234567890123456789012345678901234567891
            network: mainnet
            subgraph: mainnet
            local-db-remote: mainnet
            deployment-block: 12345
        polygon-orderbook:
            address: 0x0987654321098765432109876543210987654321
            network: polygon
            deployment-block: 12345
            subgraph: mainnet
            local-db-remote: mainnet
    "#,
            spec_version = SpecVersion::current()
        );

        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        // mainnet chain id
        let orderbooks = ob_yaml.get_orderbooks_by_chain_id(1).unwrap();
        assert_eq!(orderbooks.len(), 2);
        assert_eq!(orderbooks[0].network.key, "mainnet");
        assert_eq!(orderbooks[1].network.key, "mainnet");

        // polygon chain id
        let orderbooks = ob_yaml.get_orderbooks_by_chain_id(137).unwrap();
        assert_eq!(orderbooks.len(), 1);
        assert_eq!(orderbooks[0].network.key, "polygon");

        // arbitrum chain id has no orderbooks
        let err = ob_yaml.get_orderbooks_by_chain_id(42161).unwrap_err();
        assert_eq!(
            err,
            YamlError::NotFound("orderbook with chain-id: 42161".to_string())
        );
    }

    #[test]
    fn test_get_local_db_remote_keys() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        let keys = ob_yaml.get_local_db_remote_keys().unwrap();
        assert_eq!(keys, vec!["mainnet".to_string()]);
    }

    #[test]
    fn test_get_local_db_remotes_and_single_remote() {
        let ob_yaml =
            OrderbookYaml::new(vec![full_yaml()], OrderbookYamlValidation::default()).unwrap();

        let remotes = ob_yaml.get_local_db_remotes().unwrap();
        assert_eq!(remotes.len(), 1);
        let remote = remotes.get("mainnet").unwrap();
        assert_eq!(remote.key, "mainnet");
        assert_eq!(
            remote.url,
            Url::parse("https://example.com/localdb/mainnet").unwrap()
        );

        // Also validate the getter for a single key
        let single = ob_yaml.get_local_db_remote("mainnet").unwrap();
        assert_eq!(single.key, "mainnet");
        assert_eq!(
            single.url,
            Url::parse("https://example.com/localdb/mainnet").unwrap()
        );
    }

    #[test]
    fn test_get_local_db_remote_missing_key_error() {
        let yaml = format!(
            r#"
version: {version}
networks:
    mainnet:
        rpcs:
            - https://mainnet.infura.io
        chain-id: 1
subgraphs:
    mainnet: https://api.thegraph.com/subgraphs/name/xyz
"#,
            version = SpecVersion::current()
        );

        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();
        let err = ob_yaml.get_local_db_remote("polygon").unwrap_err();
        assert_eq!(err, YamlError::KeyNotFound("polygon".to_string()));
    }

    #[test]
    fn test_get_local_db_syncs_and_keys() {
        let yaml = format!(
            r#"
version: {version}
local-db-sync:
  test:
    batch-size: 1
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 4
    rate-limit-delay-ms: 5
    finality-depth: 6
    bootstrap-block-threshold: 7
    sync-interval-ms: 5000
"#,
            version = SpecVersion::current()
        );
        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        let keys = ob_yaml.get_local_db_sync_keys().unwrap();
        assert_eq!(keys, vec!["test".to_string()]);

        let syncs = ob_yaml.get_local_db_syncs().unwrap();
        assert_eq!(syncs.len(), 1);
        let cfg = syncs.get("test").unwrap();
        assert_eq!(cfg.key, "test");
        assert_eq!(cfg.batch_size, 1);
        assert_eq!(cfg.max_concurrent_batches, 2);
        assert_eq!(cfg.retry_attempts, 3);
        assert_eq!(cfg.retry_delay_ms, 4);
        assert_eq!(cfg.rate_limit_delay_ms, 5);
        assert_eq!(cfg.finality_depth, 6);
        assert_eq!(cfg.bootstrap_block_threshold, 7);
        assert_eq!(cfg.sync_interval_ms, 5000);
    }

    #[test]
    fn test_get_local_db_sync_by_key() {
        let yaml = format!(
            r#"
version: {version}
local-db-sync:
  test:
    batch-size: 10
    max-concurrent-batches: 20
    retry-attempts: 30
    retry-delay-ms: 40
    rate-limit-delay-ms: 50
    finality-depth: 60
    bootstrap-block-threshold: 70
    sync-interval-ms: 5000
"#,
            version = SpecVersion::current()
        );
        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        let cfg = ob_yaml.get_local_db_sync("test").unwrap();
        assert_eq!(cfg.key, "test");
        assert_eq!(cfg.batch_size, 10);
        assert_eq!(cfg.max_concurrent_batches, 20);
        assert_eq!(cfg.retry_attempts, 30);
        assert_eq!(cfg.retry_delay_ms, 40);
        assert_eq!(cfg.rate_limit_delay_ms, 50);
        assert_eq!(cfg.finality_depth, 60);
        assert_eq!(cfg.bootstrap_block_threshold, 70);
        assert_eq!(cfg.sync_interval_ms, 5000);
    }

    #[test]
    fn test_get_local_db_sync_missing_key_error() {
        let yaml = format!(
            r#"
version: {version}
local-db-sync:
  test:
    batch-size: 1
    max-concurrent-batches: 2
    retry-attempts: 3
    retry-delay-ms: 4
    rate-limit-delay-ms: 5
    finality-depth: 6
    bootstrap-block-threshold: 7
    sync-interval-ms: 5000
"#,
            version = SpecVersion::current()
        );
        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        let err = ob_yaml.get_local_db_sync("nonexistent").unwrap_err();
        assert_eq!(err, YamlError::KeyNotFound("nonexistent".to_string()));
    }

    #[test]
    fn test_get_local_db_syncs_missing_section_is_ok() {
        let yaml = format!(
            r#"
version: {version}
test: test"#,
            version = SpecVersion::current()
        );
        let ob_yaml = OrderbookYaml::new(vec![yaml], OrderbookYamlValidation::default()).unwrap();

        let syncs = ob_yaml.get_local_db_syncs().unwrap();
        assert!(syncs.is_empty());
    }
}
