// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

/// @title LibGenericPoolExchange
/// @notice Shared approve-call-revoke pattern for generic pool exchanges.
library LibGenericPoolExchange {
    using SafeERC20 for IERC20;
    using Address for address;

    /// @notice Decodes `(spender, pool, encodedFunctionCall)` from `data`,
    /// approves `spender` to spend `token`, calls `pool` forwarding the
    /// contract's entire ETH balance, then revokes the approval.
    /// @param token The token to approve for the exchange.
    /// @param data ABI-encoded `(address spender, address pool, bytes encodedFunctionCall)`.
    function exchange(address token, bytes memory data) internal {
        (address spender, address pool, bytes memory encodedFunctionCall) = abi.decode(data, (address, address, bytes));

        // Approve-call-revoke: the caller controls spender and pool, which is
        // safe because the contract holds no tokens or ETH between arb
        // operations — there is nothing for a malicious caller to extract.
        IERC20(token).forceApprove(spender, type(uint256).max);
        //slither-disable-next-line unused-return
        pool.functionCallWithValue(encodedFunctionCall, address(this).balance);
        IERC20(token).forceApprove(spender, 0);
    }
}
