// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {
    RouteProcessorOrderBookV6ArbOrderTaker
} from "../../../src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";
import {Float} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockRouteProcessor} from "test/util/concrete/MockRouteProcessor.sol";

contract RouteProcessorOrderBookV6ArbOrderTakerOnTakeOrders2DirectTest is Test {
    /// Calling onTakeOrders2 directly from an arbitrary address MUST succeed.
    /// The function has no access control by design — the contract is stateless
    /// between operations so there is nothing to exploit.
    function testOnTakeOrders2DirectCallByAttacker() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken tokenA = new MockToken("A", "A", 18);
        MockToken tokenB = new MockToken("B", "B", 18);
        MockRouteProcessor mockRp = new MockRouteProcessor();
        vm.etch(LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS, address(mockRp).code);

        RouteProcessorOrderBookV6ArbOrderTaker arb = new RouteProcessorOrderBookV6ArbOrderTaker();

        // Attacker calls onTakeOrders2 directly with zero amounts.
        // processRoute is called with 0 amountIn so no tokens move.
        bytes memory route = abi.encode(hex"");
        vm.prank(address(0xdead));
        arb.onTakeOrders2(address(tokenA), address(tokenB), Float.wrap(0), Float.wrap(0), route);

        // Contract remains empty — attacker gained nothing.
        assertEq(tokenA.balanceOf(address(arb)), 0);
        assertEq(tokenB.balanceOf(address(arb)), 0);
    }
}
