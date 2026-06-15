// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibRaindexArbFinalizeHarness} from "test/util/concrete/LibRaindexArbFinalizeHarness.sol";

/// @dev Calls the harness and reverts if it is ever sent ANY native value,
/// including zero (`Address.sendValue` performs a `call{value: 0}` even for a
/// zero amount). Used to prove `finalizeArb` skips the gas sweep when the
/// balance is zero.
contract RevertOnZeroReceive {
    function go(LibRaindexArbFinalizeHarness harness, address inputToken, address outputToken) external {
        harness.callFinalize(inputToken, 18, outputToken, 18);
    }

    receive() external payable {
        revert("no value");
    }
}
