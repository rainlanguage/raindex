use alloy::primitives::U256;
use alloy_ethers_typecast::{ReadableClient, ReadableClientError};
use dotrain::{error::ComposeError, RainDocument, Rebind};
use futures::TryFutureExt;
use proptest::{
    prelude::RngCore,
    test_runner::{RngAlgorithm, TestRng},
};
use rain_interpreter_bindings::IInterpreterStoreV3::FullyQualifiedNamespace;
use rain_interpreter_eval::{
    error::ForkCallError,
    eval::ForkEvalArgs,
    fork::{Forker, NewForkedEvm},
    trace::{RainEvalResultFromRawCallResultError, RainEvalResults},
};
use raindex_app_settings::{
    blocks::BlockError,
    rainlang::RainlangCfg,
    unit_test::TestConfig,
    yaml::{
        raindex::{RaindexYaml, RaindexYamlValidation},
        YamlError, YamlParsable,
    },
};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone)]
pub struct TestRunner {
    pub forker: Forker,
    pub dotrains: Dotrains,
    pub raindex_yamls: RaindexYamls,
    pub rng: TestRng,
    pub test_setup: TestSetup,
    pub test_config: TestConfig,
}

#[derive(Clone)]
pub struct TestSetup {
    pub block_number: u64,
    pub rainlang: Arc<RainlangCfg>,
    pub scenario_name: String,
}

#[derive(Clone)]
pub struct Dotrains {
    pub main_dotrain: String,
    pub test_dotrain: String,
}

#[derive(Clone)]
pub struct RaindexYamls {
    pub main: RaindexYaml,
    pub test: RaindexYaml,
}

