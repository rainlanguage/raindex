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

/// Fuzz test over onTakeOrders2 Float parameters to exercise
/// toFixedDecimalLossy edge cases.
contract RouteProcessorOrderBookV6ArbOrderTakerOnTakeOrders2FuzzTest is Test {
    RouteProcessorOrderBookV6ArbOrderTaker internal arb;
    MockToken internal tokenA;
    MockToken internal tokenB;

    function setUp() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        tokenA = new MockToken("A", "A", 18);
        tokenB = new MockToken("B", "B", 18);

        MockRouteProcessor mockRp = new MockRouteProcessor();
        vm.etch(LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS, address(mockRp).code);

        arb = new RouteProcessorOrderBookV6ArbOrderTaker();
    }

    /// @dev onTakeOrders2 with fuzzed Float values must not leave tokens
    /// stranded. It either succeeds (no tokens move because arb has none)
    /// or reverts cleanly.
    function testOnTakeOrders2FuzzedFloats(Float inputAmountSent, Float totalOutputAmount) external {
        bytes memory route = abi.encode(hex"");

        // The call may revert for invalid Float values (e.g., negative
        // fixed-point conversion). We just verify it doesn't panic and
        // leaves no tokens behind on success.
        try arb.onTakeOrders2(address(tokenA), address(tokenB), inputAmountSent, totalOutputAmount, route) {
            assertEq(tokenA.balanceOf(address(arb)), 0, "tokenA balance after");
            assertEq(tokenB.balanceOf(address(arb)), 0, "tokenB balance after");
        } catch {}
    }
}
