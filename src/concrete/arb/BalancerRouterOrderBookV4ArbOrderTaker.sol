// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {Address} from "openzeppelin-contracts/contracts/utils/Address.sol";

import {OrderBookV4ArbOrderTaker, OrderBookV4ArbConfigV2} from "../../abstract/OrderBookV4ArbOrderTaker.sol";

// from balancer IBatchRouter interface
// https://github.com/balancer/balancer-v3-monorepo/blob/main/pkg/interfaces/contracts/vault/IBatchRouter.sol
struct SwapPathStep {
    address pool;
    IERC20 tokenOut;
    // If true, the "pool" is an ERC4626 Buffer. Used to wrap/unwrap tokens if pool doesn't have enough liquidity.
    bool isBuffer;
}

// from balancer IBatchRouter interface
// https://github.com/balancer/balancer-v3-monorepo/blob/main/pkg/interfaces/contracts/vault/IBatchRouter.sol
struct SwapPathExactAmountIn {
    IERC20 tokenIn;
    // for each step:
    // if tokenIn == pool use removeLiquidity SINGLE_TOKEN_EXACT_IN
    // if tokenOut == pool use addLiquidity UNBALANCED
    SwapPathStep[] steps;
    uint256 exactAmountIn;
    uint256 minAmountOut;
}

// balancer IBatchRouter interface from balancer contracts
// https://github.com/balancer/balancer-v3-monorepo/blob/main/pkg/interfaces/contracts/vault/IBatchRouter.sol
interface IBatchRouter {
    /**
    * @notice Executes a swap operation involving multiple paths (steps), specifying exact input token amounts.
    * @param paths Swap paths from token in to token out, specifying exact amounts in.
    * @param deadline Deadline for the swap
    * @param wethIsEth If true, incoming ETH will be wrapped to WETH; otherwise the Vault will pull WETH tokens
    * @param userData Additional (optional) data required for the swap
    * @return pathAmountsOut Calculated amounts of output tokens corresponding to the last step of each given path
    * @return tokensOut Calculated output token addresses
    * @return amountsOut Calculated amounts of output tokens, ordered by output token address
    */
    function swapExactIn(
        SwapPathExactAmountIn[] memory paths,
        uint256 deadline,
        bool wethIsEth,
        bytes calldata userData
    ) external payable returns (uint256[] memory pathAmountsOut, address[] memory tokensOut, uint256[] memory amountsOut);
}

// permit2 interface from uniswap
// https://github.com/Uniswap/permit2/blob/main/src/interfaces/IPermit2.sol
interface IPermit2 {
    function approve(address token, address spender, uint160 amount, uint48 expiration) external;
}

contract BalancerRouterOrderBookV4ArbOrderTaker is OrderBookV4ArbOrderTaker {
    using SafeERC20 for IERC20;
    using Address for address;

    // permit2 address onchain, same for every chain
    IPermit2 public immutable iPermit2 = IPermit2(0x000000000022D473030F116dDEE9F6B43aC78BA3);

    constructor(OrderBookV4ArbConfigV2 memory config) OrderBookV4ArbOrderTaker(config) {}

    /// @inheritdoc OrderBookV4ArbOrderTaker
    function onTakeOrders(
        address inputToken,
        address outputToken,
        uint256 inputAmountSent,
        uint256 totalOutputAmount,
        bytes calldata takeOrdersData
    ) public virtual override {
        super.onTakeOrders(inputToken, outputToken, inputAmountSent, totalOutputAmount, takeOrdersData);
        (address _balancerRouter, SwapPathExactAmountIn memory route) = abi.decode(
            takeOrdersData,
            (address, SwapPathExactAmountIn)
        );

        // validate the swap tokens
        require(address(route.tokenIn) == inputToken, "Input token of the given balancer route doesnt match the order's IO");
        require(address(route.steps[route.steps.length - 1].tokenOut) == outputToken, "Output token of the given balancer route doesnt match the order's IO");

        // approve permit2 for the input token
        IERC20(inputToken).approve(address(iPermit2), type(uint256).max);
        iPermit2.approve(inputToken, _balancerRouter, type(uint160).max, 0); // 0 expiration means no expiration

        IBatchRouter balancerRouter = IBatchRouter(_balancerRouter);

        route.exactAmountIn = inputAmountSent;
        route.minAmountOut = totalOutputAmount;
        SwapPathExactAmountIn[] memory batchRoute = new SwapPathExactAmountIn[](1);
        batchRoute[0] = route;

        (uint256[] memory pathAmountsOut, address[] memory tokensOut, uint256[] memory amountsOut) = balancerRouter.swapExactIn(batchRoute, type(uint256).max, false, "0x");
        (pathAmountsOut, tokensOut, amountsOut);

        IERC20(inputToken).approve(address(iPermit2), 0);
    }

    /// Allow receiving gas.
    fallback() external {}
}
