> NOTE: Before using this guide, read the repository root `AGENTS.md` for
> authoritative agent instructions.

# Raindex – Agent Guide (Concise)

Always run commands via Nix: `nix develop -c <command>`. Never cancel
long-running tasks (45–90 min builds, 30+ min tests).

## 1. Dependency readiness (quick check)

```bash
nix develop -c cargo build
nix develop -c cargo build --target wasm32-unknown-unknown --lib -r --workspace \
  --exclude raindex_cli --exclude raindex_integration_tests
nix develop -c npm install
nix develop -c npm run build:raindex
nix develop -c npm run build:ui
```

If any step fails due to earlier lint/test issues, use the fallback below.

## 2. Development loop

- Edit code
- Rebuild dependencies you touched:
  - Rust used by `@rainlanguage/raindex` →
    `nix develop -c npm run build:raindex`
  - `@rainlanguage/ui-components` →
    `nix develop -c npm run build -w @rainlanguage/ui-components`
- Run targeted tests and lints for changed areas

## Reference: tests and lints by area

| Area                                     | Build (if needed)                                             | Lint/Check                                                                         | Tests                                                          |
| ---------------------------------------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Rust crates (`crates/*`)                 | `nix develop -c cargo build`                                  | `nix develop -c cargo clippy --workspace --all-targets --all-features -D warnings` | `nix develop -c cargo test --workspace` or `--package <crate>` |
| Raindex TS (`packages/raindex`)          | `nix develop -c npm run build:raindex`                        | `nix develop -c npm run check -w @rainlanguage/raindex`                            | `nix develop -c npm run test -w @rainlanguage/raindex`         |
| UI components (`packages/ui-components`) | `nix develop -c npm run build -w @rainlanguage/ui-components` | `nix develop -c npm run svelte-lint-format-check -w @rainlanguage/ui-components`   | `nix develop -c npm run test -w @rainlanguage/ui-components`   |
| Webapp (`packages/webapp`)               | `nix develop -c npm run build -w @rainlanguage/webapp`        | `nix develop -c npm run svelte-lint-format-check -w @rainlanguage/webapp`          | `nix develop -c npm run test -w @rainlanguage/webapp`          |
| Solidity contracts                       | `nix develop -c forge build`                                  | —                                                                                  | `nix develop -c forge test`                                    |

## Frontend verification (required when frontend changes)

- If you modify frontend code or functionality affecting the frontend, you MUST
  provide a screenshot of the built webapp reflecting your change.
- Build and preview:

```bash
nix develop -c npm run build -w @rainlanguage/webapp
nix develop -c npm run preview -w @rainlanguage/webapp
```

- If you are unable to build the webapp, you MUST provide the concrete reasons
  and errors. Workarounds are not acceptable.

## 3. End-of-session gate (comprehensive)

Partial commits are OK during the session. Before your final commit of the
session, fully mirror CI:

```bash
# Bootstrap — if either of these fails, jump to the fallback below.
nix develop -c forge soldeer install
nix develop -c forge build
nix develop -c npm run lint-format-check:all
nix develop -c npm run build:raindex   # if Rust/raindex changed
nix develop -c npm run build:ui
nix develop -c cargo test --workspace
nix develop -c npm run test
nix develop -c forge test
```

## 4. Push gate (quick recheck)

Do a short verification right before pushing:

```bash
nix develop -c npm run lint-format-check:all
nix develop -c npm run test
nix develop -c cargo test --workspace
```

## Fallback if the end-of-session gate fails early

If the gate dies before it reaches the tests, the dependency tree is not in
place. Rebuild it from scratch, one workspace at a time, so you can see which
step breaks:

```bash
nix develop -c forge soldeer install
nix develop -c forge build
nix develop -c raindex-ui-components-prelude
nix develop .#wasm-shell -c bash -c '
  set -euxo pipefail
  npm install --no-check
  npm run build -w @rainlanguage/raindex
  npm run build -w @rainlanguage/ui-components
  npm run build -w @rainlanguage/webapp
'
```

Solidity dependencies are Soldeer packages, resolved into `dependencies/` by
`forge soldeer install` (`foundry.toml` sets `libs = ['dependencies']`). They
are plain source drops — there is nothing to prepare or build inside one, so
there is no per-dependency step here.

`npm install --no-check` must run from the workspace root, and `.#wasm-shell` is
a slim shell whose `shellHook` does not run it for you — hence the explicit
first line inside the `bash -c`. `forge` and `raindex-ui-components-prelude`
come from the default `nix develop -c` shell; `.#wasm-shell` has neither.

If the problem is stale committed artifacts (`meta/`, `src/generated/`,
`crates/*/abis`, `subgraph/`) rather than a build that will not run, that is the
`copy-artifacts` regen cascade and not this bootstrap — follow the runbook in
the root `CLAUDE.md`.

Goal: all CI checks in `.github/workflows` pass. Be patient with long
builds/tests and never commit with failing lint/tests.
