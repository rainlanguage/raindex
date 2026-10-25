// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {stdError} from "forge-std-1.17.0/src/StdError.sol";
import {Strings} from "@openzeppelin-contracts-5.7.0/utils/Strings.sol";
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
import {LibTestArb} from "test/util/lib/LibTestArb.sol";
import {LibEtchRaindex} from "test/util/lib/LibEtchRaindex.sol";
import {LibTestContextWords} from "test/util/lib/LibTestContextWords.sol";
import {LibRaindexDeploy} from "../../src/lib/deploy/LibRaindexDeploy.sol";

/// @title LibRaindexArbFinalizeArbContextWordsTest
/// @notice The post-arb task sees only the sender and the arb column: every
/// Raindex context word reverts out of bounds there.
contract LibRaindexArbFinalizeArbContextWordsTest is Test {
    using Strings for address;

    function task(string memory body) internal returns (TaskV2 memory) {
        LibTestInterpreterDeploy.etchTestRainlang(vm);
        LibEtchRaindex.etchRaindex(vm);
        bytes memory bytecode = IParserV2(TEST_EXPRESSION_DEPLOYER_ADDRESS)
            .parse2(
                bytes(
                    string.concat(
                        "using-words-from ", LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS.toHexString(), "\n", body
                    )
                )
            );
        return TaskV2(
            EvaluableV4(IInterpreterV4(TEST_INTERPRETER_ADDRESS), IInterpreterStoreV3(TEST_STORE_ADDRESS), bytecode),
            new SignedContextV1[](0)
        );
    }

    function arb(TaskV2 memory post) external {
        LibTestArb.setupAndArb(vm, 80e18, 100e18, 100e18, 100e18, post, 0);
    }

    function testArbTaskBaseColumnIsSenderOnly() external {
        uint256 snapshot = vm.snapshotState();
        this.arb(task(string.concat(":ensure(equal-to(context<0 0>() ", address(this).toHexString(), ") \"sender\");")));
        vm.revertToState(snapshot);
        TaskV2 memory post = task(":set(0 context<0 1>());");
        vm.expectRevert(stdError.indexOOBError);
        this.arb(post);
    }

    function testArbTaskRaindexWordsOutOfBounds() external {
        string[] memory words = LibTestContextWords.allWords();
        for (uint256 i = 0; i < words.length; i++) {
            if (keccak256(bytes(words[i])) == keccak256("context<7 0>()")) {
                continue;
            }
            uint256 snapshot = vm.snapshotState();
            TaskV2 memory post = task(string.concat(":set(0 ", words[i], ");"));
            vm.expectRevert(stdError.indexOOBError);
            this.arb(post);
            vm.revertToState(snapshot);
        }
    }
}
