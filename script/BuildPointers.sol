// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script} from "forge-std/Script.sol";
import {LibCodeGen} from "rain.sol.codegen/lib/LibCodeGen.sol";
import {LibFs} from "rain.sol.codegen/lib/LibFs.sol";
import {RaindexV6} from "../src/concrete/raindex/RaindexV6.sol";
import {RaindexV6SubParser} from "../src/concrete/parser/RaindexV6SubParser.sol";
import {LibRaindexSubParser, EXTERN_PARSE_META_BUILD_DEPTH} from "../src/lib/LibRaindexSubParser.sol";
import {LibGenParseMeta} from "rain.interpreter.interface/lib/codegen/LibGenParseMeta.sol";
import {LibRainDeploy} from "rain.deploy/lib/LibRainDeploy.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../src/lib/deploy/LibRouteProcessor4CreationCode.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {GenericPoolRaindexV6FlashBorrower} from "../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";

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

    function buildRaindexV6Pointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(RaindexV6).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "RaindexV6",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The creation bytecode of the contract.", "CREATION_CODE", type(RaindexV6).creationCode
                ),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function buildRaindexSubParserPointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(RaindexV6SubParser).creationCode);
        RaindexV6SubParser subParser = RaindexV6SubParser(deployed);

        string memory name = "RaindexV6SubParser";

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
                        type(RaindexV6SubParser).creationCode
                    ),
                    LibCodeGen.bytesConstantString(
                        vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                    )
                ),
                string.concat(
                    LibCodeGen.describedByMetaHashConstantString(vm, name),
                    LibGenParseMeta.parseMetaConstantString(
                        vm, LibRaindexSubParser.authoringMetaV2(), EXTERN_PARSE_META_BUILD_DEPTH
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
        address deployed = LibRainDeploy.deployZoltu(type(GenericPoolRaindexV6ArbOrderTaker).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "GenericPoolRaindexV6ArbOrderTaker",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function buildRouteProcessorArbOrderTakerPointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(RouteProcessorRaindexV6ArbOrderTaker).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "RouteProcessorRaindexV6ArbOrderTaker",
            string.concat(
                addressConstantString(deployed),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev The runtime bytecode of the contract.", "RUNTIME_CODE", deployed.code
                )
            )
        );
    }

    function buildGenericPoolFlashBorrowerPointers() internal {
        address deployed = LibRainDeploy.deployZoltu(type(GenericPoolRaindexV6FlashBorrower).creationCode);

        LibFs.buildFileForContract(
            vm,
            deployed,
            "GenericPoolRaindexV6FlashBorrower",
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

        buildRaindexV6Pointers();
        buildRaindexSubParserPointers();
        buildRouteProcessor4Pointers();
        buildGenericPoolArbOrderTakerPointers();
        buildRouteProcessorArbOrderTakerPointers();
        buildGenericPoolFlashBorrowerPointers();
    }
}
