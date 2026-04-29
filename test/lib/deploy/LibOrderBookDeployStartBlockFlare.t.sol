// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployStartBlockFlareTest
/// @notice Binary-searches for the OrderBook deploy block on Flare.
/// Skipped in CI due to RPC rate limits; the isStartBlock test verifies
/// correctness cheaply.
contract LibOrderBookDeployStartBlockFlareTest is Test {
    function testStartBlockFlare() external {
        vm.skip(vm.envOr("CI", false));
        vm.createSelectFork(LibRainDeploy.FLARE);
        uint256 startBlock = LibRainDeploy.findDeployBlock(
            vm, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH, 0
        );
        assertEq(startBlock, LibOrderBookDeploy.ORDERBOOK_START_BLOCK_FLARE);
    }
}
