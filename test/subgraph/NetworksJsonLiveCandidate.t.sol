// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibBlockSearch} from "rain-deploy-0.1.25/src/lib/LibBlockSearch.sol";
import {RaindexDeploySuites} from "src/abstract/RaindexDeploySuites.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";
import {LibRaindexSubgraphNetwork} from "src/lib/deploy/LibRaindexSubgraphNetwork.sol";

/// @title NetworksJsonLiveCandidateTest
/// @notice subgraph/networks.json has an entry for exactly the supported
/// networks where the candidate RaindexV6 is live, each at its deploy block.
contract NetworksJsonLiveCandidateTest is Test, RaindexDeploySuites {
    function testNetworksJsonLiveCandidate() external {
        address raindex = LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS;
        bytes32 codehash = LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH;
        string memory json = vm.readFile("subgraph/networks.json");
        string[] memory networks = supportedNetworks();
        for (uint256 i = 0; i < networks.length; i++) {
            string memory entry = string.concat(".", LibRaindexSubgraphNetwork.subgraphNetwork(networks[i]), ".Raindex");
            vm.createSelectFork(networks[i]);
            bool live = raindex.codehash == codehash;
            assertEq(
                vm.keyExistsJson(json, entry),
                live,
                string.concat("networks.json entry vs live candidate: ", networks[i])
            );
            if (live) {
                assertTrue(
                    LibBlockSearch.isStartBlock(
                        vm, raindex, codehash, vm.parseJsonUint(json, string.concat(entry, ".startBlock"))
                    ),
                    string.concat("networks.json startBlock is not the deploy block: ", networks[i])
                );
            }
        }
    }
}
