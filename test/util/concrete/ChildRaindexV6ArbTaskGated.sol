// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ArbTaskGated, RaindexV6ArbConfig} from "../../../src/abstract/RaindexV6ArbTaskGated.sol";
import {TaskV3} from "raindex-interface-0.1.13/src/interface/IRaindexV7.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract. Exposes `_checkTaskHash` as an external function.
contract ChildRaindexV6ArbTaskGated is RaindexV6ArbTaskGated {
    constructor(RaindexV6ArbConfig memory config) RaindexV6ArbTaskGated(config) {}

    function checkTaskHash(TaskV3 memory task) external pure {
        _checkTaskHash(task);
    }
}
