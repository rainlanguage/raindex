// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6SelfTest} from "test/util/abstract/OrderBookV6SelfTest.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {NegativeVaultBalance} from "../../../src/concrete/ob/OrderBookV6.sol";

/// Direct test that `decreaseVaultBalance` reverts with `NegativeVaultBalance`
/// when the decrease exceeds the current balance.
contract OrderBookV6NegativeVaultBalanceTest is OrderBookV6SelfTest {
    /// External wrapper so vm.expectRevert can catch the internal revert.
    function externalDecreaseVaultBalance(address owner, address token, bytes32 vaultId, Float amount) external {
        decreaseVaultBalance(owner, token, vaultId, amount);
    }

    function testDecreaseVaultBalanceBelowZeroReverts() external {
        address owner = address(0xBEEF);
        address token = address(0xAAAA);
        bytes32 vaultId = bytes32(uint256(1));

        // Set vault balance to 1.
        increaseVaultBalance(owner, token, vaultId, LibDecimalFloat.packLossless(1, 0));

        // Attempt to decrease by 2 — should revert with negative result (1 - 2 = -1).
        Float balance = LibDecimalFloat.packLossless(1, 0);
        Float decreaseAmount = LibDecimalFloat.packLossless(2, 0);
        Float expectedNewBalance = LibDecimalFloat.sub(balance, decreaseAmount);
        vm.expectRevert(abi.encodeWithSelector(NegativeVaultBalance.selector, expectedNewBalance));
        this.externalDecreaseVaultBalance(owner, token, vaultId, decreaseAmount);
    }
}
