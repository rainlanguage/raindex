// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ArbOrderTaker} from "../../../src/abstract/RaindexV6ArbOrderTaker.sol";
import {RaindexV6ArbTaskGated, RaindexV6ArbConfig} from "../../../src/abstract/RaindexV6ArbTaskGated.sol";
import {TaskV2} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";

/// @dev A task-gated arb order taker: combines the `arb5` entrypoint from
/// `RaindexV6ArbOrderTaker` with the task-hash gating mixin
/// `RaindexV6ArbTaskGated`, wiring `_beforeArb` to `_checkTaskHash` (the
/// documented override mechanism). Used to test that `_beforeArb` runs before
/// the zero-orders check in `arb5`.
contract ChildGatedRaindexV6ArbOrderTaker is RaindexV6ArbOrderTaker, RaindexV6ArbTaskGated {
    constructor(RaindexV6ArbConfig memory config) RaindexV6ArbTaskGated(config) {}

    function _beforeArb(TaskV2 memory task) internal view override {
        _checkTaskHash(task);
    }
}
