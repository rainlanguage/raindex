// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {EvaluableV4, SignedContextV1, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {IParserV2} from "rain.interpreter.interface/interface/IParserV2.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";
import {LibTestArb, ArbResult} from "test/util/lib/LibTestArb.sol";

/// @title LibRaindexArbFinalizeArbTaskContextTest
/// @notice Verifies that finalizeArb passes correct Float-encoded context to
/// the post-arb task: context<1 0>=input balance, context<1 1>=output balance,
/// context<1 2>=gas balance. Column 0 is the calling context added by LibContext.
contract LibRaindexArbFinalizeArbTaskContextTest is Test {
    function testFinalizeArbTaskContextValues() external {
        LibInterpreterDeploy.etchRainlang(vm);

        IParserV2 parser = IParserV2(LibInterpreterDeploy.EXPRESSION_DEPLOYER_DEPLOYED_ADDRESS);

        // Task expression: ensure context values match expected Floats.
        // 20e18 input profit with 18 decimals → Float(20).
        // 0 output profit → Float(0).
        // 0 gas → Float(0).
        bytes memory taskBytecode = parser.parse2(
            bytes(
                string.concat(
                    ":ensure(equal-to(context<1 0>() 20) \"input\"),",
                    ":ensure(equal-to(context<1 1>() 0) \"output\"),",
                    ":ensure(equal-to(context<1 2>() 0) \"gas\");"
                )
            )
        );

        TaskV2 memory task = TaskV2({
            evaluable: EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                taskBytecode
            ),
            signedContext: new SignedContextV1[](0)
        });

        // Raindex has 100e18 output, pulls 80e18 input. Exchange has 100e18 input.
        // 20e18 input profit, 0 output profit, 0 gas.
        // Task expression will revert if context values are wrong.
        ArbResult memory result = LibTestArb.setupAndArb(vm, 80e18, 100e18, 100e18, 100e18, task, 0);

        // Sanity check balances.
        assertEq(result.inputToken.balanceOf(address(this)), 20e18, "sender inputToken profit");
        assertEq(result.inputToken.balanceOf(address(result.arb)), 0, "arb inputToken");
        assertEq(result.outputToken.balanceOf(address(result.arb)), 0, "arb outputToken");
    }
}
