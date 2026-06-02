// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    SignedContextV1,
    TaskV2
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @title RaindexV6TakeOrderBatchStateAdversarialTest
/// @notice Audit Protofire H01 (#2617): adversarial coverage of the per-owner
/// `stateOverlay` threading in `takeOrders4`. Each test pins one behaviour of
/// the fix with a discriminating total: the value differs from what the
/// pre-fix (or a mutated) contract would produce.
contract RaindexV6TakeOrderBatchStateAdversarialTest is RaindexV6ExternalRealTest {
    using LibDecimalFloat for Float;

    address internal immutable iBob = address(uint160(uint256(keccak256("bob.rain.test"))));

    function owner(string memory tag) internal pure returns (address) {
        return address(uint160(uint256(keccak256(bytes(tag)))));
    }

    /// Allow every deposit/settlement transfer.
    function mockTransfers() internal {
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));
    }

    function fund(address o) internal {
        vm.prank(o);
        iRaindex.deposit4(
            address(iToken1), bytes32(uint256(0x01)), LibDecimalFloat.packLossless(100, 0), new TaskV2[](0)
        );
    }

    function addOrder(address o, bytes memory expr) internal returns (OrderV4 memory) {
        return LibTestTakeOrder.addOrderWithExpression(
            vm, o, expr, address(iToken0), bytes32(uint256(0x01)), address(iToken1), bytes32(uint256(0x01))
        );
    }

    function take(OrderV4[] memory ordersList) internal returns (Float totalTakerInput) {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](ordersList.length);
        for (uint256 i = 0; i < ordersList.length; i++) {
            orders[i] = TakeOrderConfigV4({
                order: ordersList[i], inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)
            });
        }
        vm.prank(iBob);
        (totalTakerInput,) = iRaindex.takeOrders4(LibTestTakeOrder.defaultTakeConfig(orders));
    }

    /// Threading works across DIFFERENT orders of the same owner, not just
    /// repeats: order A writes key 0; order B (same owner, different expression)
    /// reads key 0 and only fills while unset. In one batch B must observe A's
    /// write and skip. Pre-fix: B reads stale 0 and fills → total 2.
    function testThreadingAcrossDistinctSameOwnerOrders() external {
        mockTransfers();
        address o = owner("alice");
        fund(o);
        OrderV4[] memory list = new OrderV4[](2);
        list[0] = addOrder(o, ":set(0 1),_ _:1 1;:;");
        list[1] = addOrder(o, "used:get(0),_ _:if(used 0 1) 1;:;");
        assertTrue(take(list).eq(LibDecimalFloat.packLossless(1, 0)), "B must see A's calculate write");
    }

    /// A skipped order (outputMax == 0 -> OrderZeroAmount) still runs its
    /// calculate `set`, and that write must be threaded to later same-owner
    /// orders. A writes key 0 but offers 0 (skipped); B then sees key 0 set and
    /// also skips -> total 0. Pre-fix (or if skipped orders weren't recorded):
    /// B fills -> total 1.
    function testSkippedOrderWritesAreThreaded() external {
        mockTransfers();
        address o = owner("bob.owner");
        fund(o);
        OrderV4[] memory list = new OrderV4[](2);
        list[0] = addOrder(o, ":set(0 1),_ _:0 1;:;");
        list[1] = addOrder(o, "used:get(0),_ _:if(used 0 1) 1;:;");
        assertTrue(take(list).isZero(), "skipped order's calculate write must be threaded");
    }

    /// Per-owner scoping: owner B writes key 0; owner A (who never wrote key 0)
    /// reads it and must NOT see B's write — A fills. Interleaving an unrelated
    /// owner between A's two orders also must not leak. Batch [A1, Bwriter, A2]
    /// fills all three (total 3). A flat (un-scoped) overlay would leak B's
    /// write into A2 and drop it -> total 2.
    function testCrossOwnerNoLeakInterleaved() external {
        mockTransfers();
        address a = owner("ownerA");
        address b = owner("ownerB");
        fund(a);
        fund(b);
        OrderV4[] memory list = new OrderV4[](3);
        list[0] = addOrder(a, "_ _:1 1;:;");
        list[1] = addOrder(b, ":set(0 1),_ _:1 1;:;");
        list[2] = addOrder(a, "used:get(0),_ _:if(used 0 1) 1;:;");
        assertTrue(take(list).eq(LibDecimalFloat.packLossless(3, 0)), "owner A must not see owner B's key 0 write");
    }

    /// Cumulative counter threaded across repeats: each fill requires the stored
    /// counter < 2 and increments it. Repeated 5x in one batch fills exactly
    /// twice (n = 0,1), proving values (not just a boolean flag) thread
    /// correctly and that skipped repeats still advance the counter via `set`.
    /// Pre-fix: every repeat reads 0 and fills -> total 5.
    function testCumulativeCounterThreadsAcrossRepeats() external {
        mockTransfers();
        address o = owner("carol");
        fund(o);
        OrderV4 memory order = addOrder(o, "n:get(0),:set(0 add(n 1)),_ _:if(less-than(n 2) 1 0) 1;:;");
        OrderV4[] memory list = new OrderV4[](5);
        for (uint256 i = 0; i < 5; i++) {
            list[i] = order;
        }
        assertTrue(take(list).eq(LibDecimalFloat.packLossless(2, 0)), "counter must cap fills at 2 within the batch");
    }

    /// The fix does not change cross-transaction persistence: a "fill once"
    /// order filled in one `takeOrders4` is exhausted in a later one (the
    /// calculate write is still persisted by handleIO). tx1 fills 1, tx2 fills 0.
    function testCrossTransactionPersistenceUnchanged() external {
        mockTransfers();
        address o = owner("dave");
        fund(o);
        OrderV4 memory order = addOrder(o, "used:get(0),:set(0 1),_ _:if(used 0 1) 1;:;");
        OrderV4[] memory single = new OrderV4[](1);
        single[0] = order;
        assertTrue(take(single).eq(LibDecimalFloat.packLossless(1, 0)), "tx1 fills once");
        assertTrue(take(single).isZero(), "tx2 sees persisted state and skips");
    }

    /// Threaded writes are key-specific: A writes key 0; B reads key 1 (a
    /// different key) and must miss, filling normally. Total 2. Guards against
    /// an overlay that seeds values under the wrong key.
    function testThreadingIsKeySpecific() external {
        mockTransfers();
        address o = owner("erin");
        fund(o);
        OrderV4[] memory list = new OrderV4[](2);
        list[0] = addOrder(o, ":set(0 1),_ _:1 1;:;");
        list[1] = addOrder(o, "used:get(1),_ _:if(used 0 1) 1;:;");
        assertTrue(take(list).eq(LibDecimalFloat.packLossless(2, 0)), "read of an unset key must miss");
    }

    /// A dead order (never added -> OrderNotFound) interleaved in the batch is
    /// not evaluated, so it must not advance the per-owner accumulator: a later
    /// same-owner order still sees the earlier order's write. [A writes, DEAD,
    /// B reads] -> B skips -> total 1. Catches an accumulator that indexes by
    /// loop position instead of evaluated count.
    function testDeadOrderInterleavedDoesNotCorruptAccumulator() external {
        mockTransfers();
        address o = owner("frank");
        fund(o);
        OrderV4 memory a = addOrder(o, ":set(0 1),_ _:1 1;:;");
        OrderV4 memory b = addOrder(o, "used:get(0),_ _:if(used 0 1) 1;:;");
        // A never-added order (distinct nonce) with matching tokens is dead.
        OrderV4 memory dead = OrderV4({
            owner: a.owner,
            evaluable: a.evaluable,
            validInputs: a.validInputs,
            validOutputs: a.validOutputs,
            nonce: bytes32(uint256(0xdead))
        });
        OrderV4[] memory list = new OrderV4[](3);
        list[0] = a;
        list[1] = dead;
        list[2] = b;
        assertTrue(take(list).eq(LibDecimalFloat.packLossless(1, 0)), "dead order must not break same-owner threading");
    }
}
