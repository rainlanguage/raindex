// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ArbOrderTaker, Float} from "../../abstract/RaindexV6ArbOrderTaker.sol";
import {LibGenericPoolExchange} from "../../lib/LibGenericPoolExchange.sol";

/// @title GenericPoolRaindexV6ArbOrderTaker
/// @notice Order-taker arb that swaps via an arbitrary external pool call.
/// The `takeOrdersData` is decoded as `(spender, pool, encodedFunctionCall)`.
contract GenericPoolRaindexV6ArbOrderTaker is RaindexV6ArbOrderTaker {
    constructor() {}

    /// @inheritdoc RaindexV6ArbOrderTaker
    /// @dev Decodes `takeOrdersData` as `(spender, pool, encodedFunctionCall)`
    /// and routes the swap through the specified pool via `LibGenericPoolExchange`.
    function onTakeOrders2(
        address inputToken,
        address outputToken,
        Float inputAmountSent,
        Float totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders2(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
        LibGenericPoolExchange.exchange(inputToken, takeOrdersData);
    }

    /// Allow arbitrary calls and ETH transfers to this contract without
    /// reverting. Any ETH received is swept to msg.sender by finalizeArb.
    receive() external payable {}
    fallback() external payable {}
}
