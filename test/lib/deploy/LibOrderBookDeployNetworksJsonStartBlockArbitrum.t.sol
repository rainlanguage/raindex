// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployNetworksJsonStartBlockArbitrumTest
/// @notice The startBlock for arbitrum-one in subgraph/networks.json MUST match
/// the constant in LibOrderBookDeploy.
contract LibOrderBookDeployNetworksJsonStartBlockArbitrumTest is Test {
    function testNetworksJsonStartBlockArbitrum() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        uint256 startBlock = vm.parseJsonUint(json, ".arbitrum-one.OrderBook.startBlock");
        assertEq(
            startBlock,
            LibOrderBookDeploy.ORDERBOOK_START_BLOCK_ARBITRUM,
            "networks.json startBlock mismatch: arbitrum-one"
        );
    }
}
