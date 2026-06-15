// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {TakeOrdersConfigV5, IRaindexV6} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IRaindexV6OrderTaker} from "raindex-interface-0.1.1/src/interface/IRaindexV6OrderTaker.sol";
import {Float} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// Records onTakeOrders2 invocation + the args it was called with so a test can
/// assert the callback fires only when something was taken and with the right
/// token/amount arguments.
contract RecordingTaker is IRaindexV6OrderTaker {
    uint256 public calls;
    address public lastOutputToken;
    address public lastInputToken;
    Float public lastTakerInput;
    Float public lastTakerOutput;
    bytes public lastData;
    IRaindexV6 internal immutable iRaindex;

    constructor(IRaindexV6 raindex) {
        iRaindex = raindex;
    }

    function take(TakeOrdersConfigV5 memory config) external returns (Float, Float) {
        return iRaindex.takeOrders4(config);
    }

    function onTakeOrders2(
        address outputToken,
        address inputToken,
        Float takerInput,
        Float takerOutput,
        bytes calldata data
    ) external {
        calls++;
        lastOutputToken = outputToken;
        lastInputToken = inputToken;
        lastTakerInput = takerInput;
        lastTakerOutput = takerOutput;
        lastData = data;
    }
}
