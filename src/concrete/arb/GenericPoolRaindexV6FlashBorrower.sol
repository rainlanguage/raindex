// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6FlashBorrower, TakeOrdersConfigV5} from "../../abstract/RaindexV6FlashBorrower.sol";
import {LibGenericPoolExchange} from "../../lib/LibGenericPoolExchange.sol";

/// @title GenericPoolRaindexV6FlashBorrower
/// @notice Flash-loan arb that swaps via an arbitrary external pool call.
/// @dev Implements the RaindexV6FlashBorrower interface for an external
/// liquidity source that behaves vaguely like a standard AMM. The
/// `exchangeData` from `arb` is decoded into a spender, pool and callData.
/// The `callData` is literally the encoded function call to the pool. This
/// allows the `arb` caller to process a trade against any liquidity source
/// that can swap tokens within a single function call.
/// The `spender` is the address that will be approved to spend the input token
/// on `takeOrders`, which is almost always going to be the pool itself. If you
/// are unsure, simply set it to the pool address.
contract GenericPoolRaindexV6FlashBorrower is RaindexV6FlashBorrower {
    constructor() {}

    /// @inheritdoc RaindexV6FlashBorrower
    /// @dev Decodes `exchangeData` as `(spender, pool, encodedFunctionCall)`
    /// and routes the swap through the specified pool via `LibGenericPoolExchange`.
    function _exchange(TakeOrdersConfigV5 memory takeOrders, bytes memory exchangeData) internal virtual override {
        address borrowedToken = takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token;
        LibGenericPoolExchange.exchange(borrowedToken, exchangeData);
    }

    /// Allow arbitrary calls and ETH transfers to this contract without
    /// reverting. Any ETH received is swept to msg.sender by finalizeArb.
    receive() external payable {}
    fallback() external payable {}
}
