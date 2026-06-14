// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";

import {RouteProcessorRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {Float} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IRouteProcessor} from "src/interface/IRouteProcessor.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";

/// @dev Route processor mock that records the `msg.value` it was called with in
/// storage slot 0, so a test can assert how much native ETH the arb forwarded.
contract ValueRecordingRouteProcessor is IRouteProcessor {
    function processRoute(address, uint256, address, uint256, address, bytes memory)
        external
        payable
        returns (uint256)
    {
        // Slot 0 holds the last `msg.value` seen, readable via `vm.load` from
        // the etched address.
        assembly ("memory-safe") {
            sstore(0, callvalue())
        }
        return 0;
    }
}

/// @title RouteProcessorRaindexV6ArbOrderTakerNoEthForwardTest
/// @notice Locks the documented divergence from `LibGenericPoolExchange` (#2699):
/// the route-processor arb funds `processRoute` purely via the ERC20 approval and
/// forwards NO native ETH into the swap, even while holding an ETH balance. The
/// generic-pool arbs forward `address(this).balance`; this arb does not, leaving
/// any held ETH for `finalizeArb`'s native-gas sweep instead.
contract RouteProcessorRaindexV6ArbOrderTakerNoEthForwardTest is Test {
    function testProcessRouteCalledWithNoEthValue(uint256 ethBalance) external {
        ethBalance = bound(ethBalance, 1, 1e24);

        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken tokenA = new MockToken("A", "A", 18);
        MockToken tokenB = new MockToken("B", "B", 18);

        ValueRecordingRouteProcessor recorder = new ValueRecordingRouteProcessor();
        vm.etch(LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS, address(recorder).code);
        address routeProcessor = LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS;
        // Seed slot 0 with a sentinel so a zero read is genuinely from the call.
        vm.store(routeProcessor, bytes32(0), bytes32(type(uint256).max));

        RouteProcessorRaindexV6ArbOrderTaker arb = new RouteProcessorRaindexV6ArbOrderTaker();

        // Give the arb a non-zero native ETH balance. If the arb forwarded its
        // balance like the generic-pool path, the recorder would see it.
        vm.deal(address(arb), ethBalance);

        bytes memory route = abi.encode(hex"");
        arb.onTakeOrders2(address(tokenA), address(tokenB), Float.wrap(0), Float.wrap(0), route);

        // The arb called `processRoute` with zero value: native-ETH input legs
        // are unsupported and the held ETH is NOT routed into the swap.
        assertEq(uint256(vm.load(routeProcessor, bytes32(0))), 0, "processRoute msg.value must be 0");

        // The held ETH is untouched by the swap; it stays on the arb to be swept
        // by `finalizeArb` (not exercised by this direct `onTakeOrders2` call).
        assertEq(address(arb).balance, ethBalance, "arb retains its ETH balance");
        assertEq(routeProcessor.balance, 0, "route processor received no ETH");
    }
}
