use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use alloy::sol_types::SolCall;
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_common::add_order::AddOrderArgs;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser, Clone)]
pub struct AddOrderCalldata {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    #[arg(short = 'e', long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for AddOrderCalldata {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };
        let dotrain_order = DotrainOrder::create(dotrain, settings.map(|v| vec![v])).await?;
        let dotrain_string = dotrain_order.dotrain()?;

        let config_deployment = dotrain_order
            .dotrain_yaml()
            .get_deployment(&self.deployment)?;

        let add_order_args =
            AddOrderArgs::new_from_deployment(dotrain_string, config_deployment.clone(), None)
                .await?;

        let rpcs = config_deployment
            .scenario
            .rainlang
            .network
            .rpcs
            .iter()
            .map(|rpc| rpc.to_string())
            .collect::<Vec<String>>();
        let add_order_calldata = add_order_args.try_into_call(rpcs).await?.abi_encode();

        output(&None, self.encoding.clone(), &add_order_calldata)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::dyn_abi::SolType;
    use alloy::primitives::{hex::encode_prefixed, Address};
    use alloy::sol;
    use clap::CommandFactory;
    use httpmock::MockServer;
    use rain_interpreter_bindings::IParserV2::parse2Call;
    use rain_interpreter_bindings::Rainlang::{
        expressionDeployerAddressCall, interpreterAddressCall, parserAddressCall, storeAddressCall,
    };
    use rain_orderbook_app_settings::spec_version::SpecVersion;
    use rain_orderbook_app_settings::yaml::{FieldErrorKind, YamlError};
    use serde_json::json;
    use std::io::Write;
    use std::str::FromStr;
    use tempfile::NamedTempFile;

    #[test]
    fn verify_cli() {
        AddOrderCalldata::command().debug_assert();
    }

    #[test]
    fn test_cli_args() {
        let dotrain_file = PathBuf::from_str("./some/dotrain_file.rain").unwrap();
        let settings_file = PathBuf::from_str("./some/settings_file.rain").unwrap();
        let deployment_str = "some-deployment";
        let output_str = "hex";

        let cmd = AddOrderCalldata::command();
        let result = cmd
            .try_get_matches_from(vec![
                "cmd",
                "-f",
                dotrain_file.to_str().unwrap(),
                "-c",
                settings_file.to_str().unwrap(),
                "-e",
                deployment_str,
                "-o",
                output_str,
            ])
            .unwrap();
        assert_eq!(
            result.get_one::<PathBuf>("dotrain_file"),
            Some(&dotrain_file)
        );
        assert_eq!(
            result.get_one::<PathBuf>("settings_file"),
            Some(&settings_file)
        );
        assert_eq!(
            result.get_one::<String>("deployment"),
            Some(&deployment_str.to_string())
        );
        assert_eq!(
            result.get_one::<SupportedOutputEncoding>("encoding"),
            Some(&SupportedOutputEncoding::Hex)
        );
    }

