// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {LibOrder, OrderV4} from "../../src/lib/LibOrder.sol";

/// @title LibOrderTest
/// Exercises the LibOrder library.
contract LibOrderTest is Test {
    /// Hashing should always produce the same result for the same input.
    /// forge-config: default.fuzz.runs = 100
    function testHashEqual(OrderV4 memory a) public pure {
        assertTrue(LibOrder.hash(a) == LibOrder.hash(a));
    }

    /// Hashing should always produce different results for different inputs.
    /// forge-config: default.fuzz.runs = 100
    function testHashNotEqual(OrderV4 memory a, OrderV4 memory b) public pure {
        // Only test with actually different inputs.
        vm.assume(keccak256(abi.encode(a)) != keccak256(abi.encode(b)));
        assertTrue(LibOrder.hash(a) != LibOrder.hash(b));
    }

    /// Mutating a single field should always produce a different hash.
    /// forge-config: default.fuzz.runs = 100
    function testHashNotEqualMutatedOwner(OrderV4 memory a, address otherOwner) public pure {
        vm.assume(a.owner != otherOwner);
        OrderV4 memory b = OrderV4({
            owner: otherOwner,
            evaluable: a.evaluable,
            validInputs: a.validInputs,
            validOutputs: a.validOutputs,
            nonce: a.nonce
        });
        assertTrue(LibOrder.hash(a) != LibOrder.hash(b));
    }
}
