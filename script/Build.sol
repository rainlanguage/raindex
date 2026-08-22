// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {BuildScript} from "rain-deploy-0.1.7/src/abstract/BuildScript.sol";
import {LibRainDeploySnapshot} from "rain-deploy-0.1.7/src/lib/LibRainDeploySnapshot.sol";
import {LibRainDeploy} from "rain-deploy-0.1.7/src/lib/LibRainDeploy.sol";
import {LibCodeGen} from "rain-sol-codegen-0.1.36/src/lib/LibCodeGen.sol";
import {LibFs} from "rain-sol-codegen-0.1.36/src/lib/LibFs.sol";
import {LibGenParseMeta} from "rain-interpreter-interface-0.1.0/src/lib/codegen/LibGenParseMeta.sol";
import {DeployCandidate} from "../src/abstract/RainDeploySuitesBase.sol";
import {RaindexDeploySuites} from "../src/abstract/RaindexDeploySuites.sol";
import {RaindexV6SubParser} from "../src/concrete/parser/RaindexV6SubParser.sol";
import {LibRaindexSubParser, EXTERN_PARSE_META_BUILD_DEPTH} from "../src/lib/LibRaindexSubParser.sol";

/// One contract's generated snapshot and the released-suites lib emitted from
/// its record.
struct GeneratedContract {
    /// Places the snapshot inside `src/generated/<dir>/` and names the
    /// generated released-suites lib.
    string contractName;
    /// Snapshots are written from its `sourceCreationCode` and
    /// `snapshot.dependencies`; the released lib takes its suite key and
    /// artifact path from its `snapshot`.
    DeployCandidate candidate;
}

/// @title Build
/// @notice Generates the deploy pins for every contract this repo deploys.
/// `generatedContracts()` is the only list, read by every hook below.
///
/// `run()` (what CI regenerates against) rewrites the rolling
/// `src/generated/candidate/` snapshots, the sub-parser's non-deploy codegen
/// and the released-suites libs. `cutRelease()` freezes the candidates into
/// `src/generated/<tag>/` first. Frozen snapshots are append-only historical
/// records, never regenerated here.
///
/// Unlike the org's template deploy repos there are no generated alias libs:
/// `src/lib/deploy/LibRaindexDeploy.sol` is hand-written over the candidate
/// snapshots because its constant names are consumed across this repo and its
/// crates.
contract Build is BuildScript, RaindexDeploySuites {
    /// Every contract this repo generates deploy pins for.
    /// @return The generated contracts.
    function generatedContracts() internal pure returns (GeneratedContract[] memory) {
        GeneratedContract[] memory contracts = new GeneratedContract[](6);
        contracts[0] = GeneratedContract({contractName: "RaindexV6", candidate: raindexCandidate()});
        contracts[1] = GeneratedContract({contractName: "RaindexV6SubParser", candidate: subParserCandidate()});
        contracts[2] = GeneratedContract({contractName: "RouteProcessor4", candidate: routeProcessorCandidate()});
        contracts[3] = GeneratedContract({
            contractName: "GenericPoolRaindexV6ArbOrderTaker", candidate: genericPoolArbOrderTakerCandidate()
        });
        contracts[4] = GeneratedContract({
            contractName: "RouteProcessorRaindexV6ArbOrderTaker", candidate: routeProcessorArbOrderTakerCandidate()
        });
        contracts[5] = GeneratedContract({
            contractName: "GenericPoolRaindexV6FlashBorrower", candidate: genericPoolFlashBorrowerCandidate()
        });
        return contracts;
    }

    /// @inheritdoc BuildScript
    /// @dev In declaration order — the order the aggregate emits its entries
    /// in.
    function snapshotContractNames() internal pure override returns (string[] memory) {
        GeneratedContract[] memory contracts = generatedContracts();
        string[] memory names = new string[](contracts.length);
        for (uint256 i = 0; i < contracts.length; i++) {
            names[i] = contracts[i].contractName;
        }
        return names;
    }

    /// @inheritdoc BuildScript
    /// @dev Every released-suites lib and the aggregate over them. No alias
    /// libs: `LibRaindexDeploy` is hand-written.
    function regenerateLibs() internal override {
        GeneratedContract[] memory contracts = generatedContracts();
        for (uint256 i = 0; i < contracts.length; i++) {
            LibRainDeploySnapshot.writeReleasedSuitesLib(
                vm,
                LibRainDeploySnapshot.LIB_DIR,
                recordRoot(),
                contracts[i].contractName,
                contracts[i].candidate.snapshot
            );
        }
        LibRainDeploySnapshot.writeReleasedSuitesAggregate(vm, LibRainDeploySnapshot.LIB_DIR, snapshotContractNames());
    }

    /// @inheritdoc BuildScript
    function regenerateSnapshots() internal override {
        GeneratedContract[] memory contracts = generatedContracts();
        for (uint256 i = 0; i < contracts.length; i++) {
            LibRainDeploySnapshot.writeSnapshot(
                vm,
                LibRainDeploySnapshot.CANDIDATE,
                contracts[i].contractName,
                contracts[i].candidate.sourceCreationCode,
                contracts[i].candidate.snapshot.dependencies
            );
        }
        buildSubParserPointers();
    }

    /// The sub-parser's non-deploy codegen: parse meta, function pointers and
    /// described-by meta hash. `LibRainDeploySnapshot.writeSnapshot` only ever
    /// emits deploy constants, so these live in their own generated file,
    /// `src/generated/RaindexV6SubParserPointers.sol`, consumed by
    /// `RaindexV6SubParser` itself (the usual committed-codegen cycle).
    ///
    /// The name is distinct from the contract's so the file cannot be taken
    /// for a snapshot of `RaindexV6SubParser`, and so it survives
    /// `LibFs.requireNoOrphanedArtifact` beside the candidate record.
    ///
    /// The instance read here is the one the `writeSnapshot` loop in
    /// `regenerateSnapshots` already deployed via the Zoltu factory, reached
    /// again through its deterministic address — deploying twice through the
    /// factory would collide.
    function buildSubParserPointers() internal {
        RaindexV6SubParser subParser =
            RaindexV6SubParser(LibRainDeploy.zoltuAddress(type(RaindexV6SubParser).creationCode));

        LibFs.buildFileForContract(
            vm,
            address(subParser),
            "RaindexV6SubParserPointers",
            string.concat(
                LibCodeGen.describedByMetaHashConstantString(vm, "RaindexV6SubParser"),
                LibGenParseMeta.parseMetaConstantString(
                    vm, LibRaindexSubParser.authoringMetaV2(), EXTERN_PARSE_META_BUILD_DEPTH
                ),
                LibCodeGen.subParserWordParsersConstantString(vm, subParser),
                LibCodeGen.operandHandlerFunctionPointersConstantString(vm, subParser),
                LibCodeGen.literalParserFunctionPointersConstantString(vm, subParser)
            )
        );
    }
}
