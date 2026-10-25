// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {stdError} from "forge-std-1.17.0/src/StdError.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    IOV2,
    TaskV2,
    EvaluableV4,
    SignedContextV1,
    TakeOrdersConfigV5
} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IERC20} from "@openzeppelin-contracts-5.7.0/token/ERC20/IERC20.sol";
import {Strings} from "@openzeppelin-contracts-5.7.0/utils/Strings.sol";
import {LibDecimalFloat} from "rain-math-float-0.3.10/src/lib/LibDecimalFloat.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {LibOrder} from "../../../src/lib/LibOrder.sol";
import {LibTestContextWords} from "test/util/lib/LibTestContextWords.sol";
import {
    LibNamespace,
    FullyQualifiedNamespace,
    StateNamespace
} from "rainlang-interface-0.2.13/src/lib/ns/LibNamespace.sol";

/// @title RaindexV6ContextWordEntrypointsTest
/// @notice Every context word reads its true value in the entrypoints that fill
/// its cell and reverts out of bounds in every other entrypoint.
contract RaindexV6ContextWordEntrypointsTest is RaindexV6ExternalRealTest {
    using Strings for address;
    using Strings for uint256;

    address internal constant ACTOR = address(0xD0);
    address internal constant TAKER = address(0xB0B);
    bytes32 internal constant VAULT_ID = bytes32(uint256(1));

    struct Check {
        string word;
        string value;
    }

    function ensures(Check[] memory checks) internal pure returns (string memory body) {
        for (uint256 i = 0; i < checks.length; i++) {
            body = string.concat(
                body,
                i == 0 ? "" : ",",
                ":ensure(equal-to(",
                checks[i].word,
                " ",
                checks[i].value,
                ") \"",
                checks[i].word,
                "\")"
            );
        }
        body = string.concat(body, ";");
    }

    function inFamily(Check[] memory checks, string memory word) internal pure returns (bool) {
        for (uint256 i = 0; i < checks.length; i++) {
            if (keccak256(bytes(checks[i].word)) == keccak256(bytes(word))) {
                return true;
            }
        }
        return false;
    }

    function parse(string memory body) internal view returns (bytes memory) {
        return
            iParserV2.parse2(bytes(string.concat("using-words-from ", address(iSubParser).toHexString(), "\n", body)));
    }

    function task(string memory body) internal view returns (TaskV2[] memory post) {
        post = new TaskV2[](1);
        post[0] = TaskV2(EvaluableV4(iInterpreter, iStore, parse(body)), new SignedContextV1[](0));
    }

    function orderConfig() internal view returns (OrderConfigV4 memory) {
        IOV2[] memory inputs = new IOV2[](1);
        inputs[0] = IOV2(address(iToken0), VAULT_ID);
        IOV2[] memory outputs = new IOV2[](1);
        outputs[0] = IOV2(address(iToken1), VAULT_ID);
        return OrderConfigV4(
            EvaluableV4(iInterpreter, iStore, LibTestAddOrder.VALID_ORDER_BYTECODE), inputs, outputs, bytes32(0), "", ""
        );
    }

    function configHash() internal view returns (string memory) {
        OrderConfigV4 memory config = orderConfig();
        return uint256(
                LibOrder.hash(OrderV4(ACTOR, config.evaluable, config.validInputs, config.validOutputs, config.nonce))
            ).toHexString();
    }

    function raindexCheck() internal view returns (Check memory) {
        return Check("raindex()", address(iRaindex).toHexString());
    }

    // Entrypoints.

    function doDeposit(TaskV2[] memory post) public {
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));
        vm.prank(ACTOR);
        iRaindex.deposit4(address(iToken0), VAULT_ID, LibDecimalFloat.packLossless(1, 0), post);
    }

    function doWithdraw(TaskV2[] memory post) public {
        doDeposit(new TaskV2[](0));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.prank(ACTOR);
        iRaindex.withdraw4(address(iToken0), VAULT_ID, LibDecimalFloat.packLossless(1, 0), post);
    }

    function doAddOrder(TaskV2[] memory post) public {
        vm.prank(ACTOR);
        iRaindex.addOrder4(orderConfig(), post);
    }

    function doRemoveOrder(TaskV2[] memory post) public {
        OrderConfigV4 memory config = orderConfig();
        vm.prank(ACTOR);
        iRaindex.addOrder4(config, new TaskV2[](0));
        vm.prank(ACTOR);
        iRaindex.removeOrder3(
            OrderV4(ACTOR, config.evaluable, config.validInputs, config.validOutputs, config.nonce), post
        );
    }

    function doEntask(TaskV2[] memory post) public {
        vm.prank(ACTOR);
        iRaindex.entask2(post);
    }

    /// Takes an order whose calculate source is `calculate` and handle source is
    /// `handle`, against a 10 token output vault.
    function doTake(string memory calculate, string memory handle) public returns (OrderV4 memory order) {
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));
        vm.prank(ACTOR);
        iRaindex.deposit4(address(iToken1), VAULT_ID, LibDecimalFloat.packLossless(10, 0), new TaskV2[](0));

        order = LibTestTakeOrder.addOrderWithExpression(
            vm,
            ACTOR,
            bytes(string.concat("using-words-from ", address(iSubParser).toHexString(), "\n", calculate, handle)),
            address(iToken0),
            VAULT_ID,
            address(iToken1),
            VAULT_ID
        );
        TakeOrdersConfigV5 memory config = LibTestTakeOrder.defaultTakeConfig(LibTestTakeOrder.wrapSingle(order));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transferFrom.selector), abi.encode(true));
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transfer.selector), abi.encode(true));
        vm.prank(TAKER);
        iRaindex.takeOrders4(config);
    }

    // Families.

    function depositChecks() internal view returns (Check[] memory checks) {
        checks = new Check[](7);
        checks[0] = raindexCheck();
        checks[1] = Check("depositor()", ACTOR.toHexString());
        checks[2] = Check("deposit-token()", address(iToken0).toHexString());
        checks[3] = Check("deposit-vault-id()", uint256(VAULT_ID).toHexString());
        checks[4] = Check("deposit-vault-before()", "0");
        checks[5] = Check("deposit-vault-after()", "1");
        checks[6] = Check("deposit-token-decimals()", "18");
    }

    function withdrawChecks() internal view returns (Check[] memory checks) {
        checks = new Check[](8);
        checks[0] = raindexCheck();
        checks[1] = Check("withdrawer()", ACTOR.toHexString());
        checks[2] = Check("withdraw-token()", address(iToken0).toHexString());
        checks[3] = Check("withdraw-vault-id()", uint256(VAULT_ID).toHexString());
        checks[4] = Check("withdraw-vault-before()", "1");
        checks[5] = Check("withdraw-vault-after()", "0");
        checks[6] = Check("withdraw-target-amount()", "1");
        checks[7] = Check("withdraw-token-decimals()", "18");
    }

    function orderTaskChecks() internal view returns (Check[] memory checks) {
        checks = new Check[](3);
        checks[0] = raindexCheck();
        checks[1] = Check("order-hash()", configHash());
        checks[2] = Check("order-owner()", ACTOR.toHexString());
    }

    function entaskChecks() internal view returns (Check[] memory checks) {
        checks = new Check[](1);
        checks[0] = raindexCheck();
    }

    /// Order hash depends on the bytecode holding these checks, so
    /// `testOrderEvalOrderHash` pins it separately.
    function orderEvalChecks(bool handle) internal view returns (Check[] memory checks) {
        checks = new Check[](16);
        checks[0] = raindexCheck();
        checks[1] = Check("order-owner()", ACTOR.toHexString());
        checks[2] = Check("order-counterparty()", TAKER.toHexString());
        checks[3] = Check("order-clearer()", TAKER.toHexString());
        checks[4] = Check("calculated-max-output()", handle ? "1" : "0");
        checks[5] = Check("calculated-io-ratio()", handle ? "1" : "0");
        checks[6] = Check("input-token()", address(iToken0).toHexString());
        checks[7] = Check("input-token-decimals()", "18");
        checks[8] = Check("input-vault-id()", uint256(VAULT_ID).toHexString());
        checks[9] = Check("input-vault-before()", "0");
        checks[10] = Check("input-vault-increase()", handle ? "1" : "0");
        checks[11] = Check("output-token()", address(iToken1).toHexString());
        checks[12] = Check("output-token-decimals()", "18");
        checks[13] = Check("output-vault-id()", uint256(VAULT_ID).toHexString());
        checks[14] = Check("output-vault-before()", "10");
        checks[15] = Check("output-vault-decrease()", handle ? "1" : "0");
    }

    /// Words the order eval fills but `orderEvalChecks` cannot pin by value.
    function orderEvalUnpinned(string memory word) internal pure returns (bool) {
        return keccak256(bytes(word)) == keccak256("order-hash()");
    }

    // Task entrypoint runner.

    function checkTaskFamily(function(TaskV2[] memory) external run, Check[] memory checks) internal {
        uint256 start = vm.snapshotState();
        run(task(ensures(checks)));
        vm.revertToState(start);
        string[] memory words = LibTestContextWords.allWords();
        for (uint256 i = 0; i < words.length; i++) {
            if (inFamily(checks, words[i])) {
                continue;
            }
            uint256 snapshot = vm.snapshotState();
            TaskV2[] memory post = task(string.concat(":set(0 ", words[i], ");"));
            vm.expectRevert(stdError.indexOOBError);
            run(post);
            vm.revertToState(snapshot);
        }
    }

    function testDepositContextWords() external {
        checkTaskFamily(this.doDeposit, depositChecks());
    }

    function testWithdrawContextWords() external {
        checkTaskFamily(this.doWithdraw, withdrawChecks());
    }

    function testAddOrderContextWords() external {
        checkTaskFamily(this.doAddOrder, orderTaskChecks());
    }

    function testRemoveOrderContextWords() external {
        checkTaskFamily(this.doRemoveOrder, orderTaskChecks());
    }

    function testEntaskContextWords() external {
        checkTaskFamily(this.doEntask, entaskChecks());
    }

    // Order eval runner.

    function checkOrderEvalFamily(bool handle) internal {
        Check[] memory checks = orderEvalChecks(handle);
        string memory body = ensures(checks);
        if (handle) {
            this.doTake("_ _:1 1;", body);
        } else {
            this.doTake(string.concat("_ _:1 1,", body), ":;");
        }
        string[] memory words = LibTestContextWords.allWords();
        for (uint256 i = 0; i < words.length; i++) {
            if (inFamily(checks, words[i]) || orderEvalUnpinned(words[i])) {
                continue;
            }
            uint256 snapshot = vm.snapshotState();
            string memory read = string.concat(":set(0 ", words[i], ");");
            vm.expectRevert(stdError.indexOOBError);
            if (handle) {
                this.doTake("_ _:1 1;", read);
            } else {
                this.doTake(string.concat("_ _:1 1,", read), ":;");
            }
            vm.revertToState(snapshot);
        }
    }

    function testCalculateOrderContextWords() external {
        checkOrderEvalFamily(false);
    }

    function testHandleIOContextWords() external {
        checkOrderEvalFamily(true);
    }

    /// The order hash word reads the hash of the order being evaluated, in both
    /// calculate and handle IO.
    function testOrderEvalOrderHash() external {
        bytes32 orderHash = LibOrder.hash(this.doTake("_ _:1 1,:set(1 order-hash());", ":set(2 order-hash());"));
        FullyQualifiedNamespace ns =
            LibNamespace.qualifyNamespace(StateNamespace.wrap(uint256(uint160(ACTOR))), address(iRaindex));
        assertEq(iStore.get(ns, bytes32(uint256(1))), orderHash, "calculate order hash");
        assertEq(iStore.get(ns, bytes32(uint256(2))), orderHash, "handle io order hash");
    }
}
