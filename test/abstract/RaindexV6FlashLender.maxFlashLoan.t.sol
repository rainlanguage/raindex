// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalMockTest} from "test/util/abstract/RaindexV6ExternalMockTest.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

/// @title RaindexV6FlashLenderMaxFlashLoanTest
/// Tests the maximum flash loan amount for `RaindexV6FlashLender`.
contract RaindexV6FlashLenderMaxFlashLoanTest is RaindexV6ExternalMockTest {
    /// Tests that the maximum flash loan amount for `RaindexV6FlashLender` is
    /// the balance of the token in the raindex.
    function testFlashMaxLoan(uint256 amount) public {
        vm.mockCall(
            address(iToken0), abi.encodeWithSelector(IERC20.balanceOf.selector, address(iRaindex)), abi.encode(amount)
        );
        assertEq(iRaindex.maxFlashLoan(address(iToken0)), amount);
    }
}
