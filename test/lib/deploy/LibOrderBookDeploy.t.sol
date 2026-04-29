// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibEtchOrderBook} from "test/util/lib/LibEtchOrderBook.sol";
import {OrderBookV6} from "../../../src/concrete/ob/OrderBookV6.sol";
import {OrderBookV6SubParser} from "../../../src/concrete/parser/OrderBookV6SubParser.sol";
import {
    CREATION_CODE as ORDERBOOK_CREATION_CODE,
    RUNTIME_CODE as ORDERBOOK_RUNTIME_CODE,
    DEPLOYED_ADDRESS as ORDERBOOK_GENERATED_ADDRESS
} from "../../../src/generated/OrderBookV6.pointers.sol";
import {
    CREATION_CODE as SUB_PARSER_CREATION_CODE,
    RUNTIME_CODE as SUB_PARSER_RUNTIME_CODE,
    DEPLOYED_ADDRESS as SUB_PARSER_GENERATED_ADDRESS
} from "../../../src/generated/OrderBookV6SubParser.pointers.sol";
import {
    RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_CODE,
    DEPLOYED_ADDRESS as ROUTE_PROCESSOR_GENERATED_ADDRESS,
    BYTECODE_HASH as ROUTE_PROCESSOR_GENERATED_CODEHASH
} from "../../../src/generated/RouteProcessor4.pointers.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../../../src/lib/deploy/LibRouteProcessor4CreationCode.sol";
import {GenericPoolOrderBookV6ArbOrderTaker} from "../../../src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol";
import {
    RouteProcessorOrderBookV6ArbOrderTaker
} from "../../../src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";
import {GenericPoolOrderBookV6FlashBorrower} from "../../../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";
import {
    RUNTIME_CODE as GENERIC_POOL_ARB_OT_RUNTIME_CODE,
    DEPLOYED_ADDRESS as GENERIC_POOL_ARB_OT_GENERATED_ADDRESS,
    BYTECODE_HASH as GENERIC_POOL_ARB_OT_GENERATED_CODEHASH
} from "../../../src/generated/GenericPoolOrderBookV6ArbOrderTaker.pointers.sol";
import {
    RUNTIME_CODE as RP_ARB_OT_RUNTIME_CODE,
    DEPLOYED_ADDRESS as RP_ARB_OT_GENERATED_ADDRESS,
    BYTECODE_HASH as RP_ARB_OT_GENERATED_CODEHASH
} from "../../../src/generated/RouteProcessorOrderBookV6ArbOrderTaker.pointers.sol";
import {
    RUNTIME_CODE as GENERIC_POOL_FB_RUNTIME_CODE,
    DEPLOYED_ADDRESS as GENERIC_POOL_FB_GENERATED_ADDRESS,
    BYTECODE_HASH as GENERIC_POOL_FB_GENERATED_CODEHASH
} from "../../../src/generated/GenericPoolOrderBookV6FlashBorrower.pointers.sol";

