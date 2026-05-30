#!/bin/bash
# SPDX-License-Identifier: LicenseRef-DCL-1.0
# SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd

set -euxo pipefail

mkdir -p subgraph/abis
cp out/RaindexV6.sol/RaindexV6.json subgraph/abis/Raindex.json
cp out/ERC20.sol/ERC20.json subgraph/abis/ERC20.json
cp out/DecimalFloat.sol/DecimalFloat.json subgraph/abis/DecimalFloat.json

cd subgraph
npm ci
graph codegen
