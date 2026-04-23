// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test, stdError} from "forge-std/Test.sol";

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {
    OrderConfigV4,
    OrderV4,
    IOV2,
    ClearConfigV2,
    EvaluableV4,
    SignedContextV1,
    IInterpreterV4,
    TaskV2,
    Float,
    IRaindexV6
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {NotOrderOwner, StackItem, NegativeBounty, ClearZeroAmount} from "../../../src/concrete/ob/OrderBookV6.sol";
import {LibNamespace} from "rain.interpreter.interface/lib/ns/LibNamespace.sol";
import {StateNamespace, EvalV4, SourceIndexV2} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {
    LibFixedPointDecimalArithmeticOpenZeppelin
} from "rain.math.fixedpoint/lib/LibFixedPointDecimalArithmeticOpenZeppelin.sol";
import {Math} from "openzeppelin-contracts/contracts/utils/math/Math.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

import {LibFormatDecimalFloat} from "rain.math.float/lib/format/LibFormatDecimalFloat.sol";
import {console2} from "forge-std/console2.sol";

contract MockInterpreter {
    StackItem[] internal sStack;

    constructor(StackItem[] memory stack) {
        sStack = stack;
    }

    function eval4(EvalV4 memory) external view returns (StackItem[] memory, bytes32[] memory) {
        return (sStack, new bytes32[](0));
    }
}

/// @title OrderBookV6ClearTest
/// Tests clearing an order.
contract OrderBookV6ClearTest is OrderBookV6ExternalMockTest {
    using LibFixedPointDecimalArithmeticOpenZeppelin for uint256;
    using Math for uint256;
    using LibDecimalFloat for Float;

    /// Make a deposit to the OB mocking the internal transferFrom call.
    function _depositInternal(address depositor, address token, bytes32 vaultId, Float amount) internal {
        if (vaultId != bytes32(0)) {
            uint256 amount18 = LibDecimalFloat.toFixedDecimalLossless(amount, 18);
            // The transferFrom will be used regardless of whether vaultId is 0 or
            // not, but in the nonzero case the deposit has to be done now, for the
            // zero case the clear will internally do a transfer from.
            vm.mockCall(
                token,
                abi.encodeWithSelector(
                    IERC20.transferFrom.selector, depositor, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, amount18
                ),
                abi.encode(true)
            );
            vm.prank(depositor);
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .deposit4(address(token), vaultId, amount, new TaskV2[](0));
            Float balance =
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).vaultBalance2(depositor, token, vaultId);
            assertTrue(balance.eq(amount));
        }
    }

    function conformBasicConfig(OrderConfigV4 memory aliceConfig, OrderConfigV4 memory bobConfig) internal view {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        vm.assume(bobConfig.validInputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);

        IOV2[] memory aliceValidInputs = new IOV2[](1);
        aliceValidInputs[0] = aliceConfig.validInputs[0];
        aliceConfig.validInputs = aliceValidInputs;

        IOV2[] memory aliceValidOutputs = new IOV2[](1);
        aliceValidOutputs[0] = aliceConfig.validOutputs[0];
        aliceConfig.validOutputs = aliceValidOutputs;

        IOV2[] memory bobValidInputs = new IOV2[](1);
        bobValidInputs[0] = bobConfig.validInputs[0];
        bobConfig.validInputs = bobValidInputs;

        IOV2[] memory bobValidOutputs = new IOV2[](1);
        bobValidOutputs[0] = bobConfig.validOutputs[0];
        bobConfig.validOutputs = bobValidOutputs;

        aliceConfig.evaluable.interpreter = iInterpreter;
        aliceConfig.evaluable.store = iStore;

        bobConfig.evaluable.interpreter = iInterpreter;
        bobConfig.evaluable.store = iStore;

        aliceConfig.validInputs[0].token = address(iToken0);
        aliceConfig.validOutputs[0].token = address(iToken1);

        bobConfig.validInputs[0].token = address(iToken1);
        bobConfig.validOutputs[0].token = address(iToken0);

        aliceConfig.meta = "";
        bobConfig.meta = "";
    }

    struct CheckClear {
        address alice;
        OrderConfigV4 aliceConfig;
        address bob;
        OrderConfigV4 bobConfig;
        address bountyBot;
        bytes32 aliceBountyVaultId;
        bytes32 bobBountyVaultId;
        Float aliceAmount;
        Float bobAmount;
        bytes expression;
        StackItem[] orderStackAlice;
        StackItem[] orderStackBob;
        Float expectedAliceOutput;
        Float expectedBobOutput;
        Float expectedAliceInput;
        Float expectedBobInput;
        bytes expectedError;
    }

    function checkClear(CheckClear memory clear) internal {
        vm.assume(clear.alice != clear.bob);
        vm.assume(clear.alice != clear.bountyBot);
        vm.assume(clear.bob != clear.bountyBot);
        vm.assume(clear.aliceBountyVaultId != clear.bobBountyVaultId);
        vm.assume(clear.aliceBountyVaultId != bytes32(0));
        vm.assume(clear.bobBountyVaultId != bytes32(0));

        conformBasicConfig(clear.aliceConfig, clear.bobConfig);
        vm.assume(keccak256(clear.aliceConfig.evaluable.bytecode) != keccak256(clear.bobConfig.evaluable.bytecode));

        _depositInternal(
            clear.alice,
            clear.aliceConfig.validOutputs[0].token,
            clear.aliceConfig.validOutputs[0].vaultId,
            clear.aliceAmount
        );
        _depositInternal(
            clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId, clear.bobAmount
        );

        if (clear.aliceConfig.validInputs[0].vaultId != bytes32(0)) {
            Float aliceInputBalance = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .vaultBalance2(
                    clear.alice, clear.aliceConfig.validInputs[0].token, clear.aliceConfig.validInputs[0].vaultId
                );
            assertTrue(aliceInputBalance.isZero());
        }

        if (clear.aliceConfig.validOutputs[0].vaultId != bytes32(0)) {
            Float aliceOutputBalance = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .vaultBalance2(
                    clear.alice, clear.aliceConfig.validOutputs[0].token, clear.aliceConfig.validOutputs[0].vaultId
                );
            assertTrue(aliceOutputBalance.eq(clear.aliceAmount));
        }

        if (clear.bobConfig.validInputs[0].vaultId != bytes32(0)) {
            Float bobInputBalance = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .vaultBalance2(clear.bob, clear.bobConfig.validInputs[0].token, clear.bobConfig.validInputs[0].vaultId);
            assertTrue(bobInputBalance.isZero());
        }

        if (clear.bobConfig.validOutputs[0].vaultId != bytes32(0)) {
            Float bobOutputBalance = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .vaultBalance2(
                    clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId
                );
            assertTrue(bobOutputBalance.eq(clear.bobAmount), "Bob output balance should be equal to bob amount");
        }
        {
            Float aliceBountyBalance = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .vaultBalance2(clear.bountyBot, clear.aliceConfig.validOutputs[0].token, clear.aliceBountyVaultId);
            assertTrue(aliceBountyBalance.isZero());
        }
        {
            Float bobBountyBalance = IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .vaultBalance2(clear.bountyBot, clear.bobConfig.validOutputs[0].token, clear.bobBountyVaultId);
            assertTrue(bobBountyBalance.isZero());
        }

        {
            {
                MockInterpreter aliceInterpreter = new MockInterpreter(clear.orderStackAlice);
                MockInterpreter bobInterpreter = new MockInterpreter(clear.orderStackBob);
                clear.aliceConfig.evaluable.interpreter = IInterpreterV4(address(aliceInterpreter));
                clear.bobConfig.evaluable.interpreter = IInterpreterV4(address(bobInterpreter));
            }

            OrderV4 memory aliceOrder;
            OrderV4 memory bobOrder;
            {
                (aliceOrder,) = addOrderWithChecks(clear.alice, clear.aliceConfig, clear.expression);
                (bobOrder,) = addOrderWithChecks(clear.bob, clear.bobConfig, clear.expression);
            }

            ClearConfigV2 memory configClear = ClearConfigV2({
                aliceInputIOIndex: 0,
                aliceOutputIOIndex: 0,
                bobInputIOIndex: 0,
                bobOutputIOIndex: 0,
                aliceBountyVaultId: clear.aliceBountyVaultId,
                bobBountyVaultId: clear.bobBountyVaultId
            });

            if (clear.aliceConfig.validInputs[0].vaultId == bytes32(0)) {
                // Match pushTokens logic: toFixedDecimalLossy (truncating).
                (uint256 expectedAliceInput18,) = LibDecimalFloat.toFixedDecimalLossy(clear.expectedAliceInput, 18);
                vm.mockCall(
                    clear.aliceConfig.validInputs[0].token,
                    abi.encodeWithSelector(IERC20.balanceOf.selector, clear.alice),
                    abi.encode(expectedAliceInput18)
                );
                vm.mockCall(
                    clear.aliceConfig.validInputs[0].token,
                    abi.encodeWithSelector(IERC20.allowance.selector, clear.alice),
                    abi.encode(expectedAliceInput18)
                );
                vm.mockCall(
                    clear.aliceConfig.validInputs[0].token,
                    abi.encodeWithSelector(IERC20.transfer.selector, clear.alice, expectedAliceInput18),
                    abi.encode(true)
                );

                if (clear.expectedError.length == 0) {
                    vm.expectCall(
                        clear.aliceConfig.validInputs[0].token,
                        abi.encodeWithSelector(IERC20.balanceOf.selector, clear.alice),
                        1
                    );
                    vm.expectCall(
                        clear.aliceConfig.validInputs[0].token,
                        abi.encodeWithSelector(IERC20.allowance.selector, clear.alice),
                        1
                    );
                    if (expectedAliceInput18 > 0) {
                        vm.expectCall(
                            clear.aliceConfig.validInputs[0].token,
                            abi.encodeWithSelector(IERC20.transfer.selector, clear.alice, expectedAliceInput18),
                            1
                        );
                    }
                }
            }

            if (clear.aliceConfig.validOutputs[0].vaultId == bytes32(0)) {
                // Match pullTokens logic: toFixedDecimalLossy + round up.
                (uint256 amount18, bool lossless) = LibDecimalFloat.toFixedDecimalLossy(clear.expectedAliceOutput, 18);
                if (!lossless) {
                    ++amount18;
                }
                uint256 aliceBalance18 = LibDecimalFloat.toFixedDecimalLossless(clear.aliceAmount, 18);
                vm.mockCall(
                    clear.aliceConfig.validOutputs[0].token,
                    abi.encodeWithSelector(IERC20.balanceOf.selector, clear.alice),
                    abi.encode(aliceBalance18)
                );
                vm.mockCall(
                    clear.aliceConfig.validOutputs[0].token,
                    abi.encodeWithSelector(IERC20.allowance.selector, clear.alice),
                    abi.encode(aliceBalance18)
                );
                vm.mockCall(
                    clear.aliceConfig.validOutputs[0].token,
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector,
                        clear.alice,
                        LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                        amount18
                    ),
                    abi.encode(true)
                );

                if (clear.expectedError.length == 0) {
                    vm.expectCall(
                        clear.aliceConfig.validOutputs[0].token,
                        abi.encodeWithSelector(IERC20.balanceOf.selector, clear.alice),
                        2
                    );
                    vm.expectCall(
                        clear.aliceConfig.validOutputs[0].token,
                        abi.encodeWithSelector(IERC20.allowance.selector, clear.alice),
                        2
                    );
                    if (amount18 > 0) {
                        vm.expectCall(
                            clear.aliceConfig.validOutputs[0].token,
                            abi.encodeWithSelector(
                                IERC20.transferFrom.selector,
                                clear.alice,
                                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                                amount18
                            ),
                            1
                        );
                    }
                }
            }

            if (clear.bobConfig.validInputs[0].vaultId == bytes32(0)) {
                // Match pushTokens logic: toFixedDecimalLossy (truncating).
                (uint256 expectedBobInput18,) = LibDecimalFloat.toFixedDecimalLossy(clear.expectedBobInput, 18);
                vm.mockCall(
                    clear.bobConfig.validInputs[0].token,
                    abi.encodeWithSelector(IERC20.balanceOf.selector, clear.bob),
                    abi.encode(expectedBobInput18)
                );
                vm.mockCall(
                    clear.bobConfig.validInputs[0].token,
                    abi.encodeWithSelector(IERC20.allowance.selector, clear.bob),
                    abi.encode(expectedBobInput18)
                );
                vm.mockCall(
                    clear.bobConfig.validInputs[0].token,
                    abi.encodeWithSelector(IERC20.transfer.selector, clear.bob, expectedBobInput18),
                    abi.encode(true)
                );

                if (clear.expectedError.length == 0) {
                    vm.expectCall(
                        clear.bobConfig.validInputs[0].token,
                        abi.encodeWithSelector(IERC20.balanceOf.selector, clear.bob),
                        1
                    );

                    vm.expectCall(
                        clear.bobConfig.validInputs[0].token,
                        abi.encodeWithSelector(IERC20.allowance.selector, clear.bob),
                        1
                    );
                    if (expectedBobInput18 > 0) {
                        vm.expectCall(
                            clear.bobConfig.validInputs[0].token,
                            abi.encodeWithSelector(IERC20.transfer.selector, clear.bob, expectedBobInput18),
                            1
                        );
                    }
                }
            }

            if (clear.bobConfig.validOutputs[0].vaultId == bytes32(0)) {
                // Match pullTokens logic: toFixedDecimalLossy + round up.
                (uint256 bobAmount18, bool bobLossless) =
                    LibDecimalFloat.toFixedDecimalLossy(clear.expectedBobOutput, 18);
                if (!bobLossless) {
                    ++bobAmount18;
                }
                uint256 bobBalance18 = LibDecimalFloat.toFixedDecimalLossless(clear.bobAmount, 18);
                vm.mockCall(
                    clear.bobConfig.validOutputs[0].token,
                    abi.encodeWithSelector(IERC20.balanceOf.selector, clear.bob),
                    abi.encode(bobBalance18)
                );
                vm.mockCall(
                    clear.bobConfig.validOutputs[0].token,
                    abi.encodeWithSelector(IERC20.allowance.selector, clear.bob),
                    abi.encode(bobBalance18)
                );
                vm.mockCall(
                    clear.bobConfig.validOutputs[0].token,
                    abi.encodeWithSelector(
                        IERC20.transferFrom.selector,
                        clear.bob,
                        LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                        bobAmount18
                    ),
                    abi.encode(true)
                );

                if (clear.expectedError.length == 0) {
                    vm.expectCall(
                        clear.bobConfig.validOutputs[0].token,
                        abi.encodeWithSelector(IERC20.balanceOf.selector, clear.bob),
                        2
                    );
                    vm.expectCall(
                        clear.bobConfig.validOutputs[0].token,
                        abi.encodeWithSelector(IERC20.allowance.selector, clear.bob),
                        2
                    );
                    if (bobAmount18 > 0) {
                        vm.expectCall(
                            clear.bobConfig.validOutputs[0].token,
                            abi.encodeWithSelector(
                                IERC20.transferFrom.selector,
                                clear.bob,
                                LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS,
                                bobAmount18
                            ),
                            1
                        );
                    }
                }
            }

            vm.prank(clear.bountyBot);
            if (clear.expectedError.length > 0) {
                vm.expectRevert(clear.expectedError);
            }
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                .clear3(aliceOrder, bobOrder, configClear, new SignedContextV1[](0), new SignedContextV1[](0));
        }

        if (clear.expectedError.length == 0) {
            if (clear.aliceConfig.validOutputs[0].vaultId != bytes32(0)) {
                assertTrue(
                    IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                        .vaultBalance2(
                            clear.alice,
                            clear.aliceConfig.validOutputs[0].token,
                            clear.aliceConfig.validOutputs[0].vaultId
                        ).eq(clear.aliceAmount.sub(clear.expectedAliceOutput)),
                    "Alice output vault"
                );
            }

            if (clear.aliceConfig.validInputs[0].vaultId != bytes32(0)) {
                assertTrue(
                    IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                        .vaultBalance2(
                            clear.alice,
                            clear.aliceConfig.validInputs[0].token,
                            clear.aliceConfig.validInputs[0].vaultId
                        ).eq(clear.expectedAliceInput),
                    "Alice input vault"
                );
            }

            if (clear.bobConfig.validOutputs[0].vaultId != bytes32(0)) {
                assertTrue(
                    IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                        .vaultBalance2(
                            clear.bob, clear.bobConfig.validOutputs[0].token, clear.bobConfig.validOutputs[0].vaultId
                        ).eq(clear.bobAmount.sub(clear.expectedBobOutput)),
                    "Bob output vault"
                );
            }

            if (clear.bobConfig.validInputs[0].vaultId != bytes32(0)) {
                assertTrue(
                    IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                        .vaultBalance2(
                            clear.bob, clear.bobConfig.validInputs[0].token, clear.bobConfig.validInputs[0].vaultId
                        ).eq(clear.expectedBobInput),
                    "Bob input vault"
                );
            }

            assertTrue(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(clear.bountyBot, clear.aliceConfig.validOutputs[0].token, clear.aliceBountyVaultId)
                    .eq(clear.expectedAliceOutput.sub(clear.expectedBobInput)),
                "Alice bounty"
            );
            assertTrue(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(clear.bountyBot, clear.aliceConfig.validInputs[0].token, clear.aliceBountyVaultId)
                    .isZero(),
                "Alice bounty input"
            );
            assertTrue(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(clear.bountyBot, clear.bobConfig.validOutputs[0].token, clear.bobBountyVaultId)
                    .eq(clear.expectedBobOutput.sub(clear.expectedAliceInput)),
                "Bob bounty"
            );
            assertTrue(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(clear.bountyBot, clear.bobConfig.validInputs[0].token, clear.bobBountyVaultId)
                    .eq(LibDecimalFloat.packLossless(0, 0)),
                "Bob bounty input"
            );
        }
    }

    /// forge-config: default.fuzz.runs = 1000
    function testClearSimple(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) public {
        Float aliceAmount = LibDecimalFloat.packLossless(2, 0);
        Float bobAmount = LibDecimalFloat.packLossless(2, 0);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        StackItem[] memory orderStackAlice = new StackItem[](2);
        orderStackAlice[0] = StackItem.wrap(Float.unwrap(LibDecimalFloat.packLossless(0.99e18, -18))); // orderIORatio
        orderStackAlice[1] = StackItem.wrap(Float.unwrap(LibDecimalFloat.packLossless(0.5e18, -18))); // orderOutputMax

        checkClear(
            CheckClear(
                alice,
                aliceConfig,
                bob,
                bobConfig,
                bountyBot,
                aliceBountyVaultId,
                bobBountyVaultId,
                aliceAmount,
                bobAmount,
                expression,
                orderStackAlice,
                orderStackAlice,
                LibDecimalFloat.packLossless(0.5e18, -18),
                LibDecimalFloat.packLossless(0.5e18, -18),
                LibDecimalFloat.packLossless(0.495e18, -18),
                LibDecimalFloat.packLossless(0.495e18, -18),
                ""
            )
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAliceInputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validInputs.length > 0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearBobInputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(bobConfig.validInputs.length > 0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAliceBothVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearBobBothVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(bobConfig.validInputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearBobAliceInputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(bobConfig.validInputs.length > 0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAliceOutputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validOutputs.length > 0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearBobOutputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(bobConfig.validOutputs.length > 0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAliceBobOutputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validOutputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAliceInputBobOutputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearBobInputAliceOutputVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(bobConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAliceInputBobBothVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);
        vm.assume(bobConfig.validInputs.length > 0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearBobInputAliceBothVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(bobConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAliceOutputBobBothVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validOutputs.length > 0);
        vm.assume(bobConfig.validInputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearBobOutputAliceBothVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(bobConfig.validOutputs.length > 0);
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearAllVaultIdZero(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        vm.assume(aliceConfig.validInputs.length > 0);
        vm.assume(aliceConfig.validOutputs.length > 0);
        vm.assume(bobConfig.validInputs.length > 0);
        vm.assume(bobConfig.validOutputs.length > 0);
        aliceConfig.validInputs[0].vaultId = bytes32(0);
        aliceConfig.validOutputs[0].vaultId = bytes32(0);
        bobConfig.validInputs[0].vaultId = bytes32(0);
        bobConfig.validOutputs[0].vaultId = bytes32(0);
        testClearSimple(alice, aliceConfig, bob, bobConfig, expression, bountyBot, aliceBountyVaultId, bobBountyVaultId);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearFuzzIoRatioHappy(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId,
        uint256 aliceIORatio18,
        uint256 bobIORatio18
    ) external {
        // 0 tested separately.
        aliceIORatio18 = bound(aliceIORatio18, 1, 1e18);
        bobIORatio18 = bound(bobIORatio18, 1e18, uint256(1e18).fixedPointDiv(aliceIORatio18, Math.Rounding.Floor));

        Float aliceIORatio = LibDecimalFloat.fromFixedDecimalLosslessPacked(aliceIORatio18, 18);
        Float bobIORatio = LibDecimalFloat.fromFixedDecimalLosslessPacked(bobIORatio18, 18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.expression = expression;
        checkClearStruct.aliceAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.bobAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(1, 0);
        // Alice is outputting 1 so bob will output enough to match this
        // according to his own IO ratio.
        checkClearStruct.expectedBobOutput = bobIORatio.inv().min(LibDecimalFloat.packLossless(1, 0));
        // Expected input for Alice is aliceOutput * aliceIORatio

        Float aliceOutput = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expectedAliceInput = aliceIORatio.mul(aliceOutput);
        // Expected input for Bob is Alice's output in entirety, because
        // alice IO * bob IO <= 1 and Bob is the larger ratio.
        // As Bob's ratio is >= 1 he will have his input shrunk to match
        // Alice's output. This means in this case Bob's input will be
        // 1 always, as it either = 1 anyway or matches Alice's 1.
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expectedError = "";

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio)); // orderIORatio
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutput)); // orderOutputMax

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio)); // orderIORatio
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(LibDecimalFloat.packLossless(1e18, -18))); // orderOutputMax

        checkClear(checkClearStruct);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearFuzzIoRatioError(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId,
        uint256 aliceIORatio18,
        uint256 bobIORatio18
    ) external {
        aliceIORatio18 = bound(aliceIORatio18, 1e18 + 1, 1.1e18);
        bobIORatio18 = bound(bobIORatio18, 1e18 + 1, 1.1e18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.bobAmount = LibDecimalFloat.packLossless(1, 0);
        checkClearStruct.expression = expression;

        // Ratio is bound above so save to typecast.
        // forge-lint: disable-next-line(unsafe-typecast)
        Float aliceIORatio = LibDecimalFloat.packLossless(int256(aliceIORatio18), -18);
        // Ratio is bound above so save to typecast.
        // forge-lint: disable-next-line(unsafe-typecast)
        Float bobIORatio = LibDecimalFloat.packLossless(int256(bobIORatio18), -18);

        Float aliceOutput = LibDecimalFloat.packLossless(1, 0);
        Float bobOutput = LibDecimalFloat.packLossless(1, 0);

        // Mock the interpreter.eval that is used inside clear().calculateOrderIO()
        // Produce the stack output for OB
        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio)); // orderIORatio
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutput)); // orderOutputMax

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio)); // orderIORatio
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutput)); // orderOutputMax

        // Replicate calculateClearStateAlice logic:
        // aliceInput = outputMax * ratio = 1 * ratio > bobOutputMax = 1
        // So aliceInput is capped to bobOutputMax = 1, and
        // aliceOutput = aliceInput / aliceIORatio = 1 / aliceIORatio.
        // Same logic for bob.
        checkClearStruct.expectedAliceOutput = aliceOutput.div(aliceIORatio);
        checkClearStruct.expectedBobOutput = bobOutput.div(bobIORatio);
        // Inputs are capped to the counterparty's outputMax (= 1).
        checkClearStruct.expectedAliceInput = bobOutput;
        checkClearStruct.expectedBobInput = aliceOutput;

        checkClearStruct.expectedError = abi.encodeWithSelector(NegativeBounty.selector);

        checkClear(checkClearStruct);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioAliceOnly(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        Float aliceAmount = LibDecimalFloat.packLossless(2e18, -18);
        Float bobAmount = LibDecimalFloat.packLossless(3e18, -18);

        Float aliceIORatio = LibDecimalFloat.packLossless(0, 0);
        Float bobIORatio = LibDecimalFloat.packLossless(1, 0);

        Float aliceOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);
        Float bobOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = aliceAmount;
        checkClearStruct.bobAmount = bobAmount;
        checkClearStruct.expression = expression;

        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio));
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutputMax));

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio));
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutputMax));

        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedAliceInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(0.5e18, -18);

        checkClearStruct.expectedError = "";

        checkClear(checkClearStruct);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioBobOnly(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        Float aliceAmount = Float.wrap(bytes32(uint256(2)));
        Float bobAmount = Float.wrap(bytes32(uint256(3)));

        Float aliceIORatio = LibDecimalFloat.packLossless(1, 0);
        Float bobIORatio = LibDecimalFloat.packLossless(0, 0);

        Float aliceOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);
        Float bobOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = aliceAmount;
        checkClearStruct.bobAmount = bobAmount;
        checkClearStruct.expression = expression;

        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio));
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutputMax));

        checkClearStruct.orderStackBob = new StackItem[](2);
        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio));
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutputMax));

        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedAliceInput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedError = "";

        checkClear(checkClearStruct);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClear2ZeroRatioAliceAndBob(
        address alice,
        OrderConfigV4 memory aliceConfig,
        address bob,
        OrderConfigV4 memory bobConfig,
        bytes memory expression,
        address bountyBot,
        bytes32 aliceBountyVaultId,
        bytes32 bobBountyVaultId
    ) external {
        Float aliceAmount = Float.wrap(bytes32(uint256(2)));
        Float bobAmount = Float.wrap(bytes32(uint256(3)));

        Float aliceIORatio = Float.wrap(bytes32(uint256(0)));
        Float bobIORatio = Float.wrap(bytes32(uint256(0)));

        Float aliceOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);
        Float bobOutputMax = LibDecimalFloat.packLossless(0.5e18, -18);

        CheckClear memory checkClearStruct;
        checkClearStruct.alice = alice;
        checkClearStruct.aliceConfig = aliceConfig;
        checkClearStruct.bob = bob;
        checkClearStruct.bobConfig = bobConfig;
        checkClearStruct.bountyBot = bountyBot;
        checkClearStruct.aliceBountyVaultId = aliceBountyVaultId;
        checkClearStruct.bobBountyVaultId = bobBountyVaultId;
        checkClearStruct.aliceAmount = aliceAmount;
        checkClearStruct.bobAmount = bobAmount;
        checkClearStruct.expression = expression;

        // Mock the interpreter.eval for Alice and Bob orders with zero ratio
        checkClearStruct.orderStackAlice = new StackItem[](2);
        checkClearStruct.orderStackAlice[0] = StackItem.wrap(Float.unwrap(aliceIORatio));
        checkClearStruct.orderStackAlice[1] = StackItem.wrap(Float.unwrap(aliceOutputMax));

        checkClearStruct.orderStackBob = new StackItem[](2);

        checkClearStruct.orderStackBob[0] = StackItem.wrap(Float.unwrap(bobIORatio));
        checkClearStruct.orderStackBob[1] = StackItem.wrap(Float.unwrap(bobOutputMax));

        checkClearStruct.expectedAliceOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedBobOutput = LibDecimalFloat.packLossless(0.5e18, -18);
        checkClearStruct.expectedAliceInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedBobInput = LibDecimalFloat.packLossless(0, 0);
        checkClearStruct.expectedError = "";

        checkClear(checkClearStruct);
    }
}
