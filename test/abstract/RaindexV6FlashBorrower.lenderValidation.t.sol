// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {GenericPoolRaindexV6FlashBorrower} from "../../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
import {BadLender} from "../../src/abstract/RaindexV6FlashBorrower.sol";
import {MaliciousLender} from "test/util/concrete/MaliciousLender.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";

contract RaindexV6FlashBorrowerLenderValidationTest is Test {
    /// A malicious lender calls onFlashLoan directly. The call reverts with
    /// BadLender because msg.sender is not the configured raindex.
    function testMaliciousLenderCannotExploitOnFlashLoan() external {
        MockToken token = new MockToken("Token", "TKN", 18);

        GenericPoolRaindexV6FlashBorrower arb = new GenericPoolRaindexV6FlashBorrower();

        MaliciousLender attacker = new MaliciousLender();

        vm.expectRevert(abi.encodeWithSelector(BadLender.selector, address(attacker)));
        attacker.attack(arb, address(token), 0, hex"");
    }
}
