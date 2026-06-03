// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {
    RaindexV6,
    NegativeVaultBalance,
    NegativeVaultBalanceChange,
    NegativePull,
    NegativePush
} from "src/concrete/raindex/RaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";

/// @dev Exposes RaindexV6's internal vault-balance / token helpers for testing.
contract RaindexV6VaultBalanceHarness is RaindexV6 {
    function exposedIncrease(address owner, address token, bytes32 vaultId, Float amount)
        external
        returns (Float, Float)
    {
        return increaseVaultBalance(owner, token, vaultId, amount);
    }

    function exposedDecrease(address owner, address token, bytes32 vaultId, Float amount)
        external
        returns (Float, Float)
    {
        return decreaseVaultBalance(owner, token, vaultId, amount);
    }

    function exposedPull(address account, address token, Float amount) external returns (uint256, uint8) {
        return pullTokens(account, token, amount);
    }

    function exposedPush(address account, address token, Float amount) external returns (uint256, uint8) {
        return pushTokens(account, token, amount);
    }
}

/// @title RaindexV6VaultBalanceRevertTest
/// @notice Audit A08-5 / A08-6 (#2535): the vault-balance helpers reject a
/// negative `amount` (`NegativeVaultBalanceChange`) and a decrease that would
/// drive the balance below zero (`NegativeVaultBalance`).
contract RaindexV6VaultBalanceRevertTest is Test {
    using LibDecimalFloat for Float;

    RaindexV6VaultBalanceHarness internal harness;
    address internal owner = makeAddr("owner");
    address internal token = makeAddr("token");
    // Non-zero vault id so the internal-balance branch (not the vault-0 token
    // transfer branch) is exercised.
    bytes32 internal constant VAULT = bytes32(uint256(1));

    function setUp() external {
        // pullTokens/pushTokens read token decimals via the deployed TOFU
        // contract, so it must exist before the negative-amount check is reached.
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        // Etch the runtime code so the large RaindexV6-derived harness isn't
        // subject to the EIP-170 creation size limit.
        harness = RaindexV6VaultBalanceHarness(payable(address(uint160(uint256(keccak256("vault.balance.harness"))))));
        vm.etch(address(harness), type(RaindexV6VaultBalanceHarness).runtimeCode);
    }

    /// A08-6: increase with a negative amount reverts.
    function testIncreaseNegativeAmountReverts() external {
        vm.expectRevert(
            abi.encodeWithSelector(NegativeVaultBalanceChange.selector, LibDecimalFloat.packLossless(-1, 0))
        );
        harness.exposedIncrease(owner, token, VAULT, LibDecimalFloat.packLossless(-1, 0));
    }

    /// A08-6: decrease with a negative amount reverts.
    function testDecreaseNegativeAmountReverts() external {
        vm.expectRevert(
            abi.encodeWithSelector(NegativeVaultBalanceChange.selector, LibDecimalFloat.packLossless(-1, 0))
        );
        harness.exposedDecrease(owner, token, VAULT, LibDecimalFloat.packLossless(-1, 0));
    }

    /// A08-5: decreasing below the (zero) balance reverts NegativeVaultBalance.
    function testDecreaseBelowZeroReverts() external {
        vm.expectRevert(abi.encodeWithSelector(NegativeVaultBalance.selector, LibDecimalFloat.packLossless(-1, 0)));
        harness.exposedDecrease(owner, token, VAULT, LibDecimalFloat.packLossless(1, 0));
    }

    /// A08-7: pulling a negative amount reverts NegativePull (a real token is
    /// used so the TOFU decimals read passes before the negative check).
    function testPullNegativeAmountReverts() external {
        MockToken realToken = new MockToken("Token", "TKN", 18);
        vm.expectRevert(NegativePull.selector);
        harness.exposedPull(owner, address(realToken), LibDecimalFloat.packLossless(-1, 0));
    }

    /// A08-7: pushing a negative amount reverts NegativePush.
    function testPushNegativeAmountReverts() external {
        MockToken realToken = new MockToken("Token", "TKN", 18);
        vm.expectRevert(NegativePush.selector);
        harness.exposedPush(owner, address(realToken), LibDecimalFloat.packLossless(-1, 0));
    }

    /// Increasing a vault-0 balance ALWAYS returns (0, 0): vault 0 holds no
    /// internal balance, it pushes tokens straight out to the owner, so the
    /// reported old and new balances are both exactly zero regardless of the
    /// amount. Pins the return values so a mutation echoing the amount back is
    /// caught.
    function testIncreaseVaultZeroReturnsZero() external {
        MockToken realToken = new MockToken("Token", "TKN", 18);
        // The harness pushes tokens out to the owner for vault 0, so it needs a
        // balance to transfer.
        realToken.mint(address(harness), 1e18);

        (Float oldBalance, Float newBalance) =
            harness.exposedIncrease(owner, address(realToken), bytes32(0), LibDecimalFloat.packLossless(1, 0));

        assertTrue(oldBalance.eq(Float.wrap(0)), "old balance not zero");
        assertTrue(newBalance.eq(Float.wrap(0)), "new balance not zero");
        // The owner actually received the pushed tokens.
        assertEq(realToken.balanceOf(owner), 1e18, "owner balance");
    }

    /// Decreasing a non-zero vault SUBTRACTS the amount from the old balance:
    /// it returns (oldBalance, oldBalance - amount), persists the new balance to
    /// storage, and a follow-up read reflects the reduced balance. Pins the
    /// `.sub` arithmetic, the (old, new) return tuple ordering, and the storage
    /// write — a mutation that adds instead of subtracts, returns the amount or
    /// both-new, or skips the write is caught.
    function testDecreaseNonZeroSubtracts() external {
        // Seed the vault with 10 via an increase from the zero balance.
        (Float seedOld, Float seedNew) =
            harness.exposedIncrease(owner, token, VAULT, LibDecimalFloat.packLossless(10, 0));
        assertTrue(seedOld.eq(Float.wrap(0)), "seed old");
        assertTrue(seedNew.eq(LibDecimalFloat.packLossless(10, 0)), "seed new");

        // Decrease by 3: returns (10, 7).
        (Float oldBalance, Float newBalance) =
            harness.exposedDecrease(owner, token, VAULT, LibDecimalFloat.packLossless(3, 0));
        assertTrue(oldBalance.eq(LibDecimalFloat.packLossless(10, 0)), "old balance is pre-decrease 10");
        assertTrue(newBalance.eq(LibDecimalFloat.packLossless(7, 0)), "new balance is 10 - 3 = 7");

        // The reduced balance is persisted: reading the vault returns 7.
        assertTrue(
            harness.vaultBalance2(owner, token, VAULT).eq(LibDecimalFloat.packLossless(7, 0)), "stored balance is 7"
        );

        // A second decrease compounds off the stored 7, not the original 10:
        // returns (7, 5) and persists 5.
        (Float old2, Float new2) = harness.exposedDecrease(owner, token, VAULT, LibDecimalFloat.packLossless(2, 0));
        assertTrue(old2.eq(LibDecimalFloat.packLossless(7, 0)), "second old reads stored 7");
        assertTrue(new2.eq(LibDecimalFloat.packLossless(5, 0)), "second new is 7 - 2 = 5");
        assertTrue(
            harness.vaultBalance2(owner, token, VAULT).eq(LibDecimalFloat.packLossless(5, 0)), "stored balance is 5"
        );
    }

    /// Decreasing a vault-0 balance ALWAYS returns (0, 0) and PULLS tokens from
    /// the owner into the contract (vault 0 holds no internal balance, every
    /// decrease is a compound matching deposit). Pins both the (0, 0) return and
    /// the pull side-effect (owner's tokens move to the harness) so a mutation
    /// echoing the amount back, or pushing instead of pulling, or storing a
    /// balance, is caught.
    function testDecreaseVaultZeroPullsAndReturnsZero() external {
        MockToken realToken = new MockToken("Token", "TKN", 18);
        // The owner holds tokens and has approved the harness so the pull works.
        realToken.mint(owner, 5e18);
        vm.prank(owner);
        realToken.approve(address(harness), 5e18);

        (Float oldBalance, Float newBalance) =
            harness.exposedDecrease(owner, address(realToken), bytes32(0), LibDecimalFloat.packLossless(2, 0));

        assertTrue(oldBalance.eq(Float.wrap(0)), "vault-0 old balance not zero");
        assertTrue(newBalance.eq(Float.wrap(0)), "vault-0 new balance not zero");
        // Two whole tokens were pulled from the owner into the harness.
        assertEq(realToken.balanceOf(owner), 3e18, "owner balance after pull");
        assertEq(realToken.balanceOf(address(harness)), 2e18, "harness balance after pull");
    }
}
