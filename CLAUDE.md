Before working on anything in this repository, read and follow the @AGENTS.md
file.

# Merge / Publish / Deploy runbook

The full cause-and-effect chain for a contract change: edit source → **deploy**
the new bytecode to its new CREATE2 address → that source change **publishes** a
new soldeer version → which needs its deploy constants pinned → **merge**. All of
it is routine and pre-authorized; don't stop to ask about address/codehash churn.

## Merging a PR

- Run `gh` via `nix shell nixpkgs#gh --command gh` (not on global PATH).
- **Required status checks** (the `main` ruleset): `test`, `subgraph-test`,
  `test-js-bindings`, `wasm-artifacts`, `wasm-browser-test`, `wasm-test`. NOT
  required: `rainix-sol / test`, `rainix-sol / static`, `copy-artifacts`,
  `Deploy-Preview-Push`. The ~30-min vercel preview does **not** gate a merge —
  don't wait on it. But still understand *every* red before merging; never
  `--admin` over an unexplained failure (verify each is the expected one).
- A merge-gate hook requires a `Reviewed <9-char-current-head-sha>: <substantive
  review>` PR comment **before** merging, bound to the **current** head. Re-post
  it after any branch update or new commit (the SHA changes).
- raindex is `REVIEW_REQUIRED` and the author can't self-approve →
  `gh pr merge <n> --merge --admin` (needs explicit owner authorization
  per-PR). rainix merges with plain `gh pr merge <n> --merge`.
- `--squash`/`--rebase` are hook-blocked (the default merge commit preserves
  history). `gh run rerun` is blocked — re-trigger CI with an empty commit +
  push. Use `git checkout -f -B` (not `git reset --hard`, hook-blocked). Commit
  with `git -c commit.gpgsign=false commit --no-verify`.

## Deploying a contract (the bytecode cascade)

Any source change to a CREATE2 / zoltu-deployed contract changes its on-chain
address + codehash. Do the whole cascade without asking:

1. **Regenerate artifacts** — the canonical `rainix-copy-artifacts` sequence, in
   `nix develop github:rainlanguage/rainix#sol-shell`: `forge soldeer install` →
   `script/build-meta.sh` → `forge script script/BuildPointers.sol` →
   `forge build` → `forge script script/CopyArtifacts.sol --ffi` (if present) →
   `./script/build.sh` → `forge fmt`. Stage **all** changed artifacts (pointers,
   `crates/*/abis`, subgraph, meta) or `copy-artifacts` drifts red.
2. **Deploy each changed suite**:
   `gh workflow run manual-sol-artifacts.yaml --ref <branch> -f suite=<suite>`.
   Suites: `raindex`, `subparser`, `route-processor`,
   `arb-generic-pool-order-taker`, `arb-route-processor-order-taker`,
   `arb-generic-pool-flash-borrower`. One dispatch deploys to all five chains
   (arbitrum / base / base_sepolia / flare / polygon).
   - **Never run two deploys concurrently** — they share one deployer EOA and
     race the nonce (`EOA nonce changed unexpectedly` → a partial deploy that
     only hits the first chains). Serialize: dispatch, wait for completion,
     dispatch the next.
   - A deploy run shows `completed/failure` but the **on-chain deploy
     succeeded** if the log contains `ONCHAIN EXECUTION COMPLETE & SUCCESSFUL` —
     only the etherscan `--verify` step failed (cosmetic).
3. **Re-check** — `testProdDeploy*` ("X not deployed") fork-tests stay red until
   the new addresses have code on-chain. After deploying, re-trigger PR CI with
   an empty commit.

## Publishing to soldeer (autopublish)

- `package-release.yaml` calls rainix `rainix-autopublish.yaml@main` on every
  `main` push. It content-gates the soldeer package (normalized hash of
  `forge soldeer push --dry-run` zip vs the published revision), auto-bumps
  `foundry.toml [package].version`, tags `sol-vX`, and publishes — **only** when
  the published `src/` content actually changed (a workflow/CI-only change does
  not publish).
- Every published version must carry a full pinned deploy-constant suite in
  `LibRaindexDeploy`: `*_DEPLOYED_{ADDRESS,CODEHASH}_<x>_<y>_<z>` (address +
  codehash for all six deployed contracts).
  `testAllPublishedSoldeerTagsHaveAFullConstantSuite` (via
  `script/check-published-deploy-constants.sh`, which queries the live registry)
  reds **main and every PR** on any published tag missing them. Values come from
  the current `src/generated/*.pointers.sol`; contracts unchanged since the
  prior version reuse its values. No new deploy is needed if `testProdDeploy*` is
  already green (the contracts were deployed by their own PRs).
- The content gate **strips** the pinned `*_<ver>` constant blocks before
  hashing — so pinning a version's constants is not itself a content change
  (otherwise pinning would publish the next version, which would need its own
  constants pinned: an endless bump → pin → bump loop). Only real bytecode
  changes publish.
- **To avoid a red-main follow-up**, pre-pin the *next* version's constants in
  the same contract-change PR. `next = patch(max(foundry.toml version,
  registry-latest)) + 1`. The pre-pinned constants are harmless before the
  publish (the test only checks *published* tags) and satisfy it the instant the
  version lands — so the fix, the redeploy, and the constant pin ship as one PR
  with no red main.
