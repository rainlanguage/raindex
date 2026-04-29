// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibOrderBookDeploy} from "src/lib/deploy/LibOrderBookDeploy.sol";

/// @title LibOrderBookDeployNetworksJsonAddressesTest
/// @notice Every address in subgraph/networks.json MUST match the deterministic
/// OrderBook deploy address from LibOrderBookDeploy.
contract LibOrderBookDeployNetworksJsonAddressesTest is Test {
    function testNetworksJsonAddresses() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        string[] memory networks = vm.parseJsonKeys(json, "$");
        for (uint256 i = 0; i < networks.length; i++) {
            string memory path = string.concat(".", networks[i], ".OrderBook.address");
            address addr = vm.parseJsonAddress(json, path);
            assertEq(
                addr,
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                string.concat("networks.json address mismatch: ", networks[i])
            );
        }
    }
}
