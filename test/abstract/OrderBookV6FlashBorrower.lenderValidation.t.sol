// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {GenericPoolOrderBookV6FlashBorrower} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {BadLender} from "../../src/abstract/OrderBookV6FlashBorrower.sol";
import {MaliciousLender} from "test/util/concrete/MaliciousLender.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";

contract OrderBookV6FlashBorrowerLenderValidationTest is Test {
    /// A malicious lender calls onFlashLoan directly. The call reverts with
    /// BadLender because msg.sender is not the configured orderbook.
    function testMaliciousLenderCannotExploitOnFlashLoan() external {
        MockToken token = new MockToken("Token", "TKN", 18);

        GenericPoolOrderBookV6FlashBorrower arb = new GenericPoolOrderBookV6FlashBorrower();

        MaliciousLender attacker = new MaliciousLender();

        vm.expectRevert(abi.encodeWithSelector(BadLender.selector, address(attacker)));
        attacker.attack(arb, address(token), 0, hex"");
    }
}
