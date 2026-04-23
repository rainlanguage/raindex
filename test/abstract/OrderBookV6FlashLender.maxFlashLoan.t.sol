// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";

/// @title OrderBookV6FlashLenderMaxFlashLoanTest
/// Tests the maximum flash loan amount for `OrderBookV6FlashLender`.
contract OrderBookV6FlashLenderMaxFlashLoanTest is OrderBookV6ExternalMockTest {
    /// Tests that the maximum flash loan amount for `OrderBookV6FlashLender` is
    /// the balance of the token in the order book.
    function testFlashMaxLoan(uint256 amount) public {
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.balanceOf.selector, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            abi.encode(amount)
        );
        assertEq(IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).maxFlashLoan(address(iToken0)), amount);
    }
}
