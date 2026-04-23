// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {OrderConfigV4, OrderV4, EvaluableV4, TaskV2, IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";
import {NotOrderOwner} from "../../../src/concrete/ob/OrderBookV6.sol";

contract OrderBookV6RemoveOrderOwnerTest is OrderBookV6ExternalRealTest {
    using LibOrder for OrderV4;

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderOwnerSameOrderNoop(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);

        OrderV4 memory order = OrderV4(owner, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.startPrank(owner);

        for (uint256 i = 0; i < 2; i++) {
            bool stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
            assert(stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(order.hash()));
            stateChange = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
            assert(!stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(order.hash()));

            stateChange = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(order, new TaskV2[](0));
            assert(stateChange);
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(order.hash()));
            stateChange = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(order, new TaskV2[](0));
            assert(!stateChange);
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(order.hash()));
        }

        vm.stopPrank();
    }

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderOwnerDifferentOwnerStateChange(OrderConfigV4 memory config, address alice, address bob)
        public
    {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV4 memory orderAlice =
            OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);
        OrderV4 memory orderBob = OrderV4(bob, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        {
            vm.prank(alice);
            bool stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));

            assert(stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(orderBob, new TaskV2[](0));
            assert(!stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));

            vm.prank(alice);
            stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(orderAlice, new TaskV2[](0));
            assert(stateChange);
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));
        }

        {
            vm.prank(bob);
            bool stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
            assert(stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));

            vm.prank(alice);
            stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(orderAlice, new TaskV2[](0));
            assert(!stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));

            vm.prank(bob);
            stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(orderBob, new TaskV2[](0));
            assert(stateChange);
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));
        }

        {
            vm.prank(alice);
            bool stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
            assert(stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
            assert(stateChange);
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));

            vm.prank(alice);
            stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(orderAlice, new TaskV2[](0));
            assert(stateChange);
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));
            assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));

            vm.prank(bob);
            stateChange =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(orderBob, new TaskV2[](0));
            assert(stateChange);
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderBob.hash()));
            assert(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderAlice.hash()));
        }
    }

    /// forge-config: default.fuzz.runs = 100
    function testRemoveOrderWrongOwner(OrderConfigV4 memory config, address alice, address bob) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        vm.assume(alice != bob);

        OrderV4 memory order = OrderV4(alice, config.evaluable, config.validInputs, config.validOutputs, config.nonce);

        vm.prank(alice);
        bool stateChange = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
        assert(stateChange);
        assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(order.hash()));

        vm.prank(bob);
        vm.expectRevert(abi.encodeWithSelector(NotOrderOwner.selector, alice));
        stateChange = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).removeOrder3(order, new TaskV2[](0));
        assert(!stateChange);
        assert(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(order.hash()));
    }
}
