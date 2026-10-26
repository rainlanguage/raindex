// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "@openzeppelin-contracts-5.7.0/token/ERC20/IERC20.sol";
import {RaindexV6FreshTakeOrderTest} from "test/util/abstract/RaindexV6FreshTakeOrderTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    TaskV2,
    SignedContextV1
} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.3.10/src/lib/LibDecimalFloat.sol";
import {LibNamespace} from "rainlang-interface-0.2.13/src/lib/ns/LibNamespace.sol";
import {StateNamespace} from "rainlang-interface-0.2.13/src/interface/IInterpreterV4.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";
import {Vm} from "forge-std-1.17.0/src/Test.sol";

/// A `takeOrders4` order whose calculate eval reverts, or returns too short a
/// stack, is skipped with `OrderCalculateReverted` and makes no state change;
/// later orders still take, and handle-IO reverts still revert the batch.
contract RaindexV6TakeOrderCalculateRevertTest is RaindexV6FreshTakeOrderTest {
    using LibDecimalFloat for Float;

    address internal alice = address(uint160(uint256(keccak256("alice.rain.test"))));
    address internal bob = address(uint160(uint256(keccak256("bob.rain.test"))));

    bytes32 internal constant VAULT_ID = bytes32(uint256(0x01));

    function addOrder(bytes memory expression) internal returns (OrderV4 memory) {
        return LibTestTakeOrder.addOrderWithExpression(
            vm, alice, expression, address(iToken0), VAULT_ID, address(iToken1), VAULT_ID
        );
    }

    function depositOutput() internal {
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iRaindex)),
            abi.encode(true)
        );
        vm.prank(alice);
        iRaindex.deposit4(address(iToken1), VAULT_ID, LibDecimalFloat.packLossless(10, 0), new TaskV2[](0));
    }

    function mockTakerTransfers() internal {
        vm.mockCall(
            address(iToken1), abi.encodeWithSelector(IERC20.transfer.selector, bob, uint256(1e18)), abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iRaindex), uint256(1e18)),
            abi.encode(true)
        );
    }

    function takeConfig(OrderV4 memory first, OrderV4 memory second) internal pure returns (TakeOrdersConfigV5 memory) {
        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](2);
        orders[0] = TakeOrderConfigV4(first, 0, 0, new SignedContextV1[](0));
        orders[1] = TakeOrderConfigV4(second, 0, 0, new SignedContextV1[](0));
        return LibTestTakeOrder.defaultTakeConfig(orders);
    }

    function aliceStore(bytes32 key) internal view returns (bytes32) {
        return
            iStore.get(
                LibNamespace.qualifyNamespace(StateNamespace.wrap(uint256(uint160(alice))), address(iRaindex)), key
            );
    }

    function checkSkippedThenTaken(bytes memory skippedExpression) internal {
        depositOutput();
        mockTakerTransfers();
        OrderV4 memory skipped = addOrder(skippedExpression);
        OrderV4 memory taken = addOrder("_ _:1 1;:;");

        vm.recordLogs();
        vm.prank(bob);
        (Float totalTakerInput, Float totalTakerOutput) = iRaindex.takeOrders4(takeConfig(skipped, taken));

        Vm.Log[] memory logs = vm.getRecordedLogs();
        uint256 found;
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].topics[0] == keccak256("OrderCalculateReverted(address,address,bytes32)")) {
                assertEq(logs[i].emitter, address(iRaindex), "emitter");
                assertEq(logs[i].data, abi.encode(bob, alice, LibOrder.hash(skipped)), "event data");
                found++;
            }
        }
        assertEq(found, 1, "one OrderCalculateReverted");

        assertTrue(totalTakerInput.eq(LibDecimalFloat.packLossless(1, 0)), "totalTakerInput");
        assertTrue(totalTakerOutput.eq(LibDecimalFloat.packLossless(1, 0)), "totalTakerOutput");
        assertTrue(
            iRaindex.vaultBalance2(alice, address(iToken1), VAULT_ID).eq(LibDecimalFloat.packLossless(9, 0)),
            "alice output vault"
        );
        assertTrue(
            iRaindex.vaultBalance2(alice, address(iToken0), VAULT_ID).eq(LibDecimalFloat.packLossless(1, 0)),
            "alice input vault"
        );
    }

    function testCalculateRevertSkipsOrderAndLaterOrderTakes() external {
        checkSkippedThenTaken(":ensure(0 \"calc\"),_ _:1 1;:;");
    }

    function testShortCalculateStackSkipsOrderAndLaterOrderTakes() external {
        checkSkippedThenTaken("_:1;:;");
    }

    function testCalculateRevertSkipsRevertingHandleIO() external {
        checkSkippedThenTaken(":ensure(0 \"calc\"),_ _:1 1;:ensure(0 \"skipped handle\");");
    }

    function testShortCalculateStackSkipsRevertingHandleIO() external {
        checkSkippedThenTaken("_:1;:ensure(0 \"skipped handle\");");
    }

    function testCalculateRevertSkipsHandleIOStoreWrite() external {
        checkSkippedThenTaken(":ensure(0 \"calc\"),_ _:1 1;:set(0x08 0x0a);");
        assertEq(aliceStore(bytes32(uint256(8))), bytes32(0));
    }

    function testShortCalculateStackSkipsHandleIOStoreWrite() external {
        checkSkippedThenTaken("_:1;:set(0x08 0x0a);");
        assertEq(aliceStore(bytes32(uint256(8))), bytes32(0));
    }

    function testSkippedCalculateWritesNothingToStore() external {
        checkSkippedThenTaken(":set(0x07 0x09);:;");
        assertEq(aliceStore(bytes32(uint256(7))), bytes32(0));
    }

    function testTakenCalculateWritesReachStore() external {
        depositOutput();
        mockTakerTransfers();
        OrderV4 memory order = addOrder("_ _:1 1,:set(0x07 0x09);:;");

        vm.prank(bob);
        iRaindex.takeOrders4(LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order)));

        assertEq(aliceStore(bytes32(uint256(7))), bytes32(uint256(9)));
    }

    function testHandleIORevertAfterSkippedCalculateRevertsBatch() external {
        depositOutput();
        mockTakerTransfers();
        OrderV4 memory skipped = addOrder(":ensure(0 \"calc\"),_ _:1 1;:;");
        OrderV4 memory reverting = addOrder("_ _:1 1;:ensure(0 \"handle\");");

        vm.prank(bob);
        vm.expectRevert(bytes("handle"));
        iRaindex.takeOrders4(takeConfig(skipped, reverting));
    }
}
