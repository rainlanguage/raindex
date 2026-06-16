// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    EvaluableV4,
    ClearConfigV2,
    SignedContextV1,
    TaskV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {TokenSelfTrade} from "../../../src/concrete/ob/OrderBookV6.sol";

contract OrderBookV6ClearSameTokenTest is OrderBookV6ExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testClearSameToken(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configAlice.validInputs[0].token = address(0);
        configAlice.validOutputs[0].token = address(0);
        configBob.validInputs[0].token = address(0);
        configBob.validOutputs[0].token = address(0);

        OrderV4 memory orderAlice =
            OrderV4(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV4 memory orderBob =
            OrderV4(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder4(configAlice, new TaskV2[](0));

        vm.prank(bob);
        iOrderbook.addOrder4(configBob, new TaskV2[](0));

        vm.expectRevert(abi.encodeWithSelector(TokenSelfTrade.selector));
        iOrderbook.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
