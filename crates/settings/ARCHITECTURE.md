# Settings Crate – Architecture and Responsibilities

This crate defines the configuration model, parsing, validation and update utilities for the Rain Orderbook stack. It turns one or more YAML "settings" documents into strongly‑typed Rust structures that the rest of the system can consume (CLI, services, and WASM/JS bindings). It also supports fetching and merging remote configuration (networks, tokens), contextual variable interpolation, and in‑place updates back to the underlying YAML.

At a glance:

- Input format: one or more YAML documents (via `StrictYaml` from `strict_yaml_rust`).
- Output types: `NetworkCfg`, `TokenCfg`, `OrderbookCfg`, `SubgraphCfg`, `DeployerCfg`, `OrderCfg`, `ScenarioCfg`, `DeploymentCfg`, `OrderBuilderCfg`, `ChartCfg`, `MetaboardCfg`, `AccountCfg`, plus helpers.
- Cross‑document merge: parse operations accept a vector of YAML documents and merge sections across them, rejecting duplicate keys deterministically.
- Remote sources: optional “using‑*” sections enable fetching networks/tokens from external endpoints and merging them into the local model.
- Context: a runtime context carries selected deployment/order, token selection for order builder flows, remote caches, and supports string interpolation from order paths.
- WASM/TypeScript: many types derive `Tsify` and implement WASM trait helpers for interop with the webapp.


## Parsing Framework (yaml/*)

Core traits and helpers live under `src/yaml` and are used by all config types:

- Traits
  - `ValidationConfig`: strategy for which sections to validate (networks, tokens, orders, etc.).
  - `YamlParsable`: for top‑level parsers that accept multiple YAML documents (e.g. `OrderbookYaml`, `DotrainYaml`). Provides constructors from strings/documents and helper to stringify a document.
  - `YamlParsableHash`: for map‑shaped sections (e.g. `networks`, `tokens`, `orders`). Provides `parse_all_from_yaml` and `parse_from_yaml(key)`.
  - `YamlParsableVector`: for vector‑shaped items if needed.
  - `YamlParsableString`: for single string fields with optionality (e.g. `SpecVersion`, `Sentry`).
  - `YamlParseableValue`: for single logical objects that are not a map (e.g. `OrderBuilderCfg`, `RemoteTokensCfg`).

- Context and caching
  - `Context` holds:
    - `order: Option<Arc<OrderCfg>>` – the current order for interpolation.
    - `select_tokens: Option<Vec<String>>` – allow order builder to reference tokens by key without YAML definitions.
    - `builder_context`: current deployment/order selection.
    - `yaml_cache`: remote networks/tokens cache injected by providers.
  - Interpolation: `Context::interpolate("... ${order.inputs.0.token.symbol} ...")` resolves values from the current order (inputs/outputs, token address/symbol/label/decimals, vault IDs).
  - Path resolution helpers and errors: `ContextError::{NoOrder, InvalidPath, InvalidIndex, PropertyNotFound}` with human‑readable messages.

- Cache
  - `yaml/cache.rs::Cache` stores remote networks/tokens fetched previously. The providers (`OrderbookYaml`, `DotrainYaml`) expose them to `Context` when parsing.
  - Update/get helpers return clones to keep the cache immutable from the caller’s perspective.

- YAML helpers and errors
  - Required/optional accessors: `require_string`, `optional_string`, `require_hash`, `optional_hash`, `require_vec`, `optional_vec`, `get_hash_value`, `get_hash_value_as_option`.
  - `YamlError` and `FieldErrorKind` model validation issues precisely and implement `to_readable_msg()` for end‑user feedback. Additional variants wrap module‑specific parse errors (e.g. network/token/order errors).


## Document Providers

Two top‑level providers wrap one or more YAML documents and expose a convenient API to parse sections, fetch remote data, and produce contexts.

