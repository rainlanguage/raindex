// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";
import {LibEtchRaindex} from "test/util/lib/LibEtchRaindex.sol";
import {RaindexV6} from "../../../src/concrete/raindex/RaindexV6.sol";
import {RaindexV6SubParser} from "../../../src/concrete/parser/RaindexV6SubParser.sol";
import {
    CREATION_CODE as RAINDEX_CREATION_CODE,
    RUNTIME_CODE as RAINDEX_RUNTIME_CODE,
    DEPLOYED_ADDRESS as RAINDEX_GENERATED_ADDRESS
} from "../../../src/generated/RaindexV6.pointers.sol";
import {
    CREATION_CODE as SUB_PARSER_CREATION_CODE,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_CODE,
    DEPLOYED_ADDRESS as SUB_PARSER_GENERATED_ADDRESS
} from "../../../src/generated/RaindexV6SubParser.pointers.sol";
import {
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_CODE,
    DEPLOYED_ADDRESS as ROUTE_PROCESSOR_GENERATED_ADDRESS,
    BYTECODE_HASH as ROUTE_PROCESSOR_GENERATED_CODEHASH
} from "../../../src/generated/RouteProcessor4.pointers.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../../../src/lib/deploy/LibRouteProcessor4CreationCode.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {GenericPoolRaindexV6FlashBorrower} from "../../../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";
import {
    RUNTIME_CODE as GENERIC_POOL_ARB_OT_RUNTIME_CODE,
    DEPLOYED_ADDRESS as GENERIC_POOL_ARB_OT_GENERATED_ADDRESS,
    BYTECODE_HASH as GENERIC_POOL_ARB_OT_GENERATED_CODEHASH
} from "../../../src/generated/GenericPoolRaindexV6ArbOrderTaker.pointers.sol";
import {
    RUNTIME_CODE as RP_ARB_OT_RUNTIME_CODE,
    DEPLOYED_ADDRESS as RP_ARB_OT_GENERATED_ADDRESS,
    BYTECODE_HASH as RP_ARB_OT_GENERATED_CODEHASH
} from "../../../src/generated/RouteProcessorRaindexV6ArbOrderTaker.pointers.sol";
import {
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_CODE,
    DEPLOYED_ADDRESS as GENERIC_POOL_FB_GENERATED_ADDRESS,
    BYTECODE_HASH as GENERIC_POOL_FB_GENERATED_CODEHASH
} from "../../../src/generated/GenericPoolRaindexV6FlashBorrower.pointers.sol";

