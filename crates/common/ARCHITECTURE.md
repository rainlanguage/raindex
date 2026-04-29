# raindex_common — Architecture & Reference

This crate provides the shared core for the Raindex toolchain across native (CLI, services) and WebAssembly (browser) targets. It bundles higher‑level orchestration around:

- Parsing and composing Rain language (Rainlang) and DOTRAIN YAML frontmatter.
- Building and executing Raindex contract calls (add/remove orders, deposit, withdraw), including Ledger support on native.
- Querying raindex state via the subgraph and flattening results for display/CSV export.
- A WASM‑friendly API surface for UI apps (via `wasm_bindgen_utils` and `tsify`).
- Developer ergonomics: LSP helpers, fuzz/unit‑test runners, and EVM fork utilities to parse/evaluate Rainlang and replay transactions.

The library is built as `rlib` and `cdylib`. A Git commit identifier is embedded as `GH_COMMIT_SHA` for traceability.


## Module Overview

- `add_order` — Compose Rainlang from DOTRAIN, parse to bytecode via on‑chain Parser, generate `addOrder3` call parameters, execute or simulate on a fork.
- `remove_order` — Convert subgraph `SgOrder` to `removeOrder3` call, execute or return calldata.
- `deposit` — ERC20 allowance check/approve and `deposit3` call builder/executor.
- `withdraw` — `withdraw3` call builder/executor and calldata generator.
- `transaction` — Shared tx args (RPCs, chain ID, fees), Ledger provider creation (native), and `WriteContractParameters` helpers.
- `erc20` — Typed ERC20 reads (decimals/name/symbol/allowance/balance), multicall token info, and robust revert decoding.
- `subgraph` — Thin wrapper to instantiate a raindex subgraph client from a URL.
- `raindex_client/*` — High‑level client over raindex YAML config: find networks/raindexes, fetch orders, vaults, trades, transactions; quote orders; prepare batch withdraw calldata; expose WASM‑friendly structs. The `local_db/` subtree is split into `state.rs` (runtime state, query routing via `LocalDbState`/`QuerySource`/`SyncReadiness`) and `status.rs` (UI status‑reporting types).
- `dotrain_order` — Parse and validate a DOTRAIN config; compose scenarios/deployments to Rainlang; fetch authoring metadata and pragma words; merge additional settings.
- `rainlang` — Compose Rainlang from a DOTRAIN string + bindings; optional fork‑based parser that returns encoded bytecode.
- `dotrain_add_order_lsp` — Language‑services integration for Rainlang/DOTRAIN (hover, completion, diagnostics, and fork‑parse problems).
- `types/*` — Flattened view models for CSV/export: orders, order takes, vault balance changes, token vaults, plus shared errors and constants.
- `csv` — Generic `TryIntoCsv` trait for serializing vectors of typed rows.
- `utils/*` — Formatting helpers for amounts (`U256` → string) and timestamps (seconds → UTC string).
- `fuzz` — Fuzzing/evaluation harness over Rainlang entrypoints and charts (native), with WASM‑serializable result shapes.
- `replays` — EVM fork utilities to replay an on‑chain transaction and convert raw traces to `RainEvalResult`.
- `unit_tests` — Programmatic runner that executes DOTRAIN pre/calculate‑io/handle‑io/post entrypoints on a fork for deterministic tests.
- `test_helpers` — Sample DOTRAIN used in tests.
- `lib` — Public module wiring, re‑exports, and `GH_COMMIT_SHA` env binding.

Target gating is used extensively:
- Native only: Ledger, EVM forking/eval, transaction execution, some tests.
- WASM: `tsify` types and `wasm_export` bindings, alternate `tokio` features, and JS‑oriented getters.


## Key Data Flow & Responsibilities

