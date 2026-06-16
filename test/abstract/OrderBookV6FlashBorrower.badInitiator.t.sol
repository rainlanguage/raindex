// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {GenericPoolOrderBookV6FlashBorrower} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {BadInitiator} from "../../src/abstract/OrderBookV6FlashBorrower.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";

contract OrderBookV6FlashBorrowerBadInitiatorTest is Test {
    GenericPoolOrderBookV6FlashBorrower arb;

    constructor() {
        arb = new GenericPoolOrderBookV6FlashBorrower();
    }

    /// onFlashLoan MUST revert with BadInitiator when called with an initiator
    /// that is not the flash borrower contract itself.
    function testOnFlashLoanBadInitiator(address badInitiator) external {
        vm.assume(badInitiator != address(arb));

        // Prank as the deterministic orderbook address to isolate the
        // BadInitiator check.
        vm.prank(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
        vm.expectRevert(abi.encodeWithSelector(BadInitiator.selector, badInitiator));
        arb.onFlashLoan(badInitiator, address(0), 0, 0, abi.encode(new bytes(0), new bytes(0)));
    }
}
