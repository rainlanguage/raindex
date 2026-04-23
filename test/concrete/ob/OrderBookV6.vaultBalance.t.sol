// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookV6ExternalMockTest} from "test/util/abstract/OrderBookV6ExternalMockTest.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";
import {LibOrderBookDeploy} from "../../../src/lib/deploy/LibOrderBookDeploy.sol";

/// @title OrderBookV6VaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract OrderBookV6VaultBalanceTest is OrderBookV6ExternalMockTest {
    using LibDecimalFloat for Float;

    /// Test that reading the vault balance without deposits is always zero.
    function testVaultBalanceNoDeposits(address owner, address token, bytes32 vaultId) external view {
        vm.assume(vaultId != bytes32(0));
        assertTrue(
            IRaindexV6(LibOrderBookDeploy.ORDERBOOK_DEPLOYED_ADDRESS).vaultBalance2(owner, token, vaultId).isZero()
        );
    }
}
