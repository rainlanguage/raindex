use crate::execute::Execute;
use anyhow::{anyhow, Result};
use clap::{ArgAction, Args, Parser};
use csv::Writer;
use rain_orderbook_common::dotrain_order::{AuthoringMetaV2, DotrainOrder, WordsResult};
use std::{fs::read_to_string, path::PathBuf};

/// Get words of a rainlang contract from the given inputs
#[derive(Debug, Parser)]
pub struct Words {
    #[command(flatten)]
    pub input: Input,

    #[command(flatten)]
    pub source: Source,

    /// Only get pragma words for a given scenario
    #[arg(
        long,
        requires = "scenario",
        action = ArgAction::SetTrue,
        conflicts_with_all = ["rainlang_only", "rainlang", "deployment"],
    )]
    pub pragma_only: bool,

    /// Only get rainlang words for a given scenario
    #[arg(
        long,
        requires = "scenario",
        action = ArgAction::SetTrue,
        conflicts_with_all = ["pragma_only", "rainlang", "deployment"],
    )]
    pub rainlang_only: bool,

    /// Optional metaboard subgraph url, will override the metaboard in
    /// inputs or if inputs has no metaboard specified inside
    #[arg(short = 'm', long, value_name = "URL")]
    pub metaboard_subgraph: Option<String>,

    /// Optional output file path to write the result into
    #[arg(short = 'o', long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Print the result on console (send result to std out)
    #[arg(long, action = ArgAction::SetTrue)]
    pub stdout: bool,
}

/// Group of possible input files, at least one of dotrain file or
/// setting yml file or both
#[derive(Args, Clone, Debug, PartialEq)]
#[group(required = true, multiple = true)]
pub struct Input {
    /// Path to the .rain file specifying the order
    #[arg(short = 'f', long, value_name = "PATH")]
    pub dotrain_file: Option<PathBuf>,

    /// Path to the settings yaml file
    #[arg(short = 'c', long, value_name = "PATH")]
    pub settings_file: Option<PathBuf>,
}

/// Group of possible sources, only one of rainlang or scenario or deployment
#[derive(Args, Clone, Debug, PartialEq)]
#[group(required = true, multiple = false)]
pub struct Source {
    /// Rainlang key to get its associating words
    #[arg(short = 'd', long)]
    pub rainlang: Option<String>,

    /// Scenario key, requires dotrain_file if used
    #[arg(short = 's', long, requires = "dotrain_file")]
    pub scenario: Option<String>,

    /// Deployment key, requires dotrain_file if used
    #[arg(long, requires = "dotrain_file")]
    pub deployment: Option<String>,
}

impl Execute for Words {
    async fn execute(&self) -> Result<()> {
        let dotrain = self
            .input
            .dotrain_file
            .as_ref()
            .and_then(|v| read_to_string(v).ok())
            .unwrap_or("---\n".to_string());
        let settings = match &self.input.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };

        let dotrain_order = DotrainOrder::create(dotrain, settings.map(|v| vec![v])).await?;

