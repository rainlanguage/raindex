// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {TakeOrdersConfigV5, IRaindexV6} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IRaindexV6OrderTaker} from "raindex-interface-0.1.1/src/interface/IRaindexV6OrderTaker.sol";
import {Float} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// Records whether `onTakeOrders2` was invoked, so a test can assert the
/// callback fires only when at least one order was actually taken.
contract RecordingOrderTaker is IRaindexV6OrderTaker {
    bool public called;
    IRaindexV6 internal immutable iRaindex;

    constructor(IRaindexV6 raindex) {
        iRaindex = raindex;
    }

    function take(TakeOrdersConfigV5 memory config) external returns (Float, Float) {
        return iRaindex.takeOrders4(config);
    }

    function onTakeOrders2(address, address, Float, Float, bytes calldata) external {
        called = true;
    }
}
