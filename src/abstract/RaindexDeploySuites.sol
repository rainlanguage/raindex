// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {DeployCandidate, DeploySuite, RainDeploySuitesBase} from "./RainDeploySuitesBase.sol";
import {RaindexV6} from "../concrete/raindex/RaindexV6.sol";
import {RaindexV6SubParser} from "../concrete/parser/RaindexV6SubParser.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {GenericPoolRaindexV6FlashBorrower} from "../concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../lib/deploy/LibRouteProcessor4CreationCode.sol";
import {LibRaindexDeploy} from "../lib/deploy/LibRaindexDeploy.sol";
import {LibReleasedSuites} from "../lib/LibReleasedSuites.sol";
import {LibDecimalFloatDeploy} from "rain-math-float-0.1.1/src/lib/deploy/LibDecimalFloatDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";
import {LibMetaBoardDeploy} from "rain-metadata-deploy-0.1.1/src/lib/LibMetaBoardDeploy.sol";
import {
    CREATION_CODE as RAINDEX_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as RAINDEX_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/RaindexV6.sol";
import {
    CREATION_CODE as SUB_PARSER_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/RaindexV6SubParser.sol";
import {
    CREATION_CODE as ROUTE_PROCESSOR_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/RouteProcessor4.sol";
import {
    CREATION_CODE as GENERIC_POOL_ARB_OT_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as GENERIC_POOL_ARB_OT_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/GenericPoolRaindexV6ArbOrderTaker.sol";
import {
    CREATION_CODE as RP_ARB_OT_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as RP_ARB_OT_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {
    CREATION_CODE as GENERIC_POOL_FB_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/GenericPoolRaindexV6FlashBorrower.sol";

/// @title RaindexDeploySuites
/// @notice Everything this repo deploys, declared ONCE: the six hand-written
/// candidates below, and the released side read from the generated
/// `LibReleasedSuites`, which `script/Build.sol` emits from the frozen record.
///
/// The suite keys are the `Manual sol artifacts` dispatch choices and MUST
/// stay in step with `.github/workflows/manual-sol-artifacts.yaml`.
///
/// It lives in `src/` rather than `test/` because `.soldeerignore` excludes
/// `test/` from the published package, and the deployment process is part of
/// the product.
abstract contract RaindexDeploySuites is RainDeploySuitesBase {
    /// @inheritdoc RainDeploySuitesBase
    function releasedSuites() internal pure override returns (DeploySuite[] memory) {
        return LibReleasedSuites.releasedSuites();
    }

    /// @inheritdoc RainDeploySuitesBase
    function candidateSuites() internal pure override returns (DeployCandidate[] memory) {
        DeployCandidate[] memory candidates = new DeployCandidate[](6);
        candidates[0] = raindexCandidate();
        candidates[1] = subParserCandidate();
        candidates[2] = routeProcessorCandidate();
        candidates[3] = genericPoolArbOrderTakerCandidate();
        candidates[4] = routeProcessorArbOrderTakerCandidate();
        candidates[5] = genericPoolFlashBorrowerCandidate();
        return candidates;
    }

    /// The addresses that must already have code on a network before
    /// `RaindexV6` or `RaindexV6SubParser` can be broadcast there: the decimal
    /// float log tables, the TOFU decimals reader and the metaboard.
    /// @return The dependency addresses.
    function raindexDependencies() internal pure returns (address[] memory) {
        address[] memory deps = new address[](3);
        deps[0] = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS;
        deps[1] = address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT);
        deps[2] = LibMetaBoardDeploy.META_BOARD_DEPLOYED_ADDRESS;
        return deps;
    }

    /// This repo's rolling `RaindexV6` candidate. Each candidate is a named
    /// function rather than an index into `candidateSuites`, because
    /// `script/Build.sol` emits snapshots from these candidates specifically,
    /// and because six full creation codes built in one frame do not fit the
    /// legacy codegen's stack.
    /// @return The candidate.
    function raindexCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "raindex",
                creationCode: RAINDEX_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS,
                storedBytecodeHash: LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH,
                storedRuntimeCode: RAINDEX_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/concrete/raindex/RaindexV6.sol:RaindexV6",
                dependencies: raindexDependencies()
            }),
            sourceCreationCode: type(RaindexV6).creationCode
        });
    }

    /// This repo's rolling `RaindexV6SubParser` candidate.
    /// @return The candidate.
    function subParserCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "subparser",
                creationCode: SUB_PARSER_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS,
                storedBytecodeHash: LibRaindexDeploy.SUB_PARSER_DEPLOYED_CODEHASH,
                storedRuntimeCode: SUB_PARSER_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/concrete/parser/RaindexV6SubParser.sol:RaindexV6SubParser",
                dependencies: raindexDependencies()
            }),
            sourceCreationCode: type(RaindexV6SubParser).creationCode
        });
    }

    /// This repo's rolling `RouteProcessor4` candidate. There is no local
    /// source for RouteProcessor4 — its creation code is the pinned
    /// `ROUTE_PROCESSOR_4_CREATION_CODE` constant — so the source anchor
    /// compares the snapshot to that constant.
    /// @return The candidate.
    function routeProcessorCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "route-processor",
                creationCode: ROUTE_PROCESSOR_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS,
                storedBytecodeHash: LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH,
                storedRuntimeCode: ROUTE_PROCESSOR_RUNTIME_CODE_CANDIDATE,
                artifactPath: "RouteProcessor4",
                dependencies: new address[](0)
            }),
            sourceCreationCode: ROUTE_PROCESSOR_4_CREATION_CODE
        });
    }

    /// This repo's rolling `GenericPoolRaindexV6ArbOrderTaker` candidate.
    /// @return The candidate.
    function genericPoolArbOrderTakerCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "arb-generic-pool-order-taker",
                creationCode: GENERIC_POOL_ARB_OT_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS,
                storedBytecodeHash: LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH,
                storedRuntimeCode: GENERIC_POOL_ARB_OT_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol:GenericPoolRaindexV6ArbOrderTaker",
                dependencies: new address[](0)
            }),
            sourceCreationCode: type(GenericPoolRaindexV6ArbOrderTaker).creationCode
        });
    }

    /// This repo's rolling `RouteProcessorRaindexV6ArbOrderTaker` candidate.
    /// @return The candidate.
    function routeProcessorArbOrderTakerCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "arb-route-processor-order-taker",
                creationCode: RP_ARB_OT_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS,
                storedBytecodeHash: LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH,
                storedRuntimeCode: RP_ARB_OT_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol:RouteProcessorRaindexV6ArbOrderTaker",
                dependencies: new address[](0)
            }),
            sourceCreationCode: type(RouteProcessorRaindexV6ArbOrderTaker).creationCode
        });
    }

    /// This repo's rolling `GenericPoolRaindexV6FlashBorrower` candidate.
    /// @return The candidate.
    function genericPoolFlashBorrowerCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "arb-generic-pool-flash-borrower",
                creationCode: GENERIC_POOL_FB_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS,
                storedBytecodeHash: LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH,
                storedRuntimeCode: GENERIC_POOL_FB_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol:GenericPoolRaindexV6FlashBorrower",
                dependencies: new address[](0)
            }),
            sourceCreationCode: type(GenericPoolRaindexV6FlashBorrower).creationCode
        });
    }
}
