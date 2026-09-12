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
} from "rainlang-0.2.4/test/lib/deploy/LibTestInterpreterDeploy.sol";

/// _checkTaskHash MUST pass through when iTaskHash is zero, regardless of
/// the task provided at runtime.
contract RaindexV6ArbTaskGatedCheckTaskHashPassZeroTest is Test {
    function testCheckTaskHashPassesWhenZero(bytes memory runtimeBytecode) external {
        // Construct with empty bytecode so iTaskHash == 0.
        TaskV2 memory constructTask = TaskV2(
            EvaluableV4(IInterpreterV4(TEST_INTERPRETER_ADDRESS), IInterpreterStoreV3(TEST_STORE_ADDRESS), ""),
            new SignedContextV1[](0)
        );
        ChildRaindexV6ArbTaskGated child = new ChildRaindexV6ArbTaskGated(RaindexV6ArbConfig(constructTask));

        // Any runtime task should pass.
        TaskV2 memory runtimeTask = TaskV2(
            EvaluableV4(
                IInterpreterV4(TEST_INTERPRETER_ADDRESS), IInterpreterStoreV3(TEST_STORE_ADDRESS), runtimeBytecode
            ),
            new SignedContextV1[](0)
        );
        child.checkTaskHash(runtimeTask);
    }
}
