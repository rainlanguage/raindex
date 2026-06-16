// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std/Test.sol";
import {IERC165} from "openzeppelin-contracts/contracts/utils/introspection/IERC165.sol";
import {OrderBookV6FlashLender, IERC3156FlashLender} from "../../src/abstract/OrderBookV6FlashLender.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract.
contract ChildOrderBookV6FlashLender is OrderBookV6FlashLender {
    constructor() OrderBookV6FlashLender() {}
}

contract OrderBookV6FlashLenderIERC165Test is Test {
    /// Test that ERC165 and IERC3156FlashLender are supported interfaces
    /// as per ERC165.
    function testOrderBookV6FlashLenderIERC165(bytes4 badInterfaceId) external {
        vm.assume(badInterfaceId != type(IERC165).interfaceId);
        vm.assume(badInterfaceId != type(IERC3156FlashLender).interfaceId);

        ChildOrderBookV6FlashLender flashLender = new ChildOrderBookV6FlashLender();
        assertTrue(flashLender.supportsInterface(type(IERC165).interfaceId));
        assertTrue(flashLender.supportsInterface(type(IERC3156FlashLender).interfaceId));
        assertFalse(flashLender.supportsInterface(badInterfaceId));
    }
}
