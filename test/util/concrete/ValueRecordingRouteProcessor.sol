// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IRouteProcessor} from "src/interface/IRouteProcessor.sol";

/// @dev Route processor mock that records the `msg.value` it was called with in
/// storage slot 0, so a test can assert how much native ETH the arb forwarded.
contract ValueRecordingRouteProcessor is IRouteProcessor {
    function processRoute(address, uint256, address, uint256, address, bytes memory)
        external
        payable
        returns (uint256)
    {
        // Slot 0 holds the last `msg.value` seen, readable via `vm.load` from
        // the etched address.
        assembly ("memory-safe") {
            sstore(0, callvalue())
        }
        return 0;
    }
}
