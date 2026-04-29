// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalMockTest} from "test/util/abstract/RaindexV6ExternalMockTest.sol";
import {
    IERC3156FlashBorrower,
    ON_FLASH_LOAN_CALLBACK_SUCCESS
} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

/// @title RaindexV6FlashLenderMockSuccessTest
/// Show that if the receiver is `RaindexV6FlashBorrower` and the token
/// movements do not error, then the flash loan will succeed.
contract RaindexV6FlashLenderMockSuccessTest is RaindexV6ExternalMockTest {
    /// Tests that if the receiver is `RaindexV6FlashBorrower` and the token
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
        assertTrue(iRaindex.flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data));
    }
}
