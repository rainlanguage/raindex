// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {RaindexDeploySuites} from "src/abstract/RaindexDeploySuites.sol";
import {LibRaindexSubgraphNetwork} from "src/lib/deploy/LibRaindexSubgraphNetwork.sol";

/// @title LibRaindexSubgraphNetworkSubgraphNetworkTest
/// @notice Every supported network has its own subgraph network, and any other
/// name is refused.
contract LibRaindexSubgraphNetworkSubgraphNetworkTest is Test, RaindexDeploySuites {
    function subgraphNetworkExternal(string memory network) external pure returns (string memory) {
        return LibRaindexSubgraphNetwork.subgraphNetwork(network);
    }

    function testSubgraphNetworkDistinctPerSupportedNetwork() external view {
        string[] memory networks = supportedNetworks();
        for (uint256 i = 0; i < networks.length; i++) {
            string memory a = LibRaindexSubgraphNetwork.subgraphNetwork(networks[i]);
            for (uint256 j = i + 1; j < networks.length; j++) {
                assertNotEq(a, LibRaindexSubgraphNetwork.subgraphNetwork(networks[j]), networks[i]);
            }
        }
    }

    function testSubgraphNetworkUnknown(string memory network) external {
        string[] memory networks = supportedNetworks();
        for (uint256 i = 0; i < networks.length; i++) {
            vm.assume(keccak256(bytes(network)) != keccak256(bytes(networks[i])));
        }
        vm.expectRevert(abi.encodeWithSelector(LibRaindexSubgraphNetwork.NoSubgraphNetwork.selector, network));
        this.subgraphNetworkExternal(network);
    }
}
