// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalMockTest} from "test/util/abstract/RaindexV6ExternalMockTest.sol";
import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {LibDecimalFloat, Float} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @title RaindexV6VaultBalanceVault0Test
/// @notice Vault ID 0 has no internal balance; its effective balance is the
/// MINIMUM of the owner's wallet token balance and their approval to Raindex
/// (the most Raindex could ever pull). Existing maximum-input/output tests that
/// exercise vault 0 mock `balanceOf` and `allowance` to the SAME value, so a
/// bug that returned only the balance, only the approval, or the max of the two
/// would survive them. These pin the `min(balance, approval)` selection by
/// making the two differ.
contract RaindexV6VaultBalanceVault0Test is RaindexV6ExternalMockTest {
    using LibDecimalFloat for Float;

    /// When the approval is smaller than the wallet balance, the effective
    /// vault-0 balance is the approval (Raindex can't pull more than approved).
    function testVaultBalanceVault0ApprovalLimits(address owner, uint256 balance18, uint256 approval18) external {
        // iToken0 decimals are mocked to 18 by the base.
        balance18 = bound(balance18, 1, uint256(int256(type(int224).max)));
        approval18 = bound(approval18, 0, balance18 - 1);

        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.balanceOf.selector, owner), abi.encode(balance18));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.allowance.selector, owner, address(iRaindex)),
            abi.encode(approval18)
        );

        Float effective = iRaindex.vaultBalance2(owner, address(iToken0), bytes32(0));
        // The smaller of the two (approval) is returned, not the balance.
        assertTrue(effective.eq(LibDecimalFloat.fromFixedDecimalLosslessPacked(approval18, 18)), "approval limits");
    }

    /// When the wallet balance is smaller than the approval, the effective
    /// vault-0 balance is the balance (Raindex can't pull tokens the owner
    /// doesn't hold).
    function testVaultBalanceVault0BalanceLimits(address owner, uint256 balance18, uint256 approval18) external {
        approval18 = bound(approval18, 1, uint256(int256(type(int224).max)));
        balance18 = bound(balance18, 0, approval18 - 1);

        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.balanceOf.selector, owner), abi.encode(balance18));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.allowance.selector, owner, address(iRaindex)),
            abi.encode(approval18)
        );

        Float effective = iRaindex.vaultBalance2(owner, address(iToken0), bytes32(0));
        // The smaller of the two (balance) is returned, not the approval.
        assertTrue(effective.eq(LibDecimalFloat.fromFixedDecimalLosslessPacked(balance18, 18)), "balance limits");
    }
}
