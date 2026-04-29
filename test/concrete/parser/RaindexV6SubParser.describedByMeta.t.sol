// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {RaindexV6SubParser} from "../../../src/concrete/parser/RaindexV6SubParser.sol";
import {LibRaindexDeploy} from "../../../src/lib/deploy/LibRaindexDeploy.sol";
import {LibEtchRaindex} from "test/util/lib/LibEtchRaindex.sol";

contract RaindexV6SubParserDescribedByMetaV1Test is Test {
    function setUp() public {
        LibEtchRaindex.etchRaindex(vm);
    }

    function testRaindexV6SubParserDescribedByMetaV1Happy() external view {
        bytes memory describedByMeta = vm.readFileBinary("meta/RaindexV6SubParser.rain.meta");
        RaindexV6SubParser subParser = RaindexV6SubParser(LibRaindexDeploy.SUB_PARSER_DEPLOYED_ADDRESS);

        assertEq(keccak256(describedByMeta), subParser.describedByMetaV1());
    }
}
