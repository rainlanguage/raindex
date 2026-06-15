// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6FlashBorrower} from "../../../src/abstract/RaindexV6FlashBorrower.sol";

/// @dev Deployable child of the abstract flash borrower with a no-op `_exchange`
/// so the test exercises ONLY the abstract `arb4` decimals/finalize plumbing,
/// not any concrete swap.
contract NoExchangeFlashBorrower is RaindexV6FlashBorrower {
    constructor() {}
}
