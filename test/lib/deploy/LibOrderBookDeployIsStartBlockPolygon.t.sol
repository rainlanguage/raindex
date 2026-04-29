// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployIsStartBlockPolygonTest
/// @notice Cheaply validates the known Polygon start block constant by checking
/// that code appears at the block but not the block before.
contract LibOrderBookDeployIsStartBlockPolygonTest is Test {
    function testIsStartBlockPolygon() external {
        vm.createSelectFork(LibRainDeploy.POLYGON);
        assertTrue(
            LibRainDeploy.isStartBlock(
                vm,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH,
                LibOrderBookDeploy.ORDERBOOK_START_BLOCK_POLYGON
            ),
            "not start block: Polygon"
        );
    }
}
