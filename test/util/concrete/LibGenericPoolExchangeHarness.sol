// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibGenericPoolExchange} from "src/lib/LibGenericPoolExchange.sol";

/// @dev Harness exposing the internal `exchange` for direct testing.
contract LibGenericPoolExchangeHarness {
    function exchange(address token, bytes memory data) external {
        LibGenericPoolExchange.exchange(token, data);
    }
}
