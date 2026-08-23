// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.2/src/Test.sol";
import {GeneratedContract} from "script/Build.sol";
import {DeployCandidate} from "src/abstract/RainDeploySuitesBase.sol";
import {BuildHarness} from "test/util/concrete/BuildHarness.sol";

/// @title BuildTest
/// @notice `script/Build.sol`'s own declaration.
///
/// `generatedContracts()` is the one list every hook inside `Build` reads, so
/// the regeneration, the released-lib writer and the freeze cannot disagree
/// with each other. The boundary is where they can: what this repo deploys is
/// declared in `RaindexDeploySuites.candidateSuites()`, and the generator's
/// list is a second hard-coded array in a second file.
///
/// A candidate dropped from `generatedContracts()` by a refactor or a merge
/// still compiles, because its committed snapshot is still there and still
/// imported. `cutRelease()` then freezes only the contracts the generator
/// names, `regenerateLibs()` writes a released-suites lib only for those, and
/// the release permanently omits that contract. Nothing downstream can see it:
/// `testEveryFrozenSnapshotIsReleased` walks record -> declaration, so a record
/// entry never written is invisible to it, and `RaindexDeployChainTest` is
/// never handed the omitted suite. The frozen tag then cannot be re-cut, so the
/// hole is permanent.
///
/// Deliberately nothing here calls `run()` or `cutRelease()`. Both rewrite the
/// committed `src/generated/` snapshots and `src/lib/` libs that other test
/// contracts read, and forge runs test contracts in parallel — a contract
/// rewriting what another one is reading is a race, not a check. Nothing below
/// writes anything.
///
/// There is no constant-prefix property here, unlike the org's template deploy
/// repos: this repo emits no generated alias libs, so `GeneratedContract` has
/// no prefix to collide.
contract BuildTest is Test {
    /// The harness the two declarations are read through.
    BuildHarness internal sBuild;

    function setUp() external {
        sBuild = new BuildHarness();
    }

    /// PROPERTY: the generator's list and the deploy declaration are the SAME
    /// SET of contracts, matched both ways. A candidate the generator does not
    /// name is a contract silently absent from every release cut from here, and
    /// neither the frozen-record check nor the chain check can see it, because
    /// both are only ever handed what was written.
    ///
    /// Matched by the suite key rather than by index, because a positional pass
    /// would go green on a reordered list that had silently swapped two
    /// contracts. The creation code is compared on the matched pair for the
    /// same reason: a generated entry can name the right key while carrying
    /// another contract's candidate, which writes one contract's pins into the
    /// other's snapshot file.
    function testGeneratedContractsAreExactlyTheDeclaredCandidates() external view {
        GeneratedContract[] memory generated = sBuild.externalGeneratedContracts();
        DeployCandidate[] memory candidates = sBuild.externalCandidateSuites();

        assertEq(
            generated.length, candidates.length, "a candidate is not generated, or a generated contract is not declared"
        );

        for (uint256 i = 0; i < generated.length; i++) {
            bool found = false;
            for (uint256 j = 0; j < candidates.length; j++) {
                if (
                    keccak256(bytes(generated[i].candidate.snapshot.suite))
                        == keccak256(bytes(candidates[j].snapshot.suite))
                ) {
                    assertEq(
                        keccak256(generated[i].candidate.snapshot.creationCode),
                        keccak256(candidates[j].snapshot.creationCode),
                        "generated entry carries another contract's creation code"
                    );
                    found = true;
                    break;
                }
            }
            assertTrue(
                found, string.concat("generated contract is not a declared candidate: ", generated[i].contractName)
            );
        }

        for (uint256 j = 0; j < candidates.length; j++) {
            bool found = false;
            for (uint256 i = 0; i < generated.length; i++) {
                if (
                    keccak256(bytes(generated[i].candidate.snapshot.suite))
                        == keccak256(bytes(candidates[j].snapshot.suite))
                ) {
                    found = true;
                    break;
                }
            }
            assertTrue(found, string.concat("declared candidate is not generated: ", candidates[j].snapshot.suite));
        }
    }

    /// PROPERTY: `snapshotContractNames()` is every `generatedContracts()`
    /// entry's `contractName`, positionally.
    ///
    /// It is the list `cutRelease()` freezes and the list the released-suites
    /// aggregate is emitted from, and both reach it only through `forge
    /// script`. A name list shorter than the declaration freezes one contract
    /// fewer and emits an aggregate that declares that contract's releases as
    /// nothing at all, while the test above stays green: it reads
    /// `generatedContracts()`, which this list does not pass through.
    ///
    /// Positional rather than as a set, because the aggregate documents its
    /// entries as being in declaration order and nothing else pins that.
    function testSnapshotContractNamesAreTheDeclarationInOrder() external view {
        GeneratedContract[] memory generated = sBuild.externalGeneratedContracts();
        string[] memory names = sBuild.externalSnapshotContractNames();

        assertEq(names.length, generated.length, "a different number of names than generated contracts");

        for (uint256 i = 0; i < generated.length; i++) {
            assertEq(names[i], generated[i].contractName, "the names are not the declaration in order");
        }
    }

    /// PROPERTY: no two entries name one contract. Two entries sharing a
    /// `contractName` write one snapshot file twice and freeze one file for two
    /// declarations, so the second silently replaces the first.
    function testGeneratedContractNamesAreUnique() external view {
        GeneratedContract[] memory generated = sBuild.externalGeneratedContracts();
        for (uint256 i = 0; i < generated.length; i++) {
            for (uint256 j = i + 1; j < generated.length; j++) {
                assertNotEq(
                    generated[i].contractName, generated[j].contractName, "two generated entries name one contract"
                );
            }
        }
    }

    /// PROPERTY: `contractName` is the name the snapshot path and the generated
    /// released lib are built from, and it MUST be the contract the candidate's
    /// artifact path names. A disagreement writes one contract's pins into
    /// another contract's file, which every downstream check then reads as
    /// self-consistent.
    ///
    /// The artifact path of a suite with no local Solidity source is the bare
    /// contract name, which splits to itself, so the same comparison holds for
    /// it without an exemption.
    function testGeneratedContractNameMatchesTheArtifactPath() external view {
        GeneratedContract[] memory generated = sBuild.externalGeneratedContracts();
        for (uint256 i = 0; i < generated.length; i++) {
            string[] memory components = vm.split(generated[i].candidate.snapshot.artifactPath, ":");
            assertEq(
                components[components.length - 1],
                generated[i].contractName,
                "generated name is not the artifact's contract"
            );
        }
    }
}
