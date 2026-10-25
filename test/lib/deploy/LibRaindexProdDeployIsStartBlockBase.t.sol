// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibRainDeploy} from "rain-deploy-0.1.22/src/lib/LibRainDeploy.sol";
import {LibBlockSearch} from "rain-deploy-0.1.22/src/lib/LibBlockSearch.sol";
import {LibRaindexProdDeploy} from "src/lib/deploy/LibRaindexProdDeploy.sol";

/// @title LibRaindexProdDeployIsStartBlockBaseTest
/// @notice Cheaply validates the known Base start block constant by checking
/// that code appears at the block but not the block before.
contract LibRaindexProdDeployIsStartBlockBaseTest is Test {
    function testIsStartBlockBase() external {
        vm.createSelectFork(LibRainDeploy.BASE);
        assertTrue(
            LibBlockSearch.isStartBlock(
                vm,
                LibRaindexProdDeploy.RAINDEX_DEPLOYED_ADDRESS,
                LibRaindexProdDeploy.RAINDEX_DEPLOYED_CODEHASH,
                LibRaindexProdDeploy.RAINDEX_START_BLOCK_BASE
            ),
            "not start block: Base"
        );
    }
}
