// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {RaindexV6FlashLender, IERC3156FlashLender} from "../../src/abstract/RaindexV6FlashLender.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildRaindexV6FlashLender is RaindexV6FlashLender {
    constructor() RaindexV6FlashLender() {}
}

contract RaindexV6FlashLenderIERC165Test is Test {
    /// Test that ERC165 and IERC3156FlashLender are supported interfaces
    /// as per ERC165.
    function testRaindexV6FlashLenderIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IERC3156FlashLender).interfaceId);

        ChildRaindexV6FlashLender flashLender = new ChildRaindexV6FlashLender();
        assertTrue(flashLender.supportsInterface(type(IERC165).interfaceId));
        assertTrue(flashLender.supportsInterface(type(IERC3156FlashLender).interfaceId));
        assertFalse(flashLender.supportsInterface(badInterfaceId));
    }
}
