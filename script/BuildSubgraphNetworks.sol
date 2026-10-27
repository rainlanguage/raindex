// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script} from "forge-std-1.17.0/src/Script.sol";
import {LibRainDeploy, SupportedNetwork} from "rain-deploy-0.1.25/src/lib/LibRainDeploy.sol";
import {RaindexDeploySuites} from "../src/abstract/RaindexDeploySuites.sol";
import {LibRaindexDeploy} from "../src/lib/deploy/LibRaindexDeploy.sol";
import {LibRaindexSubgraphNetwork} from "../src/lib/deploy/LibRaindexSubgraphNetwork.sol";

/// @title BuildSubgraphNetworks
/// @notice Writes `subgraph/networks.json`: one entry per supported network
/// where the candidate RaindexV6 is live, at its deploy block.
contract BuildSubgraphNetworks is Script, RaindexDeploySuites {
    function run() external {
        address raindex = LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS;
        bytes32 codehash = LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH;
        SupportedNetwork[] memory networks = LibRainDeploy.declaredNetworkConfigs(supportedNetworks());
        string memory json = "{}";
        for (uint256 i = 0; i < networks.length; i++) {
            vm.createSelectFork(networks[i].name);
            LibRainDeploy.checkNetworkChainId(networks[i]);
            if (raindex.codehash != codehash) {
                continue;
            }
            uint256 startBlock = deployBlock(raindex, codehash);
            string memory entryKey = string.concat(networks[i].name, ".Raindex");
            vm.serializeAddress(entryKey, "address", raindex);
            string memory entry = vm.serializeUint(entryKey, "startBlock", startBlock);
            string memory dataSources = vm.serializeString(networks[i].name, "Raindex", entry);
            json = vm.serializeString(
                "networks", LibRaindexSubgraphNetwork.subgraphNetwork(networks[i].name), dataSources
            );
        }
        vm.writeJson(json, "subgraph/networks.json");
        vm.writeLine("subgraph/networks.json", "");
    }

    /// Not `LibBlockSearch.findDeployBlock`: on Arbitrum chains `block.number`
    /// is the L1 block, so the head comes from the RPC. Steps back
    /// geometrically for bounded archives.
    function deployBlock(address target, bytes32 codehash) internal returns (uint256) {
        uint256 high = rpcBlockNumber();
        uint256 delta = 250000;
        uint256 low = high > delta ? high - delta : 0;
        vm.rollFork(low);
        while (target.codehash == codehash) {
            if (low == 0) {
                return 0;
            }
            high = low;
            delta *= 4;
            low = high > delta ? high - delta : 0;
            vm.rollFork(low);
        }
        while (high - low > 1) {
            uint256 mid = (low + high) / 2;
            vm.rollFork(mid);
            if (target.codehash == codehash) {
                high = mid;
            } else {
                low = mid;
            }
        }
        return high;
    }

    function rpcBlockNumber() internal returns (uint256) {
        bytes memory result = vm.rpc("eth_blockNumber", "[]");
        uint256 n = 0;
        for (uint256 i = 0; i < result.length; i++) {
            n = (n << 8) | uint8(result[i]);
        }
        return n;
    }
}