        let results = if let Some(rainlang_key) = &self.source.rainlang {
            // get rainlang from order config
            let rainlang = dotrain_order.orderbook_yaml().get_rainlang(rainlang_key)?;

            // get metaboard subgraph url
            let metaboard_url = self
                .metaboard_subgraph
                .as_ref()
                .map(|v| v.to_string())
                .or_else(|| {
                    dotrain_order
                        .orderbook_yaml()
                        .get_metaboard(&rainlang.network.key)
                        .ok()
                        .map(|metaboard| metaboard.url.to_string())
                })
                .ok_or(anyhow!("undefined metaboard subgraph url"))?;

            let rpcs = rainlang
                .network
                .rpcs
                .iter()
                .map(|rpc| rpc.to_string())
                .collect::<Vec<String>>();
            AuthoringMetaV2::fetch_for_contract(rainlang.address, rpcs, metaboard_url)
                .await?
                .words
        } else if let Some(scenario) = &self.source.scenario {
            // set the cli given metaboard url into the config
            if let Some(v) = &self.metaboard_subgraph {
                let network_name = &dotrain_order
                    .dotrain_yaml()
                    .get_scenario(scenario)?
                    .rainlang
                    .network
                    .key
                    .clone();
                dotrain_order
                    .orderbook_yaml()
                    .add_metaboard(network_name, v)?;
            }
            if self.rainlang_only {
                match dotrain_order
                    .get_rainlang_words_for_scenario(scenario)
                    .await?
                    .words
                {
                    WordsResult::Success(v) => v.words,
                    WordsResult::Error(e) => Err(anyhow!(e))?,
                }
            } else if self.pragma_only {
                let result = dotrain_order
                    .get_pragma_words_for_scenario(scenario)
                    .await?;
                let mut words = vec![];
                for p in result {
                    match p.words {
                        WordsResult::Success(v) => words.extend(v.words),
                        WordsResult::Error(e) => Err(anyhow!(e))?,
                    }
                }
                words
            } else {
                let result = dotrain_order.get_all_words_for_scenario(scenario).await?;
                let mut words = vec![];
                match result.rainlang_words.words {
                    WordsResult::Success(v) => words.extend(v.words),
                    WordsResult::Error(e) => Err(anyhow!(e))?,
                }
                for p in result.pragma_words {
                    match p.words {
                        WordsResult::Success(v) => words.extend(v.words),
                        WordsResult::Error(e) => Err(anyhow!(e))?,
                    }
                }
                words
            }
        } else if let Some(deployment) = &self.source.deployment {
            let deployment = dotrain_order.dotrain_yaml().get_deployment(deployment)?;
            let scenario = &deployment.scenario.key;

            // set the cli given metaboard url into the config
            if let Some(v) = &self.metaboard_subgraph {
                let network_key = deployment.scenario.rainlang.network.key.clone();
                dotrain_order
                    .orderbook_yaml()
                    .add_metaboard(&network_key, v)?;
            }
            let result = dotrain_order.get_all_words_for_scenario(scenario).await?;
            let mut words = vec![];
            match result.rainlang_words.words {
                WordsResult::Success(v) => words.extend(v.words),
                WordsResult::Error(e) => Err(anyhow!(e))?,
            }
            for p in result.pragma_words {
                match p.words {
                    WordsResult::Success(v) => words.extend(v.words),
                    WordsResult::Error(e) => Err(anyhow!(e))?,
                }
            }
            words
        } else {
            // clap doesnt allow this to happen since at least 1 source
            // is required which is enforced and catched by clap
            panic!("undefined source")
        };

        let mut csv_writer = Writer::from_writer(vec![]);
        for item in results.clone().into_iter() {
            csv_writer.serialize(item)?;
        }
        let text = String::from_utf8(csv_writer.into_inner()?)?;

