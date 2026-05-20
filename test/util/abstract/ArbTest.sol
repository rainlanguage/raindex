// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {Refundoor} from "test/util/concrete/Refundoor.sol";
import {
    FlashLendingMockRaindex,
    OrderV4,
    TakeOrderConfigV4,
    IOV2,
    SignedContextV1,
    EvaluableV4
} from "test/util/concrete/FlashLendingMockRaindex.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";

contract Token is ERC20 {
    constructor() ERC20("Token", "TKN") {}

    function mint(address receiver, uint256 amount) external {
        _mint(receiver, amount);
    }
}

abstract contract ArbTest is Test {
    Token immutable iTakerInput;
    Token immutable iTakerOutput;
    address immutable iRefundoor;
    FlashLendingMockRaindex immutable iRaindex;
    address payable immutable iArb;

    function buildArb() internal virtual returns (address payable);

    constructor() {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        iTakerInput = new Token();
        vm.label(address(iTakerInput), "iTakerInput");
        iTakerOutput = new Token();
        vm.label(address(iTakerOutput), "iTakerOutput");
        iRefundoor = address(new Refundoor());
        vm.label(iRefundoor, "iRefundoor");
        // Deploy the mock then etch its code at the deterministic raindex
        // address so that onFlashLoan's BadLender check passes.
        FlashLendingMockRaindex mockRaindex = new FlashLendingMockRaindex();
        vm.etch(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS, address(mockRaindex).code);
        iRaindex = FlashLendingMockRaindex(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS);
        vm.label(address(iRaindex), "iRaindex");

        iArb = buildArb();
        vm.label(iArb, "iArb");
    }

    function buildTakeOrderConfig(OrderV4 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        internal
        view
        returns (TakeOrderConfigV4[] memory)
    {
        if (order.validInputs.length == 0) {
            order.validInputs = new IOV2[](1);
        }
        if (order.validOutputs.length == 0) {
            order.validOutputs = new IOV2[](1);
        }
        inputIOIndex = bound(inputIOIndex, 0, order.validInputs.length - 1);
        outputIOIndex = bound(outputIOIndex, 0, order.validOutputs.length - 1);

        order.validInputs[inputIOIndex].token = address(iTakerOutput);
        order.validOutputs[outputIOIndex].token = address(iTakerInput);

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4(order, inputIOIndex, outputIOIndex, new SignedContextV1[](0));
        return orders;
    }

    // Allow receiving funds at end of arb.
    fallback() external {}
}