- OrderbookYaml (`yaml/orderbook.rs`)
  - Holds `documents: Vec<Arc<RwLock<StrictYaml>>>` and a `Cache`.
  - `new(sources, validation)` loads YAML strings, applies validation gates via `ValidationConfig`, and returns a provider.
  - Context initialization: `initialize_context_and_expand_remote_data()` injects remote networks/tokens from the cache into a fresh `Context`.
  - Accessors (all parse across all documents, merging maps and checking duplicates):
    - Networks: `get_network_keys`, `get_networks`, `get_network(key)`, `get_network_by_chain_id(u32)`.
    - Remote networks: `get_remote_networks` (parse `using-networks-from`).
    - Tokens: `get_token_keys`, `get_tokens`, `get_token(key)`.
    - Remote tokens: `get_remote_tokens` (parse optional `using-tokens-from`).
    - Subgraphs: `get_subgraph_keys`, `get_subgraphs`, `get_subgraph(key)`.
    - Orderbooks: `get_orderbook_keys`, `get_orderbooks`, `get_orderbook(key)`, `get_orderbook_by_address(Address)`, `get_orderbooks_by_network_key(&str)`.
    - Metaboards: `get_metaboard_keys`, `get_metaboards`, `get_metaboard(key)`, `add_metaboard(key, url)`.
    - Deployers: `get_deployer_keys`, `get_deployers`, `get_deployer(key)`.
    - Sentry: `get_sentry()` → Option<bool> from `sentry` scalar.
    - Spec version: `get_spec_version()` → string from `version` scalar.
    - Accounts: `get_account_keys`, `get_accounts`, `get_account(key)`.
  - Serde: serializes/deserializes as a sequence of YAML documents represented as strings.

- DotrainYaml (`yaml/dotrain.rs`)
  - Also wraps `documents` and a `Cache`.
  - `new(sources, validation)` selectively validates orders, scenarios, deployments.
  - Accessors:
    - Orders: `get_order_keys`, `get_orders`, `get_order(key)`, `get_order_for_builder_deployment(order_key, deployment_key)`.
    - Scenarios: `get_scenario_keys`, `get_scenarios`, `get_scenario(key)`.
    - Deployments: `get_deployment_keys`, `get_deployments`, `get_deployment(key)`.
    - Order Builder: `get_order_builder(current_deployment)` parses optional builder section with deployment‑scoped overrides and select‑tokens.
    - Charts: `get_chart_keys`, `get_charts`, `get_chart(key)`.
  - Serde mirrors `OrderbookYaml`.


## Core Config Objects

All core configs implement `YamlParsableHash` unless noted, and each instance carries a reference to the originating YAML document via `document: Arc<RwLock<StrictYaml>>`. This enables in‑place updates that preserve the source document. Equality (`PartialEq`) ignores the `document` field and compares logical data only.

### Networks (`network.rs`)

- `NetworkCfg { key, rpcs: Vec<Url>, chain_id, label?, network_id?, currency? }`
  - `dummy()` and `Default` for tests.
  - Validators: `validate_rpc(&str) -> Url`, `validate_chain_id(&str) -> u64`, `validate_network_id(&str) -> u32`.
  - Parse all: looks for `networks` map with entries shaped like:
    ```yaml
    networks:
      mainnet:
        rpcs: [https://mainnet.infura.io]
        chain-id: 1
        label: Ethereum Mainnet
        network-id: 1
        currency: ETH
    ```
  - `parse_rpcs(documents, network_key)` reads just the `rpcs` vector for a named network.
  - `update_rpcs(&mut self, Vec<String>)` updates both the YAML document and the in‑memory struct.
  - Integrates remote networks from context cache; duplicate keys cause `KeyShadowing`.
  - Specific error enum: `ParseNetworkConfigSourceError` with readable messages.

### Tokens (`token.rs`)

- `TokenCfg { key, network: Arc<NetworkCfg>, address, decimals?, label?, symbol? }`
  - Validators: `validate_address(value) -> Address`, `validate_decimals(value) -> u8`.
  - Parsing requires `tokens` map entries with `network` and `address`, optional `decimals`/`label`/`symbol`.
  - Mutations:
    - `update_address(&mut self, &str)` updates YAML and struct.
    - `add_record_to_yaml(docs, key, network_key, address, decimals?, label?, symbol?)` inserts a new token record into the first document, checking that the key doesn’t already exist and that the referenced network exists.
    - `remove_record_from_yaml(docs, key)` removes a token from whichever document contains it.
    - `parse_network_key(docs, token_key)` returns the network key for an existing token definition.
  - Merges remote tokens from context cache; conflicts produce `RemoteTokenKeyShadowing`.
  - Specific error enum: `ParseTokenConfigSourceError` with readable messages.

### Subgraphs (`subgraph.rs`)

- `SubgraphCfg { key, url }` as a simple `subgraphs:` map.
- `add_record_to_yaml(document, key, url)` helper with URL validation.

