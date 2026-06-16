// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6FreshTest} from "test/util/abstract/RaindexV6FreshTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    EvaluableV4,
    TaskV2,
    SignedContextV1
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";

/// @title RaindexV6DoPostFreshTest
/// @notice Mutation-validating coverage for the `LibRaindex.doPost` task loop as
/// exercised through `entask2`: namespace = msg.sender, empty-bytecode skip, and
/// the writes -> `store.set` step. Runs against the freshly compiled source.
contract RaindexV6DoPostFreshTest is RaindexV6FreshTest {
    using LibOrder for OrderV4;

    function actionsFor(bytes[] memory evalStrings) internal view returns (TaskV2[] memory) {
        TaskV2[] memory actions = new TaskV2[](evalStrings.length);
        for (uint256 i = 0; i < evalStrings.length; i++) {
            actions[i] =
                TaskV2(EvaluableV4(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0));
        }
        return actions;
    }

    /// A write in one task MUST be visible to a later task in the same call,
    /// proving doPost applies `store.set` for writes between tasks.
    /// forge-config: default.fuzz.runs = 20
    function testFreshEntaskWriteThenReadSameCall(address alice) external {
        bytes[] memory evals = new bytes[](2);
        evals[0] = bytes(":set(1 2);");
        evals[1] = bytes(":ensure(equal-to(get(1) 2) \"set works\");");
        vm.prank(alice);
        iRaindex.entask2(actionsFor(evals));
    }

    /// State written in one entask call MUST persist into a later entask call by
    /// the same sender (writes were committed to the store under sender's ns).
    /// forge-config: default.fuzz.runs = 20
    function testFreshEntaskWritePersistsAcrossCalls(address alice) external {
        bytes[] memory writeEvals = new bytes[](1);
        writeEvals[0] = bytes(":set(5 9);");
        vm.prank(alice);
        iRaindex.entask2(actionsFor(writeEvals));

        bytes[] memory readEvals = new bytes[](1);
        readEvals[0] = bytes(":ensure(equal-to(get(5) 9) \"persisted\");");
        vm.prank(alice);
        iRaindex.entask2(actionsFor(readEvals));
    }

    /// The store namespace is keyed by msg.sender: a write by alice MUST NOT be
    /// visible to bob. Bob reading the key alice wrote sees the default (0), so
    /// asserting it equals alice's value reverts.
    /// forge-config: default.assertions_revert = false
    /// forge-config: default.legacy_assertions = true
    /// forge-config: default.fuzz.runs = 20
    function testFreshEntaskNamespacedBySender(address alice, address bob) external {
        vm.assume(alice != bob);
        bytes[] memory writeEvals = new bytes[](1);
        writeEvals[0] = bytes(":set(7 42);");
        vm.prank(alice);
        iRaindex.entask2(actionsFor(writeEvals));

        // Alice can read her own write.
        bytes[] memory aliceRead = new bytes[](1);
        aliceRead[0] = bytes(":ensure(equal-to(get(7) 42) \"alice sees\");");
        vm.prank(alice);
        iRaindex.entask2(actionsFor(aliceRead));

        // Bob CANNOT see alice's write: get(7) is 0 in bob's namespace, so the
        // equal-to-42 ensure reverts. Build the actions BEFORE arming
        // expectRevert so the parse call is not what expectRevert catches.
        bytes[] memory bobRead = new bytes[](1);
        bobRead[0] = bytes(":ensure(equal-to(get(7) 42) \"bob must not see\");");
        TaskV2[] memory bobActions = actionsFor(bobRead);
        vm.prank(bob);
        vm.expectRevert(bytes("bob must not see"));
        iRaindex.entask2(bobActions);
    }

    /// A task with empty bytecode MUST be skipped (no eval, no revert). An empty
    /// task interleaved with a real reverting task proves the empty one is a
    /// no-op while the real one still runs.
    /// forge-config: default.assertions_revert = false
    /// forge-config: default.legacy_assertions = true
    /// forge-config: default.fuzz.runs = 20
    function testFreshEntaskEmptyBytecodeSkipped(address alice) external {
        // Empty bytecode task: skipped. By itself this MUST NOT revert.
        TaskV2[] memory emptyActions = new TaskV2[](1);
        emptyActions[0] = TaskV2(EvaluableV4(iInterpreter, iStore, hex""), new SignedContextV1[](0));
        vm.prank(alice);
        iRaindex.entask2(emptyActions);

        // An empty task followed by a reverting task: the empty one is skipped
        // and the reverting one still runs, so the call reverts. Build the
        // actions (including the parse) BEFORE arming expectRevert.
        TaskV2[] memory mixed = new TaskV2[](2);
        mixed[0] = TaskV2(EvaluableV4(iInterpreter, iStore, hex""), new SignedContextV1[](0));
        mixed[1] = TaskV2(
            EvaluableV4(iInterpreter, iStore, iParserV2.parse2(bytes(":ensure(0 \"runs\");"))), new SignedContextV1[](0)
        );
        vm.prank(alice);
        vm.expectRevert(bytes("runs"));
        iRaindex.entask2(mixed);
    }

    /// entask2 with no tasks is a clean no-op.
    /// forge-config: default.fuzz.runs = 10
    function testFreshEntaskEmptyArrayNoop(address alice) external {
        vm.prank(alice);
        iRaindex.entask2(new TaskV2[](0));
    }

    /// A revert in any entask task MUST revert the whole call (per the NatSpec:
    /// "If ANY of the expressions revert, the entire transaction MUST revert").
    /// forge-config: default.assertions_revert = false
    /// forge-config: default.legacy_assertions = true
    /// forge-config: default.fuzz.runs = 10
    function testFreshEntaskAnyRevertReverts(address alice) external {
        bytes[] memory evals = new bytes[](2);
        evals[0] = bytes("_:1;");
        evals[1] = bytes(":ensure(0 \"second reverts\");");
        // Build the actions (parse) BEFORE arming expectRevert so it catches the
        // entask2 revert, not the parse call.
        TaskV2[] memory actions = actionsFor(evals);
        vm.prank(alice);
        vm.expectRevert(bytes("second reverts"));
        iRaindex.entask2(actions);
    }
}
