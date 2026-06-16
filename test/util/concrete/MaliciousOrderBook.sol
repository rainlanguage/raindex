// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {IERC3156FlashBorrower} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {MockOrderBookBase} from "test/util/abstract/MockOrderBookBase.sol";

/// @dev Malicious orderbook that records the token allowances it has from the
/// borrower during flashLoan, before calling onFlashLoan (which will revert
/// with BadLender since this contract is not the deterministic orderbook).
contract MaliciousOrderBook is MockOrderBookBase {
    uint256 public inputAllowanceDuringFlashLoan;
    uint256 public outputAllowanceDuringFlashLoan;
    address public inputToken;
    address public outputToken;

    function setTokens(address input, address output) external {
        inputToken = input;
        outputToken = output;
    }

    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        override
        returns (bool)
    {
        // Record the allowances the arb contract granted us before the
        // lender validation in onFlashLoan fires.
        inputAllowanceDuringFlashLoan = IERC20(inputToken).allowance(address(receiver), address(this));
        outputAllowanceDuringFlashLoan = IERC20(outputToken).allowance(address(receiver), address(this));

        // This will revert with BadLender since we are not the deterministic
        // orderbook address.
        receiver.onFlashLoan(address(receiver), token, amount, 0, data);

        return true;
    }
}
