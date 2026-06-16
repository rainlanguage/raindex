# Repository Guidelines

## Project Structure & Module Organization
- Solidity contracts: `src/`, tests in `test/` with fixtures in `test-resources/`.
- Rust workspace: `crates/*` (e.g., `cli`, `common`, `bindings`, `js_api`, `quote`, `subgraph`, `settings`, `math`, `integration_tests`).
- JavaScript/Svelte: `packages/*` — `webapp`, `ui-components`, `orderbook` (wasm wrapper published to npm).
- Subgraph and tooling: `subgraph/`, `script/`, helper scripts like `prep-all.sh`.

## Build, Test, and Development Commands
- Bootstrap: `./prep-all.sh` (installs deps and builds workspaces).
- Rust: `cargo build --workspace`; tests: `cargo test`.
- Solidity (Foundry): `forge build`; tests: `forge test`.
- Webapp: `cd packages/webapp && npm run dev`.
- JS workspaces (top-level): `npm run test`, `npm run build:ui`, `npm run build:orderbook`.
- WASM bundle: `rainix-wasm-artifacts`.

## Coding Style & Naming Conventions
- Rust: format with `cargo fmt --all`; lint with `rainix-rs-static` (preconfigured flags included). Crates/modules use `snake_case`; types `PascalCase`.
- TS/Svelte: `npm run format`, `npm run lint`, `npm run check` in each package. Components `PascalCase.svelte`; files otherwise kebab/snake as appropriate.
- Solidity: `forge fmt`; compiler `solc 0.8.25` (see `foundry.toml`).

## Testing Guidelines
- Rust: `cargo test`; integration tests live in `crates/integration_tests`. Prefer `insta` snapshots and `proptest` where helpful.
- TS/Svelte: `npm run test` (Vitest). Name files `*.test.ts`/`*.spec.ts`.
- Solidity: `forge test` (add fuzz/property tests where relevant).

## Commit & Pull Request Guidelines
- PRs must: describe scope/approach, link issues, include screenshots/GIFs for UI changes, update/ add tests, and pass CI.
- Quick preflight: `npm run lint-format-check:all && rainix-rs-static`.

## Security & Configuration Tips
- Never commit secrets. Copy `.env.example` files (root, `packages/webapp`) and populate `PUBLIC_WALLETCONNECT_PROJECT_ID` as required.

## Agent-Specific Instructions
- Prefer syntax-aware search with ast-grep: Rust `sg --lang rust -p '<pattern>'`; TS `sg --lang ts -p '<pattern>'`.
- Architecture context: when working in any directory, check for an `ARCHITECTURE.md` file in the current working directory and read it first to understand local architecture before making changes.
