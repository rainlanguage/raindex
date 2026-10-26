// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RainDeployVerifyDeclaration} from "rain-deploy-0.1.25/src/abstract/RainDeployVerifyDeclaration.sol";
import {ExternalDeployment} from "rain-deploy-0.1.25/src/abstract/RainDeployDeclarationChecks.sol";
import {RaindexDeploySuites} from "../../src/abstract/RaindexDeploySuites.sol";
import {RaindexV6SubParser} from "../../src/concrete/parser/RaindexV6SubParser.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {GenericPoolRaindexV6FlashBorrower} from "../../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.4/src/lib/LibTOFUTokenDecimals.sol";
import {LibMetaBoardDeploy} from "rain-metadata-deploy-0.1.3/src/lib/LibMetaBoardDeploy.sol";

/// @title RaindexDeployDeclarationTest
/// @notice The suite fields only a dispatch, a verifier or a broadcast reads.
contract RaindexDeployDeclarationTest is RaindexDeploySuites, RainDeployVerifyDeclaration {
    function sourceCreationCode(string memory suite) internal view override returns (bytes memory) {
        bytes32 key = keccak256(bytes(suite));
        // `RaindexV6` is via-IR only; importing it here would pull this test under that restriction.
        if (key == keccak256("raindex")) return vm.getCode("src/concrete/raindex/RaindexV6.sol:RaindexV6");
        if (key == keccak256("subparser")) return type(RaindexV6SubParser).creationCode;
        if (key == keccak256("arb-generic-pool-order-taker")) {
            return type(GenericPoolRaindexV6ArbOrderTaker).creationCode;
        }
        if (key == keccak256("arb-route-processor-order-taker")) {
            return type(RouteProcessorRaindexV6ArbOrderTaker).creationCode;
        }
        if (key == keccak256("arb-generic-pool-flash-borrower")) {
            return type(GenericPoolRaindexV6FlashBorrower).creationCode;
        }
        revert(string.concat("no source for suite: ", suite));
    }

    function externalDeployments() internal pure override returns (ExternalDeployment[] memory) {
        ExternalDeployment[] memory externals = new ExternalDeployment[](2);
        externals[0] = ExternalDeployment({
            deployedAddress: address(LibTOFUTokenDecimals.TOFU_DECIMALS_DEPLOYMENT),
            codeHash: LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CODE_HASH
        });
        externals[1] = ExternalDeployment({
            deployedAddress: LibMetaBoardDeploy.META_BOARD_DEPLOYED_ADDRESS,
            codeHash: LibMetaBoardDeploy.META_BOARD_DEPLOYED_CODEHASH
        });
        return externals;
    }
}
