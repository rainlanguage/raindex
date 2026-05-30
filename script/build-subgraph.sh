#!/bin/bash
# SPDX-License-Identifier: LicenseRef-DCL-1.0
# SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd

set -euxo pipefail

mkdir -p subgraph/abis
# Strip to just `.abi` — the rest of forge's artifact JSON
# (bytecode/metadata/id) embeds build-host-specific paths and isn't
# deterministic across runners, which would defeat the
# rainix-copy-artifacts diff gate. Graph CLI / matchstick only read
# `.abi` anyway.
jq '{abi}' out/RaindexV6.sol/RaindexV6.json > subgraph/abis/Raindex.json
jq '{abi}' out/ERC20.sol/ERC20.json > subgraph/abis/ERC20.json
jq '{abi}' out/DecimalFloat.sol/DecimalFloat.json > subgraph/abis/DecimalFloat.json

cd subgraph
npm ci
graph codegen
