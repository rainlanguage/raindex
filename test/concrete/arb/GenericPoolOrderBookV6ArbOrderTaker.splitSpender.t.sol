// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibTestArb, OrderTakerSetup} from "test/util/lib/LibTestArb.sol";
import {SpenderProxy, SplitSpenderPool} from "test/util/concrete/SplitSpenderExchange.sol";

/// When spender != pool in the exchange data encoded in takeOrders.data,
/// the approval targets the spender while the call targets the pool.
contract GenericPoolOrderBookV6ArbOrderTakerSplitSpenderTest is Test {
    function testSplitSpenderExchange() external {
        SpenderProxy spender = new SpenderProxy();
        SplitSpenderPool pool = new SplitSpenderPool(spender);
        OrderTakerSetup memory setup = LibTestArb.setup(vm, address(spender), address(pool), 100e18);

        setup.arb.arb5(setup.orderBook, setup.takeOrdersConfig, LibTestArb.noopTask());

        // Spender approval was revoked after the exchange.
        assertEq(setup.outputToken.allowance(address(setup.arb), address(spender)), 0, "spender allowance revoked");
        // Pool never had approval.
        assertEq(setup.outputToken.allowance(address(setup.arb), address(pool)), 0, "pool never had allowance");
    }
}
