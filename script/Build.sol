// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script} from "forge-std-1.16.1/src/Script.sol";
import {LibCodeGen} from "rain-sol-codegen-0.1.0/src/lib/LibCodeGen.sol";
import {LibFs} from "rain-sol-codegen-0.1.0/src/lib/LibFs.sol";
import {RaindexV6} from "../src/concrete/raindex/RaindexV6.sol";
import {RaindexV6SubParser} from "../src/concrete/parser/RaindexV6SubParser.sol";
import {LibRaindexSubParser, EXTERN_PARSE_META_BUILD_DEPTH} from "../src/lib/LibRaindexSubParser.sol";
import {LibGenParseMeta} from "rain-interpreter-interface-0.1.0/src/lib/codegen/LibGenParseMeta.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {ROUTE_PROCESSOR_4_CREATION_CODE} from "../src/lib/deploy/LibRouteProcessor4CreationCode.sol";
import {GenericPoolRaindexV6ArbOrderTaker} from "../src/concrete/arb/GenericPoolRaindexV6ArbOrderTaker.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";
import {GenericPoolRaindexV6FlashBorrower} from "../src/concrete/arb/GenericPoolRaindexV6FlashBorrower.sol";

contract Build is Script {
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

    /// @notice The canonical release tag: `foundry.toml` `[package].version` with
    /// dots converted to underscores (`0.1.13` -> `0_1_13`) for the Solidity dir
    /// form — the single source of truth for the frozen-snapshot dir name.
    function deployTag() internal view returns (string memory) {
        string memory version = vm.parseTomlString(vm.readFile("foundry.toml"), ".package.version");
        bytes memory b = bytes(version);
        bytes memory out = new bytes(b.length);
        for (uint256 i = 0; i < b.length; i++) {
            out[i] = b[i] == "." ? bytes1("_") : b[i];
        }
        return string(out);
    }

    /// @notice Freeze the just-generated flat pointers into a per-release snapshot
    /// dir `src/generated/<tag>/` so each published tag keeps its own immutable
    /// deploy pins (`LibRaindexDeployTaggedConstants.t.sol` reads these). Only the
    /// CURRENT `deployTag()` dir is ever written — older tags are never touched.
    /// An existing `<tag>/` snapshot is treated as immutable: rewriting it with
    /// IDENTICAL content is a harmless no-op, but a DIFFERENT payload reverts. That
    /// only happens when a contract changed without a `[package].version` bump — the
    /// change must bump the version so a NEW `<tag>/` dir is written beside the
    /// frozen ones, rather than corrupting the history the tests pin. The flat
    /// `src/generated/*.pointers.sol` stay the current-release source consumed
    /// across the codebase.
    function freezeSnapshot() internal {
        string memory tag = deployTag();
        vm.createDir(string.concat("src/generated/", tag), true);
        string[6] memory names;
        names[0] = "RaindexV6";
        names[1] = "RaindexV6SubParser";
        names[2] = "RouteProcessor4";
        names[3] = "GenericPoolRaindexV6ArbOrderTaker";
        names[4] = "RouteProcessorRaindexV6ArbOrderTaker";
        names[5] = "GenericPoolRaindexV6FlashBorrower";
        for (uint256 i = 0; i < names.length; i++) {
            string memory frozenPath = string.concat("src/generated/", tag, "/", names[i], ".pointers.sol");
            string memory content = vm.readFile(string.concat("src/generated/", names[i], ".pointers.sol"));
            if (vm.exists(frozenPath)) {
                require(
                    keccak256(bytes(vm.readFile(frozenPath))) == keccak256(bytes(content)),
                    "Build: frozen snapshot would change; bump [package].version for a new release"
                );
            }
            vm.writeFile(frozenPath, content);
        }
    }

    function run() external {
        LibRainDeploy.etchZoltuFactory(vm);

        buildRaindexV6Pointers();
        buildRaindexSubParserPointers();
        buildRouteProcessor4Pointers();
        buildGenericPoolArbOrderTakerPointers();
        buildRouteProcessorArbOrderTakerPointers();
        buildGenericPoolFlashBorrowerPointers();

        freezeSnapshot();
    }
}
