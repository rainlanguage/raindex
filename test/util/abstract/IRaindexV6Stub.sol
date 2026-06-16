// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    IRaindexV6,
    OrderConfigV4,
    OrderV4,
    ClearConfigV2,
    SignedContextV1,
    TakeOrdersConfigV5,
    EvaluableV4,
    TaskV2,
    QuoteV2,
    Float
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IERC3156FlashLender} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashLender.sol";
import {IERC3156FlashBorrower} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";

abstract contract IRaindexV6Stub is IRaindexV6 {
    /// @inheritdoc IRaindexV6
    function entask2(TaskV2[] calldata) external pure {
        revert("eval");
    }

    /// @inheritdoc IRaindexV6
    function quote2(QuoteV2 calldata) external pure returns (bool, Float, Float) {
        revert("quote");
    }

    /// @inheritdoc IRaindexV6
    function addOrder4(OrderConfigV4 calldata, TaskV2[] calldata) external pure returns (bool) {
        revert("addOrder");
    }

    /// @inheritdoc IRaindexV6
    function orderExists(bytes32) external pure returns (bool) {
        revert("orderExists");
    }

    /// @inheritdoc IRaindexV6
    function removeOrder3(OrderV4 calldata, TaskV2[] calldata) external pure returns (bool) {
        revert("removeOrder");
    }

    /// @inheritdoc IRaindexV6
    function clear3(
        OrderV4 memory,
        OrderV4 memory,
        ClearConfigV2 calldata,
        SignedContextV1[] memory,
        SignedContextV1[] memory
    ) external pure {
        revert("clear");
    }

    /// @inheritdoc IRaindexV6
    function deposit4(address, bytes32, Float, TaskV2[] calldata) external pure {
        revert("deposit");
    }

    /// @inheritdoc IRaindexV6
    function takeOrders4(TakeOrdersConfigV5 calldata) external pure returns (Float, Float) {
        revert("takeOrders");
    }

    /// @inheritdoc IRaindexV6
    function vaultBalance2(address, address, bytes32) external pure returns (Float) {
        revert("vaultBalance");
    }

    /// @inheritdoc IRaindexV6
    function withdraw4(address, bytes32, Float, TaskV2[] calldata) external pure {
        revert("withdraw");
    }

    /// @inheritdoc IERC3156FlashLender
    function flashLoan(IERC3156FlashBorrower, address, uint256, bytes calldata) external pure returns (bool) {
        revert("flashLoan");
    }

    /// @inheritdoc IERC3156FlashLender
    function flashFee(address, uint256) external pure returns (uint256) {
        revert("flashFee");
    }

    /// @inheritdoc IERC3156FlashLender
    function maxFlashLoan(address) external pure returns (uint256) {
        revert("maxFlashLoan");
    }
}
