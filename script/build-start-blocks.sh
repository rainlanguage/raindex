#!/bin/bash
# SPDX-License-Identifier: LicenseRef-DCL-1.0
# SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd

# Rewrites subgraph/networks.json from chain (script/BuildSubgraphNetworks.sol)
# and points subgraph/subgraph.yaml at the candidate RaindexV6. Needs an
# archive RPC per supported network in the `[rpc_endpoints]` env vars.
set -euo pipefail

ADDR=$(grep -aoE 'DEPLOYED_ADDRESS = address\(0x[0-9a-fA-F]{40}' src/generated/candidate/RaindexV6.sol | grep -aoE '0x[0-9a-fA-F]{40}')
[ -n "$ADDR" ] || { echo "could not read DEPLOYED_ADDRESS from src/generated/candidate/RaindexV6.sol" >&2; exit 1; }

forge script script/BuildSubgraphNetworks.sol
sed -i -E "s/(address: \")0x[0-9a-fA-F]+(\")/\1${ADDR}\2/" subgraph/subgraph.yaml