### Orderbooks (`orderbook.rs`)

- `OrderbookCfg { key, address, network: Arc<NetworkCfg>, subgraph: Arc<SubgraphCfg>, local_db_remote?: Arc<LocalDbRemoteCfg>, label?, deployment_block }`.
- Validators: `validate_address(&str) -> Address`, `validate_deployment_block(&str) -> u64`.
- Lookup helpers: `parse_network_key(docs, orderbook_key)` returns the referenced network key or defaults to the orderbook key.
- Parses with references to previously parsed networks and subgraphs; duplicates are rejected.
- Error enum: `ParseOrderbookConfigSourceError` (invalid address, missing network/subgraph, block parse error) with readable messages.

### Local DB Remotes (`local_db_remotes.rs`)

- `local-db-remotes:` is a optional top-level map. Each entry is parsed as `LocalDbRemoteCfg { key, url }`.
- The `orderbooks[*].local-db-remote` field is optional. If omitted, it defaults to the orderbook's key. When provided explicitly, it must reference a defined remote key under `local-db-remotes`.
  - See `src/orderbook.rs` for the implementation and tests, e.g. `test_orderbook_local_db_remote_absent_defaults_to_orderbook_key`, `test_orderbook_local_db_remote_resolves`, and `test_orderbook_local_db_remote_not_found`.

### Deployers (`deployer.rs`)

- `DeployerCfg { key, address, network }`.
- Validators and `parse_network_key` similar to orderbooks (defaults to key if `network` is omitted).
- Error enum: `ParseDeployerConfigSourceError`.

### Accounts (`accounts.rs`)

- `AccountCfg { key, address }` from a simple `accounts:` map where values are addresses.
- Error enum: `ParseAccountCfgError`.

### Metaboards (`metaboard.rs`)

- `MetaboardCfg { key, url }` from `metaboards:` map of key → URL.
- `add_record_to_yaml(document, key, url)` to append entries.

### Spec Version and Sentry

- `SpecVersion` reads required root scalar `version`. Const `CURRENT_SPEC_VERSION = "3"` and helpers `current()` / `is_current()`.
- `Sentry` parses optional root scalar `sentry`. `OrderbookYaml::get_sentry()` normalizes to `Option<bool>` accepting `true/false/1/0`.


## Orders, Scenarios, Deployments

These three model how orders are defined, how they are executed (bindings, blocks, runs), and how a deployment pairs an order with a scenario under a specific deployer.

### Orders (`order.rs`)

- `OrderCfg { key, inputs: Vec<OrderIOCfg>, outputs: Vec<OrderIOCfg>, network: Arc<NetworkCfg>, deployer?: Arc<DeployerCfg>, orderbook?: Arc<OrderbookCfg> }`.
- `OrderIOCfg { token_key: String, token?: Arc<TokenCfg>, vault_id?: U256 }` – `token_key` preserves the declared token name even when the token is unresolved for select‑tokens; vault IDs are arbitrary U256 strings.
- Validation and network unification
  - Inputs/outputs must each contain `token` (unless permitted by builder select‑tokens through context) and optional `vault-id`.
  - The order’s effective `network` is inferred from first matching component (deployer/orderbook/token), and all references must match. Mismatch yields detailed errors (`DeployerNetworkDoesNotMatch`, `OrderbookNetworkDoesNotMatch`, `InputTokenNetworkDoesNotMatch`, `OutputTokenNetworkDoesNotMatch`). If no network can be determined, `NetworkNotFoundError` is raised.
  - Vault IDs are validated via `U256::from_str`.
- Mutations
  - `update_vault_id(vault_type, token_key, vault_id_opt)` updates a vault ID for a specific input/output token inside the YAML.
  - `populate_vault_ids()` fills missing input/output `vault-id`s in the YAML with a freshly generated random U256, and updates the in‑memory struct accordingly.
- Helpers
  - `parse_network_key(docs, order_key)` – resolves the expected network key by reconciling deployer/orderbook and all IO token networks; errors if any disagree.
- Error enum: `ParseOrderConfigSourceError` implements `to_readable_msg()` for user‑oriented descriptions.

### Scenarios (`scenario.rs`)

