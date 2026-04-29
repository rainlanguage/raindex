// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {OrderConfigV4, OrderV4, ClearConfigV2, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {SignedContextV1} from "rain.interpreter.interface/interface/deprecated/v1/IInterpreterCallerV2.sol";
import {SameOwner} from "../../../src/concrete/ob/OrderBookV6.sol";

/// Clearing two orders with the same owner must revert with SameOwner.
contract OrderBookV6ClearSameOwnerTest is OrderBookV6ExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testClearSameOwner(address alice, OrderConfigV4 memory configAlice, OrderConfigV4 memory configBob)
        external
    {
        assumeEtchable(alice);

        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configAlice.validOutputs[0].token = address(iToken0);
        configAlice.validInputs[0].token = address(iToken1);
        configBob.validInputs[0].token = address(iToken0);
        configBob.validOutputs[0].token = address(iToken1);

        // Add both orders as the same owner.
        vm.prank(alice);
        vm.recordLogs();
        iOrderbook.addOrder4(configAlice, new TaskV2[](0));
        OrderV4 memory orderAlice = LibTestTakeOrder.extractOrderFromLogs(vm.getRecordedLogs());

        vm.prank(alice);
        vm.recordLogs();
        iOrderbook.addOrder4(configBob, new TaskV2[](0));
        OrderV4 memory orderBob = LibTestTakeOrder.extractOrderFromLogs(vm.getRecordedLogs());

        vm.expectRevert(abi.encodeWithSelector(SameOwner.selector));
        iOrderbook.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
