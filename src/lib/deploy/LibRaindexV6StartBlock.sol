// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

/// @title LibRaindexV6StartBlock
/// @notice The block the newest released RaindexV6 was deployed at on each
/// chain the subgraph indexes.
library LibRaindexV6StartBlock {
    uint256 constant RAINDEX_START_BLOCK_ARBITRUM = 473899359;
    uint256 constant RAINDEX_START_BLOCK_BASE = 47387582;
    uint256 constant RAINDEX_START_BLOCK_FLARE = 63011168;
    uint256 constant RAINDEX_START_BLOCK_POLYGON = 88573024;
}
