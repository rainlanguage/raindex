// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    ClearConfigV2,
    SignedContextV1,
    TaskV2
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @title RaindexV6TakeOrderVaultZeroInputTest
/// @notice An order with a `vaultId == 0` INPUT is settled as a direct wallet
/// transfer to the owner (`increaseVaultBalance` -> `pushTokens`) rather than an
/// internal vault credit. The orderbook must pull every incoming token before it
/// pushes that input out: `takeOrders4` defers vault-0 input pushes until after
/// the taker payment is pulled, and `clear3` pulls both orders' outputs before
/// pushing either input. These tests use real `MockToken` ERC20s with zero
/// ambient orderbook balance, so a vault-0 input is fully funded by the
/// counterparty alone.
contract RaindexV6TakeOrderVaultZeroInputTest is RaindexV6ExternalRealTest {
    using LibDecimalFloat for Float;

    address internal owner = address(uint160(uint256(keccak256("owner.rain.test"))));
    address internal bob = address(uint160(uint256(keccak256("bob.rain.test"))));
    address internal alice = address(uint160(uint256(keccak256("alice.rain.test"))));

    bytes32 internal constant OUTPUT_VAULT_ID = bytes32(uint256(0x01));

    MockToken internal token0;
    MockToken internal token1;

    function setUp() public {
        token0 = new MockToken("Token Zero", "TK0", 18);
        token1 = new MockToken("Token One", "TK1", 18);
    }

    /// Mint `amount18` of `token` to `who`, approve the orderbook, and deposit it
    /// into `vaultId` as a Float `amount` (whole token units, 18 decimals).
    function _deposit(address who, MockToken token, bytes32 vaultId, uint256 amount) internal {
        token.mint(who, amount * 1e18);
        vm.startPrank(who);
        token.approve(address(iRaindex), amount * 1e18);
        iRaindex.deposit4(address(token), vaultId, LibDecimalFloat.packLossless(int256(amount), 0), new TaskV2[](0));
        vm.stopPrank();
    }

    /// `takeOrders4`: an order whose INPUT is `token0` at vault 0 fills against a
    /// taker who holds `token0`, while the orderbook holds no ambient `token0`.
    /// The vault-0 input is pushed to the owner only after the taker's `token0`
    /// payment has been pulled, so the trade is funded by the taker alone.
    function testVaultZeroInputSucceedsWithoutAmbientBalance() external {
        // Owner: input token0 @ vault 0 (wallet), output token1 @ a normal vault.
        _deposit(owner, token1, OUTPUT_VAULT_ID, 10);
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );

        // Taker holds token0 to pay; the orderbook holds none.
        token0.mint(bob, 1e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 1e18);
        assertEq(token0.balanceOf(address(iRaindex)), 0, "orderbook starts with zero ambient token0");

        TakeOrdersConfigV5 memory config = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        vm.prank(bob);
        (Float totalTakerInput, Float totalTakerOutput) = iRaindex.takeOrders4(config);

        assertTrue(totalTakerInput.eq(LibDecimalFloat.packLossless(1, 0)), "1 token0 filled");
        assertTrue(totalTakerOutput.eq(LibDecimalFloat.packLossless(1, 0)), "1 token1 filled");

        // The vault-0 input was pushed straight to the owner's wallet, funded by
        // the taker's payment that was pulled first.
        assertEq(token0.balanceOf(owner), 1e18, "owner received vault-0 input directly");
        assertEq(token0.balanceOf(bob), 0, "taker paid token0");
        assertEq(token1.balanceOf(bob), 1e18, "taker received token1");
        assertEq(token0.balanceOf(address(iRaindex)), 0, "token0 passed straight through the orderbook");
        assertEq(token1.balanceOf(address(iRaindex)), 9e18, "unfilled token1 stays in the owner's output vault");
    }

    /// `clear3`: Alice's INPUT is `token0` at vault 0, funded by Bob's `token0`
    /// vault-0 OUTPUT. Both orders' outputs are pulled before either input is
    /// pushed, so Alice's vault-0 input is funded by Bob's output with no ambient
    /// balance needed.
    ///
    ///   Alice: input = token0 (vault 0, wallet),  output = token1 (vault 0x01)
    ///   Bob:   input = token1 (vault 0x01),        output = token0 (vault 0, wallet)
    function testClearVaultZeroInputSucceedsWithoutAmbientBalance() external {
        // Alice funds her token1 output vault.
        _deposit(alice, token1, OUTPUT_VAULT_ID, 10);

        // Bob's output is vault-0 token0 (pulled from his wallet on clear).
        token0.mint(bob, 1e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 1e18);

        OrderV4 memory aliceOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, alice, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );
        OrderV4 memory bobOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, bob, "_ _: 1 1;:;", address(token1), OUTPUT_VAULT_ID, address(token0), bytes32(0)
        );

        assertEq(token0.balanceOf(address(iRaindex)), 0, "orderbook starts with zero ambient token0");

        iRaindex.clear3(
            aliceOrder, bobOrder, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );

        // Alice's vault-0 input was pushed to her wallet, funded by Bob's vault-0
        // output pulled first.
        assertEq(token0.balanceOf(alice), 1e18, "alice received vault-0 input directly");
        assertEq(token0.balanceOf(bob), 0, "bob paid his vault-0 output");
        assertEq(token0.balanceOf(address(iRaindex)), 0, "token0 passed straight through the orderbook");

        // Bob's token1 input was credited to his internal vault; Alice's token1
        // output vault was debited by the same amount.
        assertTrue(
            iRaindex.vaultBalance2(bob, address(token1), OUTPUT_VAULT_ID).eq(LibDecimalFloat.packLossless(1, 0)),
            "bob credited token1 input vault"
        );
        assertTrue(
            iRaindex.vaultBalance2(alice, address(token1), OUTPUT_VAULT_ID).eq(LibDecimalFloat.packLossless(9, 0)),
            "alice token1 output vault debited"
        );
    }

    function _take(OrderV4 memory a, OrderV4 memory b) internal returns (Float, Float) {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](2);
        orders[0] =
            TakeOrderConfigV4({order: a, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)});
        orders[1] =
            TakeOrderConfigV4({order: b, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)});
        vm.prank(bob);
        return iRaindex.takeOrders4(LibTestTakeOrder.defaultTakeConfig(orders));
    }

    /// Two same-owner vault-0 input orders in one batch. Their input credits
    /// accrue in the loop and settle as a single netted push to the owner after
    /// the taker's aggregate payment is pulled. With zero ambient `token0` all 3
    /// units flow taker -> orderbook -> owner; if the push were inline (before the
    /// pull) the first order would revert.
    function testVaultZeroInputMultipleSameOwnerNettedWithoutAmbientBalance() external {
        _deposit(owner, token1, OUTPUT_VAULT_ID, 10);
        OrderV4 memory order1 = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );
        // Distinct output max keeps this a distinct order from order1.
        OrderV4 memory order2 = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 2 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );

        token0.mint(bob, 3e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 3e18);
        assertEq(token0.balanceOf(address(iRaindex)), 0, "orderbook starts with zero ambient token0");

        (Float totalTakerInput, Float totalTakerOutput) = _take(order1, order2);

        assertTrue(totalTakerInput.eq(LibDecimalFloat.packLossless(3, 0)), "3 token0 filled across both orders");
        assertTrue(totalTakerOutput.eq(LibDecimalFloat.packLossless(3, 0)), "3 token1 filled across both orders");

        assertEq(token0.balanceOf(owner), 3e18, "owner received both vault-0 inputs (1 + 2), netted");
        assertEq(token0.balanceOf(bob), 0, "taker paid token0 for both");
        assertEq(token1.balanceOf(bob), 3e18, "taker received token1 for both");
        assertEq(token0.balanceOf(address(iRaindex)), 0, "token0 passed straight through the orderbook");
        assertEq(token1.balanceOf(address(iRaindex)), 7e18, "remaining token1 stays in the owner's output vault");
    }

    /// Two different-owner vault-0 input orders in one batch. Each owner's input
    /// is pushed to them after the loop (per owner, not netted across owners),
    /// all funded by the taker's single aggregate payment. Zero ambient `token0`.
    function testVaultZeroInputMultipleOwnersWithoutAmbientBalance() external {
        _deposit(owner, token1, OUTPUT_VAULT_ID, 10);
        _deposit(alice, token1, OUTPUT_VAULT_ID, 10);
        OrderV4 memory ownerOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );
        OrderV4 memory aliceOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, alice, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );

        token0.mint(bob, 2e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 2e18);
        assertEq(token0.balanceOf(address(iRaindex)), 0, "orderbook starts with zero ambient token0");

        _take(ownerOrder, aliceOrder);

        assertEq(token0.balanceOf(owner), 1e18, "owner received their own vault-0 input");
        assertEq(token0.balanceOf(alice), 1e18, "alice received their own vault-0 input");
        assertEq(token0.balanceOf(bob), 0, "taker paid token0 for both");
        assertEq(token1.balanceOf(bob), 2e18, "taker received token1 for both");
        assertEq(token0.balanceOf(address(iRaindex)), 0, "token0 passed straight through the orderbook");
    }
}
