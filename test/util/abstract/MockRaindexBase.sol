// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import {
    IRaindexV6,
    TakeOrdersConfigV5,
    OrderConfigV4,
    ClearConfigV2,
    OrderV4,
    SignedContextV1,
    TaskV2,
    QuoteV2,
    Float
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IERC3156FlashBorrower} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";

/// @dev Base for mock raindex contracts that stub all IRaindexV6 functions.
/// Inheritors override flashLoan and/or takeOrders4 with real token transfers.
abstract contract MockRaindexBase is IRaindexV6 {
    using SafeERC20 for IERC20;

    function flashLoan(IERC3156FlashBorrower, address, uint256, bytes calldata) external virtual returns (bool) {
        return false;
    }

    function takeOrders4(TakeOrdersConfigV5 calldata) external virtual returns (Float, Float) {
        return (Float.wrap(0), Float.wrap(0));
    }

    function entask2(TaskV2[] calldata) external pure {}

    function quote2(QuoteV2 calldata) external pure returns (bool, Float, Float) {
        revert("quote");
    }

    function addOrder4(OrderConfigV4 calldata, TaskV2[] calldata) external pure returns (bool) {
        return false;
    }

    function orderExists(bytes32) external pure returns (bool) {
        return false;
    }

    function clear3(
        OrderV4 memory,
        OrderV4 memory,
        ClearConfigV2 calldata,
        SignedContextV1[] memory,
        SignedContextV1[] memory
    ) external {}

    function deposit4(address, bytes32, Float, TaskV2[] calldata) external {}
    function flashFee(address, uint256) external view returns (uint256) {}
    function maxFlashLoan(address) external view returns (uint256) {}
    function removeOrder3(OrderV4 calldata, TaskV2[] calldata) external returns (bool) {}
    function vaultBalance2(address, address, bytes32) external view returns (Float) {}
    function withdraw4(address, bytes32, Float, TaskV2[] calldata) external {}
}
