#!/bin/bash
# SPDX-License-Identifier: LicenseRef-DCL-1.0
# SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd

# Post-CopyArtifacts hook for rainix-copy-artifacts. Regenerates all committed
# derived artifacts: subgraph ABIs + codegen, bindings ABIs (read by `sol!` in
# crates/bindings/src/lib.rs). Strips forge JSON to just `.abi` —
# bytecode/metadata/id embed build-host paths and aren't deterministic across
# runners, which would defeat the diff gate; consumers (alloy sol!, graph CLI,
# matchstick) read `.abi` anyway.

set -euxo pipefail

nix develop github:rainlanguage/rainix#subgraph-shell -c bash <<'INNER'
set -euxo pipefail

mkdir -p subgraph/abis
jq '{abi}' out/RaindexV6.sol/RaindexV6.json > subgraph/abis/Raindex.json
jq '{abi}' out/ERC20.sol/ERC20.json > subgraph/abis/ERC20.json
jq '{abi}' out/DecimalFloat.sol/DecimalFloat.json > subgraph/abis/DecimalFloat.json

(cd subgraph && npm ci && graph codegen)

mkdir -p crates/bindings/abis
jq '{abi}' out/IRaindexV6.sol/IRaindexV6.json > crates/bindings/abis/IRaindexV6.json
jq '{abi}' out/RaindexV6.sol/RaindexV6.json > crates/bindings/abis/RaindexV6.json
jq '{abi}' out/ERC20.sol/ERC20.json > crates/bindings/abis/ERC20.json
jq '{abi}' out/IERC20Metadata.sol/IERC20Metadata.json > crates/bindings/abis/IERC20Metadata.json
jq '{abi}' out/IInterpreterStoreV3.sol/IInterpreterStoreV3.json > crates/bindings/abis/IInterpreterStoreV3.json
INNER
