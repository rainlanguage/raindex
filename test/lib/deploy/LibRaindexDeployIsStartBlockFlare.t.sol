// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployIsStartBlockFlareTest
/// @notice Cheaply validates the known Flare start block constant by checking
/// that code appears at the block but not the block before.
contract LibRaindexDeployIsStartBlockFlareTest is Test {
    function testIsStartBlockFlare() external {
        vm.createSelectFork(LibRainDeploy.FLARE);
        assertTrue(
            LibRainDeploy.isStartBlock(
                vm,
                LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS,
                LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH,
                LibRaindexDeploy.RAINDEX_START_BLOCK_FLARE
            ),
            "not start block: Flare"
        );
    }
}
