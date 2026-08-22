Before working on anything in this repository, read and follow the @AGENTS.md
file.

# Merge / Publish / Deploy runbook

The full cause-and-effect chain for a contract change: edit source → **deploy**
the new bytecode to its new CREATE2 address → **merge**. Publishing to soldeer
is a separate, manual tag release (below), not a side effect of merging. All of
it is routine and pre-authorized; don't stop to ask about address/codehash
churn.

## Merging a PR

- Run `gh` via `nix shell nixpkgs#gh --command gh` (not on global PATH).
- **Required status checks** (the `main` ruleset): `test`, `subgraph-test`,
  `test-js-bindings`, `wasm-artifacts`, `wasm-browser-test`, `wasm-test`. NOT
  required: `rainix-sol / test`, `rainix-sol / static`, `copy-artifacts`,
  `Deploy-Preview-Push`. The ~30-min vercel preview does **not** gate a merge —
  don't wait on it. But still understand _every_ red before merging; never
  `--admin` over an unexplained failure (verify each is the expected one).
- A merge-gate hook requires a
  `Reviewed <9-char-current-head-sha>: <substantive
  review>` PR comment
  **before** merging, bound to the **current** head. Re-post it after any branch
  update or new commit (the SHA changes).
- raindex is `REVIEW_REQUIRED` and the author can't self-approve →
  `gh pr merge <n> --merge --admin` (needs explicit owner authorization per-PR).
  rainix merges with plain `gh pr merge <n> --merge`.
- Merge with the default merge commit (it preserves PR history). Re-trigger CI
  with a fresh empty commit + push. Move a branch with
  `git checkout -f -B <branch> <ref>`. Commit with
  `git -c commit.gpgsign=false commit --no-verify`. (Squash/rebase merges, CLI
  workflow re-runs, and history rewrites are blocked by hooks — these are the
  allowed equivalents; don't spell the blocked forms out, in docs or commands,
  or you'll trip those same hooks.)

## Deploying a contract (the bytecode cascade)

Any source change to a CREATE2 / zoltu-deployed contract changes its on-chain
address + codehash. Do the whole cascade without asking:

1. **Regenerate artifacts** — the canonical `rainix-copy-artifacts` sequence, in
   `nix develop github:rainlanguage/rainix#sol-shell`: `forge soldeer install` →
   `script/build-meta.sh` → `forge script script/Build.sol` → `forge build` →
   `forge script script/CopyArtifacts.sol --ffi` (if present) →
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
3. **Refresh start-block bookkeeping (only if RaindexV6 moved)** — a redeploy of
   the orderbook changes `RAINDEX_DEPLOYED_ADDRESS`, so the subgraph indexing
   bookkeeping (`RAINDEX_START_BLOCK_*` constants, `subgraph/networks.json`,
   `subgraph/subgraph.yaml`) goes stale and reds `testIsStartBlock*` /
   `testNetworksJson*` / `testSubgraphYamlAddress`. Run
   `script/build-start-blocks.sh` (binary-searches each chain's new deploy block
   over an archive RPC and rewrites all three). Needs per-chain archive RPCs via
   the foundry `rpc_endpoints` env vars (it falls back to public endpoints). Arb
   redeploys that don't move RaindexV6 (e.g. a LibRaindexArb change) skip this.
4. **Re-check** — `testProdDeploy*` ("X not deployed") fork-tests stay red until
   the new addresses have code on-chain. After deploying, re-trigger PR CI with
   an empty commit.

## Publishing to soldeer (tag release)

- Publishing is a manual tag release via rainix `rainix-tag-release.yaml@main`
  (`package-release.yaml`, `sol-v*` tags). Merges to main do NOT publish.
- To cut a release: deploy any changed suites first (above), then open a PR that
  runs `forge script ./script/Build.sol --sig 'cutRelease()'` — freezing the
  candidate pins as `src/generated/<tag>/` — and bumps
  `foundry.toml [external.package].version` to match, in one commit. Merge it,
  then push tag `sol-v<version>`.
- The tag re-runs the fork suite and `release-guard`: version == tag, the frozen
  `src/generated/<tag>/` present and byte-identical to `candidate/`, clean tree
  after regeneration, no later tag frozen. Then it publishes `raindex~<version>`
  to soldeer and cuts the GitHub release.
- `src/generated/<tag>/` snapshots are append-only — the
  `frozen-snapshots-append-only` gate in `rainix-sol-static` reds any PR that
  edits or deletes one.
