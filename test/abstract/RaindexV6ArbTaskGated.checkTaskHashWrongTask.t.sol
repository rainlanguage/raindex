// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {ChildRaindexV6ArbTaskGated} from "test/util/concrete/ChildRaindexV6ArbTaskGated.sol";
import {RaindexV6ArbConfig, WrongTask} from "../../src/abstract/RaindexV6ArbCommon.sol";
import {TaskV2, EvaluableV4, SignedContextV1} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";

/// _checkTaskHash MUST revert with WrongTask when iTaskHash is nonzero and
/// the runtime task does not match.
contract RaindexV6ArbTaskGatedCheckTaskHashWrongTaskTest is Test {
    function testCheckTaskHashRevertsWrongTask(bytes memory constructBytecode, bytes memory runtimeBytecode) external {
        vm.assume(constructBytecode.length > 0);
        vm.assume(runtimeBytecode.length > 0);

        TaskV2 memory constructTask = TaskV2(
            EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                constructBytecode
            ),
            new SignedContextV1[](0)
        );
        TaskV2 memory runtimeTask = TaskV2(
            EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                runtimeBytecode
            ),
            new SignedContextV1[](0)
        );

        // Only revert when the tasks are actually different.
        vm.assume(keccak256(abi.encode(constructTask)) != keccak256(abi.encode(runtimeTask)));

        ChildRaindexV6ArbTaskGated child = new ChildRaindexV6ArbTaskGated(RaindexV6ArbConfig(constructTask));

        vm.expectRevert(abi.encodeWithSelector(WrongTask.selector));
        child.checkTaskHash(runtimeTask);
    }
}
