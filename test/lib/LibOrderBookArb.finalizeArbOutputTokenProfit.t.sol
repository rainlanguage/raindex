// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibTestArb, ArbResult} from "test/util/lib/LibTestArb.sol";

contract LibOrderBookArbFinalizeArbOutputTokenProfitTest is Test {
    /// finalizeArb MUST transfer remaining output token profit to msg.sender.
    function testFinalizeArbTransfersOutputTokenProfit() external {
        // OB has 100e18 output, pulls 80e18 input. Exchange has 80e18 input.
        // Arb swaps only 80e18 output → 80e18 input. 20e18 output profit.
        ArbResult memory result = LibTestArb.setupAndArb(vm, 80e18, 100e18, 80e18, 80e18, LibTestArb.noopTask(), 0);

        // 20e18 output token profit swept to msg.sender by finalizeArb.
        assertEq(result.outputToken.balanceOf(address(this)), 20e18, "sender outputToken profit");
        // No input token profit (all 80e18 went to OB).
        assertEq(result.inputToken.balanceOf(address(this)), 0, "sender inputToken");
        // Arb contract is empty after finalizeArb.
        assertEq(result.inputToken.balanceOf(address(result.arb)), 0, "arb inputToken");
        assertEq(result.outputToken.balanceOf(address(result.arb)), 0, "arb outputToken");
        // OB got exactly what it pulled.
        assertEq(result.inputToken.balanceOf(address(result.orderBook)), 80e18, "OB inputToken");
        // Exchange did a partial swap.
        assertEq(result.outputToken.balanceOf(address(result.exchange)), 80e18, "exchange outputToken");
    }
}