- `ScenarioCfg { key, bindings: HashMap<String,String>, runs?: u64, blocks?: BlocksCfg, deployer: Arc<DeployerCfg> }`.
- Nested structure: scenarios can contain sub‑scenarios under `scenarios:`; keys compose as `parent.child` in the parsed map.
- Bindings
  - Parent bindings are inherited; children cannot change an existing binding’s value (shadowing causes `ParentBindingShadowedError`).
  - Values support interpolation through `Context`.
- Blocks and runs
  - `runs` is optional `u64`.
  - `blocks` can be the compact range form (`[a..b]`, `[..b]`, `[a..]`) or an object with `{ range: [...], interval: u32 }` (see Blocks below). Parser accepts either string or structured map formats.
- Deployer
  - A scenario can pick a deployer explicitly (`deployer: <key>`) or implicitly by name (scenario key matches a deployer key). A child scenario must not change the deployer chosen by its parent (`ParentDeployerShadowedError`).
- Mutations
  - `update_bindings(Map<String,String>)` updates only existing binding keys across the scenario path; new keys are appended at the lowest level of the current scenario path. Changes are persisted into YAML, then the scenario is re‑parsed and returned.

### Blocks (`blocks.rs`)

- `BlockCfg`: `Number(BlockNumber) | Genesis | Latest` with helper `to_block_number(latest)`.
- `BlockRangeCfg { start: BlockCfg, end: BlockCfg }` with `validate(latest)`.
- `BlocksCfg`: `SimpleRange(BlockRangeCfg)` or `RangeWithInterval { range, interval }`.
- Expansion: `expand_to_block_numbers(latest)` produces a concrete vector of block numbers after validating the range.

### Deployments (`deployment.rs`)

- `DeploymentCfg { key, scenario: Arc<ScenarioCfg>, order: Arc<OrderCfg> }`.
- Parsing
  - Respects optional order builder context for “current deployment” to allow per‑deployment scoping when documents contain many deployments.
  - Ensures the selected order and scenario share the same deployer; otherwise returns `ParseDeploymentConfigSourceError::NoMatch`.
  - Helper `parse_order_key(docs, deployment_key)` extracts the order name for a deployment.


## Order Builder Configuration (`order_builder.rs`)

The order builder DSL configures per‑deployment user inputs (fields), deposit presets/validation, and “select tokens” behavior for orders rendered in the UI.

- Source types (pure data) to transform into runtime types:
  - `OrderBuilderConfigSourceCfg { name, description, deployments: Map<deployment_name, OrderBuilderDeploymentSourceCfg> }`.
  - `OrderBuilderDeploymentSourceCfg { name, description, deposits: [OrderBuilderDepositSourceCfg], fields: [OrderBuilderFieldDefinitionSourceCfg], select_tokens?: [OrderBuilderSelectTokensCfg] }`.
  - `OrderBuilderDepositSourceCfg { token: String, presets?: [String], validation?: DepositValidationCfg }`.
  - `OrderBuilderFieldDefinitionSourceCfg { binding, name, description?, presets?: [OrderBuilderPresetSourceCfg], default?, show_custom_field?, validation? }`.
  - `OrderBuilderPresetSourceCfg { name?, value }`.
  - Validation enums:
    - `FieldValueValidationCfg::Number { minimum?, exclusive_minimum?, maximum?, exclusive_maximum? }`.
    - `FieldValueValidationCfg::String { min_length?, max_length? }`.
    - `FieldValueValidationCfg::Boolean`.
    - `DepositValidationCfg { minimum?, exclusive_minimum?, maximum?, exclusive_maximum? }`.

- Runtime types (used by app/wasm):
  - `OrderBuilderCfg { name, description, deployments: Map<deployment_name, OrderBuilderDeploymentCfg> }`.
  - `OrderBuilderDeploymentCfg { key, deployment: Arc<DeploymentCfg>, name, description, deposits: [OrderBuilderDepositCfg], fields: [OrderBuilderFieldDefinitionCfg], select_tokens? }`.
  - `OrderBuilderDepositCfg { token?: Arc<TokenCfg>, presets?, validation? }`.
  - `OrderBuilderFieldDefinitionCfg { binding, name, description?, presets?: [OrderBuilderPresetCfg], default?, show_custom_field?, validation? }`.
  - `OrderBuilderPresetCfg { id, name?, value }`.

