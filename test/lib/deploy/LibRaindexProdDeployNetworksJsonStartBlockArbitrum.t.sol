// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibRaindexProdDeploy} from "src/lib/deploy/LibRaindexProdDeploy.sol";

/// @title LibRaindexProdDeployNetworksJsonStartBlockArbitrumTest
/// @notice The startBlock for arbitrum-one in subgraph/networks.json MUST match
/// the constant in LibRaindexProdDeploy.
contract LibRaindexProdDeployNetworksJsonStartBlockArbitrumTest is Test {
    function testNetworksJsonStartBlockArbitrum() external view {
        string memory json = vm.readFile("subgraph/networks.json");
        uint256 startBlock = vm.parseJsonUint(json, ".arbitrum-one.Raindex.startBlock");
        assertEq(
            startBlock,
            LibRaindexProdDeploy.RAINDEX_START_BLOCK_ARBITRUM,
            "networks.json startBlock mismatch: arbitrum-one"
        );
    }
}
