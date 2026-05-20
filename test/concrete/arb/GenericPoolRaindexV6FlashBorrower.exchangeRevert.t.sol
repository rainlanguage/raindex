// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibTestFlashBorrowerArb, FlashBorrowerSetup} from "test/util/lib/LibTestFlashBorrowerArb.sol";
import {LibTestArb} from "test/util/lib/LibTestArb.sol";
import {RevertingExchange} from "test/util/concrete/RevertingExchange.sol";

/// If the exchange call reverts, the entire arb4 reverts with the
/// exchange's revert reason bubbled up.
contract GenericPoolRaindexV6FlashBorrowerExchangeRevertTest is Test {
    function testExchangeRevertPropagates() external {
        RevertingExchange exchange = new RevertingExchange();
        FlashBorrowerSetup memory setup = LibTestFlashBorrowerArb.setup(vm, address(exchange), 100e18);

        vm.expectRevert("exchange failed");
        setup.arb.arb4(setup.raindex, setup.takeOrdersConfig, setup.exchangeData, LibTestArb.noopTask());
    }
}
