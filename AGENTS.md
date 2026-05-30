# Repository Guidelines

## Project Structure & Module Organization

- Solidity contracts: `src/`, tests in `test/` with fixtures in
  `test-resources/`.
- Rust workspace: `crates/*` (e.g., `cli`, `common`, `bindings`, `js_api`,
  `quote`, `subgraph`, `settings`, `math`, `integration_tests`).
- JavaScript/Svelte: `packages/*` — `webapp`, `ui-components`, `raindex` (wasm
  wrapper published to npm).
- Subgraph and tooling: `subgraph/`, `script/`.

## Build, Test, and Development Commands

- Bootstrap:
  `nix develop -c forge soldeer install && nix develop -c forge build && nix develop -c raindex-ui-components-prelude && nix develop -c npm run build -w @rainlanguage/raindex && nix develop -c npm run build -w @rainlanguage/ui-components && nix develop -c npm run build -w @rainlanguage/webapp`
  (installs deps and builds workspaces; see README for the multi-line form).
- Rust: `cargo build --workspace`; tests: `cargo test`.
- Solidity (Foundry): `forge build`; tests: `forge test`.
- Webapp: `cd packages/webapp && npm run dev`.
- JS workspaces (top-level): `npm run test`, `npm run build:ui`,
  `npm run build:raindex`.
- WASM bundle: `rainix-wasm-artifacts`.

## Coding Style & Naming Conventions

- Rust: format with `cargo fmt --all`; lint with `rainix-rs-static`
  (preconfigured flags included). Crates/modules use `snake_case`; types
  `PascalCase`.
- TS/Svelte: `npm run format`, `npm run lint`, `npm run check` in each package.
  Components `PascalCase.svelte`; files otherwise kebab/snake as appropriate.
- Solidity: `forge fmt`; compiler `solc 0.8.25` (see `foundry.toml`).

## Testing Guidelines

- Rust: `cargo test`; integration tests live in `crates/integration_tests`.
  Prefer `insta` snapshots and `proptest` where helpful.
- TS/Svelte: `npm run test` (Vitest). Name files `*.test.ts`/`*.spec.ts`.
- Solidity: `forge test` (add fuzz/property tests where relevant).

## Commit & Pull Request Guidelines

- PRs must: describe scope/approach, link issues, include screenshots/GIFs for
  UI changes, update/ add tests, and pass CI.
- Quick preflight: `npm run lint-format-check:all && rainix-rs-static`.

## Security & Configuration Tips

- Never commit secrets. Copy `.env.example` files (root, `packages/webapp`) and
  populate `PUBLIC_WALLETCONNECT_PROJECT_ID` as required.

## CI & Workflow Conventions

GitHub Actions workflows in `.github/workflows/` follow a consistent shape. New
work should match.

### Slim shells over default devshell

- Use `nix develop .#wasm-shell` for rust+node work (cargo wasm build, npm
  workspace builds, vitest, typedoc), and `.#subgraph-shell` for graph CLI work.
  Both are local re-exports of `rainix`'s slim shells via `flake.nix`, so they
  pin to the `flake.lock` rainix rev rather than the live
  `github:rainlanguage/rainix#...` reference (which tracks rainix `main` and
  bypasses `flake.lock`).
- Default `nix develop -c` enters the heavy full devshell — only use when you
  legitimately need the full toolchain (e.g., `copilot-setup-steps`).
- `npm install --no-check` MUST run from the **workspace root** (not
  `packages/<x>`) for npm workspaces resolution. Default devshell's shellHook
  does this on shell entry; slim shells don't, so do it explicitly as the first
  command inside the `bash -c '...'`.

### Nix infrastructure

- Install nix with `nixbuild/nix-quick-install-action@v30`.
- Pull/push the shared `rainlanguage` Cachix with `cachix/cachix-action@v15`
  (set `continue-on-error: true` and `useDaemon: false` alongside
  `cache-nix-action`, else the nix DB corrupts).
- Cache the nix store with `nix-community/cache-nix-action@v7` keyed by
  `**/*.nix` + `**/flake.lock` hashes.
- Do NOT use `DeterminateSystems/nix-installer-action` or
  `DeterminateSystems/flakehub-cache-action` — they don't share the
  `rainlanguage` Cachix that every other workflow warms.

### Build caches

- `Swatinem/rust-cache@v2` after the nix-store cache step for any cargo work
  (rust-cache caches `~/.cargo/{registry,git}` + `target/`).
- `actions/cache@v4` over `~/.npm` keyed by `**/package-lock.json` for any
  `npm install` work.

### Committed derived artifacts

- ABIs that `sol!` macros read are committed under `crates/*/abis/` so cargo can
  build in slim shells without `forge soldeer install` + `forge build` on the
  test path.
- jq-strip forge JSON to deterministic fields only: `{abi}` if no `::deploy()`
  is called; `{abi, bytecode: (.bytecode | {object,
  linkReferences})}` if it
  is. **Drop `sourceMap`** — it embeds a file-ID that depends on solc's input
  ordering and differs across runners.
- Vendored solidity files (e.g.,
  `crates/test_fixtures/contracts/
  IMulticall3.sol`) keep `sol!` chains
  working in slim shells without needing the soldeer dep tree on disk.
- `script/build.sh` regenerates ALL committed derived artifacts;
  `rainix-copy-artifacts` runs it then `git diff --exit-code` to catch drift.

### Shell quoting

Pass multi-command pipelines as `nix develop ...#X -c bash -c '...'`, NOT as
`nix develop ...#X -c bash <<INNER ... INNER`. Inner commands (`graph codegen`,
`npm ci`) read stdin and consume the heredoc body, turning later lines of the
script into bogus argv. Use single-quoted outer `-c` body with double-quoted jq
filters (`jq "{abi}" ...`) inside.

## Agent-Specific Instructions

- Prefer syntax-aware search with ast-grep: Rust
  `sg --lang rust -p '<pattern>'`; TS `sg --lang ts -p '<pattern>'`.
- Architecture context: when working in any directory, check for an
  `ARCHITECTURE.md` file in the current working directory and read it first to
  understand local architecture before making changes.