### 1) DOTRAIN → Rainlang → Bytecode (add_order)
- Inputs: DOTRAIN (YAML frontmatter + Rainlang sections), selected scenario/deployment (from `raindex_app_settings`), and bindings.
- `AddOrderArgs::compose_to_rainlang` uses `rainlang::compose_to_rainlang` to produce the Rainlang snippet for order entrypoints (`calculate-io`, `handle-io`).
- Parser address is discovered via `DISPair::from_deployer`; the Rainlang text is parsed by `ParserV2::parse_text` over provided RPCs to produce bytecode.
- Metadata is generated as a Rain Meta V1 document containing `RainlangSourceV1` and CBOR‑encoded with `rain-metadata`.
- The `addOrder3` call is assembled with Evaluable (interpreter/store/bytecode), inputs/outputs vaults, random nonce/secret, and a post task (`handle-add-order`) compiled similarly.
- Execution paths:
  - Native: Build `WriteContractParameters` and execute with `WriteTransaction` via a Ledger provider, or simulate on a fork (`Forker`) per RPC.
  - Any target: Return ABI‑encoded calldata for external submission.

### 2) Remove Order
- `RemoveOrderArgs` converts `SgOrder` to `removeOrder3Call` via subgraph traits.
- Native execution mirrors `add_order` (Ledger provider + `WriteTransaction`), or return calldata only.

### 3) Deposit / Approve
- `DepositArgs` holds token, vaultId, `Float` amount, and decimals.
- `read_allowance` uses a readable provider to query ERC20 allowance.
- If allowance differs from the desired amount, `execute_approve` submits an ERC20 `approve` that overwrites the allowance with the target value.
- `execute_deposit` builds and submits `deposit3` with the exact `Float` amount converted to fixed decimal units.
- Both execution functions accept a status callback for progress.

### 4) Withdraw
- `WithdrawArgs` maps directly to `withdraw3` call. Provides execution and calldata helpers.

### 5) Transaction Plumbing
- `TransactionArgs` encapsulates RPCs, chain ID, optional Ledger derivation index, and EIP‑1559 fee caps.
- `try_fill_chain_id` reads chain ID via the readable client when absent.
- Native: `try_into_ledger_client` picks the first working RPC, connects a Ledger signer/provider, and returns the signer address.
- `try_into_write_contract_parameters` packages any typed `SolCall` into the `WriteContractParameters` used by `WriteTransaction`.

### 6) ERC20 Reads & Error Decoding
- `ERC20` wraps an `IERC20Instance` over a read provider (`mk_read_provider`). Methods: `decimals`, `name`, `symbol`, `allowance`, `balanceOf`, and `token_info` (multicall over all three metadata calls).
- Revert data is decoded via `rain_error_decoding` to produce human‑readable errors. For multicall `CallFailed`, the revert is decoded and mapped.

### 7) Subgraph Client Creation
- `SubgraphArgs::to_subgraph_client` parses the URL and returns a raindex subgraph client bound to that endpoint.

### 8) Raindex Client (raindex YAML–driven API)
- `RaindexClient::create` (async on WASM, aliased to `new` in JS) parses one or more raindex YAML strings using `RaindexYaml`, optionally with full validation. When the YAML declares `local-db-sync` sections and DB callbacks are provided, the constructor automatically sets up the local DB and starts the sync scheduler.
- `LocalDbState` encapsulates the local DB handle, scheduler, `SyncReadiness` (tracks which chains have completed a sync cycle), and the set of configured chain IDs. Query routing uses `QuerySource::LocalDb` vs `QuerySource::Subgraph` — each chain is routed to exactly one source based on configuration and readiness.
- Derives a map of networks and raindexes to build `MultiSubgraphArgs` groupings for cross‑network queries.
- Exposed operations (with WASM bindings):
  - YAML accessors: get unique chain IDs, networks, raindexes by address, accounts, and RPC URLs.
  - Orders: list with filters/pagination across networks, fetch by hash, fetch orders created in a transaction.
  - Quotes: compute per‑pair quotes for an order (`get_order_quotes` under the hood), with formatted ratios and inverses.
  - Vaults: list/query vaults for an order or raindex, fetch balance changes, prepare withdraw multicall calldata, format balances.
  - Trades and transactions: list trades (with optional time bounds), fetch trade detail, transaction detail.
- Conversion helpers map subgraph types (`Sg*`) to WASM/JS‑friendly shapes (`Raindex*`) and back when needed.
- Error surface `RaindexError` normalizes failures from YAML parsing, hex parsing, subgraph network errors, ERC20 reads, float/parse errors, etc., and provides user‑facing messages via `to_readable_msg`.

