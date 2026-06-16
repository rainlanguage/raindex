// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {stdError} from "forge-std/Test.sol";
import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {ERC20, IERC20Errors} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import {
    IERC3156FlashBorrower,
    ON_FLASH_LOAN_CALLBACK_SUCCESS
} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC3156FlashLender} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashLender.sol";
import {FlashLenderCallbackFailed} from "../../src/abstract/OrderBookV6FlashLender.sol";

contract TKN is ERC20 {
    constructor(address recipient, uint256 supply) ERC20("TKN", "TKN") {
        _mint(recipient, supply);
    }
}

interface IPull {
    function pull(address token, uint256 amount) external;
}

/// Alice has some daisy contract pull tokens from her.
/// If the tokens are returned to her then she can complete the flash loan else
/// the loan must be reverted.
contract Alice is IERC3156FlashBorrower {
    IPull immutable iPull;
    bool immutable iSuccess;

    constructor(IPull pull, bool success) {
        iPull = pull;
        iSuccess = success;
    }

    function onFlashLoan(address, address token, uint256 amount, uint256, bytes calldata)
        public
        override
        returns (bytes32)
    {
        // Approve the puller to pull the tokens.
        IERC20(token).approve(address(iPull), amount);
        iPull.pull(token, amount);
        // Approve the lender to pull the tokens back and repay the loan.
        IERC20(token).approve(msg.sender, amount);
        // Magic number for success.
        return iSuccess ? ON_FLASH_LOAN_CALLBACK_SUCCESS : bytes32(0);
    }
}

/// Bob pulls the tokens from Alice then returns them so she can repay the loan.
contract Bob is IPull {
    using SafeERC20 for IERC20;

    function pull(address token, uint256 amount) public override {
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        IERC20(token).safeTransfer(msg.sender, amount);
    }
}

/// Carol pulls tokens from Alice then returns only some of them so the loan
/// fails.
contract Carol is IPull {
    using SafeERC20 for IERC20;

    uint256 immutable iAmountWithheld;

    constructor(uint256 amountWithheld) {
        iAmountWithheld = amountWithheld;
    }

    function pull(address token, uint256 amount) public override {
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        IERC20(token).safeTransfer(msg.sender, amount - iAmountWithheld);
    }
}

/// @title OrderBookV6FlashLenderTransferTest
/// Tests the `OrderBookV6FlashLender` transfer functions.
contract OrderBookV6FlashLenderTransferTest is OrderBookV6ExternalMockTest {
    /// Alice can send tokens to Bob, who will return them and then the loan will
    /// be repaid.
    /// forge-config: default.fuzz.runs = 100
    function testFlashLoanTransferSuccess(uint256 amount, bool success) public {
        TKN tkn = new TKN(address(iOrderbook), amount);

        Bob bob = new Bob();
        Alice alice = new Alice(IPull(address(bob)), success);

        if (!success) {
            vm.expectRevert(abi.encodeWithSelector(FlashLenderCallbackFailed.selector, bytes32(0)));
        }
        bool result = iOrderbook.flashLoan(IERC3156FlashBorrower(address(alice)), address(tkn), amount, "");
        if (success) {
            assertTrue(result);
        }
    }

    /// Alice can send tokens to Carol, who will return not all of them and then
    /// the loan will fail.
    /// forge-config: default.fuzz.runs = 100
    function testFlashLoanTransferFail(uint256 amount, uint256 amountWithheld, bool success) public {
        amount = bound(amount, 1, type(uint256).max);
        amountWithheld = bound(amountWithheld, 1, amount);
        TKN tkn = new TKN(address(iOrderbook), amount);

        Carol carol = new Carol(amountWithheld);
        Alice alice = new Alice(IPull(address(carol)), success);

        if (!success) {
            vm.expectRevert(abi.encodeWithSelector(FlashLenderCallbackFailed.selector, bytes32(0)));
        } else {
            vm.expectRevert(
                abi.encodeWithSelector(
                    IERC20Errors.ERC20InsufficientBalance.selector, address(alice), amount - amountWithheld, amount
                )
            );
        }
        iOrderbook.flashLoan(IERC3156FlashBorrower(address(alice)), address(tkn), amount, "");
    }
}
