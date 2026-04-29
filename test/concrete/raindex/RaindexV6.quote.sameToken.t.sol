// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {QuoteV2, OrderConfigV4, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {TokenSelfTrade} from "../../../src/concrete/raindex/RaindexV6.sol";
import {Float} from "rain.math.float/lib/LibDecimalFloat.sol";

/// @title RaindexV6QuoteSameTokenTest
contract RaindexV6QuoteSameTokenTest is RaindexV6ExternalRealTest {
    /// Same token for input and output is error.
    /// forge-config: default.fuzz.runs = 10
    function testQuoteSameToken(QuoteV2 memory quoteConfig) external {
        vm.assume(quoteConfig.order.validInputs.length > 0);
        vm.assume(quoteConfig.order.validOutputs.length > 0);
        quoteConfig.order.validInputs[0].token = quoteConfig.order.validOutputs[0].token;
        quoteConfig.inputIOIndex = 0;
        quoteConfig.outputIOIndex = 0;
        vm.prank(quoteConfig.order.owner);
        iRaindex.addOrder4(
            OrderConfigV4({
                evaluable: quoteConfig.order.evaluable,
                validInputs: quoteConfig.order.validInputs,
                validOutputs: quoteConfig.order.validOutputs,
                nonce: quoteConfig.order.nonce,
                secret: bytes32(0),
                meta: ""
            }),
            new TaskV2[](0)
        );
        vm.expectRevert(abi.encodeWithSelector(TokenSelfTrade.selector));
        (bool success, Float maxOutput, Float ioRatio) = iRaindex.quote2(quoteConfig);
        (success, maxOutput, ioRatio);
    }
}
