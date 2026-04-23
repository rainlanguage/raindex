// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {OrderV4, TakeOrdersConfigV5, TaskV2, IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";

/// When an order evaluates to outputMax=0, the order is skipped and
/// OrderZeroAmount is emitted.
contract OrderBookV6TakeOrderZeroAmountTest is OrderBookV6ExternalRealTest {
    using LibDecimalFloat for Float;

    function testTakeOrderZeroAmount() external {
        address alice = address(uint160(uint256(keccak256("alice.rain.test"))));
        address bob = address(uint160(uint256(keccak256("bob.rain.test"))));

        // Deposit so the order has vault balance.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            abi.encode(true)
        );
        vm.prank(alice);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
            .deposit4(address(iToken1), bytes32(uint256(0x01)), LibDecimalFloat.packLossless(10, 0), new TaskV2[](0));

        // Order with outputMax=0 and IORatio=1 — zero output means skip.
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm, alice, "_ _:0 1;:;", address(iToken0), bytes32(uint256(0x01)), address(iToken1), bytes32(uint256(0x01))
        );

        TakeOrdersConfigV5 memory takeConfig = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));

        vm.prank(bob);
        vm.expectEmit(true, true, true, true);
        emit IRaindexV6.OrderZeroAmount(bob, alice, LibOrder.hash(order));
        (Float totalTakerInput, Float totalTakerOutput) =
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).takeOrders4(takeConfig);

        assertTrue(totalTakerInput.isZero(), "totalTakerInput must be zero");
        assertTrue(totalTakerOutput.isZero(), "totalTakerOutput must be zero");
    }
}
