#!/usr/bin/env bash
#
# Vercel build convention.
#
# This script produces the deployable Vercel *prebuilt* output for
# `packages/webapp` (consumed by `vercel deploy --prebuilt packages/webapp`).
#
# It is the single entry point a rainix Vercel reusable workflow references by a
# fixed path, so the build logic lives in the repo while rainix owns the generic
# wrapper around it (checkout, nix install, the nix-store / cargo / npm caches,
# and the Vercel pull + deploy). Any repo wanting a Vercel webapp deploy via
# rainix provides its own `script/vercel-build.sh` with the same contract:
#
#   - run from the repo root, with nix on PATH;
#   - `PUBLIC_WALLETCONNECT_PROJECT_ID` exported by the caller;
#   - on success, leaves the Vercel prebuilt output ready under packages/webapp.
#
set -euxo pipefail

# Workspace dependencies: wasm bindings plus the raindex / ui-components / webapp
# libraries, built in the wasm shell from the repo root.
nix develop .#wasm-shell -c bash -c '
  set -euxo pipefail
  npm install --no-check
  npm run build -w @rainlanguage/raindex
  npm run build -w @rainlanguage/ui-components
  npm run build -w @rainlanguage/webapp
'

# Deployable webapp build, run from packages/webapp in the webapp shell so the
# flake reference resolves exactly as the previous inline workflow step did.
(cd packages/webapp && nix develop .#webapp-shell -c npm run build)
