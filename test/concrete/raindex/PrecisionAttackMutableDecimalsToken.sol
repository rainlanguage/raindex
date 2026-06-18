// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/ERC20.sol";

contract PrecisionAttackMutableDecimalsToken is ERC20 {
    uint8 public dec;

    constructor(uint8 d) ERC20("Mut", "MUT") {
        dec = d;
    }

    function setDecimals(uint8 d) external {
        dec = d;
    }

    function decimals() public view override returns (uint8) {
        return dec;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
