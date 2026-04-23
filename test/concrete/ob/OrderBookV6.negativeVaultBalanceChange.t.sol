// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6SelfTest} from "test/util/abstract/OrderBookV6SelfTest.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {NegativeVaultBalanceChange} from "../../../src/concrete/ob/OrderBookV6.sol";

/// Direct tests that `increaseVaultBalance` and `decreaseVaultBalance` revert
/// with `NegativeVaultBalanceChange` when given a negative amount.
contract OrderBookV6NegativeVaultBalanceChangeTest is OrderBookV6SelfTest {
    /// External wrappers so vm.expectRevert can catch the internal revert.
    function externalIncreaseVaultBalance(address owner, address token, bytes32 vaultId, Float amount) external {
        increaseVaultBalance(owner, token, vaultId, amount);
    }

    function externalDecreaseVaultBalance(address owner, address token, bytes32 vaultId, Float amount) external {
        decreaseVaultBalance(owner, token, vaultId, amount);
    }

    function testIncreaseVaultBalanceNegativeAmountReverts() external {
        address owner = address(0xBEEF);
        address token = address(0xAAAA);
        bytes32 vaultId = bytes32(uint256(1));

        Float negativeAmount = LibDecimalFloat.packLossless(-1, 0);
        vm.expectRevert(abi.encodeWithSelector(NegativeVaultBalanceChange.selector, negativeAmount));
        this.externalIncreaseVaultBalance(owner, token, vaultId, negativeAmount);
    }

    function testDecreaseVaultBalanceNegativeAmountReverts() external {
        address owner = address(0xBEEF);
        address token = address(0xAAAA);
        bytes32 vaultId = bytes32(uint256(1));

        Float negativeAmount = LibDecimalFloat.packLossless(-1, 0);
        vm.expectRevert(abi.encodeWithSelector(NegativeVaultBalanceChange.selector, negativeAmount));
        this.externalDecreaseVaultBalance(owner, token, vaultId, negativeAmount);
    }
}
