// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    IOV2,
    TaskV2,
    EvaluableV4,
    SignedContextV1
} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IERC20} from "@openzeppelin-contracts-5.7.0/token/ERC20/IERC20.sol";
import {MessageHashUtils} from "@openzeppelin-contracts-5.7.0/utils/cryptography/MessageHashUtils.sol";
import {Strings} from "@openzeppelin-contracts-5.7.0/utils/Strings.sol";
import {LibDecimalFloat} from "rain-math-float-0.3.10/src/lib/LibDecimalFloat.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";

/// @title RaindexV6PostTaskSignedContextTest
/// @notice `signer<i>()` and `signed-context<i j>()` read the real signers and
/// signed data in every Raindex post task, so a caller cannot forge a signer by
/// placing an address in signed data.
contract RaindexV6PostTaskSignedContextTest is RaindexV6ExternalRealTest {
    using Strings for address;

    uint256 internal constant SIGNER_A_PK = 0xA11CE;
    uint256 internal constant SIGNER_B_PK = 0xB0B;
    address internal constant DATA_A = address(0xDA7A0);
    address internal constant DATA_B = address(0xDA7A1);
    address internal constant TRUSTED = address(0x7777);
    address internal constant ACTOR = address(0xD0);

    function signed(uint256 pk, bytes32[] memory ctx) internal pure returns (SignedContextV1 memory) {
        bytes32 h = MessageHashUtils.toEthSignedMessageHash(keccak256(abi.encodePacked(ctx)));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, h);
        return SignedContextV1(vm.addr(pk), ctx, abi.encodePacked(r, s, v));
    }

    function single(address a) internal pure returns (bytes32[] memory) {
        bytes32[] memory ctx = new bytes32[](1);
        ctx[0] = bytes32(uint256(uint160(a)));
        return ctx;
    }

    function task(string memory body, SignedContextV1[] memory sc) internal view returns (TaskV2[] memory) {
        TaskV2[] memory post = new TaskV2[](1);
        post[0] = TaskV2(
            EvaluableV4(
                iInterpreter,
                iStore,
                iParserV2.parse2(
                    bytes(string.concat("using-words-from ", address(iSubParser).toHexString(), "\n", body))
                )
            ),
            sc
        );
        return post;
    }

    /// Two signers, each signing its own data; the task asserts each word reads
    /// its own signer and datum.
    function realSignersTask() internal view returns (TaskV2[] memory) {
        SignedContextV1[] memory sc = new SignedContextV1[](2);
        sc[0] = signed(SIGNER_A_PK, single(DATA_A));
        sc[1] = signed(SIGNER_B_PK, single(DATA_B));
        return task(
            string.concat(
                ":ensure(equal-to(signer<0>() ",
                vm.addr(SIGNER_A_PK).toHexString(),
                ") \"signer 0\"),",
                ":ensure(equal-to(signer<1>() ",
                vm.addr(SIGNER_B_PK).toHexString(),
                ") \"signer 1\"),",
                ":ensure(equal-to(signed-context<0 0>() ",
                DATA_A.toHexString(),
                ") \"signed context 0\"),",
                ":ensure(equal-to(signed-context<1 0>() ",
                DATA_B.toHexString(),
                ") \"signed context 1\");"
            ),
            sc
        );
    }

    /// One attacker key signs three contexts, the third holding `TRUSTED`. A
    /// task that accepts only `TRUSTED` as signer must reject it.
    function forgedSignerTask() internal view returns (TaskV2[] memory) {
        SignedContextV1[] memory sc = new SignedContextV1[](3);
        sc[0] = signed(SIGNER_A_PK, single(DATA_A));
        sc[1] = signed(SIGNER_A_PK, single(DATA_A));
        sc[2] = signed(SIGNER_A_PK, single(TRUSTED));
        return task(string.concat(":ensure(equal-to(signer<0>() ", TRUSTED.toHexString(), ") \"signer trusted\");"), sc);
    }

    function deposit(TaskV2[] memory post) internal {
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, ACTOR, address(iRaindex), 1e18),
            abi.encode(true)
        );
        vm.prank(ACTOR);
        iRaindex.deposit4(address(iToken0), bytes32(uint256(1)), LibDecimalFloat.packLossless(1, 0), post);
    }

    function withdraw(TaskV2[] memory post) internal {
        deposit(new TaskV2[](0));
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, ACTOR, 1e18), abi.encode(true));
        vm.prank(ACTOR);
        iRaindex.withdraw4(address(iToken0), bytes32(uint256(1)), LibDecimalFloat.packLossless(1, 0), post);
    }

    function orderConfig() internal view returns (OrderConfigV4 memory) {
        IOV2[] memory inputs = new IOV2[](1);
        inputs[0] = IOV2(address(iToken0), bytes32(uint256(1)));
        IOV2[] memory outputs = new IOV2[](1);
        outputs[0] = IOV2(address(iToken1), bytes32(uint256(1)));
        return OrderConfigV4(
            EvaluableV4(iInterpreter, iStore, LibTestAddOrder.VALID_ORDER_BYTECODE), inputs, outputs, bytes32(0), "", ""
        );
    }

    function addOrder(TaskV2[] memory post) internal {
        vm.prank(ACTOR);
        assertTrue(iRaindex.addOrder4(orderConfig(), post));
    }

    function removeOrder(TaskV2[] memory post) internal {
        OrderConfigV4 memory config = orderConfig();
        vm.prank(ACTOR);
        iRaindex.addOrder4(config, new TaskV2[](0));
        vm.prank(ACTOR);
        assertTrue(
            iRaindex.removeOrder3(
                OrderV4(ACTOR, config.evaluable, config.validInputs, config.validOutputs, config.nonce), post
            )
        );
    }

    function entask(TaskV2[] memory post) internal {
        vm.prank(ACTOR);
        iRaindex.entask2(post);
    }

    function testDepositTaskSignerWordsReadRealSigners() external {
        deposit(realSignersTask());
    }

    function testDepositTaskRejectsSignerForgedInSignedData() external {
        TaskV2[] memory post = forgedSignerTask();
        vm.expectRevert("signer trusted");
        this.externalDeposit(post);
    }

    function testWithdrawTaskSignerWordsReadRealSigners() external {
        withdraw(realSignersTask());
    }

    function testWithdrawTaskRejectsSignerForgedInSignedData() external {
        TaskV2[] memory post = forgedSignerTask();
        vm.expectRevert("signer trusted");
        this.externalWithdraw(post);
    }

    function testAddOrderTaskSignerWordsReadRealSigners() external {
        addOrder(realSignersTask());
    }

    function testAddOrderTaskRejectsSignerForgedInSignedData() external {
        TaskV2[] memory post = forgedSignerTask();
        vm.expectRevert("signer trusted");
        this.externalAddOrder(post);
    }

    function testRemoveOrderTaskSignerWordsReadRealSigners() external {
        removeOrder(realSignersTask());
    }

    function testRemoveOrderTaskRejectsSignerForgedInSignedData() external {
        TaskV2[] memory post = forgedSignerTask();
        vm.expectRevert("signer trusted");
        this.externalRemoveOrder(post);
    }

    function testEntaskSignerWordsReadRealSigners() external {
        entask(realSignersTask());
    }

    function testEntaskRejectsSignerForgedInSignedData() external {
        TaskV2[] memory post = forgedSignerTask();
        vm.expectRevert("signer trusted");
        this.externalEntask(post);
    }

    function externalDeposit(TaskV2[] memory post) external {
        deposit(post);
    }

    function externalWithdraw(TaskV2[] memory post) external {
        withdraw(post);
    }

    function externalAddOrder(TaskV2[] memory post) external {
        addOrder(post);
    }

    function externalRemoveOrder(TaskV2[] memory post) external {
        removeOrder(post);
    }

    function externalEntask(TaskV2[] memory post) external {
        entask(post);
    }
}
