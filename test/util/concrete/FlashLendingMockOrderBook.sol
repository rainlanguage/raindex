// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {
    TakeOrderConfigV4,
    IOV2,
    OrderV4,
    SignedContextV1,
    IRaindexV6,
    TakeOrdersConfigV5,
    OrderConfigV4,
    ClearConfigV2,
    EvaluableV4,
    TaskV2,
    QuoteV2,
    Float
} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {IERC3156FlashBorrower} from "rain.raindex.interface/interface/ierc3156/IERC3156FlashBorrower.sol";

contract FlashLendingMockOrderBook is IRaindexV6 {
    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data)
        external
        returns (bool)
    {
        receiver.onFlashLoan(msg.sender, token, amount, 0, data);
        return true;
    }

    function entask2(TaskV2[] calldata) external pure {}

    /// @inheritdoc IRaindexV6
    function quote2(QuoteV2 calldata) external pure returns (bool, Float, Float) {
        revert("quote");
    }

    /// @inheritdoc IRaindexV6
    function takeOrders4(TakeOrdersConfigV5 calldata) external pure returns (Float, Float) {}

    /// @inheritdoc IRaindexV6
    function addOrder4(OrderConfigV4 calldata, TaskV2[] calldata) external pure returns (bool) {
        return false;
    }

    function orderExists(bytes32) external pure returns (bool) {
        return false;
    }

    /// @inheritdoc IRaindexV6
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