- Parsing
  - `OrderBuilderCfg::parse_from_yaml_optional(documents, context)` traverses a `builder:` map, applying deployment scoping from order builder context and order/deployment context, and builds an `OrderBuilderCfg` if present.
  - Helper queries: `check_builder_key_exists`, `parse_deployment_keys`, `parse_order_details`, `parse_deployment_details`, `parse_field_presets`, `parse_select_tokens`.
  - Integrates tokens (if present) to resolve deposit token references to `Arc<TokenCfg>`. With “select‑tokens”, the context is seeded with allowed token keys so orders may omit YAML token entries.


## Charts and Plot DSL

Charts let scenarios expose visualizations for analysis.

- `ChartCfg { key, scenario: Arc<ScenarioCfg>, plots?: [PlotCfg], metrics?: [MetricCfg] }`.
  - `MetricCfg { label, description?, unit_prefix?, unit_suffix?, value, precision? }`.
  - Parses `charts:` map; each chart may reference a scenario by key (defaults to the chart key) and include `plots:` and `metrics:`.

- Plot source DSL (`plot_source.rs`)
  - `PlotCfg { title?, subtitle?, marks: [MarkCfg], x?: AxisOptionsCfg, y?: AxisOptionsCfg, margin/…? }`.
  - `MarkCfg` variants: `Dot(DotOptionsCfg)`, `Line(LineOptionsCfg)`, `RectY(RectYOptionsCfg)`.
  - Mark options may carry an optional transform.
  - Transforms: `TransformCfg::HexBin(HexBinTransformCfg)` and `TransformCfg::BinX(BinXTransformCfg)` each with `TransformOutputsCfg` and `options` specific to the transform (e.g. `bin-width` for hexbin, `thresholds` for binx).
  - `AxisOptionsCfg` configures labels/anchors for axes.

- The chart parser (`chart.rs`) wires the YAML layout into these types and performs validation (e.g., presence and types of transform `content`, numeric fields, required `marks` vectors). Numeric strings are validated via `ChartCfg::validate_u32` where appropriate.


## Remote Data

Remote sections allow enriching local YAML with data fetched at runtime and merged into the model.

- Remote Networks (`remote_networks.rs` + `remote/chains.rs`)
  - YAML shape:
    ```yaml
    using-networks-from:
      source-name:
        url: https://…
        format: chainid
    ```
  - `RemoteNetworksCfg` parses the above. `fetch_networks(map)` issues HTTP GET requests and parses JSON into `ChainId` structures.
  - `ChainId::try_into_network_cfg` converts a chain into a `NetworkCfg` by selecting the first acceptable RPC URL (non‑WS and without `API_KEY` placeholders). Key is the chain’s `shortName`.
  - Conflicts in produced network keys across sources yield `ConflictingNetworks`.

- Remote Tokens (`remote_tokens.rs` + `remote/tokens.rs`)
  - YAML shape:
    ```yaml
    using-tokens-from:
      - https://…/tokenlist.json
      - https://…/another-list.json
    ```
  - `RemoteTokensCfg::parse_from_yaml_optional` collects/validates URLs (deduplicating across documents). `fetch_tokens(networks, cfg)` GETs each URL, merges token lists while de‑duplicating by `(chainId, addressLowercase)`, and then converts known chain IDs into `TokenCfg` by matching against provided `networks`.
  - Key format: `"<network-key>-<TokenName-with-dashes>-<addressLowercase>"`.
  - Conflicting keys across URLs produce `ConflictingTokens`.

Both remote features are surfaced to parsing via the `Context`/`Cache`. `OrderbookYaml` and `DotrainYaml` expose helper methods to fetch/update cache and then merge these into subsequent parses.


## Miscellaneous Modules

- `accounts.rs`: named EVM addresses in `accounts:`. Simple map with validation and duplicate checks.
- `sentry.rs`: optional root scalar `sentry` read as string and normalized to `Option<bool>` by `OrderbookYaml`.
- `spec_version.rs`: required root scalar `version` and helpers to compare to the current spec version (constant "3").
- `test.rs`: test helpers to construct mock networks/tokens/deployers/orderbooks.
- `unit_test.rs`: auxiliary types (`UnitTestConfigSource`, `TestConfigSource`, `ScenarioConfigSource`) used by the test harness in other crates. `TestConfigSource::into_test_config()` converts the simplified source into a `TestConfig` with an embedded `ScenarioCfg`.


## Concurrency and Update Semantics

