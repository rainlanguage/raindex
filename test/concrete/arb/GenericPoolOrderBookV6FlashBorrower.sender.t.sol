// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {GenericPoolOrderBookV6FlashBorrower} from "../../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    EvaluableV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";

contract GenericPoolOrderBookV6FlashBorrowerTest is ArbTest {
    function buildArb() internal override returns (address payable) {
        return payable(address(new GenericPoolOrderBookV6FlashBorrower()));
    }

    constructor() ArbTest() {}

    /// forge-config: default.fuzz.runs = 10
    function testGenericPoolOrderBookV6FlashBorrowerTakeOrdersSender(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex
    ) public {
        TakeOrderConfigV4[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        GenericPoolOrderBookV6FlashBorrower(iArb)
            .arb4(
                iOrderBook,
                TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: ""
            }),
                abi.encode(iRefundoor, iRefundoor, ""),
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
