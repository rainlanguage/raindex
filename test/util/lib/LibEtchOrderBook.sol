// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Vm} from "forge-std/Vm.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {RUNTIME_CODE as ORDERBOOK_RUNTIME_CODE} from "../../../src/generated/OrderBookV6.pointers.sol";
import {RUNTIME_CODE as SUB_PARSER_RUNTIME_CODE} from "../../../src/generated/OrderBookV6SubParser.pointers.sol";
import {RUNTIME_CODE as ROUTE_PROCESSOR_RUNTIME_CODE} from "../../../src/generated/RouteProcessor4.pointers.sol";

/// @title LibEtchOrderBook
/// @notice Etches the runtime bytecode of the orderbook, sub parser, and
/// route processor at their expected deterministic addresses.
library LibEtchOrderBook {
    /// @notice Etches the runtime bytecode of the orderbook, sub parser, and
    /// route processor at their expected deterministic addresses. Skips any
    /// contract whose codehash already matches.
    /// @param vm The Forge `Vm` cheatcode interface.
    function etchOrderBook(Vm vm) internal {
        if (LibOrderBookDeploy.ORDERBOOK_DEPLOYED_CODEHASH != LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS.codehash) {
            vm.etch(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS, ORDERBOOK_RUNTIME_CODE);
        }
        if (LibOrderBookDeploy.SUB_PARSER_DEPLOYED_CODEHASH != LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS.codehash)
        {
            vm.etch(LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS, SUB_PARSER_RUNTIME_CODE);
        }
        if (
            LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_CODEHASH
                != LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS.codehash
        ) {
            vm.etch(LibOrderBookDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS, ROUTE_PROCESSOR_RUNTIME_CODE);
        }
    }
}
