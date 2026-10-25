// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6FreshTakeOrderTest} from "test/util/abstract/RaindexV6FreshTakeOrderTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {OrderV4, ClearConfigV2, SignedContextV1} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rainlang-interface-0.2.13/src/interface/IInterpreterV4.sol";

/// A clear cannot drop one side, so a reverting calculate eval reverts
/// `clear3` with the eval's own revert data, and undecodable returndata reverts.
contract RaindexV6ClearCalculateRevertTest is RaindexV6FreshTakeOrderTest {
    address internal alice = address(uint160(uint256(keccak256("alice.rain.test"))));
    address internal bob = address(uint160(uint256(keccak256("bob.rain.test"))));

    function checkClearReverts(
        bytes memory aliceExpression,
        address aliceInterpreter,
        bytes memory bobExpression,
        address bobInterpreter,
        bytes memory err
    ) internal {
        OrderV4 memory aliceOrder = LibTestTakeOrder.addOrderWithInterpreter(
            vm,
            alice,
            aliceExpression,
            IInterpreterV4(aliceInterpreter),
            address(iToken0),
            bytes32(uint256(1)),
            address(iToken1),
            bytes32(uint256(1))
        );
        OrderV4 memory bobOrder = LibTestTakeOrder.addOrderWithInterpreter(
            vm,
            bob,
            bobExpression,
            IInterpreterV4(bobInterpreter),
            address(iToken1),
            bytes32(uint256(1)),
            address(iToken0),
            bytes32(uint256(1))
        );

        vm.expectRevert(err);
        iRaindex.clear3(
            aliceOrder, bobOrder, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }

    function malformedInterpreter() internal returns (address) {
        address interpreter = address(uint160(uint256(keccak256("malformed.interpreter.rain.test"))));
        vm.etch(interpreter, hex"00");
        vm.mockCall(interpreter, abi.encodeWithSelector(IInterpreterV4.eval4.selector), hex"deadbeef");
        return interpreter;
    }

    function testClearAliceCalculateRevertReverts() external {
        checkClearReverts(
            ":ensure(0 \"alice calc\"),_ _:1 1;:;",
            address(iInterpreter),
            "_ _:1 1;:;",
            address(iInterpreter),
            "alice calc"
        );
    }

    function testClearBobCalculateRevertReverts() external {
        checkClearReverts(
            "_ _:1 1;:;", address(iInterpreter), ":ensure(0 \"bob calc\"),_ _:1 1;:;", address(iInterpreter), "bob calc"
        );
    }

    function testClearAliceGarbageReturnDataReverts() external {
        checkClearReverts("_ _:1 1;:;", malformedInterpreter(), "_ _:1 1;:;", address(iInterpreter), "");
    }

    function testClearBobGarbageReturnDataReverts() external {
        checkClearReverts("_ _:1 1;:;", address(iInterpreter), "_ _:1 1;:;", malformedInterpreter(), "");
    }
}