    async fn mock_orderbook_rpc_calls(rpc_server: &MockServer) {
        // Helper to build ABI-encoded address return values.
        let build_address_return = |id: u64| {
            let addr = Address::random();
            let encoded_ret = <sol!((address,))>::abi_encode(&(addr,));
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": encode_prefixed(encoded_ret)
            })
        };

        let selector_hex = |selector: [u8; 4]| encode_prefixed(selector);

        // mock expressionDeployerAddress() call
        rpc_server.mock(|when, then| {
            when.path("/rpc")
                .body_contains(selector_hex(expressionDeployerAddressCall::SELECTOR));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(build_address_return(1));
        });

        // mock interpreterAddress() call
        rpc_server.mock(|when, then| {
            when.path("/rpc")
                .body_contains(selector_hex(interpreterAddressCall::SELECTOR));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(build_address_return(2));
        });

        // mock storeAddress() call
        rpc_server.mock(|when, then| {
            when.path("/rpc")
                .body_contains(selector_hex(storeAddressCall::SELECTOR));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(build_address_return(3));
        });

        // mock parserAddress() call
        rpc_server.mock(|when, then| {
            when.path("/rpc")
                .body_contains(selector_hex(parserAddressCall::SELECTOR));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(build_address_return(4));
        });

        // mock parse2() call
        rpc_server.mock(|when, then| {
            when.path("/rpc")
                .body_contains(selector_hex(parse2Call::SELECTOR));

            let encoded_ret = <sol!((bytes,))>::abi_encode(&(vec![0x01u8, 0x02u8],));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "jsonrpc": "2.0",
                    "id": 5,
                    "result": encode_prefixed(encoded_ret)
                }));
        });
    }

    #[tokio::test]
    async fn test_execute() {
        let rpc_server = MockServer::start_async().await;
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
    some-sg: https://www.some-sg.com

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

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

scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
        bindings:
            key: 10

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
#key !Test binding key
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;",
            rpc_server.url("/rpc").as_str(),
            spec_version = SpecVersion::current()
        );

        let mut temp_dotrain_file = NamedTempFile::new().unwrap();
        write!(temp_dotrain_file, "{}", dotrain_content).unwrap();
        let dotrain_path = temp_dotrain_file.path();

        mock_orderbook_rpc_calls(&rpc_server).await;

        let add_order_calldata = AddOrderCalldata {
            dotrain_file: dotrain_path.to_path_buf(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        add_order_calldata.execute().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_non_existent_dotrain_file() {
        let add_order_calldata = AddOrderCalldata {
            dotrain_file: PathBuf::from("./non_existent_test_dotrain.rain"),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };

        let result = add_order_calldata.execute().await;
        assert!(
            result.is_err(),
            "Expected an error due to non-existent .rain file"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No such file or directory"));
    }

    #[tokio::test]
    async fn test_execute_non_existent_settings_file() {
        let temp_dotrain_content = "
networks:
    some-network:
        rpcs:
            - http://localhost:8545
        chain-id: 1
        network-id: 1
        currency: ETH
rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
orders:
    some-order:
        rainlang: some-rainlang
        orderbook: 0x0000000000000000000000000000000000000000
        inputs: []
        outputs: []
deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
:;";
        let mut temp_dotrain_file = NamedTempFile::new().unwrap();
        write!(temp_dotrain_file, "{}", temp_dotrain_content).unwrap();
        let temp_dotrain_path = temp_dotrain_file.path();

        let add_order_calldata = AddOrderCalldata {
            dotrain_file: temp_dotrain_path.to_path_buf(),
            settings_file: Some(PathBuf::from("./non_existent_settings.yaml")),
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };

        let result = add_order_calldata.execute().await;
        assert!(
            result.is_err(),
            "Expected an error due to non-existent settings file"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No such file or directory"));
    }

    #[tokio::test]
    async fn test_execute_empty_dotrain_file() {
        let mut temp_dotrain_file = NamedTempFile::new().unwrap();
        write!(temp_dotrain_file, "").unwrap();
        let temp_dotrain_path = temp_dotrain_file.path();

        let add_order_calldata = AddOrderCalldata {
            dotrain_file: temp_dotrain_path.to_path_buf(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };

        let err = add_order_calldata.execute().await.unwrap_err();
        assert_eq!(err.to_string(), YamlError::EmptyFile.to_string());
    }

    #[tokio::test]
    async fn test_execute_invalid_yaml_dotrain_file() {
        let invalid_yaml_content = format!(
            r#"
version: {spec_version}
test: test
---
    :;
    "#,
            spec_version = SpecVersion::current()
        );

        let mut temp_dotrain_file = NamedTempFile::new().unwrap();
        write!(temp_dotrain_file, "{}", invalid_yaml_content).unwrap();
        let temp_dotrain_path = temp_dotrain_file.path();

        let add_order_calldata = AddOrderCalldata {
            dotrain_file: temp_dotrain_path.to_path_buf(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };

        let err = add_order_calldata.execute().await.unwrap_err();
        assert_eq!(
            err.to_string(),
            YamlError::Field {
                kind: FieldErrorKind::Missing("deployments".to_string()),
                location: "root".to_string()
            }
            .to_string()
        );
    }

    #[tokio::test]
    async fn test_execute_invalid_rainlang_script() {
        let rpc_server = MockServer::start_async().await;
        mock_orderbook_rpc_calls(&rpc_server).await;

        let dotrain_content_invalid_script = format!(
            "
version: {spec_version}
networks:
  some-network:
    rpcs:
        - {}
    chain-id: 1
subgraphs:
  some-subgraph: https://www.some-subgraph.com
orderbooks:
  some-orderbook:
    address: 0x0000000000000000000000000000000000000000
    network: some-network
    subgraph: some-subgraph
    deployment-block: 12345
rainlangs:
  some-rainlang:
    network: some-network
    address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
scenarios:
  some-scenario:
    network: some-network
    rainlang: some-rainlang
orders:
  some-order:
    rainlang: some-rainlang
    orderbook: some-orderbook
    inputs:
      - token: token1
    outputs:
      - token: token1
deployments:
  some-deployment:
    scenario: some-scenario
    order: some-order
tokens:
  token1:
    network: some-network
    address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
---
",
            rpc_server.url("/rpc").as_str(),
            spec_version = SpecVersion::current()
        );
        let mut temp_dotrain_file = NamedTempFile::new().unwrap();
        write!(temp_dotrain_file, "{}", dotrain_content_invalid_script).unwrap();
        let temp_dotrain_path = temp_dotrain_file.path();

        let add_order_calldata = AddOrderCalldata {
            dotrain_file: temp_dotrain_path.to_path_buf(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };

        let result = add_order_calldata.execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_no_rpc_response() {
        let dotrain_content = format!(
            r#"
version: {spec_version}
networks:
    some-network:
        rpcs:
            - http://localhost:12345/nonexistent_rpc
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

rainlangs:
    some-rainlang:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

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

scenarios:
    some-scenario:
        network: some-network
        rainlang: some-rainlang
        bindings:
            key: 10

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
#key !Test binding key
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#,
            spec_version = SpecVersion::current()
        );

        let mut temp_dotrain_file = NamedTempFile::new().unwrap();
        write!(temp_dotrain_file, "{}", dotrain_content).unwrap();
        let dotrain_path = temp_dotrain_file.path();

        let add_order_calldata = AddOrderCalldata {
            dotrain_file: dotrain_path.to_path_buf(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };

        let err = add_order_calldata.execute().await.unwrap_err();
        assert!(err
            .to_string()
            .contains("all providers failed to handle the request"));
    }
}
