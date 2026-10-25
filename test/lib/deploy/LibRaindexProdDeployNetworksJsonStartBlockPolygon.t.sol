// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibRaindexProdDeploy} from "src/lib/deploy/LibRaindexProdDeploy.sol";

/// @title LibRaindexProdDeployNetworksJsonStartBlockPolygonTest
/// @notice The startBlock for matic in subgraph/networks.json MUST match
/// the constant in LibRaindexProdDeploy.
contract LibRaindexProdDeployNetworksJsonStartBlockPolygonTest is Test {
    function testNetworksJsonStartBlockPolygon() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        uint256 startBlock = vm.parseJsonUint(json, ".matic.Raindex.startBlock");
        assertEq(
            startBlock, LibRaindexProdDeploy.RAINDEX_START_BLOCK_POLYGON, "networks.json startBlock mismatch: matic"
        );
    }
}
