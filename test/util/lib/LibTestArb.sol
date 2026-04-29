// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {Vm} from "forge-std/Vm.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    TakeOrderConfigV4,
    OrderV4,
    IOV2,
    EvaluableV4,
    SignedContextV1,
    TaskV2
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibInterpreterDeploy} from "rain.interpreter/lib/deploy/LibInterpreterDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockExchange} from "test/util/concrete/MockExchange.sol";
import {RealisticOrderTakerMockRaindex} from "test/util/concrete/RealisticOrderTakerMockRaindex.sol";

/// @dev Return value from `setupAndArb`.
struct ArbResult {
    GenericPoolRaindexV6ArbOrderTaker arb;
    MockToken inputToken;
    MockToken outputToken;
    RealisticOrderTakerMockRaindex raindex;
    MockExchange exchange;
}

/// @dev Return value from `setup`. Caller keeps their own exchange reference.
struct OrderTakerSetup {
    GenericPoolRaindexV6ArbOrderTaker arb;
    MockToken inputToken;
    MockToken outputToken;
    IRaindexV6 raindex;
    TakeOrdersConfigV5 takeOrdersConfig;
}

library LibTestArb {
    /// Deploy Zoltu + TOFU prerequisites for arb tests.
    function deployPrereqs(Vm vm) internal {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);
    }

    /// Build a no-op TaskV2 (empty bytecode, real deploy constants).
    function noopTask() internal pure returns (TaskV2 memory) {
        return TaskV2({
            evaluable: EvaluableV4(
                IInterpreterV4(LibInterpreterDeploy.INTERPRETER_DEPLOYED_ADDRESS),
                IInterpreterStoreV3(LibInterpreterDeploy.STORE_DEPLOYED_ADDRESS),
                hex""
            ),
            signedContext: new SignedContextV1[](0)
        });
    }

    /// Set up a standard arb scenario and execute arb5.
    ///
    /// @param vm The Vm cheatcode handle.
    /// @param raindexPullAmount How many inputTokens the mock Raindex pulls from arb.
    /// @param raindexOutputAmount How many outputTokens the mock Raindex has to send.
    /// @param exchangeInputAmount How many inputTokens the exchange has.
    /// @param swapAmount How many outputTokens the arb swaps at the exchange.
    /// @param task The post-arb task to run.
    /// @param ethValue ETH to send with arb5.
    function setupAndArb(
        Vm vm,
        uint256 raindexPullAmount,
        uint256 raindexOutputAmount,
        uint256 exchangeInputAmount,
        uint256 swapAmount,
        TaskV2 memory task,
        uint256 ethValue
    ) internal returns (ArbResult memory) {
        deployPrereqs(vm);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);

        RealisticOrderTakerMockRaindex raindex = new RealisticOrderTakerMockRaindex(raindexPullAmount);
        MockExchange exchange = new MockExchange();

        outputToken.mint(address(raindex), raindexOutputAmount);
        inputToken.mint(address(exchange), exchangeInputAmount);

        GenericPoolRaindexV6ArbOrderTaker arb = new GenericPoolRaindexV6ArbOrderTaker();

        bytes memory exchangeData =
            abi.encodeCall(MockExchange.swap, (IERC20(address(outputToken)), IERC20(address(inputToken)), swapAmount));

        TakeOrdersConfigV5 memory takeOrdersConfig;
        {
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

            takeOrdersConfig = TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: abi.encode(address(exchange), address(exchange), exchangeData)
            });
        }

        arb.arb5{value: ethValue}(IRaindexV6(address(raindex)), takeOrdersConfig, task);

        return
            ArbResult({
                arb: arb, inputToken: inputToken, outputToken: outputToken, raindex: raindex, exchange: exchange
            });
    }

    /// Set up an order-taker arb scenario without executing it.
    /// The caller provides their own exchange. Returns everything needed
    /// to call arb5 directly.
    ///
    /// @param vm The Vm cheatcode handle.
    /// @param exchange The exchange contract address.
    /// @param amount Token amount for the swap (18 decimals). Used as
    /// raindexPullAmount, raindexOutputAmount, exchangeInputAmount, and swapAmount.
    function setup(Vm vm, address exchange, uint256 amount) internal returns (OrderTakerSetup memory) {
        deployPrereqs(vm);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);

        RealisticOrderTakerMockRaindex raindex = new RealisticOrderTakerMockRaindex(amount);

        outputToken.mint(address(raindex), amount);
        inputToken.mint(exchange, amount);

        GenericPoolRaindexV6ArbOrderTaker arb = new GenericPoolRaindexV6ArbOrderTaker();

        bytes memory exchangeData =
            abi.encodeCall(MockExchange.swap, (IERC20(address(outputToken)), IERC20(address(inputToken)), amount));

        TakeOrdersConfigV5 memory takeOrdersConfig;
        {
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

            takeOrdersConfig = TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(0, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: abi.encode(exchange, exchange, exchangeData)
            });
        }

        return OrderTakerSetup({
            arb: arb,
            inputToken: inputToken,
            outputToken: outputToken,
            raindex: IRaindexV6(address(raindex)),
            takeOrdersConfig: takeOrdersConfig
        });
    }
}
