// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployIsStartBlockArbitrumTest
/// @notice Cheaply validates the known Arbitrum start block constant by checking
/// that code appears at the block but not the block before.
/// Skipped because free Arbitrum RPCs cannot serve full historical state needed
/// by rollFork. The start block was verified via manual cast binary search
/// against an archival RPC.
contract LibOrderBookDeployIsStartBlockArbitrumTest is Test {
    function testIsStartBlockArbitrum() external {
        vm.skip(true);
        vm.createSelectFork(LibRainDeploy.ARBITRUM_ONE);
        assertTrue(
            LibRainDeploy.isStartBlock(
                vm,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH,
                LibOrderBookDeploy.ORDERBOOK_START_BLOCK_ARBITRUM
            ),
            "not start block: Arbitrum"
        );
    }
}
