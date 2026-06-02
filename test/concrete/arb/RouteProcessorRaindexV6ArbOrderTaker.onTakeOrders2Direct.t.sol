// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";

import {RouteProcessorRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {Float} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockRouteProcessor} from "test/util/concrete/MockRouteProcessor.sol";

contract RouteProcessorRaindexV6ArbOrderTakerOnTakeOrders2DirectTest is Test {
    /// Calling onTakeOrders2 directly from an arbitrary address MUST succeed.
    /// The function has no access control by design — the contract is stateless
    /// between operations so there is nothing to exploit.
    function testOnTakeOrders2DirectCallByAttacker() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken tokenA = new MockToken("A", "A", 18);
        MockToken tokenB = new MockToken("B", "B", 18);
        MockRouteProcessor mockRp = new MockRouteProcessor();
        vm.etch(LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS, address(mockRp).code);

        RouteProcessorRaindexV6ArbOrderTaker arb = new RouteProcessorRaindexV6ArbOrderTaker();

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
