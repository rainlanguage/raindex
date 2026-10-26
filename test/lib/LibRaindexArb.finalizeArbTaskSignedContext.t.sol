// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {EvaluableV4, SignedContextV1, TaskV2} from "raindex-interface-0.1.8/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rainlang-interface-0.2.13/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rainlang-interface-0.2.13/src/interface/IInterpreterStoreV3.sol";
import {IParserV2} from "rainlang-interface-0.2.13/src/interface/IParserV2.sol";
import {
    LibTestInterpreterDeploy,
    TEST_INTERPRETER_ADDRESS,
    TEST_STORE_ADDRESS,
    TEST_EXPRESSION_DEPLOYER_ADDRESS
} from "rainlang-0.2.19/test/lib/deploy/LibTestInterpreterDeploy.sol";
import {MessageHashUtils} from "@openzeppelin-contracts-5.7.0/utils/cryptography/MessageHashUtils.sol";
import {Strings} from "@openzeppelin-contracts-5.7.0/utils/Strings.sol";
import {LibTestArb} from "test/util/lib/LibTestArb.sol";
import {LibRaindexDeploy} from "src/lib/deploy/LibRaindexDeploy.sol";
import {RUNTIME_CODE as SUB_PARSER_RUNTIME_CODE} from "src/generated/candidate/RaindexV6SubParser.sol";

/// @title LibRaindexArbFinalizeArbTaskSignedContextTest
/// @notice `signer<i>()` and `signed-context<i j>()` read the real signers and
/// signed data in the post-arb task.
contract LibRaindexArbFinalizeArbTaskSignedContextTest is Test {
    using Strings for address;

    uint256 internal constant SIGNER_A_PK = 0xA11CE;
    uint256 internal constant SIGNER_B_PK = 0xB0B;
    address internal constant DATA_A = address(0xDA7A0);
    address internal constant DATA_B = address(0xDA7A1);
    address internal constant TRUSTED = address(0x7777);

    function signed(uint256 pk, address datum) internal pure returns (SignedContextV1 memory) {
        bytes32[] memory ctx = new bytes32[](1);
        ctx[0] = bytes32(uint256(uint160(datum)));
        bytes32 h = MessageHashUtils.toEthSignedMessageHash(keccak256(abi.encodePacked(ctx)));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, h);
        return SignedContextV1(vm.addr(pk), ctx, abi.encodePacked(r, s, v));
    }

    function task(string memory body, SignedContextV1[] memory sc) internal returns (TaskV2 memory) {
        LibTestInterpreterDeploy.etchTestRainlang(vm);
        vm.etch(LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS, SUB_PARSER_RUNTIME_CODE);
        bytes memory bytecode = IParserV2(TEST_EXPRESSION_DEPLOYER_ADDRESS)
            .parse2(
                bytes(
                    string.concat(
                        "using-words-from ", LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS.toHexString(), "\n", body
                    )
                )
            );
        return TaskV2({
            evaluable: EvaluableV4(
                IInterpreterV4(TEST_INTERPRETER_ADDRESS), IInterpreterStoreV3(TEST_STORE_ADDRESS), bytecode
            ),
            signedContext: sc
        });
    }

    function testFinalizeArbTaskSignerWordsReadRealSigners() external {
        SignedContextV1[] memory sc = new SignedContextV1[](2);
        sc[0] = signed(SIGNER_A_PK, DATA_A);
        sc[1] = signed(SIGNER_B_PK, DATA_B);
        TaskV2 memory t = task(
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
        LibTestArb.setupAndArb(vm, 80e18, 100e18, 100e18, 100e18, t, 0);
    }

    function testFinalizeArbTaskRejectsSignerForgedInSignedData() external {
        SignedContextV1[] memory sc = new SignedContextV1[](3);
        sc[0] = signed(SIGNER_A_PK, DATA_A);
        sc[1] = signed(SIGNER_A_PK, DATA_A);
        sc[2] = signed(SIGNER_A_PK, TRUSTED);
        TaskV2 memory t =
            task(string.concat(":ensure(equal-to(signer<0>() ", TRUSTED.toHexString(), ") \"signer trusted\");"), sc);
        vm.expectRevert("signer trusted");
        this.externalArb(t);
    }

    function externalArb(TaskV2 memory t) external {
        LibTestArb.setupAndArb(vm, 80e18, 100e18, 100e18, 100e18, t, 0);
    }
}
