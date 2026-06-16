// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";

import {GenericPoolRaindexV6FlashBorrower} from "../../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
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
import {LibRaindexDeploy} from "../../src/lib/deploy/LibRaindexDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockExchange} from "test/util/concrete/MockExchange.sol";
import {RealisticFlashLendingMockRaindex} from "test/util/concrete/RealisticFlashLendingMockRaindex.sol";

/// arb4 reads `ordersInputToken` from `validInputs[inputIOIndex]` using the
/// INPUT-side index specifically. The existing arb4 tests all set
/// `inputIOIndex == outputIOIndex` (0/0), so reading the input token via
/// `outputIOIndex` instead selects the same slot and is indistinguishable. Here
/// the indices are DISTINCT (inputIOIndex 1, outputIOIndex 0): the real input
/// sits at validInputs[1] and the real output at validOutputs[0], with a decoy
/// input at validInputs[0].
///
/// arb4 grants the raindex a max allowance on `ordersInputToken` before the
/// flash loan, and the raindex's `takeOrders4` pulls the real input
/// (validInputs[inputIOIndex]) from the arb. If arb4 read the input token via
/// `outputIOIndex` it would approve the decoy input instead, so the raindex's
/// pull of the real input reverts on a zero allowance and the whole arb reverts.
contract RaindexV6FlashBorrowerInputIoIndexSelectionTest is Test {
    MockToken internal inputToken;
    MockToken internal outputToken;
    MockToken internal decoyInput;
    RealisticFlashLendingMockRaindex internal raindex;
    MockExchange internal exchange;
    GenericPoolRaindexV6FlashBorrower internal arb;

    function setUp() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        inputToken = new MockToken("Input", "IN", 18);
        outputToken = new MockToken("Output", "OUT", 18);
        // Decoy input occupies validInputs[0]; the real input is at index 1.
        decoyInput = new MockToken("DecoyIn", "DIN", 18);

        vm.etch(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, address(new RealisticFlashLendingMockRaindex()).code);
        raindex = RealisticFlashLendingMockRaindex(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS);
        exchange = new MockExchange();

        outputToken.mint(address(raindex), 1000e18);
        inputToken.mint(address(exchange), 100e18);

        arb = new GenericPoolRaindexV6FlashBorrower();
    }

    function noopTask() internal pure returns (TaskV2 memory) {
        return TaskV2({
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            signedContext: new SignedContextV1[](0)
        });
    }

    /// Real input at validInputs[1], real output at validOutputs[0]: the two
    /// sides use DIFFERENT indices (inputIOIndex 1, outputIOIndex 0).
    function buildOrders() internal view returns (TakeOrderConfigV4[] memory orders) {
        IOV2[] memory validInputs = new IOV2[](2);
        validInputs[0] = IOV2(address(decoyInput), bytes32(0));
        validInputs[1] = IOV2(address(inputToken), bytes32(0));
        IOV2[] memory validOutputs = new IOV2[](1);
        validOutputs[0] = IOV2(address(outputToken), bytes32(0));

        OrderV4 memory order = OrderV4({
            owner: address(0x1234),
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            validInputs: validInputs,
            validOutputs: validOutputs,
            nonce: bytes32(0)
        });

        orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 1, 0, new SignedContextV1[](0));
    }

    function exchangeData() internal view returns (bytes memory) {
        return abi.encode(
            address(exchange),
            address(exchange),
            abi.encodeCall(MockExchange.swap, (IERC20(address(outputToken)), IERC20(address(inputToken)), 100e18))
        );
    }

    function testArb4SelectsInputTokenByInputIoIndex() external {
        arb.arb4(
            IRaindexV6(address(raindex)),
            TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(100, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: buildOrders(),
                data: ""
            }),
            exchangeData(),
            noopTask()
        );

        // The real input (validInputs[inputIOIndex == 1]) drove the cycle: the
        // raindex pulled it from the arb and the exchange supplied it.
        assertEq(inputToken.balanceOf(address(raindex)), 100e18, "raindex pulled real input (inputIOIndex)");
        assertEq(outputToken.balanceOf(address(exchange)), 100e18, "exchange swapped borrowed output");
        // The decoy input (validInputs[outputIOIndex == 0]) was never approved or
        // moved.
        assertEq(decoyInput.allowance(address(arb), address(raindex)), 0, "decoy input never approved to raindex");
        assertEq(decoyInput.balanceOf(address(arb)), 0, "no decoy input held");
        assertEq(inputToken.balanceOf(address(arb)), 0, "arb empty of real input");
        assertEq(outputToken.balanceOf(address(arb)), 0, "arb empty of borrowed output");
    }
}
