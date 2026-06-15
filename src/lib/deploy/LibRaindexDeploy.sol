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

    /// The deployed address of the `RaindexV6` contract at the published `0.1.4`
    /// tag. (Unchanged from `0.1.3` — 0.1.4 only changed the arb contracts.)
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_4 = 0x4F4527919DA654C1718Ca6Ef194F6547226685f6;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.4`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_4 =
        0xab8c608cd668d2c2bcef0188d012a818b5c909f82257ccb8cdb042715e394ec7;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.5`
    /// tag. (Changed in 0.1.5 — vault-0 balance lossy-pack.)
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_5 = 0xE0203890383Ca9C0c263f42a51C4C95211Cee6F9;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.5`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_5 =
        0x914f7b9b73aeaf4e01eaa4dbef7b85de27b7ac7254364a9276a23a88c14cf575;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.6`
    /// tag. (Changed in 0.1.6 — explicit take-vs-skip flag.)
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_6 = 0x025e93Be5c1780240C11E34B3387266754017D20;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.6`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_6 =
        0x04f376a4a874d67460721f50ff58f9d20548c5e00339648ab2cfeeff9a2caa07;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.7`
    /// tag.
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_7 = 0x86594ac4319230870C6E587f4ACa48FAb575619e;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.7`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_7 =
        0x774ca4c194abf0720bca6c033580762029a80291feefd0a4b7855b16bcbbf7a9;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.8`
    /// tag. (Unchanged from `0.1.7`.)
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_8 = 0x86594ac4319230870C6E587f4ACa48FAb575619e;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.8`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_8 =
        0x774ca4c194abf0720bca6c033580762029a80291feefd0a4b7855b16bcbbf7a9;

    /// The deployed address of the `RaindexV6` contract at the published `0.1.9`
    /// tag. (Unchanged from `0.1.7`.)
    address constant RAINDEX_DEPLOYED_ADDRESS_0_1_9 = 0x86594ac4319230870C6E587f4ACa48FAb575619e;

    /// The runtime code hash of the `RaindexV6` contract at the published `0.1.9`
    /// tag.
    bytes32 constant RAINDEX_DEPLOYED_CODEHASH_0_1_9 =
        0x774ca4c194abf0720bca6c033580762029a80291feefd0a4b7855b16bcbbf7a9;

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

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.4` tag. (Unchanged from `0.1.2`.)
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_4 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.4` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_4 =
        0x704aadc1ed56f63ff918ab219e6681a5d2851d774e2ee136bbe7904ea3b2fdcd;

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.5` tag. (Unchanged from `0.1.2`.)
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_5 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.5` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_5 =
        0x704aadc1ed56f63ff918ab219e6681a5d2851d774e2ee136bbe7904ea3b2fdcd;

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.6` tag. (Unchanged from `0.1.2`.)
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_6 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.6` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_6 =
        0x704aadc1ed56f63ff918ab219e6681a5d2851d774e2ee136bbe7904ea3b2fdcd;

    /// The deployed address of the `RaindexV6SubParser` contract at the published `0.1.7`
    /// tag.
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_7 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the published `0.1.7`
    /// tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_7 =
        0x704aadc1ed56f63ff918ab219e6681a5d2851d774e2ee136bbe7904ea3b2fdcd;

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.8` tag. (Unchanged from `0.1.2`.)
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_8 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.8` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_8 =
        0x704aadc1ed56f63ff918ab219e6681a5d2851d774e2ee136bbe7904ea3b2fdcd;

    /// The deployed address of the `RaindexV6SubParser` contract at the
    /// published `0.1.9` tag. (Unchanged from `0.1.2`.)
    address constant SUB_PARSER_DEPLOYED_ADDRESS_0_1_9 = 0x09Bc7AF266012F44fb41D8Bd682da931666605e1;

    /// The runtime code hash of the `RaindexV6SubParser` contract at the
    /// published `0.1.9` tag.
    bytes32 constant SUB_PARSER_DEPLOYED_CODEHASH_0_1_9 =
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

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.4` tag. (Unchanged from `0.1.0`.)
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_4 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.4` tag. (Unchanged from `0.1.0`.)
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_4 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.5` tag. (Unchanged from `0.1.0`.)
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_5 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.5` tag. (Unchanged from `0.1.0`.)
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_5 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.6` tag. (Unchanged from `0.1.0`.)
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_6 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.6` tag. (Unchanged from `0.1.0`.)
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_6 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The deployed address of the `RouteProcessor4` contract at the published `0.1.7`
    /// tag.
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_7 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published `0.1.7`
    /// tag.
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_7 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.8` tag. (Unchanged from `0.1.7`.)
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_8 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.8` tag.
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_8 =
        0xeb3745a79c6ba48e8767b9c355b8e7b79f9d6edeca004e4bb91be4de515a7eeb;

    /// The deployed address of the `RouteProcessor4` contract at the published
    /// `0.1.9` tag. (Unchanged from `0.1.7`.)
    address constant ROUTE_PROCESSOR_DEPLOYED_ADDRESS_0_1_9 = 0x6E2d0e71d900474b262E545Bc4C98b71ab368d21;

    /// The runtime code hash of the `RouteProcessor4` contract at the published
    /// `0.1.9` tag.
    bytes32 constant ROUTE_PROCESSOR_DEPLOYED_CODEHASH_0_1_9 =
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

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.4` tag. (Changed in 0.1.4 — finalizeArb gas
    /// saturation.)
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_4 = 0x2AC0EBB152a276f27cAC11F0f5651D40f7AE4497;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.4` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_4 =
        0x1ce682bf155f1ee1d84633085ee84dd6e9e2004b3c57146c00ed3ffeb22308ba;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.5` tag. (Changed in 0.1.5 — embeds the new raindex
    /// address.)
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_5 = 0x75F97855AE1A23e91592e529800f5DBF71B29CDC;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.5` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_5 =
        0x720eb6d5e4917400c0bcaabfeb78327349b82966fc19ca50ac8c0d57ac88c437;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.6` tag. (Changed in 0.1.6 — embeds the new raindex
    /// address.)
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_6 = 0xc42e74fCC95E8E31dC5F1112d7e6E63d807e161D;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.6` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_6 =
        0xcd45f8b958069f412eb5215f499e78c22952c54c18483fc5b1aac1f3b94f5d15;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract at the published `0.1.7`
    /// tag.
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_7 = 0xc0c17e58018C80A5420dF7756acd2B2dcf37e380;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract at the published `0.1.7`
    /// tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_7 =
        0xc4492cb22d918a3f0d41f13e533744ef03a2e7f744ec0a43b79d97a04c537544;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.8` tag. (Unchanged from `0.1.7`.)
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_8 = 0xc0c17e58018C80A5420dF7756acd2B2dcf37e380;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.8` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_8 =
        0xc4492cb22d918a3f0d41f13e533744ef03a2e7f744ec0a43b79d97a04c537544;

    /// The deployed address of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.9` tag. (Unchanged from `0.1.7`.)
    address constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_9 = 0xc0c17e58018C80A5420dF7756acd2B2dcf37e380;

    /// The runtime code hash of the `GenericPoolRaindexV6ArbOrderTaker` contract
    /// at the published `0.1.9` tag.
    bytes32 constant GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_9 =
        0xc4492cb22d918a3f0d41f13e533744ef03a2e7f744ec0a43b79d97a04c537544;

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

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.4` tag. (Changed in 0.1.4 — finalizeArb
    /// gas saturation.)
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_4 =
        0xe506D9A58a491dd97e059234F4bD59EE6E37205C;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.4` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_4 =
        0xc51fc032a5d09f78b346be609646fe186670f0379ae8a2cc39595c9a1554cacf;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.5` tag. (Changed in 0.1.5 — embeds the new
    /// raindex address.)
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_5 =
        0x7dD2B0970E1057DdDCe26af8664624943F2E9076;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.5` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_5 =
        0xc31da6992ee11e25a9914f4b5057453d0847011073f3ffaf4290ee33969ca8a2;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.6` tag. (Changed in 0.1.6 — embeds the new
    /// raindex address.)
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_6 =
        0x4F30B86FdECdc6Ba5a133539Ff6Ed09e331dE67F;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.6` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_6 =
        0x8faa06b004828e8203dfd0d6141c9516beb00c47341a968fed4785386bb4d037;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker` contract at the published `0.1.7`
    /// tag.
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_7 =
        0x5Ec2625ee7c73c85fAb2bc4cf527E1434F0dcC5d;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker` contract at the published `0.1.7`
    /// tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_7 =
        0xc3fb3bb0355fee22e54cade57f5ff828478d59f34fdd8057bc0bc6a11b3d48fc;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.8` tag. (Unchanged from `0.1.7`.)
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_8 =
        0x5Ec2625ee7c73c85fAb2bc4cf527E1434F0dcC5d;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.8` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_8 =
        0xc3fb3bb0355fee22e54cade57f5ff828478d59f34fdd8057bc0bc6a11b3d48fc;

    /// The deployed address of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.9` tag. (Unchanged from `0.1.7`.)
    address constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS_0_1_9 =
        0x5Ec2625ee7c73c85fAb2bc4cf527E1434F0dcC5d;

    /// The runtime code hash of the `RouteProcessorRaindexV6ArbOrderTaker`
    /// contract at the published `0.1.9` tag.
    bytes32 constant ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH_0_1_9 =
        0xc3fb3bb0355fee22e54cade57f5ff828478d59f34fdd8057bc0bc6a11b3d48fc;

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

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.4` tag. (Changed in 0.1.4 — finalizeArb gas
    /// saturation.)
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_4 = 0x6D5c861249B3A90541f0272cbec7478fD8d61E72;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.4` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_4 =
        0x76aadbcf811cec04d1aebb56ddcf79fa8583bb25142f7654f8cd4def35505313;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.5` tag. (Changed in 0.1.5 — embeds the new raindex
    /// address.)
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_5 = 0xCcD4453E8c75A5DDE28db756405A052a0CE90F32;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.5` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_5 =
        0xc291424b97016e53276bd4aa0300228ace7c9b183d854a16a1e3cf781cf298dc;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.6` tag. (Changed in 0.1.6 — embeds the new raindex
    /// address.)
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_6 = 0x6fD23E04bC13838716F9fc9940B59b234F6C8A07;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.6` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_6 =
        0xbc77410f937b2d874b7434958e968f5f182799695ebc5bb654b1f7569a1a3131;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract at the published `0.1.7`
    /// tag.
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_7 = 0xAd8af7f90a06659E9bBFD457c6C642fB60B1F46b;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract at the published `0.1.7`
    /// tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_7 =
        0xed618063fd6ffc7558e4588ea6357356e781d660fa22caa292a2f9aeaa21996f;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.8` tag. (Unchanged from `0.1.7`.)
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_8 = 0xAd8af7f90a06659E9bBFD457c6C642fB60B1F46b;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.8` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_8 =
        0xed618063fd6ffc7558e4588ea6357356e781d660fa22caa292a2f9aeaa21996f;

    /// The deployed address of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.9` tag. (Unchanged from `0.1.7`.)
    address constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS_0_1_9 = 0xAd8af7f90a06659E9bBFD457c6C642fB60B1F46b;

    /// The runtime code hash of the `GenericPoolRaindexV6FlashBorrower` contract
    /// at the published `0.1.9` tag.
    bytes32 constant GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH_0_1_9 =
        0xed618063fd6ffc7558e4588ea6357356e781d660fa22caa292a2f9aeaa21996f;

    uint256 constant RAINDEX_START_BLOCK_ARBITRUM = 473863630;
    uint256 constant RAINDEX_START_BLOCK_BASE = 47383088;
    uint256 constant RAINDEX_START_BLOCK_FLARE = 63004971;
    uint256 constant RAINDEX_START_BLOCK_POLYGON = 88567034;
}
