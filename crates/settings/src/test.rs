use crate::local_db_remotes::LocalDbRemoteCfg;
use crate::*;
use alloy::primitives::Address;
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use subgraph::SubgraphCfg;

// Helper function to create a mock network
pub fn mock_network() -> Arc<NetworkCfg> {
    Arc::new(NetworkCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "local".into(),
        rpcs: vec![("http://127.0.0.1:8545").parse().unwrap()],
        chain_id: 1,
        label: Some("Local Testnet".into()),
        network_id: Some(1),
        currency: Some("ETH".into()),
    })
}

// Helper function to create a mock rainlang
pub fn mock_rainlang() -> Arc<RainlangCfg> {
    Arc::new(RainlangCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "Rainlang1".to_string(),
        address: Address::repeat_byte(0x03),
        network: mock_network(),
    })
}

// Helper function to create a mock orderbook
pub fn mock_orderbook() -> Arc<OrderbookCfg> {
    Arc::new(OrderbookCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "".to_string(),
        label: Some("Orderbook1".into()),
        address: Address::repeat_byte(0x04),
        subgraph: Arc::new(SubgraphCfg {
            document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
            key: "".to_string(),
            url: "https://subgraph.com".parse().unwrap(),
        }),
        network: mock_network(),
        local_db_remote: Some(Arc::new(LocalDbRemoteCfg::default())),
        deployment_block: 12345,
    })
}

// Helper function to create a mock token
pub fn mock_token(name: &str) -> Arc<TokenCfg> {
    Arc::new(TokenCfg {
        document: Arc::new(RwLock::new(StrictYaml::String("".to_string()))),
        key: "".to_string(),
        label: Some(name.into()),
        address: Address::repeat_byte(0x05),
        symbol: Some("TKN".into()),
        decimals: Some(18),
        network: mock_network(),
        logo_uri: None,
        extensions: None,
    })
}
