// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {OrderConfigV4, EvaluableV4, TaskV2, IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {UnsupportedCalculateOutputs} from "../../../src/concrete/ob/OrderBookV6.sol";

/// @title OrderBookV6AddOrderTest
/// @notice A test harness for testing the OrderBookV6 addOrder function.
contract OrderBookV6AddOrderTest is OrderBookV6ExternalRealTest {
    /// No sources deploys as we let this be a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealNoSourcesDeploys(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        config.evaluable.bytecode = hex"";
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// No handle IO reverts.
    /// This is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealNoHandleIODeploys(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2(":;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// A stack of 0 for calculate order deploys.
    /// Stack size checks are runtime.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealZeroStackCalculateReverts(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2(":;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// A stack of 1 for calculate order reverts.
    /// Stack size checks are runtime.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealOneStackCalculateReverts(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_:block-timestamp();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// A stack of 2 for calculate order deploys.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealTwoStackCalculateReverts(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _:block-timestamp() chain-id();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// A stack of 3 for calculate order deploys.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealThreeStackCalculate(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("_ _ _:block-timestamp() chain-id() block-number();:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// Inputs for calculate order. Tests one input.
    /// Deploys because this is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealCalculateInputsReverts1(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i:;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// Inputs for calculate order errors. Tests two inputs.
    /// Deploys because this is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealCalculateInputsReverts2(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i0 i1:;:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }

    /// Inputs for calculate order errors. This takes precedent over the same
    /// error in handle io.
    /// Deploys because this is a runtime check.
    /// forge-config: default.fuzz.runs = 100
    function testAddOrderRealCalculateInputsRevertsPreference(address owner, OrderConfigV4 memory config) public {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        bytes memory bytecode = iParserV2.parse2("i:;i:;");
        config.evaluable.bytecode = bytecode;
        vm.prank(owner);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).addOrder4(config, new TaskV2[](0));
    }
}