        if let Some(output) = &self.output {
            std::fs::write(output, &text)?;
        }
        if self.stdout {
            println!("{}", text);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{hex::encode_prefixed, primitives::B256, sol, sol_types::SolValue};
    use clap::CommandFactory;
    use httpmock::MockServer;
    use rain_metadata::{KnownMagic, RainMetaDocumentV1Item};
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use serde_bytes::ByteBuf;
    use serde_json::json;
    use tempfile::NamedTempFile;

    sol!(
        struct AuthoringMetaV2Sol {
            bytes32 word;
            string description;
        }
    );
    sol!(
        struct PragmaV1 { address[] usingWordsFrom; }
    );

    #[test]
    fn verify_command() {
        Words::command().debug_assert();
    }

    #[tokio::test]
    async fn test_execute_happy_with_dotrain() {
        let server = mock_server();
        let dotrain_content = format!(
            "
version: {spec_version}
networks:
    some-network:
        rpcs:
            - {}
        chain-id: 123
        network-id: 123
        currency: ETH

metaboards:
    some-network: {}

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
---
#binding
:;",
            server.url("/rpc"),
            server.url("/sg"),
            spec_version = SpecVersion::current()
        );

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain_content).unwrap();

        let words = Words {
            input: Input {
                dotrain_file: Some(dotrain_path),
                settings_file: None,
            },
            source: Source {
                rainlang: Some("some-rainlang".to_string()),
                scenario: None,
                deployment: None,
            },
            pragma_only: false,
            rainlang_only: false,
            metaboard_subgraph: None,
            output: None,
            stdout: true,
        };

        // should execute successfully
        words.execute().await.unwrap();
        assert!(words.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_happy_all() {
        let server = mock_server();
        let dotrain_content = format!(
            "
version: {spec_version}
metaboards:
    some-network: {}
---
#binding\n:;",
            server.url("/sg"),
            spec_version = SpecVersion::current()
        );
        let settings_content = format!(
            "
version: {spec_version}
networks:
    some-network:
        rpcs:
            - {}
        chain-id: 123
        network-id: 123
        currency: ETH

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba",
            server.url("/rpc"),
            spec_version = SpecVersion::current()
        );

        let settings_file = NamedTempFile::new().unwrap();
        let settings_path = settings_file.path().to_path_buf();
        std::fs::write(settings_path.clone(), settings_content).unwrap();

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain_content).unwrap();

        let words = Words {
            input: Input {
                settings_file: Some(settings_path),
                dotrain_file: Some(dotrain_path),
            },
            source: Source {
                rainlang: Some("some-rainlang".to_string()),
                scenario: None,
                deployment: None,
            },
            pragma_only: false,
            rainlang_only: false,
            metaboard_subgraph: None,
            output: None,
            stdout: true,
        };

        // should execute successfully
        words.execute().await.unwrap();
        assert!(words.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_happy_scenario_all_words() {
        let server = mock_server();
        let dotrain_content = format!(
            "
version: {spec_version}
networks:
    some-network:
        rpcs:
            - {}
        chain-id: 123
        network-id: 123
        currency: ETH

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
        bindings:
            key1: 10
---
#key1 !Test binding
#calculate-io
_ _: 1 2;
#handle-io
:;
",
            server.url("/rpc"),
            spec_version = SpecVersion::current()
        );

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain_content).unwrap();

        let words = Words {
            input: Input {
                settings_file: None,
                dotrain_file: Some(dotrain_path),
            },
            source: Source {
                rainlang: None,
                deployment: None,
                scenario: Some("some-scenario".to_string()),
            },
            pragma_only: false,
            rainlang_only: false,
            metaboard_subgraph: Some(server.url("/sg").to_string()),
            output: None,
            stdout: true,
        };

        // should execute successfully
        words.execute().await.unwrap();
        assert!(words.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_happy_scenario_rainlang_words() {
        let server = mock_server();
        let dotrain_content = format!(
            "
version: {spec_version}
metaboards:
    some-network: {}
---
#binding\n:;",
            server.url("/sg"),
            spec_version = SpecVersion::current()
        );
        let settings_content = format!(
            "
version: {spec_version}
networks:
    some-network:
        rpcs:
            - {}
        chain-id: 123
        network-id: 123
        currency: ETH

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
        bindings:
            key1: value1

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
orders:
    some-order:
        inputs:
            - token: token1
        outputs:
            - token: token1
",
            server.url("/rpc"),
            spec_version = SpecVersion::current()
        );

        let settings_file = NamedTempFile::new().unwrap();
        let settings_path = settings_file.path().to_path_buf();
        std::fs::write(settings_path.clone(), settings_content).unwrap();

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain_content).unwrap();

        let words = Words {
            input: Input {
                settings_file: Some(settings_path),
                dotrain_file: Some(dotrain_path),
            },
            source: Source {
                rainlang: None,
                deployment: None,
                scenario: Some("some-scenario".to_string()),
            },
            pragma_only: false,
            rainlang_only: true,
            metaboard_subgraph: None,
            output: None,
            stdout: true,
        };

        // should execute successfully
        assert!(words.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_happy_deployment_words() {
        let server = mock_server();
        let dotrain_content = format!(
            "
version: {spec_version}
networks:
    some-network:
        rpcs:
            - {}
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://some-sg.com

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
        bindings:
            key1: 10

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
        deployment-block: 12345

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: T2
        symbol: T2

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token2
              vault-id: 1
        rainlang: some-rainlang
        orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#key1 !Test binding
#calculate-io
_ _: 1 2;
#handle-io
:;
",
            server.url("/rpc"),
            spec_version = SpecVersion::current()
        );

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain_content).unwrap();

        let words = Words {
            input: Input {
                settings_file: None,
                dotrain_file: Some(dotrain_path),
            },
            source: Source {
                deployment: Some("some-deployment".to_string()),
                rainlang: None,
                scenario: None,
            },
            pragma_only: false,
            rainlang_only: false,
            metaboard_subgraph: Some(server.url("/sg").to_string()),
            output: None,
            stdout: true,
        };

        // should execute successfully
        words.execute().await.unwrap();
        assert!(words.execute().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unhappy() {
        let server = MockServer::start();
        // mock contract calls that doesnt implement IDescribeByMetaV1
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a701ffc9a7");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": B256::left_padding_from(&[0]).to_string()
            }));
        });

        let dotrain_content = format!(
            "
networks:
    some-network:
        rpcs:
            - {}
        chain-id: 123
        network-id: 123
        currency: ETH

metaboards:
    some-network: {}

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
---
#binding
:;",
            server.url("/rpc"),
            server.url("/sg")
        );

        let dotrain_file = NamedTempFile::new().unwrap();
        let dotrain_path = dotrain_file.path().to_path_buf();
        std::fs::write(dotrain_path.clone(), dotrain_content).unwrap();

        let words = Words {
            input: Input {
                dotrain_file: Some(dotrain_path),
                settings_file: None,
            },
            source: Source {
                rainlang: Some("some-rainlang".to_string()),
                scenario: None,
                deployment: None,
            },
            pragma_only: false,
            rainlang_only: false,
            metaboard_subgraph: None,
            output: None,
            stdout: true,
        };

        // should fail
        assert!(words.execute().await.is_err());
    }

    // helper function to mock rpc and sg response
    fn mock_server() -> MockServer {
        let server = MockServer::start();

        // mock contract calls
        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7ffffffff");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": B256::left_padding_from(&[0]).to_string()
            }));
        });

        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x01ffc9a7");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": B256::left_padding_from(&[1]).to_string()
            }));
        });

        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x6f5aa28d");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": B256::random().to_string()
            }));
        });

        server.mock(|when, then| {
            when.path("/rpc").body_contains("0x5514ca20");
            then.json_body(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": encode_prefixed(PragmaV1 { usingWordsFrom: vec![] }.abi_encode())
            }));
        });

        // mock sg query
        server.mock(|when, then| {
            when.path("/sg");
            then.status(200).json_body_obj(&serde_json::json!({
                "data": {
                    "metaV1S": [{
                        "meta": encode_prefixed(
                            RainMetaDocumentV1Item {
                                payload: ByteBuf::from(
                                    vec![
                                        AuthoringMetaV2Sol {
                                            word: B256::right_padding_from("some-word".as_bytes()),
                                            description: "some-desc".to_string(),
                                        },
                                        AuthoringMetaV2Sol {
                                            word: B256::right_padding_from("some-other-word".as_bytes()),
                                            description: "some-other-desc".to_string(),
                                        }
                                    ]
                                    .abi_encode(),
                                ),
                                magic: KnownMagic::AuthoringMetaV2,
                                content_type: rain_metadata::ContentType::OctetStream,
                                content_encoding: rain_metadata::ContentEncoding::None,
                                content_language: rain_metadata::ContentLanguage::None,
                            }
                            .cbor_encode()
                            .unwrap()
                        ),
                        "metaHash": "0x00",
                        "sender": "0x00",
                        "id": "0x00",
                        "metaBoard": {
                            "id": "0x00",
                            "metas": [],
                            "address": "0x00",
                        },
                        "subject": "0x00",
                    }]
                }
            }));
        });
        server
    }
}
