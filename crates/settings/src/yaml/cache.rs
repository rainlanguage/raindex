use crate::{NetworkCfg, TokenCfg};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Cache {
    pub remote_networks: HashMap<String, NetworkCfg>,
    pub remote_tokens: HashMap<String, TokenCfg>,
}
impl Cache {
    pub fn new() -> Self {
        Self {
            remote_networks: HashMap::new(),
            remote_tokens: HashMap::new(),
        }
    }

    pub fn update_remote_networks(&mut self, remote_networks: HashMap<String, NetworkCfg>) {
        self.remote_networks = remote_networks;
    }
    pub fn update_remote_network(&mut self, key: String, remote_network: NetworkCfg) {
        self.remote_networks.insert(key, remote_network);
    }
    pub fn get_remote_networks(&self) -> HashMap<String, NetworkCfg> {
        self.remote_networks.clone()
    }
    pub fn get_remote_network(&self, key: &str) -> Option<NetworkCfg> {
        self.remote_networks.get(key).cloned()
    }

    pub fn update_remote_tokens(&mut self, remote_tokens: HashMap<String, TokenCfg>) {
        self.remote_tokens = remote_tokens;
    }
    pub fn update_remote_token(&mut self, key: String, remote_token: TokenCfg) {
        self.remote_tokens.insert(key, remote_token);
    }
    pub fn get_remote_tokens(&self) -> HashMap<String, TokenCfg> {
        self.remote_tokens.clone()
    }
    pub fn get_remote_token(&self, key: &str) -> Option<TokenCfg> {
        self.remote_tokens.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::default_document; // For NetworkCfg and TokenCfg document field
    use alloy::primitives::Address;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Arc;
    use url::Url; // Used by TokenCfg

    // Helper to create a NetworkCfg for tests
    fn sample_network_cfg(key: &str, rpc_url_str: &str, chain_id: u32) -> NetworkCfg {
        NetworkCfg {
            document: default_document(),
            key: key.to_string(),
            rpcs: vec![Url::parse(rpc_url_str).expect("Failed to parse RPC URL for test")],
            chain_id,
            label: Some(format!("Test Label for {}", key)),
            network_id: Some(chain_id + 100), // Arbitrary distinct value
            currency: Some("TEST_ETH".to_string()),
        }
    }

    // Helper to create a TokenCfg for tests
    fn sample_token_cfg(
        key: &str,
        network: Arc<NetworkCfg>,
        address_str: &str,
        decimals: u8,
    ) -> TokenCfg {
        TokenCfg {
            document: default_document(),
            key: key.to_string(),
            network,
            address: Address::from_str(address_str).expect("Failed to parse address for test"),
            decimals: Some(decimals),
            label: Some(format!("Test Token {}", key)),
            symbol: Some(key.to_uppercase()),
            logo_uri: None,
            extensions: None,
        }
    }

    #[test]
    fn test_cache_new() {
        let cache = Cache::new();
        assert!(
            cache.remote_networks.is_empty(),
            "New cache should have empty remote_networks"
        );
        assert!(
            cache.remote_tokens.is_empty(),
            "New cache should have empty remote_tokens"
        );
    }

    #[test]
    fn test_update_remote_networks() {
        let mut cache = Cache::new();
        let mut networks_map1 = HashMap::new();
        networks_map1.insert(
            "net1".to_string(),
            sample_network_cfg("net1", "http://net1.com", 1),
        );
        networks_map1.insert(
            "net2".to_string(),
            sample_network_cfg("net2", "http://net2.com", 2),
        );

        cache.update_remote_networks(networks_map1.clone());
        assert_eq!(cache.remote_networks, networks_map1);

        let mut networks_map2 = HashMap::new();
        networks_map2.insert(
            "net3".to_string(),
            sample_network_cfg("net3", "http://net3.com", 3),
        );
        cache.update_remote_networks(networks_map2.clone());
        assert_eq!(
            cache.remote_networks, networks_map2,
            "Should overwrite with new map"
        );

        cache.update_remote_networks(HashMap::new());
        assert!(
            cache.remote_networks.is_empty(),
            "Should allow clearing the map"
        );
    }

    #[test]
    fn test_update_remote_network() {
        let mut cache = Cache::new();
        let network1 = sample_network_cfg("net1", "http://net1.com", 1);
        cache.update_remote_network("net1".to_string(), network1.clone());
        assert_eq!(cache.remote_networks.get("net1"), Some(&network1));
        assert_eq!(cache.remote_networks.len(), 1);

        let network1_updated = sample_network_cfg("net1", "http://updated-net1.com", 101);
        cache.update_remote_network("net1".to_string(), network1_updated.clone());
        assert_eq!(
            cache.remote_networks.get("net1"),
            Some(&network1_updated),
            "Should update existing network"
        );
        assert_eq!(
            cache.remote_networks.len(),
            1,
            "Length should remain 1 after update"
        );

        let network2 = sample_network_cfg("net2", "http://net2.com", 2);
        cache.update_remote_network("net2".to_string(), network2.clone());
        assert_eq!(cache.remote_networks.get("net2"), Some(&network2));
        assert_eq!(
            cache.remote_networks.len(),
            2,
            "Length should be 2 after adding a new network"
        );
    }

    #[test]
    fn test_get_remote_networks() {
        let mut cache = Cache::new();
        let mut initial_networks = HashMap::new();
        initial_networks.insert(
            "net1".to_string(),
            sample_network_cfg("net1", "http://net1.com", 1),
        );
        cache.update_remote_networks(initial_networks.clone());

        let mut retrieved_networks = cache.get_remote_networks();
        assert_eq!(
            retrieved_networks, initial_networks,
            "Should return the correct networks map"
        );

        // Modify the retrieved map and check if original is unchanged (due to clone)
        retrieved_networks.insert(
            "new_net".to_string(),
            sample_network_cfg("new_net", "http://new.com", 99),
        );
        assert_eq!(
            cache.get_remote_networks(),
            initial_networks,
            "Internal cache should not be affected by modifications to the retrieved map"
        );

        // Test with empty cache
        let empty_cache = Cache::new();
        assert!(
            empty_cache.get_remote_networks().is_empty(),
            "get_remote_networks on empty cache should return empty map"
        );
    }

    #[test]
    fn test_get_remote_network() {
        let mut cache = Cache::new();
        let network1 = sample_network_cfg("net1", "http://net1.com", 1);
        cache.update_remote_network("net1".to_string(), network1.clone());

        // Test retrieving an existing network
        let retrieved_option = cache.get_remote_network("net1");
        assert_eq!(
            retrieved_option,
            Some(network1.clone()),
            "Should retrieve the correct network"
        );

        // Test retrieving a non-existent network
        assert_eq!(
            cache.get_remote_network("non_existent_key"),
            None,
            "Should return None for non-existent key"
        );

        // Test cloning behavior of the retrieved network
        if let Some(mut retrieved_network_val) = retrieved_option {
            retrieved_network_val.label = Some("Modified Label".to_string());
            let original_network_in_cache = cache.get_remote_network("net1").unwrap();
            assert_eq!(
                original_network_in_cache.label,
                Some("Test Label for net1".to_string()),
                "Internal cache should not be modified by changes to cloned retrieved network"
            );
        } else {
            panic!("Failed to retrieve network for cloning test");
        }
    }

    #[test]
    fn test_update_remote_tokens() {
        let mut cache = Cache::new();
        let shared_network = Arc::new(sample_network_cfg("shared_net", "http://shared.com", 10));

        let mut tokens_map1 = HashMap::new();
        tokens_map1.insert(
            "token1".to_string(),
            sample_token_cfg(
                "token1",
                Arc::clone(&shared_network),
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                18,
            ),
        );
        tokens_map1.insert(
            "token2".to_string(),
            sample_token_cfg(
                "token2",
                Arc::clone(&shared_network),
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                6,
            ),
        );

        cache.update_remote_tokens(tokens_map1.clone());
        assert_eq!(cache.remote_tokens, tokens_map1);

        let mut tokens_map2 = HashMap::new();
        tokens_map2.insert(
            "token3".to_string(),
            sample_token_cfg(
                "token3",
                Arc::clone(&shared_network),
                "0xcccccccccccccccccccccccccccccccccccccccc",
                8,
            ),
        );
        cache.update_remote_tokens(tokens_map2.clone());
        assert_eq!(
            cache.remote_tokens, tokens_map2,
            "Should overwrite with new map"
        );

        cache.update_remote_tokens(HashMap::new());
        assert!(
            cache.remote_tokens.is_empty(),
            "Should allow clearing the map"
        );
    }

    #[test]
    fn test_update_remote_token() {
        let mut cache = Cache::new();
        let network1 = Arc::new(sample_network_cfg("net1", "http://net1.com", 1));
        let network2 = Arc::new(sample_network_cfg("net2", "http://net2.com", 2));

        let token1 = sample_token_cfg(
            "tkn1",
            Arc::clone(&network1),
            "0x1111111111111111111111111111111111111111",
            18,
        );
        cache.update_remote_token("tkn1".to_string(), token1.clone());
        assert_eq!(cache.remote_tokens.get("tkn1"), Some(&token1));
        assert_eq!(cache.remote_tokens.len(), 1);

        let token1_updated = sample_token_cfg(
            "tkn1",
            Arc::clone(&network1),
            "0x1111111111111111111111111111111111110000",
            17,
        ); // different address and decimals
        cache.update_remote_token("tkn1".to_string(), token1_updated.clone());
        assert_eq!(
            cache.remote_tokens.get("tkn1"),
            Some(&token1_updated),
            "Should update existing token"
        );
        assert_eq!(
            cache.remote_tokens.len(),
            1,
            "Length should remain 1 after update"
        );

        let token2 = sample_token_cfg(
            "tkn2",
            Arc::clone(&network2),
            "0x2222222222222222222222222222222222222222",
            6,
        );
        cache.update_remote_token("tkn2".to_string(), token2.clone());
        assert_eq!(cache.remote_tokens.get("tkn2"), Some(&token2));
        assert_eq!(
            cache.remote_tokens.len(),
            2,
            "Length should be 2 after adding a new token"
        );
    }

    #[test]
    fn test_get_remote_tokens() {
        let mut cache = Cache::new();
        let shared_network = Arc::new(sample_network_cfg("shared_net", "http://shared.com", 10));
        let mut initial_tokens = HashMap::new();
        initial_tokens.insert(
            "token1".to_string(),
            sample_token_cfg(
                "token1",
                Arc::clone(&shared_network),
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                18,
            ),
        );
        cache.update_remote_tokens(initial_tokens.clone());

        let mut retrieved_tokens = cache.get_remote_tokens();
        assert_eq!(
            retrieved_tokens, initial_tokens,
            "Should return the correct tokens map"
        );

        // Modify the retrieved map and check if original is unchanged (due to clone)
        retrieved_tokens.insert(
            "new_token".to_string(),
            sample_token_cfg(
                "new_token",
                Arc::clone(&shared_network),
                "0xcccccccccccccccccccccccccccccccccccccccc",
                8,
            ),
        );
        assert_eq!(
            cache.get_remote_tokens(),
            initial_tokens,
            "Internal cache should not be affected by modifications to the retrieved map"
        );

        // Test with empty cache
        let empty_cache = Cache::new();
        assert!(
            empty_cache.get_remote_tokens().is_empty(),
            "get_remote_tokens on empty cache should return empty map"
        );
    }

    #[test]
    fn test_get_remote_token() {
        let mut cache = Cache::new();
        let network1 = Arc::new(sample_network_cfg("net1", "http://net1.com", 1));
        let token1 = sample_token_cfg(
            "tkn1",
            Arc::clone(&network1),
            "0x1111111111111111111111111111111111111111",
            18,
        );
        cache.update_remote_token("tkn1".to_string(), token1.clone());

        // Test retrieving an existing token
        let retrieved_option = cache.get_remote_token("tkn1");
        assert_eq!(
            retrieved_option,
            Some(token1.clone()),
            "Should retrieve the correct token"
        );

        // Test retrieving a non-existent token
        assert_eq!(
            cache.get_remote_token("non_existent_key"),
            None,
            "Should return None for non-existent key"
        );

        // Test cloning behavior of the retrieved token
        if let Some(mut retrieved_token_val) = retrieved_option {
            retrieved_token_val.symbol = Some("MODIFIED_SYMBOL".to_string());
            let original_token_in_cache = cache.get_remote_token("tkn1").unwrap();
            assert_eq!(
                original_token_in_cache.symbol,
                Some("TKN1".to_string()),
                "Internal cache should not be modified by changes to cloned retrieved token"
            );
        } else {
            panic!("Failed to retrieve token for cloning test");
        }
    }
}
