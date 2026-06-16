// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {ChildOrderBookV6ArbTaskGated} from "test/util/concrete/ChildOrderBookV6ArbTaskGated.sol";
import {OrderBookV6ArbConfig, OrderBookV6ArbCommon} from "../../src/abstract/OrderBookV6ArbCommon.sol";
import {TaskV2, EvaluableV4, SignedContextV1} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";

/// Construction MUST emit the Construct event with the sender and config.
contract OrderBookV6ArbTaskGatedConstructTest is Test {
    function testConstructEmitsEvent(bytes memory bytecode) external {
        TaskV2 memory task = TaskV2(
            EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                bytecode
            ),
            new SignedContextV1[](0)
        );
        OrderBookV6ArbConfig memory config = OrderBookV6ArbConfig(task);

        vm.expectEmit(false, false, false, true);
        emit OrderBookV6ArbCommon.Construct(address(this), config);
        new ChildOrderBookV6ArbTaskGated(config);
    }
}
