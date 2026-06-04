// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {OrderV4} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {OrderIOCalculationV4} from "../concrete/raindex/RaindexV6.sol";

/// @title LibTakeOrdersOverlay
/// @notice Threads calculate-phase state across same-owner orders within a
/// single `takeOrders` batch by reusing the most recent same-owner order's
/// returned kvs as the next order's `stateOverlay`.
library LibTakeOrdersOverlay {
    /// @dev The most recently evaluated same-owner order's calculate kvs, or an
    /// empty overlay if the owner has no earlier order in the batch. Each eval
    /// returns the full state KV (the seeded `stateOverlay` merged with new
    /// writes), so the latest same-owner kvs already carries every earlier
    /// same-owner write in the batch and is seeded directly with no
    /// concatenation. Not-yet-evaluated and skipped (dead) entries are
    /// zero-owner so they are ignored by the scan.
    /// @param evaluatedCalculations Each evaluated order's calculation, indexed
    /// by batch position.
    /// @param owner The owner namespace to seed writes for.
    /// @return The latest same-owner calculate kvs, or an empty array.
    function latestOwnerKvs(OrderIOCalculationV4[] memory evaluatedCalculations, address owner)
        internal
        pure
        returns (bytes32[] memory)
    {
        for (uint256 j = evaluatedCalculations.length; j > 0;) {
            unchecked {
                j--;
            }
            OrderIOCalculationV4 memory calculation = evaluatedCalculations[j];
            if (calculation.order.owner == owner) {
                return calculation.kvs;
            }
        }
        return new bytes32[](0);
    }
}
