// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    TaskV2,
    EvaluableV4,
    SignedContextV1,
    IInterpreterStoreV3,
    IRaindexV6
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";

/// Tests for `LibOrderBook.doPost` behavior via the `entask2` entry point.
contract LibOrderBookDoPostTest is OrderBookV6ExternalRealTest {
    /// Empty post array is a no-op.
    function testDoPostEmptyArray() external {
        TaskV2[] memory emptyTasks = new TaskV2[](0);
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).entask2(emptyTasks);
    }

    /// Task with empty bytecode is silently skipped.
    function testDoPostEmptyBytecodeSkipped() external {
        TaskV2[] memory tasks = new TaskV2[](1);
        tasks[0] = TaskV2({
            evaluable: EvaluableV4({interpreter: iInterpreter, store: iStore, bytecode: ""}),
            signedContext: new SignedContextV1[](0)
        });
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).entask2(tasks);
    }

    /// store.set is not called when eval produces no writes.
    function testDoPostNoStoreSetWhenNoWrites() external {
        bytes memory bytecode = iParserV2.parse2("_:1;");

        TaskV2[] memory tasks = new TaskV2[](1);
        tasks[0] = TaskV2({
            evaluable: EvaluableV4({interpreter: iInterpreter, store: iStore, bytecode: bytecode}),
            signedContext: new SignedContextV1[](0)
        });

        vm.record();
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).entask2(tasks);
        (, bytes32[] memory writes) = vm.accesses(address(iStore));

        assertEq(writes.length, 0, "store.set should not be called when eval has no writes");
    }
}
