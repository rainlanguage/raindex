// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {
    RouteProcessorRaindexV6ArbOrderTaker
} from "../../../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";

/// Direct test that receive() accepts ETH transfers.
contract RouteProcessorRaindexV6ArbOrderTakerReceiveTest is Test {
    function testReceiveAcceptsEth() external {
        RouteProcessorRaindexV6ArbOrderTaker arb = new RouteProcessorRaindexV6ArbOrderTaker();
        vm.deal(address(this), 1 ether);

        (bool success,) = address(arb).call{value: 1 ether}("");
        assertTrue(success, "receive() should accept ETH");
        assertEq(address(arb).balance, 1 ether, "arb balance after receive");
    }
}
