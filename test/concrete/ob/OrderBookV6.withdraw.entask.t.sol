// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {
    OrderConfigV4,
    EvaluableV4,
    TaskV2,
    SignedContextV1,
    IRaindexV6
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibFormatDecimalFloat} from "rain.math.float/lib/format/LibFormatDecimalFloat.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/utils/ReentrancyGuard.sol";

import {console2} from "forge-std/Test.sol";

contract OrderBookV6WithdrawEvalTest is OrderBookV6ExternalRealTest {
    using Strings for address;
    using Strings for uint256;
    using LibDecimalFloat for Float;
    using LibFormatDecimalFloat for Float;

    function checkReentrancyRW(uint256 expectedReads, uint256 expectedWrites) internal view {
        (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
        // 3 reads for reentrancy guard.
        // 2 reads for deposit.
        assertEq(reads.length, expectedReads, "reads length");
        // ReentrancyGuard.REENTRANCY_GUARD_STORAGE
        bytes32 reentrancyGuardStorage = 0x9b779b17422d0df92223018b32b4d1fa46e071723d6817e2486d003becc55f00;
        assertEq(reads[0], reentrancyGuardStorage, "reads[0]");
        assertEq(reads[1], reentrancyGuardStorage, "reads[1]");
        assertEq(reads[reads.length - 1], reentrancyGuardStorage, "reads[reads.length - 1]");
        // 2 writes for reentrancy guard.
        // 1 write for deposit.
        assertEq(writes.length, expectedWrites, "writes length");
        assertEq(writes[0], reentrancyGuardStorage, "writes[0]");
        assertEq(writes[writes.length - 1], reentrancyGuardStorage, "writes[writes.length - 1]");
    }

    function checkWithdraw(
        address owner,
        bytes32 vaultId,
        Float depositAmount,
        Float targetAmount,
        bytes[] memory evalStrings,
        uint256 expectedReads,
        uint256 expectedWrites,
        bytes memory err
    ) internal {
        uint256 withdrawAmount18;
        TaskV2[] memory actions;
        {
            uint256 depositAmountAbsolute;
            {
                uint8 decimals = IERC20Metadata(address(iToken0)).decimals();
                uint256 targetAmount18 = targetAmount.toFixedDecimalLossless(decimals);
                uint256 depositAmount18 = depositAmount.toFixedDecimalLossless(decimals);
                withdrawAmount18 = depositAmount18 > targetAmount18 ? targetAmount18 : depositAmount18;
                bool lossless;
                (depositAmountAbsolute, lossless) = depositAmount.toFixedDecimalLossy(decimals);
                // Deposit roundings should round up the amount of token taken by the
                // DEX.
                if (!lossless) {
                    ++depositAmountAbsolute;
                }
            }
            {
                vm.mockCall(
                    address(iToken0),
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector,
                        owner,
                        LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                        depositAmountAbsolute
                    ),
                    abi.encode(true)
                );

                vm.startPrank(owner);

                if (depositAmountAbsolute > 0) {
                    IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                        .deposit4(address(iToken0), vaultId, depositAmount, new TaskV2[](0));
                }
            }

            actions = new TaskV2[](evalStrings.length);
            for (uint256 i = 0; i < evalStrings.length; i++) {
                actions[i] = TaskV2(
                    EvaluableV4(iInterpreter, iStore, iParserV2.parse2(evalStrings[i])), new SignedContextV1[](0)
                );
            }
        }

        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, owner, withdrawAmount18),
            abi.encode(true)
        );

        vm.record();
        if (err.length > 0) {
            vm.expectRevert(err);
        } else {
            vm.expectEmit(false, false, false, true);
            emit WithdrawV2(
                owner,
                address(iToken0),
                vaultId,
                targetAmount,
                LibDecimalFloat.fromFixedDecimalLosslessPacked(
                    withdrawAmount18, IERC20Metadata(address(iToken0)).decimals()
                ),
                withdrawAmount18
            );
        }
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
            .withdraw4(address(iToken0), vaultId, targetAmount, actions);
        if (err.length == 0) {
            checkReentrancyRW(6, 3);
            (bytes32[] memory reads, bytes32[] memory writes) = vm.accesses(address(iStore));
            assertEq(reads.length, expectedReads);
            assertEq(writes.length, expectedWrites);
        }

        vm.stopPrank();
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalEmptyNoop(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(vaultId != bytes32(0));
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);
        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, new bytes[](0), 0, 0, "");
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalOneStateless(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(vaultId != bytes32(0));
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:1;");
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals, 0, 0, "");
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalOneReadState(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(vaultId != bytes32(0));
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes("_:get(0);");
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals, 2, 1, "");
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalWriteStateSingle(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(vaultId != bytes32(0));
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        bytes[] memory evals0 = new bytes[](1);
        evals0[0] = bytes(":set(1 2);");
        // each set is 1 read and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals0, 1, 1, "");

        bytes[] memory evals1 = new bytes[](1);
        evals1[0] = bytes(":ensure(equal-to(get(1) 2) \"set works\");");
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals1, 2, 1, "");
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalWriteStateSequential(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(vaultId != bytes32(0));
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals0, 6, 4, "");

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals1, 6, 4, "");
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawEvalWriteStateDifferentOwnersNamespaced(
        address alice,
        address bob,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(alice != bob);
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        bytes[] memory evals0 = new bytes[](4);
        evals0[0] = bytes(":set(1 2);");
        evals0[1] = bytes(":ensure(equal-to(get(1) 2) \"0th set not equal\");");
        evals0[2] = bytes(":set(2 3);");
        evals0[3] = bytes(":ensure(equal-to(get(2) 3) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals0, 6, 4, "");

        bytes[] memory evals1 = new bytes[](4);
        evals1[0] = bytes(":set(1 20);");
        evals1[1] = bytes(":ensure(equal-to(get(1) 20) \"0th set not equal\");");
        evals1[2] = bytes(":set(2 30);");
        evals1[3] = bytes(":ensure(equal-to(get(2) 30) \"1st set not equal\");");
        // each set is 1 read and 1 write.
        // each get is 2 reads and 1 write.
        checkWithdraw(bob, vaultId, depositAmount, withdrawAmount, evals1, 6, 4, "");

        bytes[] memory evals2 = new bytes[](2);
        evals2[0] = bytes(":ensure(equal-to(get(1) 2) \"alice state 1\");");
        evals2[1] = bytes(":ensure(equal-to(get(2) 3) \"alice state 2\");");
        // each get is 2 reads and 1 write.
        checkWithdraw(alice, vaultId, depositAmount, withdrawAmount, evals2, 4, 2, "");

        bytes[] memory evals3 = new bytes[](2);
        evals3[0] = bytes(":ensure(equal-to(get(1) 20) \"bob state 1\");");
        evals3[1] = bytes(":ensure(equal-to(get(2) 30) \"bob state 2\");");
        // each get is 2 reads and 1 write.
        checkWithdraw(bob, vaultId, depositAmount, withdrawAmount, evals3, 4, 2, "");
    }

    /// Evals DO run if withdrawal amount ends up as 0.
    /// No withdraw => eval.
    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawalEvalZeroAmountEvalNoop(address alice, bytes32 vaultId, uint256 withdrawAmount18)
        external
    {
        vm.assume(vaultId != bytes32(0));
        withdrawAmount18 = bound(withdrawAmount18, 1, type(uint128).max);
        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"always fails\");");
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);
        checkWithdraw(alice, vaultId, Float.wrap(0), withdrawAmount, evals, 0, 0, "always fails");
    }

    /// A revert in the action prevents withdraw from being enacted.
    /// forge-config: default.fuzz.runs = 100
    function testOrderBookWithdrawalEvalRevertInAction(
        address alice,
        bytes32 vaultId,
        uint256 depositAmount18,
        uint256 withdrawAmount18
    ) external {
        vm.assume(vaultId != bytes32(0));
        depositAmount18 = bound(depositAmount18, 1, type(uint128).max);
        withdrawAmount18 = bound(withdrawAmount18, 1, depositAmount18);

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 18);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 18);

        vm.startPrank(alice);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(
                IERC20.transferFrom.selector, alice, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, depositAmount18
            ),
            abi.encode(true)
        );
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
            .deposit4(address(iToken0), vaultId, depositAmount, new TaskV2[](0));

        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, alice, withdrawAmount18),
            abi.encode(true)
        );

        bytes[] memory evals = new bytes[](1);
        evals[0] = bytes(":ensure(0 \"revert in action\");");
        TaskV2[] memory actions = evalsToActions(evals);

        assertTrue(
            depositAmount.eq(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(alice, address(iToken0), vaultId)
            )
        );

        vm.expectRevert("revert in action");
        IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
            .withdraw4(address(iToken0), vaultId, withdrawAmount, actions);

        assertTrue(
            depositAmount.eq(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(alice, address(iToken0), vaultId)
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testOrderWithdrawContext(address alice, bytes32 vaultId, uint256 depositAmount18, uint256 targetAmount18)
        external
    {
        vm.assume(vaultId != bytes32(0));
        depositAmount18 = bound(depositAmount18, 1, uint256(int256(type(int128).max)));
        targetAmount18 = bound(targetAmount18, 1, uint256(int256(type(int128).max)));
        uint256 withdrawAmount18 = depositAmount18 > targetAmount18 ? targetAmount18 : depositAmount18;

        Float depositAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(depositAmount18, 6);
        Float targetAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(targetAmount18, 6);
        Float withdrawAmount = LibDecimalFloat.fromFixedDecimalLosslessPacked(withdrawAmount18, 6);

        string memory usingWordsFrom = string.concat("using-words-from ", address(iSubParser).toHexString(), "\n");

        bytes[] memory evals = new bytes[](7);
        evals[0] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(orderbook() ",
                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.toHexString(),
                ") \"orderbook is orderbook\");"
            )
        );
        evals[1] = bytes(
            string.concat(
                usingWordsFrom, ":ensure(equal-to(withdrawer() ", alice.toHexString(), ") \"withdrawer is alice\");"
            )
        );
        evals[2] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-token() ",
                address(iToken0).toHexString(),
                ") \"withdraw token\");"
            )
        );
        evals[3] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-vault-id() ",
                uint256(vaultId).toHexString(),
                ") \"withdraw vaultId\");"
            )
        );
        evals[4] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-vault-before() ",
                depositAmount.toDecimalString(false),
                ") \"vault before\");"
            )
        );
        evals[5] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-vault-after() ",
                depositAmount.sub(withdrawAmount).toDecimalString(false),
                ") \"balance after\");"
            )
        );
        // target amount
        evals[6] = bytes(
            string.concat(
                usingWordsFrom,
                ":ensure(equal-to(withdraw-target-amount() ",
                targetAmount.toDecimalString(false),
                ") \"target amount\");"
            )
        );
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(6));

        checkWithdraw(alice, vaultId, depositAmount, targetAmount, evals, 0, 0, "");
    }
}
