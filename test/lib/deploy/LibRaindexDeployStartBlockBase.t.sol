// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployStartBlockBaseTest
/// @notice Binary-searches for the Raindex deploy block on Base.
/// Skipped in CI due to RPC rate limits; the isStartBlock test verifies
/// correctness cheaply.
contract LibRaindexDeployStartBlockBaseTest is Test {
    function testStartBlockBase() external {
        vm.skip(vm.envOr("CI", false));
        vm.createSelectFork(LibRainDeploy.BASE);
        uint256 startBlock = LibRainDeploy.findDeployBlock(
            vm, LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH, 0
        );
        assertEq(startBlock, LibRaindexDeploy.RAINDEX_START_BLOCK_BASE);
    }
}
