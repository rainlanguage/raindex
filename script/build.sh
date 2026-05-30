#!/bin/bash
# SPDX-License-Identifier: LicenseRef-DCL-1.0
# SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd

# Post-CopyArtifacts hook for rainix-copy-artifacts. Regenerates all committed
# derived artifacts: subgraph ABIs + codegen, bindings ABIs (read by `sol!` in
# crates/bindings/src/lib.rs), test_fixtures ABIs + IMulticall3.sol. Strips
# forge JSON to just `.abi` (or `.abi + .bytecode` where alloy ::deploy()
# needs it) — bytecode/metadata/id embed build-host paths and aren't
# deterministic across runners, which would defeat the diff gate; consumers
# (alloy sol!, graph CLI, matchstick) only read those fields anyway.

set -euxo pipefail

# Pass the whole pipeline via `bash -c '...'` rather than `bash <<INNER`:
# graph codegen / npm ci consume the heredoc body off stdin otherwise,
# turning the rest of the script into bogus argv for the codegen command.
nix develop github:rainlanguage/rainix#subgraph-shell -c bash -c '
set -euxo pipefail

mkdir -p subgraph/abis
jq "{abi}" out/RaindexV6.sol/RaindexV6.json > subgraph/abis/Raindex.json
jq "{abi}" out/ERC20.sol/ERC20.json > subgraph/abis/ERC20.json
jq "{abi}" out/DecimalFloat.sol/DecimalFloat.json > subgraph/abis/DecimalFloat.json

(cd subgraph && npm ci && graph codegen)

mkdir -p crates/bindings/abis
jq "{abi}" out/IRaindexV6.sol/IRaindexV6.json > crates/bindings/abis/IRaindexV6.json
jq "{abi}" out/RaindexV6.sol/RaindexV6.json > crates/bindings/abis/RaindexV6.json
jq "{abi}" out/ERC20.sol/ERC20.json > crates/bindings/abis/ERC20.json
jq "{abi}" out/IERC20Metadata.sol/IERC20Metadata.json > crates/bindings/abis/IERC20Metadata.json
jq "{abi}" out/IInterpreterStoreV3.sol/IInterpreterStoreV3.json > crates/bindings/abis/IInterpreterStoreV3.json

mkdir -p crates/test_fixtures/abis
jq "{abi, bytecode}" out/RaindexV6.sol/RaindexV6.json > crates/test_fixtures/abis/RaindexV6.json
jq "{abi, bytecode}" out/RaindexV6SubParser.sol/RaindexV6SubParser.json > crates/test_fixtures/abis/RaindexV6SubParser.json

mkdir -p crates/test_fixtures/contracts
cp dependencies/forge-std-1.16.1/src/interfaces/IMulticall3.sol crates/test_fixtures/contracts/IMulticall3.sol
'
