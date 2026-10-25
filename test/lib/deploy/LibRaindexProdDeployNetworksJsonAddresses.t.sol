// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibRaindexProdDeploy} from "src/lib/deploy/LibRaindexProdDeploy.sol";

/// @title LibRaindexProdDeployNetworksJsonAddressesTest
/// @notice Every address in subgraph/networks.json MUST match the deterministic
/// Raindex deploy address from LibRaindexProdDeploy.
contract LibRaindexProdDeployNetworksJsonAddressesTest is Test {
    function testNetworksJsonAddresses() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        string[] memory networks = vm.parseJsonKeys(json, "$");
        for (uint256 i = 0; i < networks.length; i++) {
            string memory path = string.concat(".", networks[i], ".Raindex.address");
            address addr = vm.parseJsonAddress(json, path);
            assertEq(
                addr,
                LibRaindexProdDeploy.RAINDEX_DEPLOYED_ADDRESS,
                string.concat("networks.json address mismatch: ", networks[i])
            );
        }
    }
}
