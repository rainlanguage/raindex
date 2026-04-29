// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {GenericPoolRaindexV6FlashBorrower} from "../../../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    TakeOrderConfigV4,
    OrderV4,
    IOV2,
    EvaluableV4,
    SignedContextV1,
    TaskV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @dev Exchange that re-enters arb4 when called during _exchange.
contract ReentrantExchange {
    GenericPoolRaindexV6FlashBorrower internal immutable iArb;
    IRaindexV6 internal immutable iRaindex;

    constructor(GenericPoolRaindexV6FlashBorrower arb, IRaindexV6 raindex) {
        iArb = arb;
        iRaindex = raindex;
    }

    /// Called by pool.functionCallWithValue during _exchange. Re-enters arb4.
    fallback() external payable {
        // Build minimal valid-looking args. The reentrancy guard will revert
        // before any of this is actually processed.
        IOV2[] memory ios = new IOV2[](1);
        ios[0] = IOV2(address(0x1), bytes32(0));

        OrderV4 memory order = OrderV4({
            owner: address(0x1234),
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            validInputs: ios,
            validOutputs: ios,
            nonce: bytes32(0)
        });

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));

        iArb.arb4(
            iRaindex,
            TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(1, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: ""
            }),
            hex"",
            TaskV2({
                evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
