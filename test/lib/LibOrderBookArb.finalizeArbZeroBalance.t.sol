// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibTestArb, ArbResult} from "test/util/lib/LibTestArb.sol";

contract LibOrderBookArbFinalizeArbZeroBalanceTest is Test {
    /// When OB consumes all tokens, both arb balances are zero and no
    /// safeTransfer is called for either token.
    function testFinalizeArbZeroBalanceBothTokens() external {
        // OB pulls 100e18, outputs 100e18. Exchange swaps 100e18 → 100e18.
        // Zero profit: arb has nothing left.
        ArbResult memory result = LibTestArb.setupAndArb(vm, 100e18, 100e18, 100e18, 100e18, LibTestArb.noopTask(), 0);

        // No profit swept to sender.
        assertEq(result.inputToken.balanceOf(address(this)), 0, "sender inputToken");
        assertEq(result.outputToken.balanceOf(address(this)), 0, "sender outputToken");
        // Arb contract is empty.
        assertEq(result.inputToken.balanceOf(address(result.arb)), 0, "arb inputToken");
        assertEq(result.outputToken.balanceOf(address(result.arb)), 0, "arb outputToken");
    }
}
