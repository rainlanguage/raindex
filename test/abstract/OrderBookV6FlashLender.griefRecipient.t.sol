// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {
    IERC3156FlashBorrower,
    ON_FLASH_LOAN_CALLBACK_SUCCESS
} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {FlashLenderCallbackFailed} from "../../src/abstract/OrderBookV6FlashLender.sol";

/// @title OrderBookV6FlashLenderGriefRecipientTest
/// Try to grief the recipient of the flash loan.
contract OrderBookV6FlashLenderGriefRecipientTest is OrderBookV6ExternalMockTest {
    /// Tests that no matter who the receiver is, and no matter what happens with
    /// the tokens, the flash loan will revert if the receiver is not
    /// `OrderBookV6FlashBorrower`.
    function testFlashLoanToNonReceiver(
        uint256 amount,
        bytes memory data,
        bytes32 notFlashLoanSuccess,
        bytes memory notFlashLoanSuccessBytes
    ) public {
        vm.assume(notFlashLoanSuccess != ON_FLASH_LOAN_CALLBACK_SUCCESS);
        vm.assume(keccak256(notFlashLoanSuccessBytes) != keccak256(abi.encode(ON_FLASH_LOAN_CALLBACK_SUCCESS)));

        // Return true for all transfers.
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));

        // A call to an EOA will revert with no data.
        address receiver = address(0xDEADBEEF);
        vm.expectRevert();
        iOrderbook.flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data);

        // A call to a contract that does not implement `IERC3156FlashBorrower`
        // will revert with `FlashLenderCallbackFailed`.
        vm.etch(receiver, hex"FE");
        vm.mockCall(
            receiver,
            abi.encodeWithSelector(IERC3156FlashBorrower.onFlashLoan.selector),
            abi.encode(notFlashLoanSuccess)
        );
        vm.expectRevert(abi.encodeWithSelector(FlashLenderCallbackFailed.selector, notFlashLoanSuccess));
        iOrderbook.flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data);

        // A call to a contract that does not implement `IERC3156FlashBorrower`
        // will revert with no data if the return value is not `bytes32`.
        vm.mockCall(
            receiver, abi.encodeWithSelector(IERC3156FlashBorrower.onFlashLoan.selector), notFlashLoanSuccessBytes
        );
        vm.expectRevert();
        iOrderbook.flashLoan(IERC3156FlashBorrower(receiver), address(iToken0), amount, data);
    }
}
