// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {
    OrderConfigV4,
    OrderV4,
    TaskV2,
    TakeOrdersConfigV5,
    IRaindexV6
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {UnsupportedCalculateOutputs} from "../../../src/concrete/ob/OrderBookV6.sol";

contract OrderBookV6TakeOrderBadStackTest is OrderBookV6ExternalRealTest {
    function checkBadStack(
        address alice,
        address bob,
        OrderConfigV4 memory config,
        bytes memory rainString,
        uint256 badStackHeight
    ) internal {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        config.evaluable.bytecode = iParserV2.parse2(rainString);

        OrderV4 memory order = OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        TakeOrdersConfigV5 memory takeOrdersConfig =
            LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        config.validInputs[0].token = address(iToken0);
        config.validOutputs[0].token = address(iToken1);

        vm.prank(alice);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));

        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, badStackHeight));
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).takeOrders4(takeOrdersConfig);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackEmptyStack(address alice, address bob, OrderConfigV4 memory config) external {
        checkBadStack(alice, bob, config, ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testTakeOrderBadStackOneStack(address alice, address bob, OrderConfigV4 memory config) external {
        checkBadStack(alice, bob, config, "_:1;:;", 1);
    }
}
