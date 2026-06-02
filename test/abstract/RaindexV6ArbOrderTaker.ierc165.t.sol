// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {IERC165} from "@openzeppelin-contracts-5.6.1/utils/introspection/IERC165.sol";
import {IRaindexV6ArbOrderTaker} from "raindex-interface-0.1.1/src/interface/IRaindexV6ArbOrderTaker.sol";
import {IRaindexV6OrderTaker} from "raindex-interface-0.1.1/src/interface/IRaindexV6OrderTaker.sol";
import {ChildRaindexV6ArbOrderTaker} from "../util/concrete/ChildRaindexV6ArbOrderTaker.sol";

contract RaindexV6ArbOrderTakerIERC165Test is Test {
    /// Test that ERC165 and IRaindexV6ArbOrderTaker are supported interfaces
    /// as per ERC165.
    function testRaindexV6ArbOrderTakerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IRaindexV6ArbOrderTaker).interfaceId);
        vm.assume(badInterfaceId != type(IRaindexV6OrderTaker).interfaceId);

        ChildRaindexV6ArbOrderTaker arbOrderTaker = new ChildRaindexV6ArbOrderTaker();
        assertTrue(arbOrderTaker.supportsInterface(type(IERC165).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IRaindexV6ArbOrderTaker).interfaceId));
        assertTrue(arbOrderTaker.supportsInterface(type(IRaindexV6OrderTaker).interfaceId));
        assertFalse(arbOrderTaker.supportsInterface(badInterfaceId));
    }
}
