// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std-1.16.1/src/Test.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";
import {OrderConfigV4, OrderV4, ClearConfigV2, TaskV2} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {SignedContextV1} from "rain-interpreter-interface-0.1.0/src/interface/deprecated/v1/IInterpreterCallerV2.sol";

/// @title RaindexV6ClearDeadOrderTest
/// If either order passed to `clear3` is dead (never added, or removed) the
/// clear is a no-op: it emits `OrderNotFound` for the dead order, returns
/// without reverting (so a bulk Multicall clear is not aborted), and makes no
/// vault state change. This is distinct from the take-orders dead path and was
/// previously untested for `clear3`.
contract RaindexV6ClearDeadOrderTest is RaindexV6ExternalRealTest {
    using LibOrder for OrderV4;

    /// Build two token-compatible orders for alice and bob, returning their
    /// configs. Tokens are wired so the clear passes the TokenMismatch and
    /// TokenSelfTrade guards and reaches the dead-order checks.
    function buildConfigs(OrderConfigV4 memory configAlice, OrderConfigV4 memory configBob) internal view {
        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configAlice.validOutputs[0].token = address(iToken0);
        configAlice.validInputs[0].token = address(iToken1);
        configBob.validOutputs[0].token = address(iToken1);
        configBob.validInputs[0].token = address(iToken0);
    }

    function addOrder(address owner, OrderConfigV4 memory config) internal returns (OrderV4 memory) {
        vm.prank(owner);
        vm.recordLogs();
        iRaindex.addOrder4(config, new TaskV2[](0));
        return LibTestTakeOrder.extractOrderFromLogs(vm.getRecordedLogs());
    }

    /// Alice's order is dead (never added). Bob's order is irrelevant because
    /// the alice check comes first: clear3 emits OrderNotFound for alice's hash
    /// and returns, with no other logs and no revert.
    /// forge-config: default.fuzz.runs = 10
    function testClearAliceDead(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        vm.assume(alice != bob);
        assumeEtchable(alice);
        assumeEtchable(bob);
        buildConfigs(configAlice, configBob);

        OrderV4 memory orderAlice =
            OrderV4(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV4 memory orderBob =
            OrderV4(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        assertFalse(iRaindex.orderExists(orderAlice.hash()));

        vm.expectEmit(address(iRaindex));
        emit OrderNotFound(address(this), alice, orderAlice.hash());
        vm.recordLogs();
        iRaindex.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );

        // Exactly one log: the OrderNotFound for alice. No ClearV3, no
        // AfterClearV2, no revert.
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1, "only OrderNotFound emitted");
        assertEq(logs[0].topics[0], bytes32(uint256(keccak256("OrderNotFound(address,address,bytes32)"))));
        (address sender, address owner, bytes32 orderHash) = abi.decode(logs[0].data, (address, address, bytes32));
        assertEq(sender, address(this), "sender");
        assertEq(owner, alice, "owner");
        assertEq(orderHash, orderAlice.hash(), "hash");
    }

    /// Alice's order is live, Bob's order is dead (never added). clear3 passes
    /// the alice liveness check and then emits OrderNotFound for bob's hash and
    /// returns. This pins that the SECOND (bob) dead-order check exists and uses
    /// bob's owner/hash, not alice's.
    /// forge-config: default.fuzz.runs = 10
    function testClearBobDead(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        vm.assume(alice != bob);
        assumeEtchable(alice);
        assumeEtchable(bob);
        buildConfigs(configAlice, configBob);

        // Alice is added (live); bob is not.
        OrderV4 memory orderAlice = addOrder(alice, configAlice);
        OrderV4 memory orderBob =
            OrderV4(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);
        // The two orders must not collide as the same hash.
        vm.assume(orderAlice.hash() != orderBob.hash());

        assertTrue(iRaindex.orderExists(orderAlice.hash()));
        assertFalse(iRaindex.orderExists(orderBob.hash()));

        vm.expectEmit(address(iRaindex));
        emit OrderNotFound(address(this), bob, orderBob.hash());
        vm.recordLogs();
        iRaindex.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );

        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs.length, 1, "only OrderNotFound emitted");
        assertEq(logs[0].topics[0], bytes32(uint256(keccak256("OrderNotFound(address,address,bytes32)"))));
        (address sender, address owner, bytes32 orderHash) = abi.decode(logs[0].data, (address, address, bytes32));
        assertEq(sender, address(this), "sender");
        assertEq(owner, bob, "owner");
        assertEq(orderHash, orderBob.hash(), "hash");
    }
}
