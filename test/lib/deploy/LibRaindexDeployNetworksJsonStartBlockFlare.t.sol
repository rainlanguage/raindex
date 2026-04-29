// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployNetworksJsonStartBlockFlareTest
/// @notice The startBlock for flare in subgraph/networks.json MUST match
/// the constant in LibRaindexDeploy.
contract LibRaindexDeployNetworksJsonStartBlockFlareTest is Test {
    function testNetworksJsonStartBlockFlare() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        uint256 startBlock = vm.parseJsonUint(json, ".flare.Raindex.startBlock");
        assertEq(startBlock, LibRaindexDeploy.RAINDEX_START_BLOCK_FLARE, "networks.json startBlock mismatch: flare");
    }
}
