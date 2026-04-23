// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6SelfTest} from "test/util/abstract/OrderBookV6SelfTest.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {NegativePull, NegativePush} from "../../../src/concrete/ob/OrderBookV6.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";

/// Direct tests that `pullTokens` and `pushTokens` revert with `NegativePull`
/// and `NegativePush` when given a negative Float amount.
contract OrderBookV6NegativePullPushTest is OrderBookV6SelfTest {
    address internal token;

    /// External wrappers so vm.expectRevert can catch the internal revert.
    function externalPullTokens(address account, address token_, Float amount) external {
        pullTokens(account, token_, amount);
    }

    function externalPushTokens(address account, address token_, Float amount) external {
        pushTokens(account, token_, amount);
    }

    function setUp() external {
        token = address(uint160(uint256(keccak256("token.rain.test"))));
        vm.etch(token, REVERTING_MOCK_BYTECODE);
        vm.mockCall(token, abi.encodeWithSelector(IERC20Metadata.decimals.selector), abi.encode(18));
    }

    function testPullTokensNegativeAmountReverts() external {
        Float negativeAmount = LibDecimalFloat.packLossless(-1, 0);
        vm.expectRevert(abi.encodeWithSelector(NegativePull.selector));
        this.externalPullTokens(address(0xBEEF), token, negativeAmount);
    }

    function testPushTokensNegativeAmountReverts() external {
        Float negativeAmount = LibDecimalFloat.packLossless(-1, 0);
        vm.expectRevert(abi.encodeWithSelector(NegativePush.selector));
        this.externalPushTokens(address(0xBEEF), token, negativeAmount);
    }
}