contract LibRaindexDeployTest is Test {
    /// Deploying RaindexV6 via Zoltu MUST produce the expected address and
    /// codehash.
    function testDeployAddressRaindex() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(RaindexV6).creationCode);

        assertEq(deployedAddress, LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH);
    }

    /// Deploying RaindexV6SubParser via Zoltu MUST produce the expected
    /// address and codehash.
    function testDeployAddressSubParser() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(RaindexV6SubParser).creationCode);

        assertEq(deployedAddress, LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibRaindexDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed RaindexV6 MUST match the expected
    /// codehash constant.
    function testExpectedCodeHashRaindex() external {
        RaindexV6 raindex = new RaindexV6();
        assertEq(address(raindex).codehash, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed RaindexV6SubParser MUST match the
    /// expected codehash constant.
    function testExpectedCodeHashSubParser() external {
        RaindexV6SubParser subParser = new RaindexV6SubParser();
        assertEq(address(subParser).codehash, LibRaindexDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
    }

    /// The precompiled creation code constant for RaindexV6 MUST match the
    /// compiler's creation code.
    function testCreationCodeRaindex() external pure {
        assertEq(keccak256(RAINDEX_CREATION_CODE), keccak256(type(RaindexV6).creationCode));
    }

    /// The precompiled creation code constant for RaindexV6SubParser MUST
    /// match the compiler's creation code.
    function testCreationCodeSubParser() external pure {
        assertEq(keccak256(SUB_PARSER_CREATION_CODE), keccak256(type(RaindexV6SubParser).creationCode));
    }

    /// The precompiled runtime code constant for RaindexV6 MUST match the
    /// deployed runtime bytecode.
    function testRuntimeCodeRaindex() external {
        RaindexV6 raindex = new RaindexV6();
        assertEq(keccak256(RAINDEX_RUNTIME_CODE), keccak256(address(raindex).code));
    }

    /// The precompiled runtime code constant for RaindexV6SubParser MUST
    /// match the deployed runtime bytecode.
    function testRuntimeCodeSubParser() external {
        RaindexV6SubParser subParser = new RaindexV6SubParser();
        assertEq(keccak256(SUB_PARSER_RUNTIME_CODE), keccak256(address(subParser).code));
    }

    /// The generated deployed address for RaindexV6 MUST match the deploy
    /// library constant.
    function testGeneratedDeployedAddressRaindex() external pure {
        assertEq(RAINDEX_GENERATED_ADDRESS, LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS);
    }

    /// The generated deployed address for RaindexV6SubParser MUST match the
    /// deploy library constant.
    function testGeneratedDeployedAddressSubParser() external pure {
        assertEq(SUB_PARSER_GENERATED_ADDRESS, LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
    }

    /// Deploying RouteProcessor4 via Zoltu MUST produce the expected address
    /// and codehash.
    function testDeployAddressRouteProcessor() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(ROUTE_PROCESSOR_4_CREATION_CODE);

        assertEq(deployedAddress, LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH);
    }

    /// The precompiled runtime code constant for RouteProcessor4 MUST match
    /// the deployed runtime bytecode.
    function testRuntimeCodeRouteProcessor() external {
        bytes memory creationCode = ROUTE_PROCESSOR_4_CREATION_CODE;
        address deployed;
        assembly ("memory-safe") {
            deployed := create(0, add(creationCode, 0x20), mload(creationCode))
        }
        assertTrue(deployed != address(0), "RouteProcessor4 deployment failed");
        assertEq(keccak256(ROUTE_PROCESSOR_RUNTIME_CODE), keccak256(deployed.code));
    }

    /// The generated deployed address for RouteProcessor4 MUST match the
    /// deploy library constant.
    function testGeneratedDeployedAddressRouteProcessor() external pure {
        assertEq(ROUTE_PROCESSOR_GENERATED_ADDRESS, LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS);
    }

    /// The generated codehash for RouteProcessor4 MUST match the deploy
    /// library constant.
    function testGeneratedCodehashRouteProcessor() external pure {
        assertEq(ROUTE_PROCESSOR_GENERATED_CODEHASH, LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH);
    }

    /// Deploying GenericPoolRaindexV6ArbOrderTaker via Zoltu MUST produce
    /// the expected address and codehash.
    function testDeployAddressGenericPoolArbOrderTaker() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(GenericPoolRaindexV6ArbOrderTaker).creationCode);

        assertEq(deployedAddress, LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// Deploying RouteProcessorRaindexV6ArbOrderTaker via Zoltu MUST produce
    /// the expected address and codehash.
    function testDeployAddressRouteProcessorArbOrderTaker() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(RouteProcessorRaindexV6ArbOrderTaker).creationCode);

        assertEq(deployedAddress, LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// Deploying GenericPoolRaindexV6FlashBorrower via Zoltu MUST produce
    /// the expected address and codehash.
    function testDeployAddressGenericPoolFlashBorrower() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(GenericPoolRaindexV6FlashBorrower).creationCode);

        assertEq(deployedAddress, LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed GenericPoolRaindexV6ArbOrderTaker
    /// MUST match the expected codehash constant.
    function testExpectedCodeHashGenericPoolArbOrderTaker() external {
        GenericPoolRaindexV6ArbOrderTaker arb = new GenericPoolRaindexV6ArbOrderTaker();
        assertEq(address(arb).codehash, LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed RouteProcessorRaindexV6ArbOrderTaker
    /// MUST match the expected codehash constant.
    function testExpectedCodeHashRouteProcessorArbOrderTaker() external {
        RouteProcessorRaindexV6ArbOrderTaker arb = new RouteProcessorRaindexV6ArbOrderTaker();
        assertEq(address(arb).codehash, LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed GenericPoolRaindexV6FlashBorrower
    /// MUST match the expected codehash constant.
    function testExpectedCodeHashGenericPoolFlashBorrower() external {
        GenericPoolRaindexV6FlashBorrower fb = new GenericPoolRaindexV6FlashBorrower();
        assertEq(address(fb).codehash, LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH);
    }

    /// The precompiled runtime code for GenericPoolRaindexV6ArbOrderTaker
    /// MUST match the deployed runtime bytecode.
    function testRuntimeCodeGenericPoolArbOrderTaker() external {
        GenericPoolRaindexV6ArbOrderTaker arb = new GenericPoolRaindexV6ArbOrderTaker();
        assertEq(keccak256(GENERIC_POOL_ARB_OT_RUNTIME_CODE), keccak256(address(arb).code));
    }

    /// The precompiled runtime code for RouteProcessorRaindexV6ArbOrderTaker
    /// MUST match the deployed runtime bytecode.
    function testRuntimeCodeRouteProcessorArbOrderTaker() external {
        RouteProcessorRaindexV6ArbOrderTaker arb = new RouteProcessorRaindexV6ArbOrderTaker();
        assertEq(keccak256(RP_ARB_OT_RUNTIME_CODE), keccak256(address(arb).code));
    }

    /// The precompiled runtime code for GenericPoolRaindexV6FlashBorrower
    /// MUST match the deployed runtime bytecode.
    function testRuntimeCodeGenericPoolFlashBorrower() external {
        GenericPoolRaindexV6FlashBorrower fb = new GenericPoolRaindexV6FlashBorrower();
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_CODE), keccak256(address(fb).code));
    }

    /// The generated deployed address for GenericPoolRaindexV6ArbOrderTaker
    /// MUST match the deploy library constant.
    function testGeneratedDeployedAddressGenericPoolArbOrderTaker() external pure {
        assertEq(GENERIC_POOL_ARB_OT_GENERATED_ADDRESS, LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS);
    }

    /// The generated deployed address for RouteProcessorRaindexV6ArbOrderTaker
    /// MUST match the deploy library constant.
    function testGeneratedDeployedAddressRouteProcessorArbOrderTaker() external pure {
        assertEq(RP_ARB_OT_GENERATED_ADDRESS, LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS);
    }

    /// The generated deployed address for GenericPoolRaindexV6FlashBorrower
    /// MUST match the deploy library constant.
    function testGeneratedDeployedAddressGenericPoolFlashBorrower() external pure {
        assertEq(GENERIC_POOL_FB_GENERATED_ADDRESS, LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS);
    }

    /// The generated codehash for GenericPoolRaindexV6ArbOrderTaker MUST
    /// match the deploy library constant.
    function testGeneratedCodehashGenericPoolArbOrderTaker() external pure {
        assertEq(
            GENERIC_POOL_ARB_OT_GENERATED_CODEHASH, LibRaindexDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH
        );
    }

    /// The generated codehash for RouteProcessorRaindexV6ArbOrderTaker MUST
    /// match the deploy library constant.
    function testGeneratedCodehashRouteProcessorArbOrderTaker() external pure {
        assertEq(RP_ARB_OT_GENERATED_CODEHASH, LibRaindexDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// The generated codehash for GenericPoolRaindexV6FlashBorrower MUST
    /// match the deploy library constant.
    function testGeneratedCodehashGenericPoolFlashBorrower() external pure {
        assertEq(GENERIC_POOL_FB_GENERATED_CODEHASH, LibRaindexDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH);
    }

    /// After calling etchRaindex, all three contracts MUST have the expected
    /// codehash at their expected addresses.
    function testEtchRaindex() external {
        LibEtchRaindex.etchRaindex(vm);

        assertEq(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS.codehash, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH);
        assertEq(LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibRaindexDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
        assertEq(
            LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash,
            LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
        );
    }

    /// Calling etchRaindex twice MUST be idempotent — codehashes remain
    /// correct on the second call.
    function testEtchRaindexIdempotent() external {
        LibEtchRaindex.etchRaindex(vm);
        LibEtchRaindex.etchRaindex(vm);

        assertEq(LibRaindexDeploy.RAINDEX_DEPLOYED_ADDRESS.codehash, LibRaindexDeploy.RAINDEX_DEPLOYED_CODEHASH);
        assertEq(LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibRaindexDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
        assertEq(
            LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash,
            LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
        );
    }
}
