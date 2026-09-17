use crate::NetworkCfg;
use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;
use thiserror::Error;
use url::Url;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainId {
    pub name: String,
    pub chain: String,
    pub icon: Option<String>,
    pub rpc: Vec<Url>,
    pub features: Option<Vec<Features>>,
    pub faucets: Option<Vec<String>>,
    pub native_currency: NativeCurrency,
    #[serde(rename = "infoURL")]
    pub info_url: String,
    pub short_name: String,
    pub chain_id: u32,
    pub network_id: u32,
    pub slip44: Option<u64>,
    pub ens: Option<ENS>,
    pub explorers: Option<Vec<Explorer>>,
    pub red_flags: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Features {
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NativeCurrency {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ENS {
    pub registry: Address,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Explorer {
    pub name: String,
    pub url: Url,
    pub icon: Option<String>,
    pub standard: String,
}

#[derive(Error, Debug, PartialEq)]
pub enum ChainIdError {
    #[error("provided rpc urls are not supported")]
    UnsupportedRpcUrls,
    #[error("cannot find any rpc urls for this chain")]
    NoRpc,
}

impl ChainId {
    pub fn try_into_network_cfg(
        self,
        document: Arc<RwLock<StrictYaml>>,
    ) -> Result<NetworkCfg, ChainIdError> {
        if self.rpc.is_empty() {
            return Err(ChainIdError::NoRpc);
        }
        let block_explorer = self
            .explorers
            .as_ref()
            .and_then(|explorers| {
                explorers.iter().find(|explorer| {
                    explorer.standard.eq_ignore_ascii_case("EIP3091")
                        && matches!(explorer.url.scheme(), "http" | "https")
                })
            })
            .map(|explorer| explorer.url.clone());
        for rpc in &self.rpc {
            if !rpc.path().contains("API_KEY") && !rpc.scheme().starts_with("ws") {
                return Ok(NetworkCfg {
                    document: document.clone(),
                    key: self.short_name,
                    rpcs: vec![rpc.clone()],
                    chain_id: self.chain_id,
                    label: Some(self.name),
                    network_id: Some(self.network_id),
                    currency: Some(self.native_currency.symbol),
                    block_explorer,
                });
            }
        }
        Err(ChainIdError::UnsupportedRpcUrls)
    }
}

#[derive(Error, Debug)]
pub enum RemoteNetworkError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Unknown format: {}", 0)]
    UnknownFormat(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum RemoteNetworks {
    ChainId(Vec<ChainId>),
}
