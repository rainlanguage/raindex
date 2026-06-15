// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6, OrderIOCalculationV4} from "src/concrete/raindex/RaindexV6.sol";
import {Float} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {OrderV4, SignedContextV1} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {TOFUOutcome} from "rain-tofu-erc20-decimals-0.1.1/src/interface/ITOFUTokenDecimals.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";

/// @dev Exposes RaindexV6's internal `calculateOrderIO` and seeds the TOFU
/// decimals store so a fresh-compiled RaindexV6 (etched runtime code) can be
/// mutation-tested for the calculate-context construction, the output-max vault
/// cap, the calculations pre-fill, the MIN_OUTPUTS guard, the (IORatio,
/// outputMax) read-back, and the per-token TOFU decimals checks.
contract RaindexV6CalculateOrderIOHarness is RaindexV6 {
    function exposedCalculateOrderIO(
        OrderV4 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        address counterparty,
        SignedContextV1[] memory signedContext
    ) external view returns (OrderIOCalculationV4 memory) {
        return calculateOrderIO(order, inputIOIndex, outputIOIndex, counterparty, signedContext, new bytes32[](0));
    }

    /// Persist the token's decimals into the TOFU store via the mutating read,
    /// so a later `decimalsForTokenReadOnly` can detect inconsistency.
    function seedTofu(address token) external returns (TOFUOutcome, uint8) {
        return LibTOFUTokenDecimals.decimalsForToken(token);
    }

    /// Credit a non-zero vault balance so the output-max vault cap has a finite
    /// balance to clamp against.
    function exposedIncrease(address owner, address token, bytes32 vaultId, Float amount) external {
        increaseVaultBalance(owner, token, vaultId, amount);
    }
}
