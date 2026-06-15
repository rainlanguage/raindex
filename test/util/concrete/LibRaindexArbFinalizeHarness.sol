// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibRaindexArb} from "src/lib/LibRaindexArb.sol";
import {TaskV2, SignedContextV1, EvaluableV4} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterStoreV3.sol";

/// @dev Exposes the internal `finalizeArb` so its zero-balance sweep guards can
/// be exercised directly. `finalizeArb` sends to `msg.sender`, which here is the
/// caller of `callFinalize` (the test, or a reverting-receiver proxy).
contract LibRaindexArbFinalizeHarness {
    function callFinalize(address inputToken, uint8 inputDecimals, address outputToken, uint8 outputDecimals) external {
        TaskV2 memory task = TaskV2({
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            signedContext: new SignedContextV1[](0)
        });
        LibRaindexArb.finalizeArb(task, inputToken, inputDecimals, outputToken, outputDecimals);
    }

    receive() external payable {}
}
