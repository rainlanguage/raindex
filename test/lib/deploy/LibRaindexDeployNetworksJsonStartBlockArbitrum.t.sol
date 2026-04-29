// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployNetworksJsonStartBlockArbitrumTest
/// @notice The startBlock for arbitrum-one in subgraph/networks.json MUST match
/// the constant in LibRaindexDeploy.
contract LibRaindexDeployNetworksJsonStartBlockArbitrumTest is Test {
    function testNetworksJsonStartBlockArbitrum() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        uint256 startBlock = vm.parseJsonUint(json, ".arbitrum-one.Raindex.startBlock");
        assertEq(
            startBlock, LibRaindexDeploy.RAINDEX_START_BLOCK_ARBITRUM, "networks.json startBlock mismatch: arbitrum-one"
        );
    }
}
