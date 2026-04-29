// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {ArbTest} from "./ArbTest.sol";
import {
    RouteProcessorOrderBookV6ArbOrderTaker
} from "../../../src/concrete/arb/RouteProcessorOrderBookV6ArbOrderTaker.sol";

contract RouteProcessorOrderBookV6ArbOrderTakerTest is ArbTest {
    function buildArb() internal override returns (address payable) {
        return payable(address(new RouteProcessorOrderBookV6ArbOrderTaker()));
    }

    constructor() ArbTest() {}
}
