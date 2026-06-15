// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6, OrderIOCalculationV4} from "src/concrete/raindex/RaindexV6.sol";
import {ClearStateChangeV2} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {Float} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @dev Exposes RaindexV6's internal pure clear-state-change math so it can be
/// tested against a FRESH-COMPILED copy of the source (not the etched committed
/// bytecode the external mock/real suites run, which is blind to src/*.sol
/// mutations).
contract RaindexV6ClearStateChangeHarness is RaindexV6 {
    function exposedCalculateClearStateChange(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) external pure returns (ClearStateChangeV2 memory) {
        return calculateClearStateChange(aliceOrderIOCalculation, bobOrderIOCalculation);
    }

    function exposedCalculateClearStateAlice(
        OrderIOCalculationV4 memory aliceOrderIOCalculation,
        OrderIOCalculationV4 memory bobOrderIOCalculation
    ) external pure returns (Float aliceInput, Float aliceOutput) {
        return calculateClearStateAlice(aliceOrderIOCalculation, bobOrderIOCalculation);
    }
}
