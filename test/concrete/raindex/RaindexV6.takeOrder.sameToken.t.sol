// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    TaskV2,
    TakeOrderConfigV4,
    SignedContextV1,
    TakeOrdersConfigV5
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {TokenSelfTrade} from "../../../src/concrete/raindex/RaindexV6.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

contract RaindexV6TakeOrderSameTokenTest is RaindexV6ExternalRealTest {
    /// forge-config: default.fuzz.runs = 10
    function testTakeOrderSameToken(address alice, OrderConfigV4 memory configAlice) external {
        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        configAlice.validInputs[0].token = address(0);
        configAlice.validOutputs[0].token = address(0);

        OrderV4 memory orderAlice =
            OrderV4(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);

        vm.prank(alice);
        iRaindex.addOrder4(configAlice, new TaskV2[](0));

        TakeOrderConfigV4[] memory takeOrders = new TakeOrderConfigV4[](1);
        takeOrders[0] = TakeOrderConfigV4({
            order: orderAlice, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)
        });

        TakeOrdersConfigV5 memory takeOrdersConfig = TakeOrdersConfigV5({
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            orders: takeOrders,
            data: ""
        });

        vm.expectRevert(abi.encodeWithSelector(TokenSelfTrade.selector));
        iRaindex.takeOrders4(takeOrdersConfig);
    }
}
