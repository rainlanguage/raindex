// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {ChildGatedRaindexV6ArbOrderTaker} from "test/util/concrete/ChildGatedRaindexV6ArbOrderTaker.sol";
import {RaindexV6ArbConfig, WrongTask} from "../../src/abstract/RaindexV6ArbCommon.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    TakeOrderConfigV4,
    TaskV2,
    EvaluableV4,
    SignedContextV1
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterStoreV3.sol";
import {LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {LibInterpreterDeploy} from "rainlang-0.1.2/src/lib/deploy/LibInterpreterDeploy.sol";

/// @title RaindexV6ArbOrderTakerBeforeArbOrderTest
/// @notice Pins the ORDERING of `arb5`: `_beforeArb(task)` is called BEFORE the
/// zero-orders check. On a task-gated arb, supplying empty orders together with
/// a task that does NOT match the configured hash MUST revert `WrongTask` (the
/// gate runs first), NOT `NoOrders`. If `_beforeArb` were moved below the
/// `orders.length == 0` check, the empty-orders path would revert `NoOrders`
/// before the gate ran, so the discriminator is the SELECTOR of the revert.
contract RaindexV6ArbOrderTakerBeforeArbOrderTest is Test {
    function _gatedTask(bytes memory bytecode) internal pure returns (TaskV2 memory) {
        return TaskV2({
            evaluable: EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                bytecode
            ),
            signedContext: new SignedContextV1[](0)
        });
    }

    /// With empty orders AND a non-matching task, the gate reverts WrongTask
    /// (it ran before the zero-orders check).
    function testBeforeArbRunsBeforeZeroOrdersCheck() external {
        // Configure the gate with a nonzero-bytecode task so iTaskHash != 0.
        TaskV2 memory configuredTask = _gatedTask(hex"01");
        ChildGatedRaindexV6ArbOrderTaker arb = new ChildGatedRaindexV6ArbOrderTaker(RaindexV6ArbConfig(configuredTask));

        // A DIFFERENT runtime task so the hash will not match -> gate reverts.
        TaskV2 memory wrongTask = _gatedTask(hex"02");

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](0);
        TakeOrdersConfigV5 memory takeOrdersConfig = TakeOrdersConfigV5({
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            orders: orders,
            data: ""
        });

        vm.expectRevert(abi.encodeWithSelector(WrongTask.selector));
        arb.arb5(IRaindexV6(address(0xdead)), takeOrdersConfig, wrongTask);
    }

    /// Control: with empty orders and a MATCHING task, the gate passes and the
    /// zero-orders check then reverts NoOrders. This proves the WrongTask above
    /// is the gate firing (not an unconditional revert), and that an
    /// empty-orders call still reverts NoOrders once the gate is satisfied.
    function testGatePassesThenNoOrders() external {
        TaskV2 memory configuredTask = _gatedTask(hex"01");
        ChildGatedRaindexV6ArbOrderTaker arb = new ChildGatedRaindexV6ArbOrderTaker(RaindexV6ArbConfig(configuredTask));

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](0);
        TakeOrdersConfigV5 memory takeOrdersConfig = TakeOrdersConfigV5({
            minimumIO: LibDecimalFloat.packLossless(0, 0),
            maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
            maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
            IOIsInput: true,
            orders: orders,
            data: ""
        });

        vm.expectRevert(abi.encodeWithSelector(IRaindexV6.NoOrders.selector));
        arb.arb5(IRaindexV6(address(0xdead)), takeOrdersConfig, configuredTask);
    }
}
