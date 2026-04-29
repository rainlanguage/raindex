// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {IMetaV1_2} from "rain.metadata/lib/LibMeta.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {IRaindexV6Stub} from "test/util/abstract/IRaindexV6Stub.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {IRaindexV6, OrderConfigV4, OrderV4, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibEtchOrderBook} from "test/util/lib/LibEtchOrderBook.sol";
import {EvaluableV4} from "rain.interpreter.interface/interface/IInterpreterCallerV4.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";

/// @title OrderBookV6ExternalTest
/// Abstract contract that performs common setup needed for testing an orderbook
/// from its external interface.
///
/// Notably:
/// - Deploys a real orderbook contract with correct meta.
/// - Deploys several mockable token contracts.
/// - Deploys a mockable deployer contract for a DISpair.
///
/// Inherits from Test so that it can be used as a base contract for other tests.
/// Implements IRaindexV6 so that it has access to all the relevant events.
abstract contract OrderBookV6ExternalMockTest is Test, IMetaV1_2, IRaindexV6Stub {
    IInterpreterV4 immutable iInterpreter;
    IInterpreterStoreV3 immutable iStore;
    IRaindexV6 immutable iOrderbook;
    IERC20 immutable iToken0;
    IERC20 immutable iToken1;

    constructor() {
        vm.pauseGasMetering();

        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        iInterpreter = IInterpreterV4(address(uint160(uint256(keccak256("interpreter.rain.test")))));
        vm.etch(address(iInterpreter), REVERTING_MOCK_BYTECODE);
        iStore = IInterpreterStoreV3(address(uint160(uint256(keccak256("store.rain.test")))));
        vm.etch(address(iStore), REVERTING_MOCK_BYTECODE);
        LibEtchOrderBook.etchOrderBook(vm);
        iOrderbook = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);

        iToken0 = IERC20(address(uint160(uint256(keccak256("token0.rain.test")))));
        vm.etch(address(iToken0), REVERTING_MOCK_BYTECODE);
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));
        iToken1 = IERC20(address(uint160(uint256(keccak256("token1.rain.test")))));
        vm.etch(address(iToken1), REVERTING_MOCK_BYTECODE);
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(uint8(18)));
        vm.resumeGasMetering();
    }

    /// Boilerplate to add an order with a mocked deployer and checks events and
    /// storage accesses.
    function addOrderWithChecks(address owner, OrderConfigV4 memory config, bytes memory)
        internal
        returns (OrderV4 memory, bytes32)
    {
        (OrderV4 memory order, bytes32 orderHash) = LibTestAddOrder.expectedOrder(owner, config);
        assertTrue(!iOrderbook.orderExists(orderHash));
        vm.expectEmit(false, false, false, true);
        emit AddOrderV3(owner, orderHash, order);
        if (config.meta.length > 0) {
            vm.expectEmit(false, false, true, false);
            // The subject of the meta is the order hash.
            emit MetaV1_2(owner, orderHash, config.meta);
        }
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        assertTrue(iOrderbook.addOrder4(config, new TaskV2[](0)));
        // MetaV1 is NOT emitted if the meta is empty.
        assertEq(vm.getRecordedLogs().length, config.meta.length > 0 ? 2 : 1);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check, 1x for live write.
        assertEq(reads.length, 5);
        // 2x for reentrancy guard, 1x for live write.
        assertEq(writes.length, 3);
        assertTrue(iOrderbook.orderExists(orderHash));

        // Adding the same order again MUST NOT change state. This MAY be
        // impossible to encounter for a real expression deployer, as the
        // deployer MAY NOT return the same address twice, but it is possible to
        // mock.
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        assertFalse(iOrderbook.addOrder4(config, new TaskV2[](0)));
        assertEq(vm.getRecordedLogs().length, 0);
        (reads, writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check.
        assertEq(reads.length, 4);
        // 2x for reentrancy guard.
        assertEq(writes.length, 2);
        assertTrue(iOrderbook.orderExists(orderHash));

        return (order, orderHash);
    }

    /// Boilerplate to remove an order with a mocked deployer and checks events
    /// and storage accesses.
    function removeOrderWithChecks(address owner, OrderV4 memory order) internal {
        bytes32 orderHash = LibOrder.hash(order);
        // This check assumes the order exists before we try to remove it.
        assertTrue(iOrderbook.orderExists(orderHash));
        vm.expectEmit(false, false, false, true);
        emit RemoveOrderV3(owner, orderHash, order);
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        // An order was removed so this is true as there is a state change.
        assertTrue(iOrderbook.removeOrder3(order, new TaskV2[](0)));
        assertEq(vm.getRecordedLogs().length, 1);
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check, 1x for dead write.
        assertEq(reads.length, 5);
        // 2x for reentrancy guard, 1x for dead write.
        assertEq(writes.length, 3);
        assertFalse(iOrderbook.orderExists(orderHash));

        // Removing the same order again MUST NOT change state.
        vm.record();
        vm.recordLogs();
        vm.prank(owner);
        // There is no state change so this is false.
        assertFalse(iOrderbook.removeOrder3(order, new TaskV2[](0)));
        assertEq(vm.getRecordedLogs().length, 0);
        (reads, writes) = vm.accesses(address(iOrderbook));
        // 3x for reentrancy guard, 1x for dead order check.
        assertEq(reads.length, 4);
        // 2x for reentrancy guard.
        assertEq(writes.length, 2);
        assertFalse(iOrderbook.orderExists(orderHash));
    }
}