#[derive(Error, Debug)]
pub enum TestRunnerError {
    #[error("Scenario not found")]
    ScenarioNotFound(String),
    #[error(transparent)]
    ReadableClientHttpError(#[from] ReadableClientError),
    #[error(transparent)]
    BlockError(#[from] BlockError),
    #[error(transparent)]
    ForkCallError(Box<ForkCallError>),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    ComposeError(#[from] ComposeError),
    #[error("Invalid input args: {0}")]
    InvalidArgs(String),
    #[error(transparent)]
    YamlError(#[from] YamlError),
    #[error(transparent)]
    RainEvalResultFromRawCallResult(#[from] RainEvalResultFromRawCallResultError),
}

impl From<ForkCallError> for TestRunnerError {
    fn from(err: ForkCallError) -> Self {
        Self::ForkCallError(Box::new(err))
    }
}

impl TestRunner {
    pub async fn new(
        dotrain: &str,
        test_dotrain: &str,
        test_config: &TestConfig,
        seed: Option<[u8; 32]>,
    ) -> Result<Self, TestRunnerError> {
        let main_raindex_yaml = RaindexYaml::new(
            vec![RainDocument::get_front_matter(dotrain)
                .unwrap_or("")
                .to_string()],
            RaindexYamlValidation::default(),
        )?;
        let test_raindex_yaml = RaindexYaml::new(
            vec![RainDocument::get_front_matter(test_dotrain)
                .unwrap_or("")
                .to_string()],
            RaindexYamlValidation::default(),
        )?;

        Ok(Self {
            forker: Forker::new().unwrap(),
            dotrains: Dotrains {
                main_dotrain: dotrain.into(),
                test_dotrain: test_dotrain.into(),
            },
            raindex_yamls: RaindexYamls {
                main: main_raindex_yaml,
                test: test_raindex_yaml,
            },
            rng: TestRng::from_seed(RngAlgorithm::ChaCha, &seed.unwrap_or([0; 32])),
            test_setup: TestSetup {
                block_number: 0,
                rainlang: Arc::new(RainlangCfg::dummy()),
                scenario_name: String::new(),
            },
            test_config: test_config.clone(),
        })
    }

    fn get_final_bindings(&mut self, is_test_dotrain: bool) -> Vec<Rebind> {
        let scenario_bindings: Vec<Rebind> = self
            .test_config
            .scenario
            .bindings
            .clone()
            .into_iter()
            .map(|(k, v)| Rebind(k, v))
            .collect();
        let mut final_bindings: Vec<Rebind> = vec![];

        let dotrain = if is_test_dotrain {
            self.dotrains.test_dotrain.clone()
        } else {
            self.dotrains.main_dotrain.clone()
        };

        let rain_document =
            RainDocument::create(dotrain, None, None, Some(scenario_bindings.clone()));
        let elided_binding_keys = Arc::new(
            rain_document
                .namespace()
                .iter()
                .filter(|(_, v)| v.is_elided_binding())
                .map(|(k, _)| k.clone())
                .collect::<Vec<String>>(),
        );

        let elided_binding_keys = Arc::clone(&elided_binding_keys);
        let scenario_bindings = scenario_bindings.clone();

        for elided_binding in elided_binding_keys.as_slice() {
            let mut val: [u8; 32] = [0; 32];
            self.rng.fill_bytes(&mut val);
            let hex = alloy::primitives::hex::encode_prefixed(val);
            final_bindings.push(Rebind(elided_binding.to_string(), hex));
        }

        final_bindings.extend(scenario_bindings);
        final_bindings
    }

    async fn run_pre_entrypoint(&mut self) -> Result<RainEvalResults, TestRunnerError> {
        let final_bindings = self.get_final_bindings(true);

        let dotrain = Arc::new(self.dotrains.test_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let rainlang_cfg = Arc::clone(&self.test_setup.rainlang);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string =
                RainDocument::compose_text(&dotrain, &["pre"], None, Some(final_bindings))?;

            let args = ForkEvalArgs {
                rainlang_string,
                source_index: 0,
                rainlang: rainlang_cfg.address,
                namespace: FullyQualifiedNamespace::default(),
                context: vec![vec![U256::from(0); 1]; 1],
                decode_errors: true,
                inputs: vec![],
                state_overlay: vec![],
            };
            fork_clone
                .fork_eval(args)
                .map_err(|e| TestRunnerError::ForkCallError(Box::new(e)))
                .await
        });

        Ok(vec![handle.await??.try_into()?].into())
    }

    async fn run_calculate_entrypoint(
        &mut self,
        pre_stack: RainEvalResults,
    ) -> Result<RainEvalResults, TestRunnerError> {
        let input_token = pre_stack.results[0].stack[2];
        let output_token = pre_stack.results[0].stack[1];
        let output_cap = pre_stack.results[0].stack[0];

        let final_bindings = self.get_final_bindings(false);

        let dotrain = Arc::new(self.dotrains.main_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let rainlang_cfg = Arc::clone(&self.test_setup.rainlang);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string = RainDocument::compose_text(
                &dotrain,
                &["calculate-io"],
                None,
                Some(final_bindings),
            )?;

            let mut context = vec![vec![U256::from(0); 5]; 5];
            // output cap
            context[2][0] = output_cap;
            // input token
            context[3][0] = input_token;
            // output token
            context[4][0] = output_token;

            let args = ForkEvalArgs {
                rainlang_string,
                source_index: 0,
                rainlang: rainlang_cfg.address,
                namespace: FullyQualifiedNamespace::default(),
                context,
                decode_errors: true,
                inputs: vec![],
                state_overlay: vec![],
            };
            fork_clone
                .fork_eval(args)
                .map_err(|e| TestRunnerError::ForkCallError(Box::new(e)))
                .await
        });

        Ok(vec![handle.await??.try_into()?].into())
    }

    async fn run_handle_entrypoint(
        &mut self,
        pre_stack: RainEvalResults,
        calculate_stack: RainEvalResults,
    ) -> Result<RainEvalResults, TestRunnerError> {
        let output_cap = pre_stack.results[0].stack[0];
        let max_output = calculate_stack.results[0].stack[1];
        let _io_ratio = calculate_stack.results[0].stack[0];

        let final_bindings = self.get_final_bindings(false);

        let dotrain = Arc::new(self.dotrains.main_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let rainlang_cfg = Arc::clone(&self.test_setup.rainlang);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string =
                RainDocument::compose_text(&dotrain, &["handle-io"], None, Some(final_bindings))?;

            let mut context = vec![vec![U256::from(0); 5]; 5];

            // output vault decrease
            context[4][4] = U256::min(max_output, output_cap);

            let args = ForkEvalArgs {
                rainlang_string,
                source_index: 0,
                rainlang: rainlang_cfg.address,
                namespace: FullyQualifiedNamespace::default(),
                context,
                decode_errors: true,
                inputs: vec![],
                state_overlay: vec![],
            };
            fork_clone
                .fork_eval(args)
                .map_err(|e| TestRunnerError::ForkCallError(Box::new(e)))
                .await
        });

        Ok(vec![handle.await??.try_into()?].into())
    }

    async fn run_post_entrypoint(
        &mut self,
        pre_stack: RainEvalResults,
        calculate_stack: RainEvalResults,
    ) -> Result<RainEvalResults, TestRunnerError> {
        let input_token = pre_stack.results[0].stack[2];
        let output_token = pre_stack.results[0].stack[1];
        let output_cap = pre_stack.results[0].stack[0];
        let max_output = calculate_stack.results[0].stack[1];
        let io_ratio = calculate_stack.results[0].stack[0];

        let final_bindings = self.get_final_bindings(true);

        let dotrain = Arc::new(self.dotrains.test_dotrain.clone());
        self.forker
            .roll_fork(Some(self.test_setup.block_number), None)?;
        let fork = Arc::new(self.forker.clone());
        let fork_clone = Arc::clone(&fork);
        let rainlang_cfg = Arc::clone(&self.test_setup.rainlang);
        let dotrain = Arc::clone(&dotrain);

        let handle = tokio::spawn(async move {
            let rainlang_string =
                RainDocument::compose_text(&dotrain, &["post"], None, Some(final_bindings))?;

            let mut context = vec![vec![U256::from(0); 20]; 20];

            // input token
            context[3][0] = input_token;
            // output token
            context[4][0] = output_token;
            // max output
            context[2][0] = max_output;
            // io ratio
            context[2][1] = io_ratio;
            // output vault decrease
            context[4][4] = U256::min(max_output, output_cap);

            let args = ForkEvalArgs {
                rainlang_string,
                source_index: 0,
                rainlang: rainlang_cfg.address,
                namespace: FullyQualifiedNamespace::default(),
                context,
                decode_errors: true,
                inputs: vec![],
                state_overlay: vec![],
            };
            fork_clone
                .fork_eval(args)
                .map_err(|e| TestRunnerError::ForkCallError(Box::new(e)))
                .await
        });

        Ok(vec![handle.await??.try_into()?].into())
    }

    async fn create_fork(
        &mut self,
        rpcs: Vec<String>,
        block_number: u64,
    ) -> Result<(), TestRunnerError> {
        let mut last_err = None;
        let mut fork_success = false;
        for rpc in &rpcs {
            match self
                .forker
                .add_or_select(
                    NewForkedEvm {
                        fork_url: rpc.clone(),
                        fork_block_number: Some(block_number),
                    },
                    None,
                )
                .await
            {
                Ok(_) => {
                    fork_success = true;
                    break;
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }
        if !fork_success {
            return Err(TestRunnerError::InvalidArgs(format!(
                "Failed to create fork: {last_err:?}"
            )));
        }
        Ok(())
    }

    pub async fn run_unit_test(&mut self) -> Result<RainEvalResults, TestRunnerError> {
        self.test_setup.rainlang = Arc::new(
            self.raindex_yamls
                .main
                .get_rainlang(&self.test_config.scenario_name)?,
        );

        // Fetch the latest block number
        let rpcs = self
            .test_setup
            .rainlang
            .network
            .rpcs
            .iter()
            .map(|rpc| rpc.to_string())
            .collect::<Vec<String>>();

        let block_number = ReadableClient::new_from_http_urls(rpcs.clone())?
            .get_block_number()
            .await?;

        let blocks = self
            .test_config
            .scenario
            .blocks
            .as_ref()
            .map_or(Ok(vec![block_number]), |b| {
                b.expand_to_block_numbers(block_number)
            })?;
        self.test_setup.block_number = blocks[0];

        self.create_fork(rpcs, block_number).await?;

        let pre_stack = self.run_pre_entrypoint().await?;
        let calculate_stack = self.run_calculate_entrypoint(pre_stack.clone()).await?;
        let _handle_stack = self
            .run_handle_entrypoint(pre_stack.clone(), calculate_stack.clone())
            .await?;
        let results = self.run_post_entrypoint(pre_stack, calculate_stack).await?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raindex_app_settings::{spec_version::SpecVersion, unit_test::UnitTestConfigSource};
    use raindex_test_fixtures::LocalEvm;

    fn get_test_config(test_dotrain: &str) -> TestConfig {
        let frontmatter = RainDocument::get_front_matter(test_dotrain).unwrap();
        let source = serde_yaml::from_str::<UnitTestConfigSource>(frontmatter).unwrap();
        source.test.into_test_config()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn test_test_runner() {
        let local_evm = LocalEvm::new().await;
        let test_dotrain = format!(
            r#"
version: {spec_version}
test:
    # calculate-entrypoint: some-custom-entrypoint
    # handle-entrypoint: some-custom-entrypoint
    scenario-name: some-key
    scenario:
        bindings:
            raindex-subparser: {raindex_subparser}
            second-binding: 999
---
#raindex-subparser !
#second-binding !

#pre
input-token: 0x01,
output-token: 0x02,
output-cap: 10;

#post
using-words-from raindex-subparser

/* calculate io stack */
:ensure(equal-to(calculated-io-ratio() 999) "io ratio should be 999"),
:ensure(equal-to(calculated-max-output() 10) "max output should be 10"),
:ensure(equal-to(output-token() 0x02) "output token should be 0x02"),
:ensure(equal-to(input-token() 0x01) "input token should be 0x01"),

/* handle io stack */
:ensure(equal-to(output-vault-decrease() 10) "output cap should be 10");
    "#,
            raindex_subparser = local_evm.raindex_subparser.address(),
            spec_version = SpecVersion::current()
        );
        let dotrain = format!(
            r#"
version: {spec_version}
rainlangs:
    some-key:
        address: {rainlang_address}
networks:
    some-key:
        rpcs:
            - {rpc_url}
        chain-id: 123
scenarios:
    some-key:
        bindings:
            raindex-subparser: {raindex_subparser}
            second-binding: 20
---
#raindex-subparser !
#second-binding !

#calculate-io
using-words-from raindex-subparser

_: input-token(),
_: output-token(),
a: 10,
b: second-binding;
#handle-io
using-words-from raindex-subparser

_: output-vault-decrease();
    "#,
            rpc_url = local_evm.url(),
            rainlang_address = local_evm.rainlang,
            raindex_subparser = local_evm.raindex_subparser.address(),
            spec_version = SpecVersion::current()
        );

        let test_config = get_test_config(&test_dotrain);

        let mut runner = TestRunner::new(&dotrain, &test_dotrain, &test_config, None)
            .await
            .unwrap();

        runner
            .run_unit_test()
            .await
            .map_err(|e| println!("{:#?}", e))
            .unwrap();
    }
}
