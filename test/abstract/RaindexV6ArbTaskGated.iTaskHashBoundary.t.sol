// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {ChildRaindexV6ArbTaskGated} from "test/util/concrete/ChildRaindexV6ArbTaskGated.sol";
import {RaindexV6ArbConfig} from "../../src/abstract/RaindexV6ArbCommon.sol";
import {TaskV2, EvaluableV4, SignedContextV1} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterStoreV3.sol";
import {LibInterpreterDeploy} from "rainlang-0.1.2/src/lib/deploy/LibInterpreterDeploy.sol";

/// The constructor branch on `bytecode.length != 0` MUST trip at the exact
/// boundary of one byte: empty bytecode leaves `iTaskHash` at zero (branch not
/// taken), but a SINGLE byte of bytecode is enough to store the keccak hash.
/// This pins the comparator to `!= 0` (i.e. `>= 1`); an off-by-one mutation
/// such as `> 1` would wrongly leave a one-byte task ungated.
contract RaindexV6ArbTaskGatedITaskHashBoundaryTest is Test {
    function buildTask(bytes memory bytecode) internal pure returns (TaskV2 memory) {
        return TaskV2(
            EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                bytecode
            ),
            new SignedContextV1[](0)
        );
    }

    /// Zero-length bytecode: hash stays exactly zero.
    function testITaskHashBoundaryZeroLength() external {
        TaskV2 memory task = buildTask("");
        ChildRaindexV6ArbTaskGated child = new ChildRaindexV6ArbTaskGated(RaindexV6ArbConfig(task));
        assertEq(child.iTaskHash(), bytes32(0), "empty bytecode must leave iTaskHash zero");
    }

    /// One-byte bytecode is over the boundary: hash is the keccak of the task
    /// and is necessarily nonzero.
    function testITaskHashBoundaryOneByteLength() external {
        TaskV2 memory task = buildTask(hex"01");
        ChildRaindexV6ArbTaskGated child = new ChildRaindexV6ArbTaskGated(RaindexV6ArbConfig(task));
        bytes32 expected = keccak256(abi.encode(task));
        assertEq(child.iTaskHash(), expected, "one byte of bytecode must store the keccak task hash");
        assertTrue(child.iTaskHash() != bytes32(0), "one-byte task hash must be nonzero");
    }
}
