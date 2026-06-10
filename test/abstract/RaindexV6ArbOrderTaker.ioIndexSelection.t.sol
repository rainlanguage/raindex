// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";

import {GenericPoolRaindexV6ArbOrderTaker} from "../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    TakeOrderConfigV4,
    OrderV4,
    IOV2,
    EvaluableV4,
    SignedContextV1,
    TaskV2
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterStoreV3.sol";
import {LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockExchange} from "test/util/concrete/MockExchange.sol";
import {RealisticOrderTakerMockRaindex} from "test/util/concrete/RealisticOrderTakerMockRaindex.sol";

/// arb5 reads the order's input token from `validInputs[inputIOIndex]` and the
/// output token from `validOutputs[outputIOIndex]`. A single order with two IOs
/// per side (index 0 = decoy, index 1 = real) and indices set to 1 must run the
/// full cycle on the index-1 tokens.
///
/// The scenario leaves BOTH a real-input leftover (raindex pulls less than the
/// swap produced) and a real-output leftover (the exchange swaps less than the
/// raindex sent), so finalize must sweep both real tokens. If arb5 read the
/// decoy input (index 0) the raindex's pull reverts on a missing allowance; if
/// arb5 read the decoy output (index 0) finalize sweeps the zero-balance decoy
/// and strands the real-output leftover in the arb.
contract RaindexV6ArbOrderTakerIoIndexSelectionTest is Test {
    receive() external payable {}

    function noopTask() internal pure returns (TaskV2 memory) {
        return TaskV2({
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            signedContext: new SignedContextV1[](0)
        });
    }

    function testArb5SelectsTokensByConfiguredIoIndices() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);
        // Decoys placed at IO index 0; the real tokens sit at index 1.
        MockToken decoyInput = new MockToken("DecoyIn", "DIN", 18);
        MockToken decoyOutput = new MockToken("DecoyOut", "DOUT", 18);

        // Raindex sends 100e18 output and pulls 80e18 input.
        RealisticOrderTakerMockRaindex raindex = new RealisticOrderTakerMockRaindex(80e18);
        MockExchange exchange = new MockExchange();

        outputToken.mint(address(raindex), 100e18);
        // Exchange can only fill an 80e18 swap, so 20e18 real output is left in
        // the arb to be finalized.
        inputToken.mint(address(exchange), 80e18);

        GenericPoolRaindexV6ArbOrderTaker arb = new GenericPoolRaindexV6ArbOrderTaker();

        IOV2[] memory validInputs = new IOV2[](2);
        validInputs[0] = IOV2(address(decoyInput), bytes32(0));
        validInputs[1] = IOV2(address(inputToken), bytes32(0));
        IOV2[] memory validOutputs = new IOV2[](2);
        validOutputs[0] = IOV2(address(decoyOutput), bytes32(0));
        validOutputs[1] = IOV2(address(outputToken), bytes32(0));

        OrderV4 memory order = OrderV4({
            owner: address(0x1234),
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            validInputs: validInputs,
            validOutputs: validOutputs,
            nonce: bytes32(0)
        });

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        // inputIOIndex 1, outputIOIndex 1: the REAL tokens.
        orders[0] = TakeOrderConfigV4(order, 1, 1, new SignedContextV1[](0));

        bytes memory exchangeData =
            abi.encodeCall(MockExchange.swap, (IERC20(address(outputToken)), IERC20(address(inputToken)), 80e18));

        arb.arb5(
            IRaindexV6(address(raindex)),
            TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: abi.encode(address(exchange), address(exchange), exchangeData)
            }),
            noopTask()
        );

        // The cycle ran on the index-1 (real) tokens: raindex pulled the real
        // input, the exchange swapped 80e18 of the real output, and finalize
        // swept the 20e18 real-output leftover to the sender.
        assertEq(inputToken.balanceOf(address(raindex)), 80e18, "raindex pulled real input");
        assertEq(outputToken.balanceOf(address(exchange)), 80e18, "exchange swapped real output");
        assertEq(outputToken.balanceOf(address(this)), 20e18, "real output leftover swept to sender");
        assertEq(inputToken.balanceOf(address(this)), 0, "no real input leftover");
        // The arb is left empty of the real output (it was finalized, not
        // stranded by reading a decoy index).
        assertEq(outputToken.balanceOf(address(arb)), 0, "arb empty of real output");

        // The decoys (index 0) were never touched: no balances moved, no
        // allowance was granted to the raindex on the decoy input.
        assertEq(decoyInput.allowance(address(arb), address(raindex)), 0, "decoy input never approved to raindex");
        assertEq(decoyInput.balanceOf(address(arb)), 0, "no decoy input held");
        assertEq(decoyOutput.balanceOf(address(this)), 0, "no decoy output swept");
    }
}
