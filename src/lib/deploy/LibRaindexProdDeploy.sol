// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {BYTECODE_HASH as RAINDEX_HASH, DEPLOYED_ADDRESS as RAINDEX_ADDR} from "../../generated/0_1_15/RaindexV6.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR
} from "../../generated/0_1_15/RaindexV6SubParser.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH,
    DEPLOYED_ADDRESS as ROUTE_PROCESSOR_ADDR
} from "../../generated/0_1_15/RouteProcessor4.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_OT_HASH,
    DEPLOYED_ADDRESS as GENERIC_POOL_ARB_OT_ADDR
} from "../../generated/0_1_15/GenericPoolRaindexV6ArbOrderTaker.sol";
import {
    BYTECODE_HASH as RP_ARB_OT_HASH,
    DEPLOYED_ADDRESS as RP_ARB_OT_ADDR
} from "../../generated/0_1_15/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH,
    DEPLOYED_ADDRESS as GENERIC_POOL_FB_ADDR
} from "../../generated/0_1_15/GenericPoolRaindexV6FlashBorrower.sol";

/// @title LibRaindexProdDeploy
/// @notice The Raindex contracts of the latest release, the one
/// `[external.package].version` names, aliased from its frozen snapshot, and
/// the blocks RaindexV6 was deployed at on each chain. `LibRaindexDeploy` is
/// the unreleased candidate.
library LibRaindexProdDeploy {
    address constant RAINDEX_DEPLOYED_ADDRESS = RAINDEX_ADDR;
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH = RAINDEX_HASH;

    address constant SUB_PARSER_DEPLOYED_ADDRESS = SUB_PARSER_ADDR;
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH = SUB_PARSER_HASH;

    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS = ROUTE_PROCESSOR_ADDR;
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH = ROUTE_PROCESSOR_HASH;

    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS = GENERIC_POOL_ARB_OT_ADDR;
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH = GENERIC_POOL_ARB_OT_HASH;

    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS = RP_ARB_OT_ADDR;
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH = RP_ARB_OT_HASH;

    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS = GENERIC_POOL_FB_ADDR;
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH = GENERIC_POOL_FB_HASH;

    uint256 constant RAINDEX_START_BLOCK_ARBITRUM = 473899359;
    uint256 constant RAINDEX_START_BLOCK_BASE = 47387582;
    uint256 constant RAINDEX_START_BLOCK_FLARE = 63011168;
    uint256 constant RAINDEX_START_BLOCK_POLYGON = 88573024;
}
