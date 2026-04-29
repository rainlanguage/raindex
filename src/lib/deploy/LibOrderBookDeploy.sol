// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {
    BYTECODE_HASH as ORDERBOOK_HASH,
    DEPLOYED_ADDRESS as ORDERBOOK_ADDR
} from "../../generated/OrderBookV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR
} from "../../generated/OrderBookV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH,
    DEPLOYED_ADDRESS as ROUTE_PROCESSOR_ADDR
} from "../../generated/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_OT_HASH,
    DEPLOYED_ADDRESS as GENERIC_POOL_ARB_OT_ADDR
} from "../../generated/GenericPoolOrderBookV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_OT_HASH,
    DEPLOYED_ADDRESS as RP_ARB_OT_ADDR
} from "../../generated/RouteProcessorOrderBookV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH,
    DEPLOYED_ADDRESS as GENERIC_POOL_FB_ADDR
} from "../../generated/GenericPoolOrderBookV6FlashBorrower.pointers.sol";

/// @title LibOrderBookDeploy
/// @notice A library containing the deployed address and code hash of the
/// OrderBook contracts when deployed with the rain standard zoltu deployer.
/// This allows idempotent deployments against precommitted addresses and hashes
/// that can be easily verified automatically in tests and scripts rather than
/// relying on registries or manual verification.
library LibOrderBookDeploy {
    /// The address of the `OrderBookV6` contract when deployed with the rain
    /// standard zoltu deployer.
    address constant ORDERBOOK_DEPLOYED_ADDRESS = ORDERBOOK_ADDR;

    /// The code hash of the `OrderBookV6` contract when deployed with the rain
    /// standard zoltu deployer.
    bytes32 constant ORDERBOOK_DEPLOYED_CODEHASH = ORDERBOOK_HASH;

    /// The address of the `OrderBookV6SubParser` contract when deployed with
    /// the rain standard zoltu deployer.
    address constant SUB_PARSER_DEPLOYED_ADDRESS = SUB_PARSER_ADDR;

    /// The code hash of the `OrderBookV6SubParser` contract when deployed with
    /// the rain standard zoltu deployer.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH = SUB_PARSER_HASH;

    /// The address of the `RouteProcessor4` contract when deployed with the
    /// rain standard zoltu deployer.
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS = ROUTE_PROCESSOR_ADDR;

    /// The code hash of the `RouteProcessor4` contract when deployed with the
    /// rain standard zoltu deployer.
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH = ROUTE_PROCESSOR_HASH;

    /// The address of the `GenericPoolOrderBookV6ArbOrderTaker` contract when
    /// deployed with the rain standard zoltu deployer.
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS = GENERIC_POOL_ARB_OT_ADDR;

    /// The code hash of the `GenericPoolOrderBookV6ArbOrderTaker` contract when
    /// deployed with the rain standard zoltu deployer.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH = GENERIC_POOL_ARB_OT_HASH;

    /// The address of the `RouteProcessorOrderBookV6ArbOrderTaker` contract
    /// when deployed with the rain standard zoltu deployer.
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS = RP_ARB_OT_ADDR;

    /// The code hash of the `RouteProcessorOrderBookV6ArbOrderTaker` contract
    /// when deployed with the rain standard zoltu deployer.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH = RP_ARB_OT_HASH;

    /// The address of the `GenericPoolOrderBookV6FlashBorrower` contract when
    /// deployed with the rain standard zoltu deployer.
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS = GENERIC_POOL_FB_ADDR;

    /// The code hash of the `GenericPoolOrderBookV6FlashBorrower` contract when
    /// deployed with the rain standard zoltu deployer.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH = GENERIC_POOL_FB_HASH;

    uint256 constant ORDERBOOK_START_BLOCK_ARBITRUM = 441612693;
    uint256 constant ORDERBOOK_START_BLOCK_BASE = 43339885;
    uint256 constant ORDERBOOK_START_BLOCK_FLARE = 56972130;
    uint256 constant ORDERBOOK_START_BLOCK_POLYGON = 84174550;
}
