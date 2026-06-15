// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

/// @dev Accepts any call (including empty calldata) so the empty-encoded-call
/// path can be observed.
contract FallbackPool {
    bool public called;

    fallback() external payable {
        called = true;
    }

    receive() external payable {
        called = true;
    }
}
