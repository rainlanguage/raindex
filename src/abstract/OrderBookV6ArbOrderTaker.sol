// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.19;

import {ERC165, IERC165} from "openzeppelin-contracts/contracts/utils/introspection/ERC165.sol";
import {ReentrancyGuard} from "openzeppelin-contracts/contracts/utils/ReentrancyGuard.sol";
import {IERC20, SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IRaindexV6ArbOrderTaker, TaskV2} from "rain.raindex.interface/interface/IRaindexV6ArbOrderTaker.sol";
import {TakeOrdersConfigV5, Float} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {OrderBookV6ArbCommon} from "./OrderBookV6ArbCommon.sol";
import {LibOrderBookArb} from "../lib/LibOrderBookArb.sol";
import {IRaindexV6OrderTaker} from "rain.raindex.interface/interface/IRaindexV6OrderTaker.sol";
import {LibTOFUTokenDecimals} from "rain.tofu.erc20-decimals/lib/LibTOFUTokenDecimals.sol";

/// @title OrderBookV6ArbOrderTaker
/// @notice Arb contract that takes orders directly from an `OrderBook` without
/// flash loans. Inheritors implement the strategy for sourcing the input token
/// (e.g. routing through a DEX).
abstract contract OrderBookV6ArbOrderTaker is
    IRaindexV6OrderTaker,
    IRaindexV6ArbOrderTaker,
    ReentrancyGuard,
    ERC165,
    OrderBookV6ArbCommon
{
    using SafeERC20 for IERC20;

    constructor() {}

    /// @inheritdoc IERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return (interfaceId == type(IRaindexV6OrderTaker).interfaceId)
            || (interfaceId == type(IRaindexV6ArbOrderTaker).interfaceId) || super.supportsInterface(interfaceId);
    }

    /// @inheritdoc IRaindexV6ArbOrderTaker
    function arb5(IRaindexV6 orderBook, TakeOrdersConfigV5 calldata takeOrders, TaskV2 calldata task)
        external
        payable
        nonReentrant
    {
        _beforeArb(task);
        // Mimic what OB would do anyway if called with zero orders.
        if (takeOrders.orders.length == 0) {
            revert IRaindexV6.NoOrders();
        }

        address ordersInputToken = takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token;
        address ordersOutputToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;

        IERC20(ordersInputToken).forceApprove(address(orderBook), type(uint256).max);
        //slither-disable-next-line unused-return
        orderBook.takeOrders4(takeOrders);
        IERC20(ordersInputToken).forceApprove(address(orderBook), 0);

        LibOrderBookArb.finalizeArb(
            task,
            ordersInputToken,
            LibTOFUTokenDecimals.safeDecimalsForToken(ordersInputToken),
            ordersOutputToken,
            LibTOFUTokenDecimals.safeDecimalsForToken(ordersOutputToken)
        );
    }

    /// @inheritdoc IRaindexV6OrderTaker
    /// @dev Empty no-op. The contract holds no value between operations and the
    /// caller chooses which orderbook to interact with, so there is nothing to
    /// protect via `msg.sender` validation here.
    function onTakeOrders2(address, address, Float, Float, bytes calldata) public virtual override {}
}
