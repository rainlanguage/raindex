// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";

/// @param task The task to run as post for each arb.
struct RaindexV6ArbConfig {
    TaskV2 task;
}

/// Thrown when the task does not match the expected hash.
error WrongTask();

/// @title RaindexV6ArbCommon
/// @notice Common base for arb contracts that interact with `Raindex`.
/// Provides a `_beforeArb` hook that is called at the start of every arb
/// operation. The base implementation is a no-op; task-gated variants
/// override it to validate the task hash.
abstract contract RaindexV6ArbCommon {
    /// @notice Emitted on construction with the full config.
    /// @param sender The deployer address.
    /// @param config The arb config used to construct this contract.
    event Construct(address sender, RaindexV6ArbConfig config);

    /// @dev Hook called at the start of every arb. Base implementation is a
    /// no-op. Overridden by `RaindexV6ArbTaskGated` to validate the task
    /// hash.
    function _beforeArb(TaskV2 memory task) internal virtual {}
}
