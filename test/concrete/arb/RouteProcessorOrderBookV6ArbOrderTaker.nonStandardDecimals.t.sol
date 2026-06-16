// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

import {
    RouteProcessorOrderBookV6ArbOrderTaker
} from "../../../src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";
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
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockRouteProcessor} from "test/util/concrete/MockRouteProcessor.sol";
import {RealisticOrderTakerMockOrderBook} from "test/util/concrete/RealisticOrderTakerMockOrderBook.sol";

/// arb5 with RouteProcessor MUST correctly convert Float amounts to
/// fixed-point using the token's decimals, not hardcoded 18.
contract RouteProcessorOrderBookV6ArbOrderTakerNonStandardDecimalsTest is Test {
    function testRouteProcessorArb5SixDecimalTokens() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        // 6-decimal tokens (like USDC/USDT).
        MockToken inputToken = new MockToken("USDC", "USDC", 6);
        MockToken outputToken = new MockToken("USDT", "USDT", 6);

        // OB will send 100 USDT (100e6) to the taker, then pull 100 USDC.
        RealisticOrderTakerMockOrderBook orderBook = new RealisticOrderTakerMockOrderBook(100e6);
        MockRouteProcessor mockRp = new MockRouteProcessor();
        vm.etch(LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS, address(mockRp).code);
        address routeProcessor = LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS;

        // OB has outputToken to send to taker.
        outputToken.mint(address(orderBook), 100e6);
        // RouteProcessor has inputToken to give back after swap.
        inputToken.mint(routeProcessor, 100e6);

        RouteProcessorOrderBookV6ArbOrderTaker arb = new RouteProcessorOrderBookV6ArbOrderTaker();

        IOV2[] memory validInputs = new IOV2[](1);
        validInputs[0] = IOV2(address(inputToken), bytes32(0));
        IOV2[] memory validOutputs = new IOV2[](1);
        validOutputs[0] = IOV2(address(outputToken), bytes32(0));

        OrderV4 memory order = OrderV4({
            owner: address(0x1234),
            evaluable: EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                hex""
            ),
            validInputs: validInputs,
            validOutputs: validOutputs,
            nonce: bytes32(0)
        });

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, 0, 0, new SignedContextV1[](0));

        bytes memory takeOrdersData = abi.encode(hex"");

        arb.arb5(
            IRaindexV6(address(orderBook)),
            TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(100, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: takeOrdersData
            }),
            TaskV2({
                evaluable: EvaluableV4(
                    IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                    IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                    hex""
                ),
                signedContext: new SignedContextV1[](0)
            })
        );

        // Arb contract has no remaining tokens.
        assertEq(inputToken.balanceOf(address(arb)), 0, "arb inputToken");
        assertEq(outputToken.balanceOf(address(arb)), 0, "arb outputToken");
        // OB swapped outputToken for inputToken.
        assertEq(inputToken.balanceOf(address(orderBook)), 100e6, "OB inputToken");
        assertEq(outputToken.balanceOf(address(orderBook)), 0, "OB outputToken");
        // RouteProcessor sent all its inputToken.
        assertEq(inputToken.balanceOf(routeProcessor), 0, "RP inputToken");
        assertEq(outputToken.balanceOf(routeProcessor), 0, "RP outputToken");
        // Test contract received profit via finalizeArb.
        assertEq(outputToken.balanceOf(address(this)), 100e6, "test outputToken");
    }
}
