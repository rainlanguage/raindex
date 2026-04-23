// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";

/// @title OrderBookV6FlashLenderFeeTest
/// Tests the fee charged by `OrderBookV6FlashLender`.
contract OrderBookV6FlashLenderFeeTest is OrderBookV6ExternalMockTest {
    /// Tests that the fee charged by `OrderBookV6FlashLender` is 0.
    function testFlashFee(address token, uint256 amount) public view {
        assertEq(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).flashFee(token, amount), 0);
    }
}
