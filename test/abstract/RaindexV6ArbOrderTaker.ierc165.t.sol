// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {IERC165} from "@openzeppelin-contracts-5.7.0/utils/introspection/IERC165.sol";
import {IRaindexV7ArbOrderTaker} from "raindex-interface-0.1.11/src/interface/IRaindexV7ArbOrderTaker.sol";
import {IRaindexV7OrderTaker} from "raindex-interface-0.1.11/src/interface/IRaindexV7OrderTaker.sol";
import {ChildRaindexV6ArbOrderTaker} from "../util/concrete/ChildRaindexV6ArbOrderTaker.sol";

contract RaindexV6ArbOrderTakerIERC165Test is Test {
    /// Test that ERC165 and IRaindexV7ArbOrderTaker are supported interfaces
    /// as per ERC165.
    function testRaindexV6ArbOrderTakerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IRaindexV7ArbOrderTaker).interfaceId);
        vm.assume(badInterfaceId != type(IRaindexV7OrderTaker).interfaceId);

        ChildRaindexV6ArbOrderTaker arbOrderTaker = new ChildRaindexV6ArbOrderTaker();
        assertTrue(arbOrderTaker.supportsInterface(type(IERC165).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IRaindexV7ArbOrderTaker).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IRaindexV7OrderTaker).interfaceId));
        assertFalse(arbOrderTaker.supportsInterface(badInterfaceId));
    }
}
