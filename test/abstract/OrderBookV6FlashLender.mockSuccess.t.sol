// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";
import {
    IERC3156FlashBorrower,
    ON_FLASH_LOAN_CALLBACK_SUCCESS
} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

/// @title OrderBookV6FlashLenderMockSuccessTest
/// Show that if the receiver is `OrderBookV6FlashBorrower` and the token
/// movements do not error, then the flash loan will succeed.
contract OrderBookV6FlashLenderMockSuccessTest is OrderBookV6ExternalMockTest {
    /// Tests that if the receiver is `OrderBookV6FlashBorrower` and the token
    /// movements do not error, then the flash loan will succeed.
    function testFlashLoanToReceiver(uint256 amount, bytes memory data) public {
        // Return true for all transfers.
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));

        // A call to a contract that implements `IERC3156FlashBorrower` will
        // succeed if the return value is `ON_FLASH_LOAN_CALLBACK_SUCCESS`.
        address receiver = address(0xDEADBEEF);
        vm.etch(receiver, hex"FE");
        vm.mockCall(
            receiver,
            abi.encodeWithSelector(IERC3156FlashBorrower.onFlashLoan.selector),
            abi.encode(ON_FLASH_LOAN_CALLBACK_SUCCESS)
        );
        assertTrue(
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data)
        );
    }
}