### 9) DOTRAIN Order Utilities
- `DotrainOrder::create` extracts frontmatter from a DOTRAIN text, validates spec version, hydrates remote networks/tokens if configured, and builds both `DotrainYaml` and `RaindexYaml` caches.
- Compose helpers:
  - `compose_scenario_to_rainlang` and `compose_deployment_to_rainlang` create Rainlang with scenario/deployment bindings applied.
  - `compose_scenario_to_post_task_rainlang` produces post‑task (`handle-add-order`) Rainlang.
- Queries:
  - `get_pragmas_for_scenario` and `get_contract_authoring_meta_v2_for_scenario` read pragma addresses and authoring metadata (supports metaboard subgraph).
- Errors map to readable messages for UI consumption.

### 10) Rainlang Composition & Fork Parse
- `rainlang::compose_to_rainlang` uses language services’ meta store and `RainDocument::create` with optional rebinding to render specified entrypoints.
- Native: `fork_parse::parse_rainlang_on_fork` lazily initializes a global `FORKER`, selects/creates an EVM fork per RPC, and returns ABI‑encoded expression config. Reverts are decoded to typed errors.

### 11) LSP Integration for Add Order
- `DotrainAddOrderLsp` stores a `TextDocumentItem` plus optional bindings. Methods:
  - `hover`, `completion` via `RainLanguageServices`.
  - `problems` composes entrypoints; if composition fails, reports structured errors; else optionally fork‑parses Rainlang against a selected deployment to provide diagnostics.

### 12) Fuzzing, Unit Tests, and Replays
- `fuzz::FuzzRunner` composes entrypoints for scenarios (replacing elided bindings with random data), creates a fork at configured block(s), and runs multiple iterations. Results flatten to tables for charting.
- `unit_tests::TestRunner` orchestrates a four‑phase evaluation (pre → calculate‑io → handle‑io → post) with controlled context injection; designed for deterministic contract‑level testing of Rainlang logic.
- `replays::TradeReplayer` builds a fork and replays a given transaction hash, returning converted `RainEvalResult` traces.

### 13) Flattened Types and CSV
- `types/order_detail_extended.rs` — Wraps `SgOrder` and optionally decoded Rainlang source from meta.
- `types/orders_list_flattened.rs` — `OrderFlattened` for list views: timestamps, owner, interpreters/stores, valid vault IDs and token symbols, and first add‑event transaction, with `TryFrom<SgOrder>`.
- `types/order_takes_list_flattened.rs` — `OrderTakeFlattened` summarizing a trade (input/output amounts, tokens, and formatted values).
- `types/vault_balance_change_flattened.rs` — Normalized view of vault balance changes across deposits/withdrawals/trades with signed formatted amounts and type labels.
- `types/token_vault_flattened.rs` — Vault + token metadata and formatted balance.
- All implement `TryIntoCsv<T>` on `Vec<T>` to produce headers + rows via `serde::Serialize` and `csv::Writer`.

### 14) Utilities
- `utils/amount_formatter.rs` — `format_amount_u256` using `alloy::primitives::utils::format_units` and `remove_trailing_zeros` to clean fixed‑decimal strings. Error type wraps unit and parse errors.
- `utils/timestamp.rs` — Formatting helpers: `format_bigint_timestamp_display` (string seconds → UTC) and `format_timestamp_display_utc` with strict bounds and parse errors.


## Public API Highlights

- Call builders and calldata
  - `AddOrderArgs::{try_into_call, get_add_order_call_parameters, execute, get_add_order_calldata, simulate_execute}`
  - `RemoveOrderArgs::{execute, get_rm_order_calldata}`
  - `DepositArgs::{read_allowance, execute_approve, execute_deposit}` and `WithdrawArgs::{execute, get_withdraw_calldata}`
  - `RaindexVaultsList::get_withdraw_calldata` for multicall batch withdraws

- DOTRAIN/Rainlang
  - `DotrainOrder::create`, `compose_*_to_rainlang`, YAML accessors, pragma/meta queries
  - `rainlang::compose_to_rainlang` and native `fork_parse::parse_rainlang_on_fork`

