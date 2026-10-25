// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6FreshTakeOrderTest} from "test/util/abstract/RaindexV6FreshTakeOrderTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {OrderV4, QuoteV2, SignedContextV1} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rainlang-interface-0.2.13/src/interface/IInterpreterV4.sol";

/// `quote2` surfaces a reverting calculate eval as the eval's own revert data,
/// and reverts on a no-code interpreter or undecodable returndata.
contract RaindexV6QuoteCalculateRevertTest is RaindexV6FreshTakeOrderTest {
    address internal alice = address(uint160(uint256(keccak256("alice.rain.test"))));

    function addOrder(bytes memory expression, address interpreter) internal returns (OrderV4 memory) {
        return LibTestTakeOrder.addOrderWithInterpreter(
            vm,
            alice,
            expression,
            IInterpreterV4(interpreter),
            address(iToken0),
            bytes32(uint256(1)),
            address(iToken1),
            bytes32(uint256(1))
        );
    }

    function testQuoteCalculateRevertReverts() external {
        OrderV4 memory order = addOrder(":ensure(0 \"calc\"),_ _:1 1;:;", address(iInterpreter));

        vm.expectRevert(bytes("calc"));
        iRaindex.quote2(QuoteV2(order, 0, 0, new SignedContextV1[](0)));
    }

    function testQuoteNoCodeInterpreterReverts() external {
        address interpreter = address(uint160(uint256(keccak256("no.code.rain.test"))));
        assertEq(interpreter.code.length, 0);
        OrderV4 memory order = addOrder("_ _:1 1;:;", interpreter);

        try iRaindex.quote2(QuoteV2(order, 0, 0, new SignedContextV1[](0))) {
            fail();
        } catch (bytes memory err) {
            // forge -vvv's revert diagnostic replaces exactly this empty revert data.
            assertTrue(
                err.length == 0
                    || keccak256(err)
                        == keccak256(
                            abi.encode(string.concat("call to non-contract address ", vm.toString(interpreter)))
                        )
            );
        }
    }

    function testQuoteGarbageReturnDataReverts() external {
        address interpreter = address(uint160(uint256(keccak256("malformed.interpreter.rain.test"))));
        vm.etch(interpreter, hex"00");
        vm.mockCall(interpreter, abi.encodeWithSelector(IInterpreterV4.eval4.selector), hex"deadbeef");
        OrderV4 memory order = addOrder("_ _:1 1;:;", interpreter);

        vm.expectRevert(bytes(""));
        iRaindex.quote2(QuoteV2(order, 0, 0, new SignedContextV1[](0)));
    }
}
