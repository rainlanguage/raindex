// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {GenericPoolOrderBookV6FlashBorrower} from "../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
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
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibOrderBookDeploy} from "../../src/lib/deploy/LibOrderBookDeploy.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {FlashLendingMockOrderBook} from "test/util/concrete/FlashLendingMockOrderBook.sol";
import {ReentrantExchange} from "test/util/concrete/ReentrantExchange.sol";

contract OrderBookV6FlashBorrowerReentrancyTest is Test {
    /// arb4 MUST revert when re-entered via the pool call in _exchange.
    function testArb4Reentrancy() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);

        // The flash borrower checks msg.sender == ORDERBOOK_DEPLOYED_ADDRESS in
        // onFlashLoan. Deploy a FlashLendingMockOrderBook and etch it there.
        FlashLendingMockOrderBook orderBook = new FlashLendingMockOrderBook();
        vm.etch(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, address(orderBook).code);

        // Mint output tokens to the etched OB so it can flash-lend them.
        outputToken.mint(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, 100e18);

        GenericPoolOrderBookV6FlashBorrower arb = new GenericPoolOrderBookV6FlashBorrower();

        // Deploy the reentrant exchange that will call arb4 when invoked.
        ReentrantExchange exchange =
            new ReentrantExchange(arb, IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS));

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

        // exchangeData: spender=exchange, pool=exchange, callData=empty (triggers fallback).
        bytes memory exchangeData = abi.encode(address(exchange), address(exchange), hex"");

        vm.expectRevert(abi.encodeWithSelector(bytes4(keccak256("ReentrancyGuardReentrantCall()"))));
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
            exchangeData,
            TaskV2({
                evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                signedContext: new SignedContextV1[](0)
            })
        );
    }
}
