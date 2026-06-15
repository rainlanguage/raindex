// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {MockRaindexBase} from "test/util/abstract/MockRaindexBase.sol";

/// @dev Mock raindex that returns false from flashLoan (inherits default stub).
contract FalseFlashLoanMockRaindex is MockRaindexBase {}
