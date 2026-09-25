// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.2/src/Test.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployNetworksJsonStartBlockRobinhoodTest
/// @notice The startBlock for robinhood in subgraph/networks.json MUST
/// match the constant in LibRaindexDeploy.
contract LibRaindexDeployNetworksJsonStartBlockRobinhoodTest is Test {
    function testNetworksJsonStartBlockRobinhood() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        uint256 startBlock = vm.parseJsonUint(json, ".robinhood.Raindex.startBlock");
        assertEq(
            startBlock,
            LibRaindexDeploy.RAINDEX_START_BLOCK_ROBINHOOD,
            "networks.json startBlock mismatch: robinhood"
        );
    }
}
