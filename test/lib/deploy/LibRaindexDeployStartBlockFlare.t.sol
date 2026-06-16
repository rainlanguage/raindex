// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployStartBlockFlareTest
/// @notice Binary-searches for the Raindex deploy block on Flare.
/// Skipped in CI due to RPC rate limits; the isStartBlock test verifies
/// correctness cheaply.
contract LibRaindexDeployStartBlockFlareTest is Test {
    function testStartBlockFlare() external {
        vm.skip(vm.envOr("CI", false));
        vm.createSelectFork(LibRainDeploy.FLARE);
        uint256 startBlock = LibRainDeploy.findDeployBlock(
            vm, LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH, 0
        );
        assertEq(startBlock, LibRaindexDeploy.RAINDEX_START_BLOCK_FLARE);
    }
}
