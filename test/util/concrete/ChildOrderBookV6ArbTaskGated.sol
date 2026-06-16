// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ArbTaskGated, OrderBookV6ArbConfig} from "../../../src/abstract/OrderBookV6ArbTaskGated.sol";
import {TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";

/// @dev We need a contract that is deployable in order to test the abstract
/// base contract. Exposes `_checkTaskHash` as an external function.
contract ChildOrderBookV6ArbTaskGated is OrderBookV6ArbTaskGated {
    constructor(OrderBookV6ArbConfig memory config) OrderBookV6ArbTaskGated(config) {}

    function checkTaskHash(TaskV2 memory task) external view {
        _checkTaskHash(task);
    }
}
