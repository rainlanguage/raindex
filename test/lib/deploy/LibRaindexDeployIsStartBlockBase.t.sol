// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployIsStartBlockBaseTest
/// @notice Cheaply validates the known Base start block constant by checking
/// that code appears at the block but not the block before.
contract LibRaindexDeployIsStartBlockBaseTest is Test {
    function testIsStartBlockBase() external {
        vm.createSelectFork(LibRainDeploy.BASE);
        assertTrue(
            LibRainDeploy.isStartBlock(
                vm,
                LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS,
                LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH,
                LibRaindexDeploy.RAINDEX_START_BLOCK_BASE
            ),
            "not start block: Base"
        );
    }
}
