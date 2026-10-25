// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6FreshTakeOrderTest} from "test/util/abstract/RaindexV6FreshTakeOrderTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {OrderV4, QuoteV2, SignedContextV1} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";

/// `quote2` surfaces a reverting calculate eval as the eval's own revert data.
contract RaindexV6QuoteCalculateRevertTest is RaindexV6FreshTakeOrderTest {
    function testQuoteCalculateRevertReverts() external {
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm,
            address(uint160(uint256(keccak256("alice.rain.test")))),
            ":ensure(0 \"calc\"),_ _:1 1;:;",
            address(iToken0),
            bytes32(uint256(1)),
            address(iToken1),
            bytes32(uint256(1))
        );

        vm.expectRevert(bytes("calc"));
        iRaindex.quote2(QuoteV2(order, 0, 0, new SignedContextV1[](0)));
    }
}
