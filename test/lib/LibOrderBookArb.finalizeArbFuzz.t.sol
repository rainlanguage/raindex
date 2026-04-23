// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibTestArb, ArbResult} from "test/util/lib/LibTestArb.sol";

contract LibOrderBookArbFinalizeArbFuzzTest is Test {
    /// Fuzz finalizeArb with varying amounts. After arb, all tokens must be
    /// accounted for: arb contract empty, sender gets profit, OB and exchange
    /// hold the rest.
    function testFinalizeArbFuzz(uint256 obOutputAmount, uint256 swapAmount, uint256 obPullAmount) external {
        // Bound to avoid zero-amount edge cases and overflow.
        obOutputAmount = bound(obOutputAmount, 1, 1e30);
        swapAmount = bound(swapAmount, 1, obOutputAmount);
        obPullAmount = bound(obPullAmount, 1, swapAmount);

        // Exchange always has enough to fulfill the swap.
        uint256 exchangeInputAmount = swapAmount;

        ArbResult memory result = LibTestArb.setupAndArb(
            vm, obPullAmount, obOutputAmount, exchangeInputAmount, swapAmount, LibTestArb.noopTask(), 0
        );

        // Arb contract must be empty.
        assertEq(result.inputToken.balanceOf(address(result.arb)), 0, "arb inputToken");
        assertEq(result.outputToken.balanceOf(address(result.arb)), 0, "arb outputToken");

        // Input token profit = swapAmount - obPullAmount.
        uint256 expectedInputProfit = swapAmount - obPullAmount;
        assertEq(result.inputToken.balanceOf(address(this)), expectedInputProfit, "sender inputToken profit");

        // Output token profit = obOutputAmount - swapAmount.
        uint256 expectedOutputProfit = obOutputAmount - swapAmount;
        assertEq(result.outputToken.balanceOf(address(this)), expectedOutputProfit, "sender outputToken profit");

        // OB got what it pulled.
        assertEq(result.inputToken.balanceOf(address(result.orderBook)), obPullAmount, "OB inputToken");

        // Exchange got what was swapped in.
        assertEq(result.outputToken.balanceOf(address(result.exchange)), swapAmount, "exchange outputToken");
    }
}
