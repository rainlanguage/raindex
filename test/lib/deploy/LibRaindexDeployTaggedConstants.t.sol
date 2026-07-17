// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_4,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_4,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_4,
    CREATION_CODE as RAINDEX_CREATION_0_1_4
} from "../../../src/generated/0_1_4/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_4,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_4,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_4,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_4
} from "../../../src/generated/0_1_4/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_4,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_4
} from "../../../src/generated/0_1_4/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_4,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_4
} from "../../../src/generated/0_1_4/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_4,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_4
} from "../../../src/generated/0_1_4/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_4,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_4
} from "../../../src/generated/0_1_4/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_5,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_5,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_5,
    CREATION_CODE as RAINDEX_CREATION_0_1_5
} from "../../../src/generated/0_1_5/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_5,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_5,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_5,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_5
} from "../../../src/generated/0_1_5/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_5,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_5
} from "../../../src/generated/0_1_5/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_5,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_5
} from "../../../src/generated/0_1_5/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_5,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_5
} from "../../../src/generated/0_1_5/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_5,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_5
} from "../../../src/generated/0_1_5/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_6,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_6,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_6,
    CREATION_CODE as RAINDEX_CREATION_0_1_6
} from "../../../src/generated/0_1_6/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_6,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_6,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_6,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_6
} from "../../../src/generated/0_1_6/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_6,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_6
} from "../../../src/generated/0_1_6/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_6,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_6
} from "../../../src/generated/0_1_6/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_6,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_6
} from "../../../src/generated/0_1_6/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_6,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_6
} from "../../../src/generated/0_1_6/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_7,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_7,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_7,
    CREATION_CODE as RAINDEX_CREATION_0_1_7
} from "../../../src/generated/0_1_7/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_7,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_7,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_7,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_7
} from "../../../src/generated/0_1_7/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_7,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_7
} from "../../../src/generated/0_1_7/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_7,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_7
} from "../../../src/generated/0_1_7/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_7,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_7
} from "../../../src/generated/0_1_7/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_7,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_7
} from "../../../src/generated/0_1_7/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_8,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_8,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_8,
    CREATION_CODE as RAINDEX_CREATION_0_1_8
} from "../../../src/generated/0_1_8/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_8,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_8,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_8,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_8
} from "../../../src/generated/0_1_8/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_8,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_8
} from "../../../src/generated/0_1_8/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_8,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_8
} from "../../../src/generated/0_1_8/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_8,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_8
} from "../../../src/generated/0_1_8/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_8,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_8
} from "../../../src/generated/0_1_8/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_9,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_9,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_9,
    CREATION_CODE as RAINDEX_CREATION_0_1_9
} from "../../../src/generated/0_1_9/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_9,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_9,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_9,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_9
} from "../../../src/generated/0_1_9/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_9,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_9
} from "../../../src/generated/0_1_9/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_9,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_9
} from "../../../src/generated/0_1_9/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_9,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_9
} from "../../../src/generated/0_1_9/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_9,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_9
} from "../../../src/generated/0_1_9/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_10,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_10,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_10,
    CREATION_CODE as RAINDEX_CREATION_0_1_10
} from "../../../src/generated/0_1_10/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_10,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_10,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_10,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_10
} from "../../../src/generated/0_1_10/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_10,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_10
} from "../../../src/generated/0_1_10/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_10,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_10
} from "../../../src/generated/0_1_10/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_10,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_10
} from "../../../src/generated/0_1_10/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_10,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_10
} from "../../../src/generated/0_1_10/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_11,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_11,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_11,
    CREATION_CODE as RAINDEX_CREATION_0_1_11
} from "../../../src/generated/0_1_11/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_11,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_11,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_11,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_11
} from "../../../src/generated/0_1_11/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_11,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_11
} from "../../../src/generated/0_1_11/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_11,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_11
} from "../../../src/generated/0_1_11/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_11,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_11
} from "../../../src/generated/0_1_11/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_11,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_11
} from "../../../src/generated/0_1_11/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_12,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_12,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_12,
    CREATION_CODE as RAINDEX_CREATION_0_1_12
} from "../../../src/generated/0_1_12/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_12,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_12,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_12,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_12
} from "../../../src/generated/0_1_12/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_12,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_12
} from "../../../src/generated/0_1_12/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_12,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_12
} from "../../../src/generated/0_1_12/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_12,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_12
} from "../../../src/generated/0_1_12/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_12,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_12
} from "../../../src/generated/0_1_12/GenericPoolRaindexV6FlashBorrower.pointers.sol";
import {
    BYTECODE_HASH as RAINDEX_HASH_0_1_13,
    RUNTIME_CODE as RAINDEX_RUNTIME_0_1_13,
    DEPLOYED_ADDRESS as RAINDEX_ADDR_0_1_13,
    CREATION_CODE as RAINDEX_CREATION_0_1_13
} from "../../../src/generated/0_1_13/RaindexV6.pointers.sol";
import {
    BYTECODE_HASH as SUB_PARSER_HASH_0_1_13,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_0_1_13,
    DEPLOYED_ADDRESS as SUB_PARSER_ADDR_0_1_13,
    CREATION_CODE as SUB_PARSER_CREATION_0_1_13
} from "../../../src/generated/0_1_13/RaindexV6SubParser.pointers.sol";
import {
    BYTECODE_HASH as ROUTE_PROCESSOR_HASH_0_1_13,
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_0_1_13
} from "../../../src/generated/0_1_13/RouteProcessor4.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_ARB_HASH_0_1_13,
    RUNTIME_CODE as GENERIC_POOL_ARB_RUNTIME_0_1_13
} from "../../../src/generated/0_1_13/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as RP_ARB_HASH_0_1_13,
    RUNTIME_CODE as RP_ARB_RUNTIME_0_1_13
} from "../../../src/generated/0_1_13/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    BYTECODE_HASH as GENERIC_POOL_FB_HASH_0_1_13,
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_0_1_13
} from "../../../src/generated/0_1_13/GenericPoolRaindexV6FlashBorrower.pointers.sol";

