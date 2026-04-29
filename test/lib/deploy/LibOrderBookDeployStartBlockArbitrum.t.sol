// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployStartBlockArbitrumTest
/// @notice Binary-searches for the OrderBook deploy block on Arbitrum.
/// Always skipped because Foundry's rollFork maps to L1 block numbers, not L2.
/// The Arbitrum start block was found via manual binary search using
/// eth_getCode RPC calls against L2 block numbers.
contract LibOrderBookDeployStartBlockArbitrumTest is Test {
    /// Arbitrum Nitro genesis block. Archive RPCs can't serve blocks before this.
    uint256 constant ARBITRUM_NITRO_GENESIS_BLOCK = 22207817;

    // function testStartBlockArbitrum() external {
    //     vm.createSelectFork(LibRainDeploy.ARBITRUM_ONE);
    //     uint256 startBlock = LibRainDeploy.findDeployBlock(
    //         vm,
    //         LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
    //         LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH,
    //         ARBITRUM_NITRO_GENESIS_BLOCK
    //     );
    //     assertEq(startBlock, LibOrderBookDeploy.ORDERBOOK_START_BLOCK_ARBITRUM);
    // }
}
