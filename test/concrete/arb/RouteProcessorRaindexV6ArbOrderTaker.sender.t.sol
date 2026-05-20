// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    RouteProcessorRaindexV6ArbOrderTakerTest
} from "test/util/abstract/RouteProcessorRaindexV6ArbOrderTakerTest.sol";
import {
    OrderV4,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";

contract RouteProcessorRaindexV6ArbOrderTakerSenderTest is RouteProcessorRaindexV6ArbOrderTakerTest {
    /// forge-config: default.fuzz.runs = 100
    function testRouteProcessorTakeOrdersSender(OrderV4 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        RouteProcessorRaindexV6ArbOrderTaker(iArb)
            .arb5(
                iRaindex,
                TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: abi.encode(bytes("0x00"))
            }),
                TaskV2({
                evaluable: EvaluableV4(
                    IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                    IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                    ""
                ),
                signedContext: new SignedContextV1[](0)
            })
            );
    }
}
