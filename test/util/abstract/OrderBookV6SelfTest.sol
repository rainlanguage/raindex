// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";

import {OrderBookV6} from "../../../src/concrete/ob/OrderBookV6.sol";

/// @title OrderBookV6SelfTest
/// Abstract contract that is an `OrderBookV6` and can be used to test itself.
/// Inherits from Test so that it can be used as a base contract for other tests.
/// Deploys TOFU singleton so internal functions that touch token decimals work.
abstract contract OrderBookV6SelfTest is Test, OrderBookV6 {
    constructor() {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);
    }
}
