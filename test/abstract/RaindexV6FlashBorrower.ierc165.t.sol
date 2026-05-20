// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {RaindexV6FlashBorrower, IERC3156FlashBorrower} from "../../src/abstract/RaindexV6FlashBorrower.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildRaindexV6FlashBorrower is RaindexV6FlashBorrower {
    constructor() {}
}

contract RaindexV6FlashBorrowerIERC165Test is Test {
    /// Test that ERC165 and IERC3156FlashBorrower are supported interfaces
    /// as per ERC165.
    function testRaindexV6FlashBorrowerIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IERC3156FlashBorrower).interfaceId);

        ChildRaindexV6FlashBorrower flashBorrower = new ChildRaindexV6FlashBorrower();
        assertTrue(flashBorrower.supportsInterface(type(IERC165).interfaceId));
        assertTrue(flashBorrower.supportsInterface(type(IERC3156FlashBorrower).interfaceId));
        assertFalse(flashBorrower.supportsInterface(badInterfaceId));
    }
}
