// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

import {GenericPoolRaindexV6FlashBorrower} from "../../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
import {BadLender} from "../../src/abstract/RaindexV6FlashBorrower.sol";
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
import {MockToken} from "test/util/concrete/MockToken.sol";
import {MaliciousRaindex} from "test/util/concrete/MaliciousRaindex.sol";

/// A malicious raindex passed to arb4 gets forceApprove before the
/// BadLender check in onFlashLoan fires. The entire tx reverts atomically,
/// so the approvals never persist.
contract RaindexV6FlashBorrowerBadLenderApprovalTest is Test {
    function testBadLenderRevertsWithApproval() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        MockToken inputToken = new MockToken("Input", "IN", 18);
        MockToken outputToken = new MockToken("Output", "OUT", 18);

        MaliciousRaindex malicious = new MaliciousRaindex();
        malicious.setTokens(address(inputToken), address(outputToken));

        GenericPoolRaindexV6FlashBorrower arb = new GenericPoolRaindexV6FlashBorrower();

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

        bytes memory exchangeData = abi.encode(address(0), address(0), "");

        // arb4 reverts with BadLender because the malicious raindex is not
        // the deterministic raindex address.
        vm.expectRevert(abi.encodeWithSelector(BadLender.selector, address(malicious)));
        arb.arb4(
            IRaindexV6(address(malicious)),
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

        // After the revert, no approvals persist.
        assertEq(inputToken.allowance(address(arb), address(malicious)), 0);
        assertEq(outputToken.allowance(address(arb), address(malicious)), 0);
    }
}
