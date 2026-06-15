// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6} from "src/concrete/raindex/RaindexV6.sol";

/// @dev A RaindexV6 harness etched from freshly-compiled runtime code so the
/// external deposit/withdraw paths exercise the SOURCE under test (the committed
/// `src/generated/RaindexV6.pointers.sol` bytecode that the mock-based suites
/// etch is regenerated separately, so source edits are invisible to it).
contract RaindexV6DepositWithdrawHarness is RaindexV6 {}
