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

/// @title RaindexV6TakeOrderBatchStateBypassTest
/// @notice Audit Protofire H01 (#2617): `takeOrders4` evaluates every order's
/// calculate entrypoint with an empty `stateOverlay` and only persists the
/// calculate-phase `kvs` later in `handleIO`, after the whole order loop. With
/// no per-batch uniqueness, a taker can repeat the same order in one call and
/// each calculate evaluation sees the same stale store state — so a stateful
/// per-order cap enforced in the calculate phase is bypassed within a batch.
///
/// This test demonstrates the bypass with a "fill once" order: the calculate
/// phase reads a store flag, marks it via `set`, and offers output only while
/// the flag is unset. Across separate transactions the cap holds, but repeating
/// the order three times in a single `takeOrders4` fills three times.
contract RaindexV6TakeOrderBatchStateBypassTest is RaindexV6ExternalRealTest {
    using LibDecimalFloat for Float;

    /// A "fill once" order: outputMax is 1 only while store key 0 is unset; the
    /// calculate phase reads the flag (before setting it) and `set`s it. Because
    /// the calculate `kvs` are deferred to handleIO, repeated evaluations in the
    /// same batch all read the unset flag.
    function testH01RepeatedOrderBypassesCalculatePhaseCap() external {
        address owner = address(uint160(uint256(keccak256("owner.rain.test"))));
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));

        // Fund the order's output vault generously so the vault balance is never
        // the binding constraint — only the stateful cap should limit fills.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iRaindex)),
            abi.encode(true)
        );
        vm.prank(owner);
        iRaindex.deposit4(
            address(iToken1), bytes32(uint256(0x01)), LibDecimalFloat.packLossless(10, 0), new TaskV2[](0)
        );

        // calculate: read flag, set flag, offer 1 only while flag is unset (ratio 1).
        // handle: no-op.
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm,
            owner,
            "used:get(0),:set(0 1),_ _:if(used 0 1) 1;:;",
            address(iToken0),
            bytes32(uint256(0x01)),
            address(iToken1),
            bytes32(uint256(0x01))
        );

        // The taker submits the SAME order three times in one batch.
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](3);
        for (uint256 i = 0; i < 3; i++) {
            orders[i] = TakeOrderConfigV4({
                order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)
            });
        }
        TakeOrdersConfigV5 memory config = LibTestTakeOrder.defaultTakeConfig(orders);

        // Allow the settlement transfers (taker receives token1, pays token0).
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transfer.selector, bob), abi.encode(true));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iRaindex)),
            abi.encode(true)
        );

        vm.prank(bob);
        (Float totalTakerInput,) = iRaindex.takeOrders4(config);

        // The order's calculate-phase cap intends to fill at most once (total
        // output 1). H01: the repeated order bypasses it and fills three times.
        // This assertion is the intended behaviour and currently FAILS (yields
        // 3) — it should pass once calculate-phase writes are threaded forward
        // via `stateOverlay`.
        assertTrue(
            totalTakerInput.eq(LibDecimalFloat.packLossless(1, 0)),
            "stateful calculate-phase cap must hold within a single takeOrders4 batch"
        );
    }
}
