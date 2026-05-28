// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {TOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/concrete/TOFUTokenDecimals.sol";

/// @dev Forces the concrete `TOFUTokenDecimals` from the rain-tofu-erc20-decimals
/// Soldeer dependency into raindex's compilation set so `forge build` emits
/// out/TOFUTokenDecimals.sol/TOFUTokenDecimals.json. The Rust test fixtures load
/// that artifact's deployed bytecode to mock the token on a local EVM. Referencing
/// the creation code keeps the import used and gives this file its own artifact.
library TOFUTokenDecimalsArtifact {
    function creationCode() internal pure returns (bytes memory) {
        return type(TOFUTokenDecimals).creationCode;
    }
}
