// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

/// @dev Approval target that can pull tokens on behalf of whoever approved it.
contract SpenderProxy {
    using SafeERC20 for IERC20;

    function pullFrom(IERC20 token, address from, address to, uint256 amount) external {
        token.safeTransferFrom(from, to, amount);
    }
}

/// @dev Pool that executes swaps by pulling tokenIn via a separate SpenderProxy
/// and sending tokenOut back to the caller. Mimics DEXes where the approval
/// target differs from the contract you call (e.g. Permit2, TokenTransferProxy).
contract SplitSpenderPool {
    using SafeERC20 for IERC20;

    SpenderProxy public immutable iSpender;

    constructor(SpenderProxy spender) {
        iSpender = spender;
    }

    function swap(IERC20 tokenIn, IERC20 tokenOut, uint256 amount) external payable {
        iSpender.pullFrom(tokenIn, msg.sender, address(this), amount);
        tokenOut.safeTransfer(msg.sender, amount);
    }

    receive() external payable {}
}
