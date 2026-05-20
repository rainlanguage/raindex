// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {
    OrderV4,
    IOV2,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    EvaluableV4,
    SignedContextV1
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {TokenMismatch} from "../../../src/concrete/raindex/RaindexV6.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @title RaindexV6TakeOrderTokenMismatchTest
/// @notice A test harness for testing the RaindexV6 takeOrder function.
/// Focuses on the token mismatch case.
contract RaindexV6TakeOrderTokenMismatchTest is RaindexV6ExternalRealTest {
    /// It is only possible to get a token mismatch when there are at least two
    /// orders. This is because `takeOrders` is interactive so we assume that
    /// the caller's desired input and output tokens match the first order they
    /// pass in.
    /// Test a mismatch in the input tokens.
    /// forge-config: default.fuzz.runs = 10
    function testTokenMismatchInputs(
        OrderV4 memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        OrderV4 memory b,
        uint256 bInputIOIndex,
        uint256 bOutputIOIndex,
        uint256 maxTakerInput18,
        uint256 maxIORatio
    ) external {
        vm.assume(a.validInputs.length > 0);
        aInputIOIndex = bound(aInputIOIndex, 0, a.validInputs.length - 1);
        vm.assume(b.validInputs.length > 0);
        bInputIOIndex = bound(bInputIOIndex, 0, b.validInputs.length - 1);
        vm.assume(a.validOutputs.length > 0);
        aOutputIOIndex = bound(aOutputIOIndex, 0, a.validOutputs.length - 1);
        vm.assume(b.validOutputs.length > 0);
        bOutputIOIndex = bound(bOutputIOIndex, 0, b.validOutputs.length - 1);
        maxTakerInput18 = bound(maxTakerInput18, 1, uint256(int256(type(int224).max)));
        maxIORatio = bound(maxIORatio, 0, uint256(int256(type(int224).max)));

        vm.assume(a.validInputs[aInputIOIndex].token != a.validOutputs[aOutputIOIndex].token);

        // Mismatch on inputs across orders taken.
        vm.assume(a.validInputs[aInputIOIndex].token != b.validInputs[bInputIOIndex].token);
        // Line up outputs so we don't trigger that code path.
        b.validOutputs[bOutputIOIndex].token = a.validOutputs[aOutputIOIndex].token;

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](2);
        orders[0] = TakeOrderConfigV4(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfigV4(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV5 memory config = TakeOrdersConfigV5({
            minimumIO: Float.wrap(0),
            maximumIO: LibDecimalFloat.fromFixedDecimalLosslessPacked(maxTakerInput18, 18),
            maximumIORatio: LibDecimalFloat.fromFixedDecimalLosslessPacked(maxIORatio, 18),
            IOIsInput: true,
            orders: orders,
            data: ""
        });
        vm.expectRevert(abi.encodeWithSelector(TokenMismatch.selector));
        (Float totalTakerInput, Float totalTakerOutput) = iRaindex.takeOrders4(config);
        (totalTakerInput, totalTakerOutput);
    }

    /// Test a mismatch in the output tokens.
    /// forge-config: default.fuzz.runs = 10
    function testTokenDecimalMismatchOutputs(
        OrderV4 memory a,
        uint256 aInputIOIndex,
        uint256 aOutputIOIndex,
        OrderV4 memory b,
        uint256 bInputIOIndex,
        uint256 bOutputIOIndex,
        uint256 maxTakerInput18,
        uint256 maxIORatio18
    ) external {
        vm.assume(a.validInputs.length > 0);
        aInputIOIndex = bound(aInputIOIndex, 0, a.validInputs.length - 1);
        vm.assume(b.validInputs.length > 0);
        bInputIOIndex = bound(bInputIOIndex, 0, b.validInputs.length - 1);
        vm.assume(a.validOutputs.length > 0);
        aOutputIOIndex = bound(aOutputIOIndex, 0, a.validOutputs.length - 1);
        vm.assume(b.validOutputs.length > 0);
        bOutputIOIndex = bound(bOutputIOIndex, 0, b.validOutputs.length - 1);
        maxTakerInput18 = bound(maxTakerInput18, 1, uint256(int256(type(int224).max)));
        maxIORatio18 = bound(maxIORatio18, 0, uint256(int256(type(int224).max)));

        vm.assume(a.validOutputs[aOutputIOIndex].token != a.validInputs[aInputIOIndex].token);

        // Mismatch on outputs across orders taken.
        vm.assume(a.validOutputs[aOutputIOIndex].token != b.validOutputs[bOutputIOIndex].token);
        // Line up inputs so we don't trigger that code path.
        b.validInputs[bInputIOIndex].token = a.validInputs[aInputIOIndex].token;

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](2);
        orders[0] = TakeOrderConfigV4(a, aInputIOIndex, aOutputIOIndex, new SignedContextV1[](0));
        orders[1] = TakeOrderConfigV4(b, bInputIOIndex, bOutputIOIndex, new SignedContextV1[](0));
        TakeOrdersConfigV5 memory config = TakeOrdersConfigV5({
            minimumIO: Float.wrap(0),
            maximumIO: LibDecimalFloat.fromFixedDecimalLosslessPacked(maxTakerInput18, 18),
            maximumIORatio: LibDecimalFloat.fromFixedDecimalLosslessPacked(maxIORatio18, 18),
            IOIsInput: true,
            orders: orders,
            data: ""
        });
        vm.expectRevert(abi.encodeWithSelector(TokenMismatch.selector));
        (Float totalTakerInput, Float totalTakerOutput) = iRaindex.takeOrders4(config);
        (totalTakerInput, totalTakerOutput);
    }
}
