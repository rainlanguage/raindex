// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexDeployNetworksJsonAddressesTest
/// @notice Every address in subgraph/networks.json MUST match the deterministic
/// Raindex deploy address from LibRaindexDeploy.
contract LibRaindexDeployNetworksJsonAddressesTest is Test {
    function testNetworksJsonAddresses() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        string[] memory networks = vm.parseJsonKeys(json, "$");
        for (uint256 i = 0; i < networks.length; i++) {
            string memory path = string.concat(".", networks[i], ".Raindex.address");
            address addr = vm.parseJsonAddress(json, path);
            assertEq(
                addr,
                LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS,
                string.concat("networks.json address mismatch: ", networks[i])
            );
        }
    }
}
