// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.0;

/// @title IRouteProcessor
/// @notice Minimal ABI surface for Sushi's RouteProcessor, vendored here so
/// raindex does not depend on the `sushixswap-v2` git submodule. The signatures
/// match Sushi's `IRouteProcessor` so the deterministic RouteProcessor4
/// deployment can be called for ABI compatibility.
interface IRouteProcessor {
    struct RouteProcessorData {
        address tokenIn;
        uint256 amountIn;
        address tokenOut;
        uint256 amountOutMin;
        address to;
        bytes route;
    }

    /// @notice Process a swap with passed route on RouteProcessor
    /// @param tokenIn The address of the token to swap from
    /// @param amountIn The amount of token to swap from
    /// @param tokenOut The address of the token to swap to
    /// @param amountOutMin The minimum amount of token to receive
    /// @param to The address to send the swapped token to
    /// @param route The route to use for the swap
    function processRoute(
        address tokenIn,
        uint256 amountIn,
        address tokenOut,
        uint256 amountOutMin,
        address to,
        bytes memory route
    ) external payable returns (uint256 amountOut);
}
