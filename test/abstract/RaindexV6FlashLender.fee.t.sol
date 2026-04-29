// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalMockTest} from "test/util/abstract/RaindexV6ExternalMockTest.sol";

/// @title RaindexV6FlashLenderFeeTest
/// Tests the fee charged by `RaindexV6FlashLender`.
contract RaindexV6FlashLenderFeeTest is RaindexV6ExternalMockTest {
    /// Tests that the fee charged by `RaindexV6FlashLender` is 0.
    function testFlashFee(address token, uint256 amount) public view {
        assertEq(iRaindex.flashFee(token, amount), 0);
    }
}
