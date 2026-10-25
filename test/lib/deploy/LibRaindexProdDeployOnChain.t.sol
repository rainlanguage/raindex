// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibRainDeploy} from "rain-deploy-0.1.22/src/lib/LibRainDeploy.sol";
import {LibRaindexProdDeploy} from "../../../src/lib/deploy/LibRaindexProdDeploy.sol";

/// @title LibRaindexProdDeployOnChainTest
/// @notice Forks each supported network and verifies that all raindex
/// contracts are deployed at the expected addresses with the expected codehash.
contract LibRaindexProdDeployOnChainTest is Test {
    function _checkAllContracts() internal view {
        assertTrue(LibRaindexProdDeploy.RAINDEX_DEPLOYED_ADDRESS.code.length > 0, "RaindexV6 not deployed");
        assertEq(LibRaindexProdDeploy.RAINDEX_DEPLOYED_ADDRESS.codehash, LibRaindexProdDeploy.RAINDEX_DEPLOYED_CODEHASH);

        assertTrue(LibRaindexProdDeploy.SUB_PARSER_DEPLOYED_ADDRESS.code.length > 0, "SubParser not deployed");
        assertEq(
            LibRaindexProdDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibRaindexProdDeploy.SUB_PARSER_DEPLOYED_CODEHASH
        );

        assertTrue(
            LibRaindexProdDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.code.length > 0, "RouteProcessor4 not deployed"
        );
        assertEq(
            LibRaindexProdDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash,
            LibRaindexProdDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
        );

        assertTrue(
            LibRaindexProdDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS.code.length > 0,
            "GenericPoolArbOrderTaker not deployed"
        );
        assertEq(
            LibRaindexProdDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS.codehash,
            LibRaindexProdDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH
        );

        assertTrue(
            LibRaindexProdDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS.code.length > 0,
            "RouteProcessorArbOrderTaker not deployed"
        );
        assertEq(
            LibRaindexProdDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS.codehash,
            LibRaindexProdDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH
        );

        assertTrue(
            LibRaindexProdDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS.code.length > 0,
            "GenericPoolFlashBorrower not deployed"
        );
        assertEq(
            LibRaindexProdDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS.codehash,
            LibRaindexProdDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH
        );
    }

    /// All contracts MUST be deployed on Arbitrum.
    function testProdDeployArbitrum() external {
        vm.createSelectFork(LibRainDeploy.ARBITRUM_ONE);
        _checkAllContracts();
    }

    /// All contracts MUST be deployed on Base.
    function testProdDeployBase() external {
        vm.createSelectFork(LibRainDeploy.BASE);
        _checkAllContracts();
    }

    /// All contracts MUST be deployed on Base Sepolia.
    function testProdDeployBaseSepolia() external {
        vm.createSelectFork(LibRainDeploy.BASE_SEPOLIA);
        _checkAllContracts();
    }

    /// All contracts MUST be deployed on Flare.
    function testProdDeployFlare() external {
        vm.createSelectFork(LibRainDeploy.FLARE);
        _checkAllContracts();
    }

    /// All contracts MUST be deployed on Polygon.
    function testProdDeployPolygon() external {
        vm.createSelectFork(LibRainDeploy.POLYGON);
        _checkAllContracts();
    }
}
