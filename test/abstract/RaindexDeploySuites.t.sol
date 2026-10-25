// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DeployCandidate, DeployDependency, DeploySuite} from "../../src/abstract/RainDeploySuitesBase.sol";
import {RaindexDeploySuites} from "../../src/abstract/RaindexDeploySuites.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.4/src/lib/LibTOFUTokenDecimals.sol";
import {LibMetaBoardDeploy} from "rain-metadata-deploy-0.1.3/src/lib/LibMetaBoardDeploy.sol";

/// @title RaindexDeploySuitesTest
/// @notice Properties of the deploy declaration that `rain-deploy` ships no
/// assertion for.
contract RaindexDeploySuitesTest is Test, RaindexDeploySuites {
    uint256 constant EIP170_MAX_RUNTIME_CODE_SIZE = 0x6000;

    /// PROPERTY: every candidate's runtime code fits EIP-170.
    function testCandidateRuntimeCodeFitsEip170() external view {
        DeployCandidate[] memory candidates = checkedCandidateSuites();
        for (uint256 i = 0; i < candidates.length; i++) {
            assertLe(
                candidates[i].snapshot.storedRuntimeCode.length,
                EIP170_MAX_RUNTIME_CODE_SIZE,
                string.concat("runtime code over EIP-170: ", candidates[i].snapshot.suite)
            );
        }
    }

    /// Every deployment a candidate's code could hard-code a call to, paired
    /// with the code hash that belongs at it.
    function knownDeployments() internal view returns (address[] memory, bytes32[] memory) {
        DeployCandidate[] memory candidates = checkedCandidateSuites();
        DeploySuite[] memory released = releasedSuites();
        uint256 length = candidates.length + released.length + 2;
        address[] memory addresses = new address[](length);
        bytes32[] memory codeHashes = new bytes32[](length);
        uint256 cursor = 0;
        for (uint256 i = 0; i < candidates.length; i++) {
            addresses[cursor] = candidates[i].snapshot.storedDeployedAddress;
            codeHashes[cursor] = candidates[i].snapshot.storedBytecodeHash;
            cursor++;
        }
        for (uint256 i = 0; i < released.length; i++) {
            addresses[cursor] = released[i].storedDeployedAddress;
            codeHashes[cursor] = released[i].storedBytecodeHash;
            cursor++;
        }
        addresses[cursor] = address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT);
        codeHashes[cursor] = LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CODE_HASH;
        cursor++;
        addresses[cursor] = LibMetaBoardDeploy.META_BOARD_DEPLOYED_ADDRESS;
        codeHashes[cursor] = LibMetaBoardDeploy.META_BOARD_DEPLOYED_CODEHASH;
        return (addresses, codeHashes);
    }

    /// Every operand of a PUSH1..PUSH20 in `code`, read as an address.
    function pushedAddresses(bytes memory code) internal pure returns (address[] memory) {
        address[] memory pushed = new address[](code.length);
        uint256 count = 0;
        uint256 i = 0;
        while (i < code.length) {
            uint8 op = uint8(code[i]);
            if (op >= 0x60 && op <= 0x7f) {
                uint256 size = op - 0x5f;
                if (size <= 20 && i + size < code.length) {
                    uint160 operand = 0;
                    for (uint256 j = 1; j <= size; j++) {
                        operand = (operand << 8) | uint160(uint8(code[i + j]));
                    }
                    pushed[count] = address(operand);
                    count++;
                }
                i += size + 1;
            } else {
                i++;
            }
        }
        assembly ("memory-safe") {
            mstore(pushed, count)
        }
        return pushed;
    }

    function contains(address[] memory list, address target) internal pure returns (bool) {
        for (uint256 i = 0; i < list.length; i++) {
            if (list[i] == target) {
                return true;
            }
        }
        return false;
    }

    function declares(DeployDependency[] memory dependencies, address target) internal pure returns (bool) {
        for (uint256 i = 0; i < dependencies.length; i++) {
            if (dependencies[i].deployedAddress == target) {
                return true;
            }
        }
        return false;
    }

    /// PROPERTY: each candidate declares exactly the known deployments its
    /// runtime code hard-codes, once each, with the code that belongs there.
    function testCandidateDependenciesAreTheDeploymentsItsCodeCalls() external view {
        (address[] memory addresses, bytes32[] memory codeHashes) = knownDeployments();
        DeployCandidate[] memory candidates = checkedCandidateSuites();
        for (uint256 i = 0; i < candidates.length; i++) {
            DeploySuite memory suite = candidates[i].snapshot;
            address[] memory pushed = pushedAddresses(suite.storedRuntimeCode);
            for (uint256 k = 0; k < addresses.length; k++) {
                if (addresses[k] == suite.storedDeployedAddress) {
                    continue;
                }
                assertEq(
                    declares(suite.dependencies, addresses[k]),
                    contains(pushed, addresses[k]),
                    string.concat(suite.suite, " declares iff its code calls ", vm.toString(addresses[k]))
                );
            }
            for (uint256 d = 0; d < suite.dependencies.length; d++) {
                DeployDependency memory dependency = suite.dependencies[d];
                bool known = false;
                for (uint256 k = 0; k < addresses.length; k++) {
                    if (addresses[k] == dependency.deployedAddress) {
                        known = true;
                        assertEq(
                            keccak256(dependency.runtimeCode),
                            codeHashes[k],
                            string.concat(suite.suite, " dependency code at ", vm.toString(addresses[k]))
                        );
                        break;
                    }
                }
                assertTrue(
                    known, string.concat(suite.suite, " unknown dependency ", vm.toString(dependency.deployedAddress))
                );
                for (uint256 e = d + 1; e < suite.dependencies.length; e++) {
                    assertTrue(
                        suite.dependencies[e].deployedAddress != dependency.deployedAddress,
                        string.concat(suite.suite, " duplicate dependency ", vm.toString(dependency.deployedAddress))
                    );
                }
            }
        }
    }
}
