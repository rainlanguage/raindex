// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

import {GenericPoolOrderBookV6FlashBorrower} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {FlashLoanFailed} from "../../src/abstract/OrderBookV6FlashBorrower.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    TakeOrderConfigV4,
    OrderV4,
    IOV2,
    EvaluableV4,
    SignedContextV1,
    TaskV2,
    Float
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockOrderBookBase} from "test/util/abstract/MockOrderBookBase.sol";

/// @dev Mock orderbook that returns false from flashLoan (inherits default stub).
contract FalseFlashLoanMockOrderBook is MockOrderBookBase {}

contract OrderBookV6FlashBorrowerFlashLoanFailedTest is Test {
    /// arb4 reverts with FlashLoanFailed when the orderbook's flashLoan
    /// returns false.
    function testFlashLoanFailed() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);

        FalseFlashLoanMockOrderBook mockOb = new FalseFlashLoanMockOrderBook();
        vm.etch(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, address(mockOb).code);

        GenericPoolOrderBookV6FlashBorrower arb = new GenericPoolOrderBookV6FlashBorrower();

        IOV2[] memory validInputs = new IOV2[](1);
        validInputs[0] = IOV2(address(inputToken), bytes32(0));
        IOV2[] memory validOutputs = new IOV2[](1);
        validOutputs[0] = IOV2(address(outputToken), bytes32(0));

        OrderV4 memory order = OrderV4({
            owner: address(0x1234),
            evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
            validInputs: validInputs,
            validOutputs: validOutputs,
            nonce: bytes32(0)
        });

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));

        vm.expectRevert(FlashLoanFailed.selector);
        arb.arb4(
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(100, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: ""
            }),
            "",
            TaskV2({
                evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
