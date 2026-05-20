// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ArbOrderTaker} from "../../../src/abstract/RaindexV6ArbOrderTaker.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildRaindexV6ArbOrderTaker is RaindexV6ArbOrderTaker {
    constructor() {}
}
