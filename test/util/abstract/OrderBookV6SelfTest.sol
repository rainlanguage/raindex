// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";

import {REVERTING_MOCK_BYTECODE} from "test/util/lib/LibTestConstants.sol";

import {OrderBookV6} from "../../../src/concrete/ob/OrderBookV6.sol";

/// @title OrderBookV6SelfTest
/// Abstract contract that is an `OrderBookV6` and can be used to test itself.
/// Inherits from Test so that it can be used as a base contract for other tests.
/// Mocks all externalities during construction.
abstract contract OrderBookV6SelfTest is Test, OrderBookV6 {}