- Raindex client (selected)
  - YAML: `get_unique_chain_ids`, `get_all_networks`, `get_network_by_chain_id`, `get_raindex_by_address`, `get_all_accounts`
  - Orders: `get_orders`, `get_order_by_hash`, `get_add_orders_for_transaction`
  - Quotes: `RaindexOrder::get_quotes`
  - Vaults: `get_vaults_list`, `get_raindex_vaults_list`, `RaindexVault::{get_balance_changes, get_deposit_calldata, get_withdraw_calldata}`, `RaindexVault::get_account_balance`
  - Trades/Tx: `RaindexOrder::{get_trades_list, get_trades_count}`, `RaindexOrder::get_trade_detail`, `RaindexClient::get_transaction`

- CSV & types
  - `TryIntoCsv` implemented on vectors of flattened types for export.

- Constants
  - `RAINDEX_ORDER_ENTRYPOINTS` = [`calculate-io`, `handle-io`]
  - `RAINDEX_ADDORDER_POST_TASK_ENTRYPOINTS` = [`handle-add-order`]
  - `types::vault::NO_SYMBOL` — fallback when token symbol is absent
  - `GH_COMMIT_SHA` — compile‑time commit id


## Error Handling

Each domain defines focused error enums with `thiserror::Error`:
- `AddOrderArgsError`, `RemoveOrderArgsError`, `DepositError`, `WritableTransactionExecuteError`, `TransactionArgsError` — transactional and parsing failures.
- `erc20::Error` — revert decoding, provider/multicall errors, and typed ABI decode errors.
- `RaindexError` — umbrella for YAML/subgraph/hex/float/amount formatting/ERC20/tx errors; exposes `to_readable_msg` for UI.
- `types::FlattenError` — conversion/formatting failures when building flattened view models.
- `TryDecodeRainlangSourceError` — meta decoding and content validation of Rainlang source.
- Fuzz/unit‑test/replay errors normalize fork issues, abi‑decoded reverts, and YAML/spec mismatches.

Errors typically bubble with `#[from]` to preserve sources and are turned into WASM‑friendly structures via `From<...> for WasmEncodedError` where applicable.


## WASM vs Native Surface

- WASM builds derive `Tsify` for public structs and expose getters with JS‑friendly types (`Hex`, `Address`, `number[]`, `Map<..>`) using `wasm_bindgen_utils` annotations.
- Native builds enable:
  - `tokio` full features.
  - Ledger transport and provider setup for transaction execution.
  - EVM forking (`rain_interpreter_eval`) to parse/eval Rainlang, simulate calls, and replay txs.

Where functionality cannot run in WASM, equivalent calldata generation methods are provided so frontends can submit via their own providers.


## Subgraph, YAML, and Quoting

- Subgraph clients are constructed from raindex YAML to ensure per‑network routing and naming.
- The client supports multi‑network fan‑out via `MultiRaindexSubgraphClient` and paginated queries.
- Quotes use on‑chain multicall under the hood; results are converted to `Float` and pre‑formatted strings, including inverse ratios and empty/infinite cases.


## Testing & Quality Notes

- The crate has extensive unit tests across modules (including mocked RPC/subgraph) validating:
  - Error surfaces and message mapping.
  - ABI encoding/decoding of calls and calldata consistency.
  - CSV/flattening correctness and edge cases (missing symbols, invalid timestamps, invalid hex/ABI).
  - DOTRAIN spec version gating and settings merging.
  - ERC20 revert handling and multicall decoding.
  - EVM‑fork‑based parsing/evaluation and replay correctness (native).
- Fuzz and unit‑test runners demonstrate example patterns for evaluating Rainlang over controlled contexts.


## Notable Dependencies (workspace crates)

