// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DeploySuite} from "rain-deploy-0.1.22/src/abstract/RainDeploySuitesBase.sol";
import {LibRainDeploySnapshot} from "rain-deploy-0.1.22/src/lib/LibRainDeploySnapshot.sol";
import {LibRaindexProdDeploy} from "src/lib/deploy/LibRaindexProdDeploy.sol";
import {LibRaindexV6Released} from "src/lib/LibRaindexV6Released.sol";
import {LibRaindexV6SubParserReleased} from "src/lib/LibRaindexV6SubParserReleased.sol";
import {LibRouteProcessor4Released} from "src/lib/LibRouteProcessor4Released.sol";
import {LibGenericPoolRaindexV6ArbOrderTakerReleased} from "src/lib/LibGenericPoolRaindexV6ArbOrderTakerReleased.sol";
import {
    LibRouteProcessorRaindexV6ArbOrderTakerReleased
} from "src/lib/LibRouteProcessorRaindexV6ArbOrderTakerReleased.sol";
import {LibGenericPoolRaindexV6FlashBorrowerReleased} from "src/lib/LibGenericPoolRaindexV6FlashBorrowerReleased.sol";

/// @title LibRaindexProdDeployTest
/// @notice `LibRaindexProdDeploy` MUST pin the newest frozen release, and that
/// release MUST be the one `[external.package].version` names.
contract LibRaindexProdDeployTest is Test {
    function checkPinsPackageRelease(DeploySuite[] memory released, address deployedAddress, bytes32 codehash)
        internal
        view
    {
        assertTrue(released.length > 0, "no release frozen");
        DeploySuite memory latest = released[released.length - 1];
        string[] memory parts = vm.split(latest.suite, "@");
        assertEq(parts[parts.length - 1], LibRainDeploySnapshot.deployTag(vm), latest.suite);
        assertEq(deployedAddress, latest.storedDeployedAddress, latest.suite);
        assertEq(codehash, latest.storedBytecodeHash, latest.suite);
    }

    function testRaindexPinsPackageRelease() external view {
        checkPinsPackageRelease(
            LibRaindexV6Released.releasedSuites(),
            LibRaindexProdDeploy.RAINDEX_DEPLOYED_ADDRESS,
            LibRaindexProdDeploy.RAINDEX_DEPLOYED_CODEHASH
        );
    }

    function testSubParserPinsPackageRelease() external view {
        checkPinsPackageRelease(
            LibRaindexV6SubParserReleased.releasedSuites(),
            LibRaindexProdDeploy.SUB_PARSER_DEPLOYED_ADDRESS,
            LibRaindexProdDeploy.SUB_PARSER_DEPLOYED_CODEHASH
        );
    }

    function testRouteProcessorPinsPackageRelease() external view {
        checkPinsPackageRelease(
            LibRouteProcessor4Released.releasedSuites(),
            LibRaindexProdDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS,
            LibRaindexProdDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
        );
    }

    function testGenericPoolArbOrderTakerPinsPackageRelease() external view {
        checkPinsPackageRelease(
            LibGenericPoolRaindexV6ArbOrderTakerReleased.releasedSuites(),
            LibRaindexProdDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS,
            LibRaindexProdDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH
        );
    }

    function testRouteProcessorArbOrderTakerPinsPackageRelease() external view {
        checkPinsPackageRelease(
            LibRouteProcessorRaindexV6ArbOrderTakerReleased.releasedSuites(),
            LibRaindexProdDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS,
            LibRaindexProdDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH
        );
    }

    function testGenericPoolFlashBorrowerPinsPackageRelease() external view {
        checkPinsPackageRelease(
            LibGenericPoolRaindexV6FlashBorrowerReleased.releasedSuites(),
            LibRaindexProdDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS,
            LibRaindexProdDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH
        );
    }
}
