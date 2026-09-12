// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "test/util/abstract/ArbTest.sol";

import {GenericPoolRaindexV6FlashBorrower} from "../../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
import {ArbRequiresInputIO} from "../../src/abstract/RaindexV6FlashBorrower.sol";
import {
    IRaindexV6,
    EvaluableV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    IInterpreterV4,
    IInterpreterStoreV3,
    TaskV2,
    SignedContextV1
} from "raindex-interface-0.1.5/src/interface/IRaindexV6.sol";
import {LibDecimalFloat} from "rain-math-float-0.2.1/src/lib/LibDecimalFloat.sol";
import {
    TEST_INTERPRETER_ADDRESS,
    TEST_STORE_ADDRESS
} from "rainlang-0.2.4/test/lib/deploy/LibTestInterpreterDeploy.sol";

contract RaindexV6FlashBorrowerIOIsInputTest is ArbTest {
    function buildArb() internal override returns (address payable) {
        return payable(address(new GenericPoolRaindexV6FlashBorrower()));
    }

    constructor() ArbTest() {}

    /// arb4 derives the flash-loan amount from `minimumIO` as the order output the
    /// taker receives, which only matches when `minimumIO` is the taker input. It
    /// MUST revert with `ArbRequiresInputIO` when `IOIsInput == false`, before any
    /// flash loan is taken. The single order is never inspected (the guard reverts
    /// first), so a default entry suffices.
    function testArb4RevertsWhenIOIsInputFalse() external {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);

        vm.expectRevert(abi.encodeWithSelector(ArbRequiresInputIO.selector));
        GenericPoolRaindexV6FlashBorrower(iArb)
            .arb4(
                iRaindex,
                TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: false,
                orders: orders,
                data: ""
            }),
                "",
                TaskV2({
                evaluable: EvaluableV4(
                    IInterpreterV4(TEST_INTERPRETER_ADDRESS), IInterpreterStoreV3(TEST_STORE_ADDRESS), ""
                ),
                signedContext: new SignedContextV1[](0)
            })
            );
    }
}
