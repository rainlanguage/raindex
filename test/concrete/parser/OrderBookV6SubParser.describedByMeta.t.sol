// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {OrderBookV6SubParser} from "../../../src/concrete/parser/OrderBookV6SubParser.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibEtchOrderBook} from "test/util/lib/LibEtchOrderBook.sol";

contract OrderBookV6SubParserDescribedByMetaV1Test is Test {
    function setUp() public {
        LibEtchOrderBook.etchOrderBook(vm);
    }

    function testOrderBookV6SubParserDescribedByMetaV1Happy() external view {
        bytes memory describedByMeta = vm.readFileBinary("meta/OrderBookV6SubParser.rain.meta");
        OrderBookV6SubParser subParser = OrderBookV6SubParser(LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);

        assertEq(keccak256(describedByMeta), subParser.describedByMetaV1());
    }
}