Every config item carries `document: Arc<RwLock<StrictYaml>>` pointing to the YAML document where it originated. “Update” methods (`TokenCfg::update_address`, `NetworkCfg::update_rpcs`, `OrderCfg::update_vault_id`, `OrderCfg::populate_vault_ids`, `MetaboardCfg::add_record_to_yaml`, `SubgraphCfg::add_record_to_yaml`, `TokenCfg::add_record_to_yaml`/`remove_record_from_yaml`) acquire a write lock, mutate the tree in place, and update the in‑memory struct for consistency. All read operations acquire read locks.


## Duplicate Keys and Merging

For map‑shaped sections parsed across multiple documents, the crate enforces unique keys within the merged result. If a key appears twice across any documents, parsing fails with `YamlError::KeyShadowing(key, section)`. This guarantees deterministic provenance and avoids silent overrides.


## Error Reporting Philosophy

All parser and validator errors convert to `YamlError` or module‑specific `*Parse*Error` enums with a `to_readable_msg()` that is safe to show to end users. `FieldErrorKind::{Missing, InvalidType, InvalidValue}` always include a precise human‑readable location (e.g., "order 'MyOrder'", "output index '0' in order 'MyOrder'", "order builder deployment 'MyDeployment'").


## WASM/TypeScript Interop

When building for `wasm32`, many types derive `Tsify` and implement WASM trait helpers via `wasm_bindgen_utils::impl_wasm_traits!`. This allows the web UI to import the same configuration model with strong typing (including union types and optional fields) and to consume results produced by this crate.


## Typical Workflows

- Validate orderbook YAML for networks/tokens/etc. and query objects:
  1. Load strings into `OrderbookYaml::new([...], OrderbookYamlValidation::full())`.
  2. Optionally fetch remote networks/tokens, store in cache, and then call `get_*` methods to retrieve `NetworkCfg`, `TokenCfg`, `OrderbookCfg`, etc.
  3. Use update helpers to persist changes back to YAML documents.

- Validate dotrain YAML and build a deployment plan:
  1. Load strings into `DotrainYaml::new([...], DotrainYamlValidation::full())`.
  2. Resolve `OrderCfg`, `ScenarioCfg`, and `DeploymentCfg`; ensure network/deployer invariants hold.
  3. Optional Order Builder: parse `OrderBuilderCfg` scoped to a specific deployment and seed context with `select-tokens`.
  4. Optional Charts: parse `ChartCfg` for visualization.


## YAML Shape Reference (informative)

- Root keys seen across modules (some optional depending on usage):
  - `version: "3"`
  - `using-networks-from: { name: { url, format: chainid } }`
  - `using-tokens-from: [ url, ... ]`
  - `networks: { key: { rpcs: [url,...], chain-id, label?, network-id?, currency? } }`
  - `tokens: { key: { network, address, decimals?, label?, symbol? } }`
  - `subgraphs: { key: url }`
  - `orderbooks: { key: { address, network?, subgraph?, label?, deployment-block } }`
  - `metaboards: { key: url }`
  - `deployers: { key: { address, network? } }`
  - `accounts: { key: address }`
  - `orders: { key: { inputs: [{ token, vault-id? }, ...], outputs: [...], deployer?, orderbook? } }`
  - `scenarios: { key: { bindings: {k:v}, runs?, blocks?, deployer?, scenarios?: {...} } }`
  - `deployments: { key: { scenario, order } }`
  - `builder: { name, description, deployments: { key: { name, description, deposits: [...], fields: [...], select-tokens?: [...] } } }`
  - `charts: { key: { scenario?, plots?: {...}, metrics?: [...] } }`
  - `sentry: true|false|1|0`


## Testing

The crate ships extensive unit tests for every parser and update path, including error paths with precise messages. Test helpers in `src/test.rs` construct mock networks/deployers/tokens/orderbooks; parser modules provide happy‑path and negative test cases (duplicate keys, missing/invalid fields, range validation for blocks, builder validation, remote fetch flows with http mocks, etc.).


## Summary

The settings crate provides a single, well‑typed interface over YAML configuration for the Rain Orderbook ecosystem: robust parsing across multiple files, strict validation with user‑friendly errors, safe in‑place updates, optional remote augmentation, contextual interpolation, builder and chart DSLs, and WASM interop. Other crates consume these types to build CLIs, runtimes, and UIs without re‑implementing YAML logic.
