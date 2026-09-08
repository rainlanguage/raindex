// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.2/src/Test.sol";
import {ChildRaindexV6ArbTaskGated} from "test/util/concrete/ChildRaindexV6ArbTaskGated.sol";
import {RaindexV6ArbConfig} from "../../src/abstract/RaindexV6ArbCommon.sol";
import {TaskV2, EvaluableV4, SignedContextV1} from "raindex-interface-0.1.5/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rainlang-interface-0.2.8/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rainlang-interface-0.2.8/src/interface/IInterpreterStoreV3.sol";
import {
    TEST_INTERPRETER_ADDRESS,
    TEST_STORE_ADDRESS
} from "rainlang-0.2.1/test/lib/deploy/LibTestInterpreterDeploy.sol";

/// When constructed with empty bytecode the task hash MUST be zero.
contract RaindexV6ArbTaskGatedITaskHashZeroTest is Test {
    function testITaskHashZeroEmptyBytecode() external {
        TaskV2 memory task = TaskV2(
            EvaluableV4(IInterpreterV4(TEST_INTERPRETER_ADDRESS), IInterpreterStoreV3(TEST_STORE_ADDRESS), ""),
            new SignedContextV1[](0)
        );
        ChildRaindexV6ArbTaskGated child = new ChildRaindexV6ArbTaskGated(RaindexV6ArbConfig(task));
        assertEq(child.iTaskHash(), bytes32(0));
    }
}
