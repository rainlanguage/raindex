// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    IOV2,
    EvaluableV4,
    TaskV2,
    IRaindexV6
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotRainMetaV1, META_MAGIC_NUMBER_V1} from "rain.metadata/interface/unstable/IMetaV1_2.sol";
import {LibMeta} from "rain.metadata/lib/LibMeta.sol";
import {IExpressionDeployerV3} from "rain.interpreter.interface/interface/deprecated/v1/IExpressionDeployerV3.sol";

/// @title OrderBookV6AddOrderMockTest
/// @notice Tests the addOrder function of the OrderBookV6 contract.
contract OrderBookV6AddOrderMockTest is OrderBookV6ExternalMockTest {
    /// Adding an order without calculations does not revert.
    /// This is a runtime error.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderWithoutCalculationsDeploys(address owner, OrderConfigV4 memory config) public {
        vm.prank(owner);
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = "";
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
        (OrderV4 memory order, bytes32 orderHash) = LibTestAddOrder.expectedOrder(owner, config);
        (order);
        assertTrue(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderHash));
    }

    /// Adding an order without inputs reverts.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderWithoutInputsReverts(address owner, OrderConfigV4 memory config) public {
        vm.prank(owner);
        config.evaluable.bytecode = hex"02000000040000000000000000";
        config.validInputs = new IOV2[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoInputs.selector));
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
        (OrderV4 memory order, bytes32 orderHash) = LibTestAddOrder.expectedOrder(owner, config);
        (order);
        assertTrue(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderHash));
    }

    /// Adding an order without token outputs reverts.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderWithoutOutputsReverts(address owner, OrderConfigV4 memory config) public {
        vm.prank(owner);
        config.evaluable.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        config.validOutputs = new IOV2[](0);
        vm.expectRevert(abi.encodeWithSelector(OrderNoOutputs.selector));
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
        (OrderV4 memory order, bytes32 orderHash) = LibTestAddOrder.expectedOrder(owner, config);
        (order);
        assertTrue(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderHash));
    }

    /// Adding an order with calculations, inputs and outputs will succeed if
    /// the expression is valid according to the deployer. The resulting order
    /// MUST be emitted. This test assumes empty meta.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderWithCalculationsInputsAndOutputsSucceeds(
        address owner,
        OrderConfigV4 memory config,
        bytes memory expression
    ) public {
        config.evaluable.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        config.meta = new bytes(0);
        (OrderV4 memory order, bytes32 orderhash) = addOrderWithChecks(owner, config, expression);
        (order);
        (orderhash);
    }

    /// Adding a valid order with a non-empty meta MUST revert if the meta is
    /// not self describing as a rain meta document.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderWithNonEmptyMetaReverts(address owner, OrderConfigV4 memory config, bytes memory) public {
        vm.prank(owner);
        config.evaluable.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        vm.assume(!LibMeta.isRainMetaV1(config.meta));
        vm.assume(config.meta.length > 0);

        vm.expectRevert(abi.encodeWithSelector(NotRainMetaV1.selector, config.meta));
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));

        (OrderV4 memory order, bytes32 orderHash) = LibTestAddOrder.expectedOrder(owner, config);
        (order);
        assertTrue(!IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).orderExists(orderHash));
    }

    /// Adding a valid order with a non-empty meta MUST emit MetaV1 if the meta
    /// is self describing as a rain meta document.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderWithNonEmptyMetaEmitsMetaV1(
        address owner,
        OrderConfigV4 memory config,
        bytes memory expression
    ) public {
        config.evaluable.bytecode = hex"02000000040000000000000000";
        vm.assume(config.validInputs.length > 0);
        vm.assume(config.validOutputs.length > 0);
        vm.assume(config.meta.length > 0);

        // This is a bit of a hack, but it's the easiest way to get a valid
        // meta document.
        config.meta = abi.encodePacked(META_MAGIC_NUMBER_V1, config.meta);

        (OrderV4 memory order, bytes32 orderHash) = addOrderWithChecks(owner, config, expression);
        (order);
        (orderHash);
    }

    /// Alice and Bob can add orders with the same config. The resulting orders
    /// MUST be different.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderTwoAccountsWithSameConfig(
        address alice,
        address bob,
        OrderConfigV4 memory config,
        bytes memory expression
    ) public {
        vm.assume(alice != bob);
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        (OrderV4 memory aliceOrder, bytes32 aliceOrderHash) = addOrderWithChecks(alice, config, expression);
        (OrderV4 memory bobOrder, bytes32 bobOrderHash) = addOrderWithChecks(bob, config, expression);
        (aliceOrder);
        (bobOrder);
        assertTrue(aliceOrderHash != bobOrderHash);
    }

    /// Alice and Bob can add orders with different configs. The resulting orders
    /// MUST be different.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderTwoAccountsWithDifferentConfig(
        address alice,
        address bob,
        OrderConfigV4 memory aliceConfig,
        OrderConfigV4 memory bobConfig,
        bytes memory aliceExpression,
        bytes memory bobExpression
    ) public {
        vm.assume(alice != bob);
        LibTestAddOrder.conformConfig(aliceConfig, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(bobConfig, iInterpreter, iStore);
        (OrderV4 memory aliceOrder, bytes32 aliceOrderHash) = addOrderWithChecks(alice, aliceConfig, aliceExpression);
        (OrderV4 memory bobOrder, bytes32 bobOrderHash) = addOrderWithChecks(bob, bobConfig, bobExpression);
        (aliceOrder);
        (bobOrder);
        assertTrue(aliceOrderHash != bobOrderHash);
    }

    /// Alice can add orders with different configs. The resulting orders MUST
    /// be different.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderSameAccountWithDifferentConfig(
        address alice,
        OrderConfigV4 memory configOne,
        OrderConfigV4 memory configTwo,
        bytes memory expressionOne,
        bytes memory expressionTwo
    ) public {
        LibTestAddOrder.conformConfig(configOne, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configTwo, iInterpreter, iStore);
        (OrderV4 memory expectedOrderOne, bytes32 expectedOrderOneHash) =
            LibTestAddOrder.expectedOrder(alice, configOne);
        (OrderV4 memory expectedOrderTwo, bytes32 expectedOrderTwoHash) =
            LibTestAddOrder.expectedOrder(alice, configTwo);
        (expectedOrderOne);
        (expectedOrderTwo);
        assertTrue(expectedOrderOneHash != expectedOrderTwoHash);
        (OrderV4 memory orderOne, bytes32 orderOneHash) = addOrderWithChecks(alice, configOne, expressionOne);
        (OrderV4 memory orderTwo, bytes32 orderTwoHash) = addOrderWithChecks(alice, configTwo, expressionTwo);
        (orderOne);
        (orderTwo);
        assertTrue(orderOneHash != orderTwoHash);
    }
}
