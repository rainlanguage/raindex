// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6} from "src/concrete/raindex/RaindexV6.sol";
import {Float} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

contract RaindexV6DustCreditHarness is RaindexV6 {
    function exposedPull(address account, address token, Float amount) external returns (uint256, uint8) {
        return pullTokens(account, token, amount);
    }

    function exposedPush(address account, address token, Float amount) external returns (uint256, uint8) {
        return pushTokens(account, token, amount);
    }

    function exposedDustCredit(address user, address token) external view returns (Float) {
        return sDustCredit[user][token];
    }
}
