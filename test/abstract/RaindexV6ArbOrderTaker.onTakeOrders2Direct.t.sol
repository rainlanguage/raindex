// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {GenericPoolRaindexV6ArbOrderTaker} from "../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {Float} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";

contract RaindexV6ArbOrderTakerOnTakeOrders2DirectTest is Test {
    /// Calling onTakeOrders2 directly MUST succeed (no-op) and not modify
    /// any state. Anyone can call it — there is no access control by design.
    function testOnTakeOrders2DirectCallSucceeds() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken tokenA = new MockToken("A", "A", 18);
        MockToken tokenB = new MockToken("B", "B", 18);

        GenericPoolRaindexV6ArbOrderTaker arb = new GenericPoolRaindexV6ArbOrderTaker();

        // Call from an arbitrary address — should not revert.
        // takeOrdersData is abi.encode(spender, pool, encodedFunctionCall).
        // Use the arb's own address as pool (it has a fallback) so the call
        // succeeds without deploying another contract.
        bytes memory takeOrdersData = abi.encode(address(arb), address(arb), hex"");
        vm.prank(address(0xdead));
        arb.onTakeOrders2(address(tokenA), address(tokenB), Float.wrap(0), Float.wrap(0), takeOrdersData);

        // Contract remains empty — attacker gained nothing.
        assertEq(tokenA.balanceOf(address(arb)), 0);
        assertEq(tokenB.balanceOf(address(arb)), 0);
    }
}
