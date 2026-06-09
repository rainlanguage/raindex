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
    TaskV2,
    Float
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterStoreV3.sol";
import {LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "../../src/lib/deploy/LibRaindexDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MockExchange} from "test/util/concrete/MockExchange.sol";
import {RealisticOrderTakerMockRaindex} from "test/util/concrete/RealisticOrderTakerMockRaindex.sol";

contract RaindexV6ArbOrderTakerOnTakeOrders2Test is Test {
    /// arb5 completes a full order-taker cycle with real ERC20 transfers:
    /// takeOrders, onTakeOrders2 callback with exchange, and finalize.
    function testArb5RealTokenTransfers() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);

        // arb5 only trusts the canonical raindex deployment, so etch the mock's
        // runtime code (immutables included) at that address.
        RealisticOrderTakerMockRaindex mockRaindex = new RealisticOrderTakerMockRaindex(100e18);
        vm.etch(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, address(mockRaindex).code);
        RealisticOrderTakerMockRaindex raindex =
            RealisticOrderTakerMockRaindex(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS);
        MockExchange exchange = new MockExchange();

        outputToken.mint(address(raindex), 100e18);
        inputToken.mint(address(exchange), 100e18);

        GenericPoolRaindexV6ArbOrderTaker arb = new GenericPoolRaindexV6ArbOrderTaker();

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

        bytes memory takeOrdersData = abi.encode(
            address(exchange),
            address(exchange),
            abi.encodeCall(MockExchange.swap, (IERC20(address(outputToken)), IERC20(address(inputToken)), 100e18))
        );

        arb.arb5(
            IRaindexV6(address(raindex)),
            TakeOrdersConfigV5({
                minimumIO: LibDecimalFloat.packLossless(100, 0),
                maximumIO: LibDecimalFloat.packLossless(type(int224).max, 0),
                maximumIORatio: LibDecimalFloat.packLossless(type(int224).max, 0),
                IOIsInput: true,
                orders: orders,
                data: takeOrdersData
            }),
            TaskV2({
                evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                signedContext: new SignedContextV1[](0)
            })
        );

        // Raindex received all inputToken from arb.
        assertEq(inputToken.balanceOf(address(raindex)), 100e18);
        // Exchange received all outputToken from arb.
        assertEq(outputToken.balanceOf(address(exchange)), 100e18);
        // Arb contract has no remaining tokens.
        assertEq(inputToken.balanceOf(address(arb)), 0);
        assertEq(outputToken.balanceOf(address(arb)), 0);
    }
}
