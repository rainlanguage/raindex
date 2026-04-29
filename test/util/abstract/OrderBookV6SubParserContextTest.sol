// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {StackAllocationMismatch} from "rain.interpreter/error/ErrIntegrity.sol";
import {OpTest, StackItem} from "rain.interpreter/../test/abstract/OpTest.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {LibOrderBookSubParserContextFixture} from "test/util/fixture/LibOrderBookSubParserContextFixture.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {LibEtchOrderBook} from "test/util/lib/LibEtchOrderBook.sol";

abstract contract OrderBookV6SubParserContextTest is OpTest {
    using Strings for address;

    function setUp() public {
        LibEtchOrderBook.etchOrderBook(vm);
    }

    function word() internal pure virtual returns (string memory);

    function testSubParserContextHappy() external view {
        string memory w = word();
        address subParserAddress = LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS;

        StackItem[] memory expectedStack = new StackItem[](1);
        expectedStack[0] = StackItem.wrap(keccak256(bytes(w)));

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", subParserAddress.toHexString(), " _: ", w, "();"));

        checkHappy(rainlang, LibOrderBookSubParserContextFixture.hashedNamesContext(), expectedStack, w);
    }

    function testSubParserContextUnhappyDisallowedOperand() external {
        string memory w = word();
        address subParserAddress = LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS;

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", subParserAddress.toHexString(), " _: ", w, "<1>();"));

        checkDisallowedOperand(rainlang);
    }

    function testSubParserContextUnhappyDisallowedInputs() external {
        string memory w = word();
        address subParserAddress = LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS;

        bytes memory rainlang =
            bytes(string.concat("using-words-from ", subParserAddress.toHexString(), " _: ", w, "(1);"));

        checkUnhappyParse2(rainlang, abi.encodeWithSelector(StackAllocationMismatch.selector, 2, 1));
    }
}
