// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployIsStartBlockBaseTest
/// @notice Cheaply validates the known Base start block constant by checking
/// that code appears at the block but not the block before.
contract LibOrderBookDeployIsStartBlockBaseTest is Test {
    function testIsStartBlockBase() external {
        vm.createSelectFork(LibRainDeploy.BASE);
        assertTrue(
            LibRainDeploy.isStartBlock(
                vm,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH,
                LibOrderBookDeploy.ORDERBOOK_START_BLOCK_BASE
            ),
            "not start block: Base"
        );
    }
}
