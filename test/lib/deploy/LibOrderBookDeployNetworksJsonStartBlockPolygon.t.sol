// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployNetworksJsonStartBlockPolygonTest
/// @notice The startBlock for matic in subgraph/networks.json MUST match
/// the constant in LibOrderBookDeploy.
contract LibOrderBookDeployNetworksJsonStartBlockPolygonTest is Test {
    function testNetworksJsonStartBlockPolygon() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        uint256 startBlock = vm.parseJsonUint(json, ".matic.OrderBook.startBlock");
        assertEq(
            startBlock, LibOrderBookDeploy.ORDERBOOK_START_BLOCK_POLYGON, "networks.json startBlock mismatch: matic"
        );
    }
}