contract LibOrderBookDeployTest is Test {
    /// Deploying OrderBookV6 via Zoltu MUST produce the expected address and
    /// codehash.
    function testDeployAddressOrderBook() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(OrderBookV6).creationCode);

        assertEq(deployedAddress, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH);
    }

    /// Deploying OrderBookV6SubParser via Zoltu MUST produce the expected
    /// address and codehash.
    function testDeployAddressSubParser() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(OrderBookV6SubParser).creationCode);

        assertEq(deployedAddress, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed OrderBookV6 MUST match the expected
    /// codehash constant.
    function testExpectedCodeHashOrderBook() external {
        OrderBookV6 ob = new OrderBookV6();
        assertEq(address(ob).codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed OrderBookV6SubParser MUST match the
    /// expected codehash constant.
    function testExpectedCodeHashSubParser() external {
        OrderBookV6SubParser subParser = new OrderBookV6SubParser();
        assertEq(address(subParser).codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH);
    }

    /// The precompiled creation code constant for OrderBookV6 MUST match the
    /// compiler's creation code.
    function testCreationCodeOrderBook() external pure {
        assertEq(keccak256(ORDERBOOK_CREATION_CODE), keccak256(type(OrderBookV6).creationCode));
    }

    /// The precompiled creation code constant for OrderBookV6SubParser MUST
    /// match the compiler's creation code.
    function testCreationCodeSubParser() external pure {
        assertEq(keccak256(SUB_PARSER_CREATION_CODE), keccak256(type(OrderBookV6SubParser).creationCode));
    }

    /// The precompiled runtime code constant for OrderBookV6 MUST match the
    /// deployed runtime bytecode.
    function testRuntimeCodeOrderBook() external {
        OrderBookV6 ob = new OrderBookV6();
        assertEq(keccak256(ORDERBOOK_RUNTIME_CODE), keccak256(address(ob).code));
    }

    /// The precompiled runtime code constant for OrderBookV6SubParser MUST
    /// match the deployed runtime bytecode.
    function testRuntimeCodeSubParser() external {
        OrderBookV6SubParser subParser = new OrderBookV6SubParser();
        assertEq(keccak256(SUB_PARSER_RUNTIME_CODE), keccak256(address(subParser).code));
    }

    /// The generated deployed address for OrderBookV6 MUST match the deploy
    /// library constant.
    function testGeneratedDeployedAddressOrderBook() external pure {
        assertEq(ORDERBOOK_GENERATED_ADDRESS, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS);
    }

    /// The generated deployed address for OrderBookV6SubParser MUST match the
    /// deploy library constant.
    function testGeneratedDeployedAddressSubParser() external pure {
        assertEq(SUB_PARSER_GENERATED_ADDRESS, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
    }

    /// Deploying RouteProcessor4 via Zoltu MUST produce the expected address
    /// and codehash.
    function testDeployAddressRouteProcessor() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(ROUTE_PROCESSOR_4_CREATION_CODE);

        assertEq(deployedAddress, LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH);
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
        assertEq(ROUTE_PROCESSOR_GENERATED_ADDRESS, LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS);
    }

    /// The generated codehash for RouteProcessor4 MUST match the deploy
    /// library constant.
    function testGeneratedCodehashRouteProcessor() external pure {
        assertEq(ROUTE_PROCESSOR_GENERATED_CODEHASH, LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH);
    }

    /// Deploying GenericPoolOrderBookV6ArbOrderTaker via Zoltu MUST produce
    /// the expected address and codehash.
    function testDeployAddressGenericPoolArbOrderTaker() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(GenericPoolOrderBookV6ArbOrderTaker).creationCode);

        assertEq(deployedAddress, LibOrderBookDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibOrderBookDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// Deploying RouteProcessorOrderBookV6ArbOrderTaker via Zoltu MUST produce
    /// the expected address and codehash.
    function testDeployAddressRouteProcessorArbOrderTaker() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(RouteProcessorOrderBookV6ArbOrderTaker).creationCode);

        assertEq(deployedAddress, LibOrderBookDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(
            address(deployedAddress).codehash, LibOrderBookDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH
        );
    }

    /// Deploying GenericPoolOrderBookV6FlashBorrower via Zoltu MUST produce
    /// the expected address and codehash.
    function testDeployAddressGenericPoolFlashBorrower() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployedAddress = LibRainDeploy.deployZoltu(type(GenericPoolOrderBookV6FlashBorrower).creationCode);

        assertEq(deployedAddress, LibOrderBookDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");
        assertEq(address(deployedAddress).codehash, LibOrderBookDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed GenericPoolOrderBookV6ArbOrderTaker
    /// MUST match the expected codehash constant.
    function testExpectedCodeHashGenericPoolArbOrderTaker() external {
        GenericPoolOrderBookV6ArbOrderTaker arb = new GenericPoolOrderBookV6ArbOrderTaker();
        assertEq(address(arb).codehash, LibOrderBookDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed RouteProcessorOrderBookV6ArbOrderTaker
    /// MUST match the expected codehash constant.
    function testExpectedCodeHashRouteProcessorArbOrderTaker() external {
        RouteProcessorOrderBookV6ArbOrderTaker arb = new RouteProcessorOrderBookV6ArbOrderTaker();
        assertEq(address(arb).codehash, LibOrderBookDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// The codehash of a freshly deployed GenericPoolOrderBookV6FlashBorrower
    /// MUST match the expected codehash constant.
    function testExpectedCodeHashGenericPoolFlashBorrower() external {
        GenericPoolOrderBookV6FlashBorrower fb = new GenericPoolOrderBookV6FlashBorrower();
        assertEq(address(fb).codehash, LibOrderBookDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH);
    }

    /// The precompiled runtime code for GenericPoolOrderBookV6ArbOrderTaker
    /// MUST match the deployed runtime bytecode.
    function testRuntimeCodeGenericPoolArbOrderTaker() external {
        GenericPoolOrderBookV6ArbOrderTaker arb = new GenericPoolOrderBookV6ArbOrderTaker();
        assertEq(keccak256(GENERIC_POOL_ARB_OT_RUNTIME_CODE), keccak256(address(arb).code));
    }

    /// The precompiled runtime code for RouteProcessorOrderBookV6ArbOrderTaker
    /// MUST match the deployed runtime bytecode.
    function testRuntimeCodeRouteProcessorArbOrderTaker() external {
        RouteProcessorOrderBookV6ArbOrderTaker arb = new RouteProcessorOrderBookV6ArbOrderTaker();
        assertEq(keccak256(RP_ARB_OT_RUNTIME_CODE), keccak256(address(arb).code));
    }

    /// The precompiled runtime code for GenericPoolOrderBookV6FlashBorrower
    /// MUST match the deployed runtime bytecode.
    function testRuntimeCodeGenericPoolFlashBorrower() external {
        GenericPoolOrderBookV6FlashBorrower fb = new GenericPoolOrderBookV6FlashBorrower();
        assertEq(keccak256(GENERIC_POOL_FB_RUNTIME_CODE), keccak256(address(fb).code));
    }

    /// The generated deployed address for GenericPoolOrderBookV6ArbOrderTaker
    /// MUST match the deploy library constant.
    function testGeneratedDeployedAddressGenericPoolArbOrderTaker() external pure {
        assertEq(
            GENERIC_POOL_ARB_OT_GENERATED_ADDRESS, LibOrderBookDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_ADDRESS
        );
    }

    /// The generated deployed address for RouteProcessorOrderBookV6ArbOrderTaker
    /// MUST match the deploy library constant.
    function testGeneratedDeployedAddressRouteProcessorArbOrderTaker() external pure {
        assertEq(RP_ARB_OT_GENERATED_ADDRESS, LibOrderBookDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_ADDRESS);
    }

    /// The generated deployed address for GenericPoolOrderBookV6FlashBorrower
    /// MUST match the deploy library constant.
    function testGeneratedDeployedAddressGenericPoolFlashBorrower() external pure {
        assertEq(GENERIC_POOL_FB_GENERATED_ADDRESS, LibOrderBookDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_ADDRESS);
    }

    /// The generated codehash for GenericPoolOrderBookV6ArbOrderTaker MUST
    /// match the deploy library constant.
    function testGeneratedCodehashGenericPoolArbOrderTaker() external pure {
        assertEq(
            GENERIC_POOL_ARB_OT_GENERATED_CODEHASH, LibOrderBookDeploy.GENERIC_POOL_ARB_ORDER_TAKER_DEPLOYED_CODEHASH
        );
    }

    /// The generated codehash for RouteProcessorOrderBookV6ArbOrderTaker MUST
    /// match the deploy library constant.
    function testGeneratedCodehashRouteProcessorArbOrderTaker() external pure {
        assertEq(RP_ARB_OT_GENERATED_CODEHASH, LibOrderBookDeploy.ROUTE_PROCESSOR_ARB_ORDER_TAKER_DEPLOYED_CODEHASH);
    }

    /// The generated codehash for GenericPoolOrderBookV6FlashBorrower MUST
    /// match the deploy library constant.
    function testGeneratedCodehashGenericPoolFlashBorrower() external pure {
        assertEq(GENERIC_POOL_FB_GENERATED_CODEHASH, LibOrderBookDeploy.GENERIC_POOL_FLASH_BORROWER_DEPLOYED_CODEHASH);
    }

    /// After calling etchOrderBook, all three contracts MUST have the expected
    /// codehash at their expected addresses.
    function testEtchOrderBook() external {
        LibEtchOrderBook.etchOrderBook(vm);

        assertEq(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH);
        assertEq(
            LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH
        );
        assertEq(
            LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash,
            LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
        );
    }

    /// Calling etchOrderBook twice MUST be idempotent — codehashes remain
    /// correct on the second call.
    function testEtchOrderBookIdempotent() external {
        LibEtchOrderBook.etchOrderBook(vm);
        LibEtchOrderBook.etchOrderBook(vm);

        assertEq(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH);
        assertEq(
            LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash, LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH
        );
        assertEq(
            LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash,
            LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
        );
    }
}
