// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {Reenteroor} from "test/util/concrete/Reenteroor.sol";
import {
    IRaindexV6,
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

    /// Pool safety across REPEATED takes: a vault-0 input is a pure pass-through
    /// funded by the taker, so the orderbook's holdings of the input token must
    /// not change across the batch — and must never dip into OTHER users' pooled
    /// balances of that token. A third party parks 50 token0 in a normal vault;
    /// the owner then fills a vault-0 input order twice. Each take the owner must
    /// receive exactly one unit and the pool must stay at exactly 50.
    ///
    /// This is the strongest guard on the transient vault-0 slot. If take 1 left
    /// the slot non-zero (a dangling credit), take 2 would re-push it: the owner
    /// would receive 2 on the second take and the pool would drop to 49. If the
    /// push ever exceeded the accrued amount, the pool would drop on take 1.
    /// Either way the orderbook would be paying out tokens it does not own —
    /// draining the third party — and this test catches it to the wei.
    function testVaultZeroInputPoolUntouchedAcrossRepeatedTakes() external {
        address carol = address(uint160(uint256(keccak256("carol.rain.test"))));
        bytes32 poolVaultId = bytes32(uint256(0x99));
        // Carol's token0 sits in the orderbook as a pooled vault balance.
        _deposit(carol, token0, poolVaultId, 50);

        _deposit(owner, token1, OUTPUT_VAULT_ID, 10);
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );

        // Taker funds exactly two units across two takes.
        token0.mint(bob, 2e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 2e18);
        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "orderbook holds only carol's pool pre-take");

        TakeOrdersConfigV5 memory config = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));

        vm.prank(bob);
        iRaindex.takeOrders4(config);
        assertEq(token0.balanceOf(owner), 1e18, "take 1: owner receives exactly one unit");
        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "take 1: pool intact");

        vm.prank(bob);
        iRaindex.takeOrders4(config);
        assertEq(token0.balanceOf(owner), 2e18, "take 2: owner receives exactly one more unit, no dangling re-push");
        assertEq(token0.balanceOf(bob), 0, "taker funded exactly two units total");
        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "take 2: pool still intact, no drain from a dangling slot");
        assertTrue(
            iRaindex.vaultBalance2(carol, address(token0), poolVaultId).eq(LibDecimalFloat.packLossless(50, 0)),
            "carol's pooled vault balance untouched across both takes"
        );
    }

    /// Same pool safety for `clear3`'s vault-0 input push: Alice's vault-0 token0
    /// input is funded entirely by Bob's token0 output, so a third party's pooled
    /// token0 is untouched. A vault-0 push that exceeded what Bob supplied would
    /// drain the pool.
    function testClearVaultZeroInputDoesNotTouchPooledBalance() external {
        address carol = address(uint160(uint256(keccak256("carol.rain.test"))));
        bytes32 poolVaultId = bytes32(uint256(0x99));
        _deposit(carol, token0, poolVaultId, 50);

        _deposit(alice, token1, OUTPUT_VAULT_ID, 10);
        token0.mint(bob, 1e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 1e18);

        OrderV4 memory aliceOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, alice, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );
        OrderV4 memory bobOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, bob, "_ _: 1 1;:;", address(token1), OUTPUT_VAULT_ID, address(token0), bytes32(0)
        );

        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "orderbook holds only carol's pool pre-clear");

        iRaindex.clear3(
            aliceOrder, bobOrder, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );

        assertEq(token0.balanceOf(alice), 1e18, "alice received her vault-0 input");
        assertEq(token0.balanceOf(bob), 0, "bob funded his vault-0 output in full");
        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "clear vault-0 push must not touch the pooled balance");
        assertTrue(
            iRaindex.vaultBalance2(carol, address(token0), poolVaultId).eq(LibDecimalFloat.packLossless(50, 0)),
            "carol's pooled vault balance is untouched"
        );
    }

    /// The OUTPUT side of vault 0: a `vaultId == 0` OUTPUT is funded directly from
    /// the order owner's wallet (`recordVaultOutput` -> `decreaseVaultBalance`
    /// pulls the token in) and handed to the taker, so a third party's pooled
    /// balance of the output token is untouched. Owner outputs token1 from vault
    /// 0; the taker supplies token0 and receives token1; carol's pooled token1 is
    /// unchanged and the token0 input is credited to the owner's normal vault.
    function testVaultZeroOutputDoesNotTouchPooledBalance() external {
        address carol = address(uint160(uint256(keccak256("carol.rain.test"))));
        bytes32 poolVaultId = bytes32(uint256(0x99));
        bytes32 inputVaultId = bytes32(uint256(0x02));
        _deposit(carol, token1, poolVaultId, 50);

        // Owner funds the vault-0 output from their wallet.
        token1.mint(owner, 1e18);
        vm.prank(owner);
        token1.approve(address(iRaindex), 1e18);
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), inputVaultId, address(token1), bytes32(0)
        );

        // Taker supplies token0 (the order's input), receives token1 (the output).
        token0.mint(bob, 1e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 1e18);
        assertEq(token1.balanceOf(address(iRaindex)), 50e18, "orderbook holds only carol's token1 pool pre-take");

        TakeOrdersConfigV5 memory config = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        vm.prank(bob);
        iRaindex.takeOrders4(config);

        assertEq(token1.balanceOf(owner), 0, "owner's wallet funded the vault-0 output");
        assertEq(token1.balanceOf(bob), 1e18, "taker received the vault-0 output");
        assertEq(token0.balanceOf(bob), 0, "taker supplied token0");
        assertEq(token1.balanceOf(address(iRaindex)), 50e18, "vault-0 output must not touch the token1 pool");
        assertTrue(
            iRaindex.vaultBalance2(carol, address(token1), poolVaultId).eq(LibDecimalFloat.packLossless(50, 0)),
            "carol's pooled token1 untouched"
        );
        assertTrue(
            iRaindex.vaultBalance2(owner, address(token0), inputVaultId).eq(LibDecimalFloat.packLossless(1, 0)),
            "owner's token0 input vault credited"
        );
    }

    /// A capped (partial) fill accrues only the ACTUAL filled vault-0 input, not
    /// the order's offered max. The order offers up to 5 but the owner's output
    /// vault holds only 2 token1, so the output (and thus the matching token0
    /// input) is capped at 2. The owner must receive exactly 2 token0 — funded by
    /// the taker — and the pooled token0 must be untouched. Accruing the offered
    /// max instead of the filled amount would push 5 and drain the pool by 3.
    function testVaultZeroInputPartialFillAccruesActualNotMax() external {
        address carol = address(uint160(uint256(keccak256("carol.rain.test"))));
        bytes32 poolVaultId = bytes32(uint256(0x99));
        _deposit(carol, token0, poolVaultId, 50);

        // Output vault holds only 2, capping the order's offered-5 output at 2.
        _deposit(owner, token1, OUTPUT_VAULT_ID, 2);
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 5 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );

        // Taker could fund 5 but only 2 will fill.
        token0.mint(bob, 5e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 5e18);
        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "orderbook holds only carol's pool pre-take");

        TakeOrdersConfigV5 memory config = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        vm.prank(bob);
        iRaindex.takeOrders4(config);

        assertEq(token0.balanceOf(owner), 2e18, "owner received exactly the 2 filled, not the offered 5");
        assertEq(token0.balanceOf(bob), 3e18, "taker spent only the 2 that filled, keeps 3");
        assertEq(token1.balanceOf(bob), 2e18, "taker received the 2 available token1");
        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "partial vault-0 fill must not touch the pool");
        assertTrue(
            iRaindex.vaultBalance2(carol, address(token0), poolVaultId).eq(LibDecimalFloat.packLossless(50, 0)),
            "carol's pooled token0 untouched"
        );
    }

    /// Settlement isolation: one owner fills two same-token (token0) input orders
    /// in one batch — A via vault 0 (pushed to the wallet) and B via a normal
    /// vault (credited internally). The two must settle to DISTINCT places: A's
    /// unit lands in the owner's wallet and B's unit in the internal vault, with
    /// neither leaking into the other and a third party's pooled token0 untouched.
    /// Over-pushing the vault-0 slot would put 2 in the wallet instead of 1.
    function testVaultZeroAndInternalInputSettleToDistinctPlaces() external {
        address carol = address(uint160(uint256(keccak256("carol.rain.test"))));
        bytes32 poolVaultId = bytes32(uint256(0x99));
        bytes32 internalVaultId = bytes32(uint256(0x02));
        _deposit(carol, token0, poolVaultId, 50);

        _deposit(owner, token1, OUTPUT_VAULT_ID, 10);
        OrderV4 memory vaultZeroOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );
        OrderV4 memory internalOrder = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), internalVaultId, address(token1), OUTPUT_VAULT_ID
        );

        token0.mint(bob, 2e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 2e18);

        _take(vaultZeroOrder, internalOrder);

        // A (vault 0) -> owner's wallet; B (internal) -> owner's internal vault.
        assertEq(token0.balanceOf(owner), 1e18, "only the vault-0 input reached the owner's wallet");
        assertTrue(
            iRaindex.vaultBalance2(owner, address(token0), internalVaultId).eq(LibDecimalFloat.packLossless(1, 0)),
            "the internal input was credited to the internal vault, not the wallet"
        );
        // The vault-0 slot read for the wallet must not double-count the internal credit.
        assertEq(token0.balanceOf(bob), 0, "taker funded both units");
        assertEq(token1.balanceOf(bob), 2e18, "taker received both outputs");
        // Orderbook holds the pool (50) plus the internally-credited unit (1).
        assertEq(token0.balanceOf(address(iRaindex)), 51e18, "orderbook holds pool + internal credit only");
        assertTrue(
            iRaindex.vaultBalance2(carol, address(token0), poolVaultId).eq(LibDecimalFloat.packLossless(50, 0)),
            "carol's pooled token0 untouched"
        );
    }

    /// An untaken (zero-amount) order moves NO tokens at the real-token level: it
    /// runs no recordVaultInput/Output and no handleIO, so it neither charges the
    /// taker, credits the owner, nor touches any vault. Batch [untaken A, taken
    /// B]: only B's single unit flows; A contributes nothing. The overlay-removal
    /// adversarial suite asserts the state (kvs) no-op under mocked transfers;
    /// this pins the token-movement no-op with real ERC20s.
    function testUntakenOrderContributesNoTokenMovement() external {
        address carol = address(uint160(uint256(keccak256("carol.rain.test"))));
        bytes32 poolVaultId = bytes32(uint256(0x99));
        _deposit(carol, token0, poolVaultId, 50);

        _deposit(owner, token1, OUTPUT_VAULT_ID, 10);
        OrderV4 memory untaken = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 0 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );
        OrderV4 memory taken = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );

        // Taker is funded for two units; only one must be spent.
        token0.mint(bob, 2e18);
        vm.prank(bob);
        token0.approve(address(iRaindex), 2e18);

        (Float totalTakerInput,) = _take(untaken, taken);

        assertTrue(totalTakerInput.eq(LibDecimalFloat.packLossless(1, 0)), "only the taken order filled");
        assertEq(token0.balanceOf(bob), 1e18, "taker charged for the taken order only");
        assertEq(token1.balanceOf(bob), 1e18, "taker received the taken order's output only");
        assertEq(token0.balanceOf(owner), 1e18, "owner received only the taken order's vault-0 input");
        assertEq(token0.balanceOf(address(iRaindex)), 50e18, "untaken order moved nothing; pool intact");
        assertTrue(
            iRaindex.vaultBalance2(owner, address(token1), OUTPUT_VAULT_ID).eq(LibDecimalFloat.packLossless(9, 0)),
            "only the taken order drew from the output vault"
        );
    }

    /// The flash-swap reentrancy window: takeOrders4 pushes the order outputs to
    /// the taker and invokes its `onTakeOrders2` callback BEFORE it pulls the
    /// taker's payment and BEFORE it flushes the vault-0 input slots. So during
    /// the callback the vault-0 slot is accrued but not yet settled — the most
    /// dangerous moment to re-enter. A malicious taker that re-enters takeOrders4
    /// from its callback must be stopped by the nonReentrant guard, so it can
    /// never act on the mid-flight slot. (The existing arb reentrancy test uses a
    /// mock orderbook; this exercises the real RaindexV6 guard with a live vault-0
    /// slot.)
    function testTakeOrdersReentrancyViaCallbackBlocked() external {
        _deposit(owner, token1, OUTPUT_VAULT_ID, 10);
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(token0), bytes32(0), address(token1), OUTPUT_VAULT_ID
        );

        Reenteroor taker = new Reenteroor();

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4({
            order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)
        });
        // Non-empty data makes takeOrders4 invoke the onTakeOrders2 callback.
        TakeOrdersConfigV5 memory config = TakeOrdersConfigV5({
            orders: orders,
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            data: hex"01"
        });

        // The taker's callback re-enters takeOrders4 (its config is irrelevant:
        // the nonReentrant guard reverts at function entry).
        taker.reenterWith(abi.encodeWithSelector(IRaindexV6.takeOrders4.selector, config));

        vm.expectRevert(abi.encodeWithSelector(bytes4(keccak256("ReentrancyGuardReentrantCall()"))));
        vm.prank(address(taker));
        iRaindex.takeOrders4(config);
    }
}
