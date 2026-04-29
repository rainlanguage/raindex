// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {OrderV4, TakeOrdersConfigV5, TaskV2, IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";

/// When an order's IORatio exceeds the taker's maximumIORatio, the order
/// is skipped and OrderExceedsMaxRatio is emitted.
contract OrderBookV6TakeOrderExceedsMaxRatioTest is OrderBookV6ExternalRealTest {
    using LibDecimalFloat for Float;

    function testTakeOrderExceedsMaxRatio() external {
        address alice = address(uint160(uint256(keccak256("alice.rain.test"))));
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));

        // Deposit so the order can fill.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iOrderbook)),
            abi.encode(true)
        );
        vm.prank(alice);
        iOrderbook.deposit4(
            address(iToken1), bytes32(uint256(0x01)), LibDecimalFloat.packLossless(10, 0), new TaskV2[](0)
        );

        // Order with outputMax=1 and IORatio=100.
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm,
            alice,
            "_ _:1 100;:;",
            address(iToken0),
            bytes32(uint256(0x01)),
            address(iToken1),
            bytes32(uint256(0x01))
        );

        // maximumIORatio = 1, but order ratio is 100 -> skip.
        TakeOrdersConfigV5 memory takeConfig = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        takeConfig.maximumIORatio = LibDecimalFloat.packLossless(1, 0);

        vm.prank(bob);
        vm.expectEmit(true, true, true, true);
        emit IRaindexV6.OrderExceedsMaxRatio(bob, alice, LibOrder.hash(order));
        (Float totalTakerInput, Float totalTakerOutput) = iOrderbook.takeOrders4(takeConfig);

        assertTrue(totalTakerInput.isZero(), "totalTakerInput must be zero");
        assertTrue(totalTakerOutput.isZero(), "totalTakerOutput must be zero");
    }
}
