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
    TakeOrdersConfigV5,
    QuoteV2
} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IERC20} from "@openzeppelin-contracts-5.7.0/token/ERC20/IERC20.sol";
import {Strings} from "@openzeppelin-contracts-5.7.0/utils/Strings.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.3.10/src/lib/LibDecimalFloat.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {LibTestContextWords} from "test/util/lib/LibTestContextWords.sol";

/// @title RaindexV6ContextWordEntrypointsTest
/// @notice Every deposit context word reads its true value in deposit tasks and
/// reverts out of bounds in every other entrypoint.
contract RaindexV6ContextWordEntrypointsTest is RaindexV6ExternalRealTest {
    using Strings for address;
    using Strings for uint256;
    using LibDecimalFloat for Float;

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
        (Float totalTakerInput,) = iRaindex.takeOrders4(config);
        assertTrue(totalTakerInput.gt(LibDecimalFloat.FLOAT_ZERO), "order taken");
    }

    /// `quote2` bubbles a calculate revert that `takeOrders4` skips.
    function doQuote(string memory calculate) public {
        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm,
            ACTOR,
            bytes(string.concat("using-words-from ", address(iSubParser).toHexString(), "\n", calculate, ":;")),
            address(iToken0),
            VAULT_ID,
            address(iToken1),
            VAULT_ID
        );
        iRaindex.quote2(QuoteV2(order, 0, 0, new SignedContextV1[](0)));
    }

    // Families.

    function depositChecks() internal view returns (Check[] memory checks) {
        checks = new Check[](6);
        checks[0] = Check("depositor()", ACTOR.toHexString());
        checks[1] = Check("deposit-token()", address(iToken0).toHexString());
        checks[2] = Check("deposit-vault-id()", uint256(VAULT_ID).toHexString());
        checks[3] = Check("deposit-vault-before()", "0");
        checks[4] = Check("deposit-vault-after()", "1");
        checks[5] = Check("deposit-token-decimals()", "18");
    }

    // Task entrypoint runner.

    function checkTaskFamily(function(TaskV2[] memory) external run, Check[] memory checks) internal {
        uint256 start = vm.snapshotState();
        run(task(checks.length > 0 ? ensures(checks) : ":;"));
        vm.revertToState(start);
        string[] memory words = LibTestContextWords.depositWords();
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
        checkTaskFamily(this.doWithdraw, new Check[](0));
    }

    function testAddOrderContextWords() external {
        checkTaskFamily(this.doAddOrder, new Check[](0));
    }

    function testRemoveOrderContextWords() external {
        checkTaskFamily(this.doRemoveOrder, new Check[](0));
    }

    function testEntaskContextWords() external {
        checkTaskFamily(this.doEntask, new Check[](0));
    }

    // Order eval runner.

    function checkOrderEvalFamily(bool handle) internal {
        uint256 start = vm.snapshotState();
        this.doTake("_ _:1 1;", ":;");
        vm.revertToState(start);
        string[] memory words = LibTestContextWords.depositWords();
        for (uint256 i = 0; i < words.length; i++) {
            uint256 snapshot = vm.snapshotState();
            string memory read = string.concat(":set(0 ", words[i], ");");
            vm.expectRevert(stdError.indexOOBError);
            if (handle) {
                this.doTake("_ _:1 1;", read);
            } else {
                this.doQuote(string.concat("_ _:1 1,", read));
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
}
