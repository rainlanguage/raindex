// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {
    IRaindexV6,
    OrderConfigV4,
    OrderV4,
    EvaluableV4,
    ClearConfigV2,
    SignedContextV1,
    TaskV2
} from "raindex-interface-0.1.3/src/interface/IRaindexV6.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";

contract RaindexV6ClearSameTokenTest is RaindexV6ExternalRealTest {
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
        iRaindex.addOrder4(configAlice, new TaskV2[](0));

        vm.prank(bob);
        iRaindex.addOrder4(configBob, new TaskV2[](0));

        vm.expectRevert(abi.encodeWithSelector(IRaindexV6.TokenSelfTrade.selector));
        iRaindex.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
