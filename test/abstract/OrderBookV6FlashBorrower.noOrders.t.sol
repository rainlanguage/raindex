// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {GenericPoolOrderBookV6FlashBorrower} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {
    IRaindexV6,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";

contract OrderBookV6FlashBorrowerNoOrdersTest is ArbTest {
    function buildArb() internal override returns (address payable) {
        return payable(address(new GenericPoolOrderBookV6FlashBorrower()));
    }

    constructor() ArbTest() {}

    /// arb4 MUST revert with NoOrders when given an empty orders array.
    function testArb4NoOrders() external {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](0);

        vm.expectRevert(abi.encodeWithSelector(IRaindexV6.NoOrders.selector));
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
                "",
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
