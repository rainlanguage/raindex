// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {LibTestArb, OrderTakerSetup} from "test/util/lib/LibTestArb.sol";
import {MockExchange} from "test/util/concrete/MockExchange.sol";
import {ZeroExchangeSpender, ZeroExchangePool} from "src/lib/LibGenericPoolExchange.sol";

/// The order-taker routes `takeOrdersConfig.data` into `onTakeOrders2 ->
/// LibGenericPoolExchange.exchange`. A zero spender/pool in that data reverts
/// through the real `arb5` path (the guards fire after take-orders, before any
/// approval/call).
contract GenericPoolRaindexV6ArbOrderTakerZeroAddressTest is Test {
    function testArbZeroSpenderReverts() external {
        MockExchange exchange = new MockExchange();
        OrderTakerSetup memory setup = LibTestArb.setup(vm, address(exchange), 100e18);

        setup.takeOrdersConfig.data = abi.encode(address(0), address(exchange), bytes(""));
        vm.expectRevert(ZeroExchangeSpender.selector);
        setup.arb.arb5(setup.raindex, setup.takeOrdersConfig, LibTestArb.noopTask());
    }

    function testArbZeroPoolReverts() external {
        MockExchange exchange = new MockExchange();
        OrderTakerSetup memory setup = LibTestArb.setup(vm, address(exchange), 100e18);

        setup.takeOrdersConfig.data = abi.encode(address(exchange), address(0), bytes(""));
        vm.expectRevert(ZeroExchangePool.selector);
        setup.arb.arb5(setup.raindex, setup.takeOrdersConfig, LibTestArb.noopTask());
    }
}
