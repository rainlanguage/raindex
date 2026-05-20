// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployStartBlockPolygonTest
/// @notice Binary-searches for the Raindex deploy block on Polygon.
/// Skipped in CI due to RPC rate limits; the isStartBlock test verifies
/// correctness cheaply.
contract LibRaindexDeployStartBlockPolygonTest is Test {
    function testStartBlockPolygon() external {
        vm.skip(vm.envOr("CI", false));
        vm.createSelectFork(LibRainDeploy.POLYGON);
        uint256 startBlock = LibRainDeploy.findDeployBlock(
            vm, LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH, 0
        );
        assertEq(startBlock, LibRaindexDeploy.RAINDEX_START_BLOCK_POLYGON);
    }
}
