// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IRaindexV6
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {SignedContextV1} from "rain-interpreter-interface-0.1.0/src/interface/deprecated/v1/IInterpreterCallerV2.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// `takeOrders4` guards a zero maximum IO before dereferencing any order IO
/// index, so a zero max with an out-of-range index reverts with the explicit
/// `ZeroMaximumIO` rather than a generic out-of-bounds bounds-check panic.
contract RaindexV6TakeOrderZeroMaxOOBIndexTest is RaindexV6ExternalRealTest {
    using LibDecimalFloat for Float;

    address internal alice = address(uint160(uint256(keccak256("alice.rain.test"))));
    address internal bob = address(uint160(uint256(keccak256("bob.rain.test"))));

    function _addOrder(bytes memory expression) internal returns (OrderV4 memory) {
        return LibTestTakeOrder.addOrderWithExpression(
            vm, alice, expression, address(iToken0), bytes32(uint256(0x01)), address(iToken1), bytes32(uint256(0x01))
        );
    }

    /// A zero maximum IO reverts with `ZeroMaximumIO` even when the take config's
    /// `inputIOIndex` is out of range for the order's `validInputs`. The zero-max
    /// guard runs before the index is dereferenced.
    function testZeroMaxWithOOBInputIOIndex() external {
        // The order has a single input (index 0); index 1 is out of range.
        OrderV4 memory order = _addOrder("_ _:1e0 1e0;:;");

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 1, 0, new SignedContextV1[](0));

        TakeOrdersConfigV5 memory config = TakeOrdersConfigV5({
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            orders: orders,
            data: ""
        });

        vm.prank(bob);
        vm.expectRevert(IRaindexV6.ZeroMaximumIO.selector);
        iRaindex.takeOrders4(config);
    }
}
