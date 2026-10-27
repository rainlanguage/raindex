// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {RaindexDeploySuites} from "src/abstract/RaindexDeploySuites.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";
import {LibRaindexSubgraphNetwork} from "src/lib/deploy/LibRaindexSubgraphNetwork.sol";

/// @title NetworksJsonCandidateAddressTest
/// @notice Every subgraph/networks.json entry is a supported network's subgraph
/// network, holding only the candidate RaindexV6 address.
contract NetworksJsonCandidateAddressTest is Test, RaindexDeploySuites {
    function testNetworksJsonCandidateAddress() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        string[] memory keys = vm.parseJsonKeys(json, "$");
        string[] memory networks = supportedNetworks();
        for (uint256 i = 0; i < keys.length; i++) {
            bool supported = false;
            for (uint256 j = 0; j < networks.length; j++) {
                if (
                    keccak256(bytes(keys[i]))
                        == keccak256(bytes(LibRaindexSubgraphNetwork.subgraphNetwork(networks[j])))
                ) {
                    supported = true;
                }
            }
            assertTrue(supported, string.concat("networks.json entry for no supported network: ", keys[i]));
            string memory entry = string.concat(".", keys[i]);
            assertEq(vm.parseJsonKeys(json, entry).length, 1, string.concat("networks.json data sources: ", keys[i]));
            assertEq(
                vm.parseJsonAddress(json, string.concat(entry, ".Raindex.address")),
                LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS,
                string.concat("networks.json address is not the candidate: ", keys[i])
            );
        }
    }
}
