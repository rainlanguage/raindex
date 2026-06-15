// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6FlashLender} from "../../../src/abstract/RaindexV6FlashLender.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildRaindexV6FlashLender is RaindexV6FlashLender {
    constructor() RaindexV6FlashLender() {}
}
