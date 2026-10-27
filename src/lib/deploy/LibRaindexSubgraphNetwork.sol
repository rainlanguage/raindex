// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {LibRainDeploy} from "rain-deploy-0.1.25/src/lib/LibRainDeploy.sol";

/// @title LibRaindexSubgraphNetwork
/// @notice The subgraph network each deploy network is indexed under, as the
/// `subgraph/networks.json` key and the Ormi network id
/// (https://ormilabs.com/docs/supported-chains/subgraphs).
library LibRaindexSubgraphNetwork {
    /// @param network The deploy network with no subgraph network.
    error NoSubgraphNetwork(string network);

    /// @param network A deploy network, as an `[rpc_endpoints]` alias.
    /// @return The subgraph network it is indexed under.
    function subgraphNetwork(string memory network) internal pure returns (string memory) {
        bytes32 name = keccak256(bytes(network));
        if (name == keccak256(bytes(LibRainDeploy.ARBITRUM_ONE))) return "arbitrum-one";
        if (name == keccak256(bytes(LibRainDeploy.BASE))) return "base";
        if (name == keccak256(bytes(LibRainDeploy.BASE_SEPOLIA))) return "base-sepolia";
        if (name == keccak256(bytes(LibRainDeploy.BSC))) return "bsc";
        if (name == keccak256(bytes(LibRainDeploy.ETHEREUM))) return "mainnet";
        if (name == keccak256(bytes(LibRainDeploy.FLARE))) return "flare";
        if (name == keccak256(bytes(LibRainDeploy.HYPEREVM))) return "hyperevm";
        if (name == keccak256(bytes(LibRainDeploy.POLYGON))) return "matic";
        if (name == keccak256(bytes(LibRainDeploy.ROBINHOOD))) return "robinhood";
        revert NoSubgraphNetwork(network);
    }
}
