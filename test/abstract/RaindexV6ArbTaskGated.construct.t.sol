// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {ChildRaindexV6ArbTaskGated} from "test/util/concrete/ChildRaindexV6ArbTaskGated.sol";
import {RaindexV6ArbConfig, RaindexV6ArbCommon} from "../../src/abstract/RaindexV6ArbCommon.sol";
import {TaskV2, EvaluableV4, SignedContextV1} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";

/// Construction MUST emit the Construct event with the sender and config.
contract RaindexV6ArbTaskGatedConstructTest is Test {
    function testConstructEmitsEvent(bytes memory bytecode) external {
        TaskV2 memory task = TaskV2(
            EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                bytecode
            ),
            new SignedContextV1[](0)
        );
        RaindexV6ArbConfig memory config = RaindexV6ArbConfig(task);

        vm.expectEmit(false, false, false, true);
        emit RaindexV6ArbCommon.Construct(address(this), config);
        new ChildRaindexV6ArbTaskGated(config);
    }
}
