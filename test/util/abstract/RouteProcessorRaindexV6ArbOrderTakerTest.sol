// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {RouteProcessorRaindexV6ArbOrderTaker} from "../../../src/concrete/arb/RouteProcessorRaindexV6ArbOrderTaker.sol";

contract RouteProcessorRaindexV6ArbOrderTakerTest is ArbTest {
    function buildArb() internal override returns (address payable) {
        return payable(address(new RouteProcessorRaindexV6ArbOrderTaker()));
    }

    constructor() ArbTest() {}
}
