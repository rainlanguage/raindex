// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script, console2} from "forge-std/Script.sol";
import {RaindexV6SubParser} from "../src/concrete/parser/RaindexV6SubParser.sol";
import {IMetaBoardV1_2} from "rain.metadata/interface/unstable/IMetaBoardV1_2.sol";
import {LibDescribedByMeta} from "rain.metadata/lib/LibDescribedByMeta.sol";
import {LibMetaBoardDeploy} from "rain.metadata/lib/deploy/LibMetaBoardDeploy.sol";
import {LibDecimalFloatDeploy} from "rain.math.float/lib/deploy/LibDecimalFloatDeploy.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "../src/lib/deploy/LibRaindexDeploy.sol";
import {CREATION_CODE as RAINDEX_CREATION_CODE} from "../src/generated/RaindexV6.pointers.sol";
import {CREATION_CODE as SUB_PARSER_CREATION_CODE} from "../src/generated/RaindexV6SubParser.pointers.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../src/lib/deploy/LibRouteProcessor4CreationCode.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {GenericPoolRaindexV6FlashBorrower} from "../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";

/// @dev Deploy only the RaindexV6 (raindex) contract.
bytes32 constant DEPLOYMENT_SUITE_RAINDEX = keccak256("raindex");
/// @dev Deploy only the RaindexV6SubParser contract.
bytes32 constant DEPLOYMENT_SUITE_SUBPARSER = keccak256("subparser");
/// @dev Deploy only the RouteProcessor4 contract.
bytes32 constant DEPLOYMENT_SUITE_ROUTE_PROCESSOR = keccak256("route-processor");
/// @dev Deploy only GenericPoolRaindexV6ArbOrderTaker.
bytes32 constant DEPLOYMENT_SUITE_ARB_GENERIC_POOL_ORDER_TAKER = keccak256("arb-generic-pool-order-taker");
/// @dev Deploy only RouteProcessorRaindexV6ArbOrderTaker.
bytes32 constant DEPLOYMENT_SUITE_ARB_ROUTE_PROCESSOR_ORDER_TAKER = keccak256("arb-route-processor-order-taker");
/// @dev Deploy only GenericPoolRaindexV6FlashBorrower.
bytes32 constant DEPLOYMENT_SUITE_ARB_GENERIC_POOL_FLASH_BORROWER = keccak256("arb-generic-pool-flash-borrower");

/// @title Deploy
/// @notice Foundry script that deploys raindex contracts. Controlled by the
/// `DEPLOYMENT_SUITE` env var to select which subset to deploy, and
/// `DEPLOYMENT_KEY` for the deployer private key.
contract Deploy is Script {
    /// @dev Tracks on-chain code hashes of dependencies per network, passed to
    /// `LibRainDeploy.deployAndBroadcast` to verify dependency integrity.
    mapping(string => mapping(address => bytes32)) internal sDepCodeHashes;

    /// @dev Entry point. Reads `DEPLOYMENT_KEY` and `DEPLOYMENT_SUITE` from env
    /// then deploys the selected contracts via `LibRainDeploy`.
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("DEPLOYMENT_KEY");
        string memory suiteString = vm.envString("DEPLOYMENT_SUITE");
        bytes32 suite = keccak256(bytes(suiteString));

        if (suite == DEPLOYMENT_SUITE_RAINDEX) {
            console2.log("Deploying RaindexV6...");
            address[] memory deps = new address[](3);
            deps[0] = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS;
            deps[1] = address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT);
            deps[2] = LibMetaBoardDeploy.METABOARD_DEPLOYED_ADDRESS;
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                RAINDEX_CREATION_CODE,
                "src/concrete/raindex/RaindexV6.sol:RaindexV6",
                LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS,
                LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
        } else if (suite == DEPLOYMENT_SUITE_SUBPARSER) {
            console2.log("Deploying RaindexV6SubParser...");
            address[] memory deps = new address[](3);
            deps[0] = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS;
            deps[1] = address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT);
            deps[2] = LibMetaBoardDeploy.METABOARD_DEPLOYED_ADDRESS;
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                SUB_PARSER_CREATION_CODE,
                "src/concrete/parser/RaindexV6SubParser.sol:RaindexV6SubParser",
                LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS,
                LibRaindexDeploy.SUB_PARSER_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
            IMetaBoardV1_2 metaboard = IMetaBoardV1_2(LibMetaBoardDeploy.METABOARD_DEPLOYED_ADDRESS);
            vm.startBroadcast(deployerPrivateKey);
            bytes memory subParserDescribedByMeta = vm.readFileBinary("meta/RaindexV6SubParser.rain.meta");
            LibDescribedByMeta.emitForDescribedAddress(
                metaboard, RaindexV6SubParser(LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS), subParserDescribedByMeta
            );
            vm.stopBroadcast();
        } else if (suite == DEPLOYMENT_SUITE_ROUTE_PROCESSOR) {
            console2.log("Deploying RouteProcessor4...");
            address[] memory deps = new address[](0);
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                ROUTE_PROCESSOR_4_CREATION_CODE,
                "RouteProcessor4",
                LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS,
                LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
        } else if (suite == DEPLOYMENT_SUITE_ARB_GENERIC_POOL_ORDER_TAKER) {
            console2.log("Deploying GenericPoolRaindexV6ArbOrderTaker...");
            address[] memory deps = new address[](0);
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                type(GenericPoolRaindexV6ArbOrderTaker).creationCode,
                "src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol:GenericPoolRaindexV6ArbOrderTaker",
                LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS,
                LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
        } else if (suite == DEPLOYMENT_SUITE_ARB_ROUTE_PROCESSOR_ORDER_TAKER) {
            console2.log("Deploying RouteProcessorRaindexV6ArbOrderTaker...");
            address[] memory deps = new address[](0);
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                type(RouteProcessorRaindexV6ArbOrderTaker).creationCode,
                "src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol:RouteProcessorRaindexV6ArbOrderTaker",
                LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS,
                LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
        } else if (suite == DEPLOYMENT_SUITE_ARB_GENERIC_POOL_FLASH_BORROWER) {
            console2.log("Deploying GenericPoolRaindexV6FlashBorrower...");
            address[] memory deps = new address[](0);
            LibRainDeploy.deployAndBroadcast(
                vm,
                LibRainDeploy.supportedNetworks(),
                deployerPrivateKey,
                type(GenericPoolRaindexV6FlashBorrower).creationCode,
                "src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol:GenericPoolRaindexV6FlashBorrower",
                LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS,
                LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH,
                deps,
                sDepCodeHashes
            );
        } else {
            revert("Unknown deployment suite");
        }
    }
}
