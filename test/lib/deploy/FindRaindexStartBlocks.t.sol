// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test, console2} from "forge-std-1.16.1/src/Test.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";

/// TEMPORARY: prints the exact deploy block of the current RaindexV6 on each
/// mainnet via binary search, to capture the new start-block constants after a
/// redeploy. The old start blocks are valid lower bounds (the redeploy is later).
/// Removed before merge; the permanent generator is a follow-up.
contract FindRaindexStartBlocksTest is Test {
    function _find(string memory network, uint256 floor) internal {
        vm.createSelectFork(network);
        uint256 b = LibRainDeploy.findDeployBlock(
            vm, LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH, floor
        );
        console2.log(string.concat("FOUND_START_BLOCK ", network), b);
    }

    function testFindRaindexStartBlocks() external {
        _find(LibRainDeploy.ARBITRUM_ONE, LibRaindexDeploy.RAINDEX_START_BLOCK_ARBITRUM);
        _find(LibRainDeploy.BASE, LibRaindexDeploy.RAINDEX_START_BLOCK_BASE);
        _find(LibRainDeploy.FLARE, LibRaindexDeploy.RAINDEX_START_BLOCK_FLARE);
        _find(LibRainDeploy.POLYGON, LibRaindexDeploy.RAINDEX_START_BLOCK_POLYGON);
    }
}
