// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IRouteProcessor} from "sushixswap-v2/src/interfaces/IRouteProcessor.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";

import {RaindexV6ArbOrderTaker, Float} from "../../abstract/RaindexV6ArbOrderTaker.sol";
import {LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {LibRaindexDeploy} from "../../lib/deploy/LibRaindexDeploy.sol";

/// @title RouteProcessorRaindexV6ArbOrderTaker
/// @notice Order-taker arb that swaps via the deterministic Sushi
/// RouteProcessor4 deployment.
contract RouteProcessorRaindexV6ArbOrderTaker is RaindexV6ArbOrderTaker {
    using SafeERC20 for IERC20;

    constructor() {}

    /// @inheritdoc RaindexV6ArbOrderTaker
    function onTakeOrders2(
        address inputToken,
        address outputToken,
        Float inputAmountSent,
        Float totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders2(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
        address routeProcessor = LibRaindexDeploy.ROUTE_PROCESSOR_DEPLOYED_ADDRESS;
        IERC20(inputToken).forceApprove(routeProcessor, type(uint256).max);
        bytes memory route = abi.decode(takeOrdersData, (bytes));
        // Input amount precision loss is acceptable as the route processor
        // only needs an approximate amount to execute the swap.
        //slither-disable-next-line unused-return
        (uint256 inputTokenAmount,) =
            LibDecimalFloat.toFixedDecimalLossy(inputAmountSent, IERC20Metadata(inputToken).decimals());
        (uint256 outputTokenAmount, bool lossless) =
            LibDecimalFloat.toFixedDecimalLossy(totalOutputAmount, IERC20Metadata(outputToken).decimals());
        if (!lossless) {
            outputTokenAmount++;
        }
        //slither-disable-next-line unused-return
        IRouteProcessor(routeProcessor)
            .processRoute(inputToken, inputTokenAmount, outputToken, outputTokenAmount, address(this), route);
        IERC20(inputToken).forceApprove(routeProcessor, 0);
    }

    /// Allow arbitrary calls and ETH transfers to this contract without
    /// reverting. Any ETH received is swept to msg.sender by finalizeArb.
    receive() external payable {}
    fallback() external payable {}
}
