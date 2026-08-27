Read and follow @AGENTS.md.

# Merge / Publish / Deploy runbook

Contract change chain: edit source → **deploy** the new bytecode to its new
CREATE2 address → **merge**. Publishing to soldeer is a separate manual tag
release (below), never a side effect of merging. All of it is routine and
pre-authorized; don't stop to ask about address/codehash churn.

## Merging a PR

- Run `gh` via `nix shell nixpkgs#gh --command gh` (not on global PATH).
- Required checks (`main` ruleset): `test`, `subgraph-test`, `test-js-bindings`,
  `wasm-artifacts`, `wasm-browser-test`, `wasm-test`. `rainix-sol / *`,
  `git-clean`, `Deploy-Preview-Push` and the ~30-min vercel preview do NOT gate
  a merge. Still understand every red before merging; never `--admin` over an
  unexplained failure.
- A merge-gate hook requires a
  `Reviewed <9-char-head-sha>: <substantive
  review>` PR comment bound to the
  **current** head; re-post after any new commit.
- raindex is `REVIEW_REQUIRED` and the author can't self-approve →
  `gh pr merge <n> --merge --admin` (needs explicit owner authorization per-PR).
  rainix merges with plain `gh pr merge <n> --merge`.
- Merge with the default merge commit. Re-trigger CI with a fresh empty commit +
  push. Move a branch with `git checkout -f -B <branch> <ref>`. Commit with
  `git -c commit.gpgsign=false commit --no-verify`. (Squash/rebase merges, CLI
  workflow re-runs and history rewrites are hook-blocked — these are the allowed
  equivalents; don't spell the blocked forms out or you'll trip those hooks.)

## Deploying a contract (the bytecode cascade)

Any source change to a CREATE2/zoltu-deployed contract moves its on-chain
address + codehash. Do the whole cascade without asking:

1. Regenerate artifacts — run the `rainix-copy-artifacts` step sequence (see
   that workflow) in `nix develop github:rainlanguage/rainix#sol-shell`. Stage
   ALL changed artifacts or `git-clean` drifts red.
2. Deploy each changed suite:
   `gh workflow run manual-sol-artifacts.yaml --ref <branch> -f suite=<x>`. One
   dispatch deploys all seven chains. **Never run two deploys concurrently** —
   they share one deployer EOA and race the nonce, leaving a partial deploy;
   serialize dispatches. A `completed/failure` run still deployed if the log has
   `ONCHAIN EXECUTION COMPLETE & SUCCESSFUL` — only etherscan `--verify` failed
   (cosmetic).
3. If RaindexV6 moved, run `script/build-start-blocks.sh` — rewrites the
   `RAINDEX_START_BLOCK_*` constants, `subgraph/networks.json` and
   `subgraph/subgraph.yaml` (else `testIsStartBlock*` / `testNetworksJson*` /
   `testSubgraphYamlAddress` red). Uses foundry `rpc_endpoints` env vars,
   falling back to public archive RPCs.
4. `testProdDeploy*` fork tests stay red until the new addresses have code
   on-chain; re-trigger PR CI with an empty commit after deploying.

## Publishing to soldeer (tag release)

- Manual tag release via rainix `rainix-tag-release.yaml@main`
  (`package-release.yaml`, `sol-v*` tags). Merges to main do NOT publish.
- To cut a release: deploy changed suites first, then one PR that runs
  `forge script ./script/Build.sol --sig 'cutRelease()'` — freezing the
  candidate pins as `src/generated/<tag>/` — and bumps
  `foundry.toml [external.package].version` to match. Merge, then push tag
  `sol-v<version>`.
- The tag re-runs the fork suite and `release-guard` (version == tag, frozen dir
  present and byte-identical to `candidate/`, clean tree after regeneration, no
  later tag frozen), then publishes `raindex~<version>` and cuts the GitHub
  release.
- `src/generated/<tag>/` snapshots are append-only — the
  `frozen-snapshots-append-only` gate reds any PR that edits or deletes one.