/// @title LibRaindexDeployTaggedConstantsTest
/// @notice Each frozen per-tag pointer snapshot under `src/generated/<tag>/`
/// must be self-consistent and reproducible: its recorded `BYTECODE_HASH` is the
/// keccak of its recorded `RUNTIME_CODE`, and (for the contracts whose snapshot
/// carries `CREATION_CODE`) Zoltu-deploying that creation code lands at the recorded
/// `DEPLOYED_ADDRESS` with the recorded codehash. Network-free (no registry FFI): a
/// new release freezes a `<tag>/` snapshot (via BuildPointers) and adds a test block
/// here. This is the raindex analogue of rain.factory's tagged-constants test.
contract LibRaindexDeployTaggedConstantsTest is Test {
    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.4 - the pin is internally consistent.
    function testRaindexV6_0_1_4_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_4), RAINDEX_HASH_0_1_4);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.4 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_4_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_4);
        assertEq(deployed, RAINDEX_ADDR_0_1_4);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_4);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_4);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.4 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_4_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_4), SUB_PARSER_HASH_0_1_4);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.4 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_4_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_4);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_4);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_4);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_4);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.4 - the pin is internally consistent.
    function testRouteProcessor4_0_1_4_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_4), ROUTE_PROCESSOR_HASH_0_1_4);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.4 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_4_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_4), GENERIC_POOL_ARB_HASH_0_1_4);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.4 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_4_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_4), RP_ARB_HASH_0_1_4);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.4 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_4_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_4), GENERIC_POOL_FB_HASH_0_1_4);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.5 - the pin is internally consistent.
    function testRaindexV6_0_1_5_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_5), RAINDEX_HASH_0_1_5);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.5 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_5_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_5);
        assertEq(deployed, RAINDEX_ADDR_0_1_5);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_5);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_5);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.5 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_5_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_5), SUB_PARSER_HASH_0_1_5);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.5 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_5_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_5);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_5);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_5);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_5);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.5 - the pin is internally consistent.
    function testRouteProcessor4_0_1_5_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_5), ROUTE_PROCESSOR_HASH_0_1_5);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.5 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_5_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_5), GENERIC_POOL_ARB_HASH_0_1_5);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.5 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_5_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_5), RP_ARB_HASH_0_1_5);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.5 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_5_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_5), GENERIC_POOL_FB_HASH_0_1_5);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.6 - the pin is internally consistent.
    function testRaindexV6_0_1_6_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_6), RAINDEX_HASH_0_1_6);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.6 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_6_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_6);
        assertEq(deployed, RAINDEX_ADDR_0_1_6);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_6);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_6);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.6 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_6_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_6), SUB_PARSER_HASH_0_1_6);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.6 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_6_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_6);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_6);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_6);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_6);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.6 - the pin is internally consistent.
    function testRouteProcessor4_0_1_6_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_6), ROUTE_PROCESSOR_HASH_0_1_6);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.6 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_6_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_6), GENERIC_POOL_ARB_HASH_0_1_6);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.6 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_6_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_6), RP_ARB_HASH_0_1_6);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.6 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_6_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_6), GENERIC_POOL_FB_HASH_0_1_6);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.7 - the pin is internally consistent.
    function testRaindexV6_0_1_7_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_7), RAINDEX_HASH_0_1_7);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.7 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_7_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_7);
        assertEq(deployed, RAINDEX_ADDR_0_1_7);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_7);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_7);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.7 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_7_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_7), SUB_PARSER_HASH_0_1_7);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.7 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_7_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_7);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_7);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_7);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_7);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.7 - the pin is internally consistent.
    function testRouteProcessor4_0_1_7_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_7), ROUTE_PROCESSOR_HASH_0_1_7);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.7 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_7_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_7), GENERIC_POOL_ARB_HASH_0_1_7);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.7 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_7_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_7), RP_ARB_HASH_0_1_7);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.7 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_7_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_7), GENERIC_POOL_FB_HASH_0_1_7);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.8 - the pin is internally consistent.
    function testRaindexV6_0_1_8_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_8), RAINDEX_HASH_0_1_8);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.8 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_8_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_8);
        assertEq(deployed, RAINDEX_ADDR_0_1_8);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_8);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_8);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.8 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_8_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_8), SUB_PARSER_HASH_0_1_8);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.8 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_8_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_8);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_8);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_8);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_8);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.8 - the pin is internally consistent.
    function testRouteProcessor4_0_1_8_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_8), ROUTE_PROCESSOR_HASH_0_1_8);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.8 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_8_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_8), GENERIC_POOL_ARB_HASH_0_1_8);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.8 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_8_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_8), RP_ARB_HASH_0_1_8);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.8 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_8_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_8), GENERIC_POOL_FB_HASH_0_1_8);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.9 - the pin is internally consistent.
    function testRaindexV6_0_1_9_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_9), RAINDEX_HASH_0_1_9);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.9 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_9_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_9);
        assertEq(deployed, RAINDEX_ADDR_0_1_9);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_9);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_9);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.9 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_9_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_9), SUB_PARSER_HASH_0_1_9);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.9 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_9_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_9);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_9);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_9);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_9);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.9 - the pin is internally consistent.
    function testRouteProcessor4_0_1_9_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_9), ROUTE_PROCESSOR_HASH_0_1_9);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.9 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_9_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_9), GENERIC_POOL_ARB_HASH_0_1_9);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.9 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_9_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_9), RP_ARB_HASH_0_1_9);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.9 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_9_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_9), GENERIC_POOL_FB_HASH_0_1_9);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.10 - the pin is internally consistent.
    function testRaindexV6_0_1_10_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_10), RAINDEX_HASH_0_1_10);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.10 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_10_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_10);
        assertEq(deployed, RAINDEX_ADDR_0_1_10);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_10);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_10);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.10 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_10_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_10), SUB_PARSER_HASH_0_1_10);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.10 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_10_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_10);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_10);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_10);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_10);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.10 - the pin is internally consistent.
    function testRouteProcessor4_0_1_10_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_10), ROUTE_PROCESSOR_HASH_0_1_10);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.10 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_10_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_10), GENERIC_POOL_ARB_HASH_0_1_10);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.10 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_10_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_10), RP_ARB_HASH_0_1_10);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.10 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_10_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_10), GENERIC_POOL_FB_HASH_0_1_10);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.11 - the pin is internally consistent.
    function testRaindexV6_0_1_11_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_11), RAINDEX_HASH_0_1_11);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.11 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_11_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_11);
        assertEq(deployed, RAINDEX_ADDR_0_1_11);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_11);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_11);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.11 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_11_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_11), SUB_PARSER_HASH_0_1_11);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.11 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_11_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_11);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_11);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_11);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_11);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.11 - the pin is internally consistent.
    function testRouteProcessor4_0_1_11_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_11), ROUTE_PROCESSOR_HASH_0_1_11);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.11 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_11_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_11), GENERIC_POOL_ARB_HASH_0_1_11);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.11 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_11_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_11), RP_ARB_HASH_0_1_11);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.11 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_11_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_11), GENERIC_POOL_FB_HASH_0_1_11);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.12 - the pin is internally consistent.
    function testRaindexV6_0_1_12_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_12), RAINDEX_HASH_0_1_12);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.12 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_12_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_12);
        assertEq(deployed, RAINDEX_ADDR_0_1_12);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_12);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_12);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.12 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_12_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_12), SUB_PARSER_HASH_0_1_12);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.12 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_12_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_12);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_12);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_12);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_12);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.12 - the pin is internally consistent.
    function testRouteProcessor4_0_1_12_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_12), ROUTE_PROCESSOR_HASH_0_1_12);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.12 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_12_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_12), GENERIC_POOL_ARB_HASH_0_1_12);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.12 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_12_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_12), RP_ARB_HASH_0_1_12);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.12 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_12_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_12), GENERIC_POOL_FB_HASH_0_1_12);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6 0.1.13 - the pin is internally consistent.
    function testRaindexV6_0_1_13_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RAINDEX_RUNTIME_0_1_13), RAINDEX_HASH_0_1_13);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6 0.1.13 reproduces its recorded address + codehash.
    function testRaindexV6_0_1_13_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(RAINDEX_CREATION_0_1_13);
        assertEq(deployed, RAINDEX_ADDR_0_1_13);
        assertEq(deployed.codehash, RAINDEX_HASH_0_1_13);
        assertEq(keccak256(deployed.code), RAINDEX_HASH_0_1_13);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RaindexV6SubParser 0.1.13 - the pin is internally consistent.
    function testRaindexV6SubParser_0_1_13_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(SUB_PARSER_RUNTIME_0_1_13), SUB_PARSER_HASH_0_1_13);
    }

    /// Zoltu-deploying the frozen CREATION_CODE for RaindexV6SubParser 0.1.13 reproduces its recorded address + codehash.
    function testRaindexV6SubParser_0_1_13_CreationDeploysToPinnedAddress() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(SUB_PARSER_CREATION_0_1_13);
        assertEq(deployed, SUB_PARSER_ADDR_0_1_13);
        assertEq(deployed.codehash, SUB_PARSER_HASH_0_1_13);
        assertEq(keccak256(deployed.code), SUB_PARSER_HASH_0_1_13);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessor4 0.1.13 - the pin is internally consistent.
    function testRouteProcessor4_0_1_13_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_0_1_13), ROUTE_PROCESSOR_HASH_0_1_13);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6ArbOrderTaker 0.1.13 - the pin is internally consistent.
    function testGenericPoolRaindexV6ArbOrderTaker_0_1_13_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_ARB_RUNTIME_0_1_13), GENERIC_POOL_ARB_HASH_0_1_13);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for RouteProcessorRaindexV6ArbOrderTaker 0.1.13 - the pin is internally consistent.
    function testRouteProcessorRaindexV6ArbOrderTaker_0_1_13_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(RP_ARB_RUNTIME_0_1_13), RP_ARB_HASH_0_1_13);
    }

    /// keccak256(RUNTIME_CODE) == BYTECODE_HASH for GenericPoolRaindexV6FlashBorrower 0.1.13 - the pin is internally consistent.
    function testGenericPoolRaindexV6FlashBorrower_0_1_13_RuntimeHashesToBytecodeHash() external pure {
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_0_1_13), GENERIC_POOL_FB_HASH_0_1_13);
    }
}
