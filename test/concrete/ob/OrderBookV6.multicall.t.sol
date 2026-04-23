// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {Multicall} from "openzeppelin-contracts/contracts/utils/Multicall.sol";
import {OrderBookV6ExternalRealTest} from "test/util/abstract/OrderBookV6ExternalRealTest.sol";
import {IRaindexV6, TaskV2} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";
import {Float, LibDecimalFloat} from "rain.math.float/lib/LibDecimalFloat.sol";

/// Test that the Multicall inherited from OpenZeppelin works correctly on the
/// OrderBook, allowing multiple deposit calls in a single transaction.
contract OrderBookV6MulticallTest is OrderBookV6ExternalRealTest {
    function testMulticallDeposits() external {
        address alice = address(uint160(uint256(keccak256("alice.rain.test"))));

        // Mock transferFrom for both tokens.
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS),
            abi.encode(true)
        );

        // Encode two deposit4 calls.
        bytes[] memory calls = new bytes[](2);
        calls[0] = abi.encodeWithSelector(
            IRaindexV6.deposit4.selector,
            address(iToken0),
            bytes32(uint256(0x01)),
            LibDecimalFloat.packLossless(10, 0),
            new TaskV2[](0)
        );
        calls[1] = abi.encodeWithSelector(
            IRaindexV6.deposit4.selector,
            address(iToken1),
            bytes32(uint256(0x02)),
            LibDecimalFloat.packLossless(20, 0),
            new TaskV2[](0)
        );

        vm.prank(alice);
        Multicall(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).multicall(calls);

        // Verify both vault balances were set.
        assertEq(
            Float.unwrap(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(alice, address(iToken0), bytes32(uint256(0x01)))
            ),
            Float.unwrap(LibDecimalFloat.packLossless(10, 0))
        );
        assertEq(
            Float.unwrap(
                IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS)
                    .vaultBalance2(alice, address(iToken1), bytes32(uint256(0x02)))
            ),
            Float.unwrap(LibDecimalFloat.packLossless(20, 0))
        );
    }
}
