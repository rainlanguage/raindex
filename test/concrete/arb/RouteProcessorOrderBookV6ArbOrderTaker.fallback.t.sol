// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {
    RouteProcessorOrderBookV6ArbOrderTaker
} from "../../../src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";

/// Direct test that fallback() accepts ETH transfers with non-empty calldata.
contract RouteProcessorOrderBookV6ArbOrderTakerFallbackTest is Test {
    function testFallbackAcceptsEthWithData() external {
        RouteProcessorOrderBookV6ArbOrderTaker arb = new RouteProcessorOrderBookV6ArbOrderTaker();
        vm.deal(address(this), 1 ether);

        (bool success,) = address(arb).call{value: 1 ether}(hex"deadbeef");
        assertTrue(success, "fallback() should accept ETH with data");
        assertEq(address(arb).balance, 1 ether, "arb balance after fallback");
    }
}
