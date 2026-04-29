// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {OrderConfigV4, OrderV4, EvaluableV4, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";

contract OrderBookV6AddOrderOwnerTest is OrderBookV6ExternalRealTest {
    using LibOrder for OrderV4;

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderOwnerSameOrderNoop(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        OrderV4 memory order = OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.prank(owner);
        bool stateChange = iOrderbook.addOrder4(config, new TaskV2[](0));
        assert(stateChange);
        assert(iOrderbook.orderExists(order.hash()));

        vm.prank(owner);
        stateChange = iOrderbook.addOrder4(config, new TaskV2[](0));
        assert(!stateChange);
        assert(iOrderbook.orderExists(order.hash()));
    }

    /// forge-config: default.fuzz.runs = 100
    function testAddOrderOwnerDifferentOwnerStateChange(OrderConfigV4 memory config, address alice, address bob)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV4 memory orderAlice =
            OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
        OrderV4 memory orderBob = OrderV4(bob, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.prank(alice);
        bool stateChange = iOrderbook.addOrder4(config, new TaskV2[](0));
        assert(stateChange);
        assert(iOrderbook.orderExists(orderAlice.hash()));

        vm.prank(bob);
        stateChange = iOrderbook.addOrder4(config, new TaskV2[](0));
        assert(stateChange);
        assert(iOrderbook.orderExists(orderBob.hash()));
    }
}
