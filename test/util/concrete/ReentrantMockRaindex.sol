// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    TaskV2,
    EvaluableV4,
    SignedContextV1,
    Float
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IInterpreterV4} from "rain.interpreter.interface/interface/IInterpreterV4.sol";
import {IInterpreterStoreV3} from "rain.interpreter.interface/interface/IInterpreterStoreV3.sol";
import {MockRaindexBase} from "test/util/abstract/MockRaindexBase.sol";

/// @dev Mock Raindex whose takeOrders4 callback re-enters arb5 on the taker.
contract ReentrantMockRaindex is MockRaindexBase {
    using SafeERC20 for IERC20;

    bool internal sReentered;

    function takeOrders4(TakeOrdersConfigV5 calldata config) external override returns (Float, Float) {
        address ordersOutputToken = config.orders[0].order.validOutputs[config.orders[0].outputIOIndex].token;

        // Send output token to taker so callback has tokens to work with.
        uint256 bal = IERC20(ordersOutputToken).balanceOf(address(this));
        IERC20(ordersOutputToken).safeTransfer(msg.sender, bal);

        if (!sReentered) {
            sReentered = true;
            // Re-enter arb5 from the Raindex callback.
            GenericPoolRaindexV6ArbOrderTaker(payable(msg.sender))
                .arb5(
                    IRaindexV6(address(this)),
                    config,
                    TaskV2({
                    evaluable: EvaluableV4(IInterpreterV4(address(0)), IInterpreterStoreV3(address(0)), hex""),
                    signedContext: new SignedContextV1[](0)
                })
                );
        }

        return (Float.wrap(0), Float.wrap(0));
    }
}
