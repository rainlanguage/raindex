// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script} from "forge-std/Script.sol";
import {LibCodeGen} from "rain.sol.codegen/lib/LibCodeGen.sol";
import {LibFs} from "rain.sol.codegen/lib/LibFs.sol";
import {OrderBookV6} from "../src/concrete/ob/OrderBookV6.sol";
import {OrderBookV6SubParser} from "../src/concrete/parser/OrderBookV6SubParser.sol";
import {LibOrderBookSubParser, EXTERN_PARSE_META_BUILD_DEPTH} from "../src/lib/LibOrderBookSubParser.sol";
import {LibGenParseMeta} from "rain.interpreter.interface/lib/codegen/LibGenParseMeta.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../src/lib/deploy/LibRouteProcessor4CreationCode.sol";
import {GenericPoolOrderBookV6ArbOrderTaker} from "../src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol";
import {RouteProcessorOrderBookV6ArbOrderTaker} from "../src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";
import {GenericPoolOrderBookV6FlashBorrower} from "../src/concrete/arb/GenericPoolOrderBookV6FlashBorrower.sol";

contract BuildPointers is Script {
    function addressConstantString(address addr) internal pure returns (string memory) {
        return string.concat(
            "\n",
            "/// @dev The deterministic deploy address of the contract when deployed via\n",
            "/// the Zoltu factory.\n",
            "address constant DEPLOYED_ADDRESS = address(",
            vm.toString(addr),
            ");\n"
        );
    }

    function buildOrderBookV6Pointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(OrderBookV6).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "OrderBookV6",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm,
                    "/// @dev The creation bytecode of the contract.",
                    "CREATION_CODE",
                    type(OrderBookV6).creationCode
                ),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function buildOrderBookSubParserPointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(OrderBookV6SubParser).creationCode);
        OrderBookV6SubParser subParser = OrderBookV6SubParser(deployed);

        string memory name = "OrderBookV6SubParser";

        LibFs.buildFileForContract(
            vm,
            deployed,
            name,
            string.concat(
                string.concat(
                    addressConstantString(deployed),
                    LibCodeGen.bytesConstantString(
                        vm,
                        "/// @dev The creation bytecode of the contract.",
                        "CREATION_CODE",
                        type(OrderBookV6SubParser).creationCode
                    ),
                    LibCodeGen.bytesConstantString(
                        vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                    )
                ),
                string.concat(
                    LibCodeGen.describedByMetaHashConstantString(vm, name),
                    LibGenParseMeta.parseMetaConstantString(
                        vm, LibOrderBookSubParser.authoringMetaV2(), EXTERN_PARSE_META_BUILD_DEPTH
                    ),
                    LibCodeGen.subParserWordParsersConstantString(vm, subParser),
                    LibCodeGen.operandHandlerFunctionPointersConstantString(vm, subParser),
                    LibCodeGen.literalParserFunctionPointersConstantString(vm, subParser)
                )
            )
        );
    }

    function buildRouteProcessor4Pointers() internal {
        address deployed = LibRainDeploy.deployZoltu(ROUTE_PROCESSOR_4_CREATION_CODE);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "RouteProcessor4",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function buildGenericPoolArbOrderTakerPointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(GenericPoolOrderBookV6ArbOrderTaker).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "GenericPoolOrderBookV6ArbOrderTaker",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function buildRouteProcessorArbOrderTakerPointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(RouteProcessorOrderBookV6ArbOrderTaker).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "RouteProcessorOrderBookV6ArbOrderTaker",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function buildGenericPoolFlashBorrowerPointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(GenericPoolOrderBookV6FlashBorrower).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "GenericPoolOrderBookV6FlashBorrower",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function run() external {
        LibRainDeploy.etchZoltuFactory(vm);

        buildOrderBookV6Pointers();
        buildOrderBookSubParserPointers();
        buildRouteProcessor4Pointers();
        buildGenericPoolArbOrderTakerPointers();
        buildRouteProcessorArbOrderTakerPointers();
        buildGenericPoolFlashBorrowerPointers();
    }
}
