// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployStartBlockBaseTest
/// @notice Binary-searches for the OrderBook deploy block on Base.
/// Skipped in CI due to RPC rate limits; the isStartBlock test verifies
/// correctness cheaply.
contract LibOrderBookDeployStartBlockBaseTest is Test {
    function testStartBlockBase() external {
        vm.skip(vm.envOr("CI", false));
        vm.createSelectFork(LibRainDeploy.BASE);
        uint256 startBlock = LibRainDeploy.findDeployBlock(
            vm, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH, 0
        );
        assertEq(startBlock, LibOrderBookDeploy.ORDERBOOK_START_BLOCK_BASE);
    }
}