- `raindex_bindings` — Strongly‑typed ABI for Raindex and ERC20 contracts (`addOrder3`, `removeOrder3`, `deposit3`, `withdraw3`, multicall).
- `raindex_subgraph_client` — GraphQL types and clients for raindex data; provides `Sg*` models and helpers.
- `raindex_app_settings` — DOTRAIN/raindex YAML structures, validation, and spec versioning.
- `raindex_quote` — Batch quote engine for orders.
- `rain_interpreter_*` — Parser, eval, DISP pair, bindings used to compile/evaluate Rainlang.
- `rain_metadata` — CBOR‑encoded metadata with magic prefixes; used to embed Rainlang source.
- `rain_error_decoding` — ABI error decoding to readable types/names.
- `rain_math_float` — Arbitrary‑precision floats with hex encoding for on‑chain compatibility and pretty formatting.
- `alloy` & `alloy_ethers_typecast` — EVM primitives, providers, signers (Ledger), and Read/Write contract helpers.
- `wasm_bindgen_utils`, `tsify` — WASM/JS interop helpers and type generation.


## Typical End‑to‑End Flows

- Add an order (native or UI):
  1) Build `DotrainOrder` from DOTRAIN text; pick a deployment; build `AddOrderArgs` via `new_from_deployment`.
  2) Compose Rainlang and parse to bytecode via parser address (from `DISPair`).
  3) Generate metadata bytes and `addOrder3Call` with post task.
  4) Execute with `WriteTransaction` (native) or obtain `abi_encode()` and submit via external provider (WASM).

- Remove an order:
  1) Fetch `SgOrder` from subgraph; convert to `removeOrder3Call` and execute or export calldata.

- Deposit/Withdraw:
  1) For deposit, check allowance and, when it differs from the intended amount, approve the exact target before calling `deposit3`.
  2) For withdraw, construct `withdraw3` calldata or execute; batch multiple via `RaindexVaultsList::get_withdraw_calldata`.

- Explore raindex data in a UI:
  1) Instantiate `RaindexClient` from YAML configs.
  2) List orders across networks; fetch vaults, trades, and quotes for selected orders.
  3) Render CSV exports using `TryIntoCsv` on flattened types.


## Constants & Build Flags

- `GH_COMMIT_SHA` is set at compile time via the `COMMIT_SHA` env var and can be displayed for “About/Version” screens.
- `crate-type = ["rlib", "cdylib"]` enables consumption as a Rust lib and a WASM/FFI‑ready dynamic library.
- Conditional `tokio` features are selected for WASM vs native targets in Cargo.toml.


## Notes & Caveats

- Many error types intentionally capture inner variants to preserve exact failure causes (RPC transport vs. ABI decode vs. parse errors). UIs should prefer `to_readable_msg` for end users.
- Fork‑based helpers rely on at least one working RPC; functions return descriptive errors if all providers fail.
- Non‑WASM execution paths interact with hardware Ledger devices and therefore are compiled out for browser targets.
- Some performance‑related queries are stubbed/disabled (see TODOs referencing issue 1989) and kept for future reinstatement.


## File Map (quick reference)

- API layers
  - `src/add_order.rs`, `src/remove_order.rs`, `src/deposit.rs`, `src/withdraw.rs`, `src/transaction.rs`, `src/erc20.rs`
- DOTRAIN/Rainlang
  - `src/dotrain_order.rs`, `src/rainlang.rs`, `src/dotrain_add_order_lsp.rs`
- Client & data access
  - `src/raindex_client/` (orders, quotes, vaults, trades, transactions, YAML)
  - `src/raindex_client/local_db/state.rs` (runtime state: `LocalDbState`, `QuerySource`, `SyncReadiness`, `ClassifiedChains`)
  - `src/raindex_client/local_db/status.rs` (UI status types: `LocalDbStatus`, `SchedulerState`, `NetworkSyncStatus`, etc.)
  - `src/subgraph.rs`
- Data views & export
  - `src/types/` (flattened rows + errors), `src/csv.rs`, `src/utils/*`
- Eval/fork tooling
  - `src/fuzz/*`, `src/replays.rs`, `src/unit_tests.rs`
- Surfacing
  - `src/lib.rs` (pub mod graph, wasm re‑exports), `GH_COMMIT_SHA`

This document covers all publicly exposed modules and their roles so new contributors and integrators can navigate the crate quickly and correctly wire native/WASM consumers.


Last Updated: 2026-03-03 — Updated for RaindexClient local DB refactor (single-step async construction, deterministic query routing, state.rs/status.rs split).
