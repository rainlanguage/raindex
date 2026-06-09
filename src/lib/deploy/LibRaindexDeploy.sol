// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {BYTECODE_HASH as RAINDEX_HASH, DEPLOYED_ADDRESS as RAINDEX_ADDR} from "../../generated/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR
} from "../../generated/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH,
    DEPLOYED_ADDRESS as ROUTE_PROCESSOR_ADDR
} from "../../generated/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_OT_HASH,
    DEPLOYED_ADDRESS as GENERIC_POOL_ARB_OT_ADDR
} from "../../generated/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_OT_HASH,
    DEPLOYED_ADDRESS as RP_ARB_OT_ADDR
} from "../../generated/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH,
    DEPLOYED_ADDRESS as GENERIC_POOL_FB_ADDR
} from "../../generated/GenericPoolRaindexV6FlashBorrower.pointers.sol";

/// @title LibRaindexDeploy
/// @notice A library containing the deployed address and code hash of the
/// Raindex contracts when deployed with the rain standard zoltu deployer.
/// This allows idempotent deployments against precommitted addresses and hashes
/// that can be easily verified automatically in tests and scripts rather than
/// relying on registries or manual verification.
library LibRaindexDeploy {
    /// The address of the `RaindexV6` contract when deployed with the rain
    /// standard zoltu deployer.
    address constant RAINDEX_DEPLOYED_ADDRESS = RAINDEX_ADDR;

    /// The code hash of the `RaindexV6` contract when deployed with the rain
    /// standard zoltu deployer.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH = RAINDEX_HASH;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.0`
    /// tag. Pinned as a literal (not derived from the current pointers) so it
    /// keeps referencing the `0.1.0` deployment after `RAINDEX_DEPLOYED_ADDRESS`
    /// above advances to a newer version.
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_0 = 0xb05D73E6BCc26AEB5b67Ff68C6E9C6151073e3cE;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.0`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_0 =
        0xf5d6441f79933283777975573d385f5fc48c71f50ec85ef900dc947995e2f33d;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.2`
    /// tag.
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_2 = 0x4F4527919DA654C1718Ca6Ef194F6547226685f6;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.2`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_2 =
        0xab8c608cd668d2c2bcef0188d012a818b5c909f82257ccb8cdb042715e394ec7;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.3`
    /// tag. (Unchanged from `0.1.2` — that release only pinned 0.1.2's own
    /// constants, no bytecode change.)
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_3 = 0x4F4527919DA654C1718Ca6Ef194F6547226685f6;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.3`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_3 =
        0xab8c608cd668d2c2bcef0188d012a818b5c909f82257ccb8cdb042715e394ec7;

    /// The address of the `RaindexV6SubParser` contract when deployed with
    /// the rain standard zoltu deployer.
    address constant SUB_PARSER_DEPLOYED_ADDRESS = SUB_PARSER_ADDR;

    /// The code hash of the `RaindexV6SubParser` contract when deployed with
    /// the rain standard zoltu deployer.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH = SUB_PARSER_HASH;

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.0` tag.
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_0 = 0xBdb9cC5FF04A2bA28a4B0369d5592c469B9dFF53;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.0` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_0 =
        0x59e436b1a58c24ce88745ff6a49800ae74b98ac6c1f78032c509ec926de32140;

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.2` tag.
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_2 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.2` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_2 =
        0x704aadc1ed56f63ff918ab219e6681a5d2851d774e2ee136bbe7904ea3b2fdcd;

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.3` tag. (Unchanged from `0.1.2`.)
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_3 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.3` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_3 =
        0x704aadc1ed56f63ff918ab219e6681a5d2851d774e2ee136bbe7904ea3b2fdcd;

    /// The address of the `RouteProcessor4` contract when deployed with the
    /// rain standard zoltu deployer.
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS = ROUTE_PROCESSOR_ADDR;

    /// The code hash of the `RouteProcessor4` contract when deployed with the
    /// rain standard zoltu deployer.
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH = ROUTE_PROCESSOR_HASH;

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.0` tag.
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_0 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.0` tag.
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_0 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.2` tag. (Unchanged from `0.1.0`.)
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_2 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.2` tag. (Unchanged from `0.1.0`.)
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_2 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.3` tag. (Unchanged from `0.1.0`.)
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_3 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.3` tag. (Unchanged from `0.1.0`.)
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_3 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The address of the `GenericPoolRaindexV6ArbOrderTaker` contract when
    /// deployed with the rain standard zoltu deployer.
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS = GENERIC_POOL_ARB_OT_ADDR;

    /// The code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract when
    /// deployed with the rain standard zoltu deployer.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH = GENERIC_POOL_ARB_OT_HASH;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.0` tag.
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_0 = 0xA7299e32D8B89E064211FCDdE21A69cd0d54D0eb;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.0` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_0 =
        0xba5ce714baf93e1405a4e93d4ace1717b8ff04ee4fcfbfcff6ada6210928397c;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.2` tag.
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_2 = 0xdD8a5075476dbE40EDaB9b711133828Fdc74c972;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.2` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_2 =
        0xb3bb9236040b5fd7354c56923dd6ddf790e8fa86638d8091abc5e1958227bf4f;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.3` tag. (Unchanged from `0.1.2`.)
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_3 = 0xdD8a5075476dbE40EDaB9b711133828Fdc74c972;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.3` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_3 =
        0xb3bb9236040b5fd7354c56923dd6ddf790e8fa86638d8091abc5e1958227bf4f;

    /// The address of the `RouteProcessorRaindexV6ArbOrderTaker` contract
    /// when deployed with the rain standard zoltu deployer.
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS = RP_ARB_OT_ADDR;

    /// The code hash of the `RouteProcessorRaindexV6ArbOrderTaker` contract
    /// when deployed with the rain standard zoltu deployer.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH = RP_ARB_OT_HASH;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.0` tag.
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_0 =
        0x3dbef2Fb2ADFE64048Aa7E177F1645Ce7Bf5216d;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.0` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_0 =
        0x728605dabf8d0f4235999e961f288b6e05765c8c210a302e81f1b4fea2754bc8;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.2` tag.
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_2 =
        0xaaA2266C98DD65e51f90079f3Dd42464f8A8BaD5;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.2` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_2 =
        0x8181a5b98afff1113b831d4dc7f912e3e4a24bd440526b1350dbd7111df7284c;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.3` tag. (Unchanged from `0.1.2`.)
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_3 =
        0xaaA2266C98DD65e51f90079f3Dd42464f8A8BaD5;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.3` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_3 =
        0x8181a5b98afff1113b831d4dc7f912e3e4a24bd440526b1350dbd7111df7284c;

    /// The address of the `GenericPoolRaindexV6FlashBorrower` contract when
    /// deployed with the rain standard zoltu deployer.
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS = GENERIC_POOL_FB_ADDR;

    /// The code hash of the `GenericPoolRaindexV6FlashBorrower` contract when
    /// deployed with the rain standard zoltu deployer.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH = GENERIC_POOL_FB_HASH;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.0` tag.
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_0 = 0x8bdEd5aC59070533ec538a950e38B1a6F6cFA36c;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.0` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_0 =
        0xa1bf69e21c50d0f6c30fb102d78c768de58f733ca86ee745560aff7069c155c2;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.2` tag.
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_2 = 0x3A60DEa68c69B9C2338F12235fd87320D17bCF6a;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.2` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_2 =
        0xe887c3af02ae0170b43a7c70a291032f31b7709ec1088e66119cd1253600c0f8;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.3` tag. (Unchanged from `0.1.2`.)
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_3 = 0x3A60DEa68c69B9C2338F12235fd87320D17bCF6a;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.3` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_3 =
        0xe887c3af02ae0170b43a7c70a291032f31b7709ec1088e66119cd1253600c0f8;

    uint256 constant RAINDEX_START_BLOCK_ARBITRUM = 469964122;
    uint256 constant RAINDEX_START_BLOCK_BASE = 46893385;
    uint256 constant RAINDEX_START_BLOCK_FLARE = 62235114;
    uint256 constant RAINDEX_START_BLOCK_POLYGON = 87914243;
}
