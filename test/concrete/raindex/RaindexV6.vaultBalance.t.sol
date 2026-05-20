// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6ExternalMockTest} from "test/util/abstract/RaindexV6ExternalMockTest.sol";
import {LibDecimalFloat, Float} from "rain.math.float/lib/LibDecimalFloat.sol";
import {IRaindexV6} from "rain.raindex.interface/interface/IRaindexV6.sol";

/// @title RaindexV6VaultBalanceTest
/// Tests the basic functionality of reading from the vault balance.
contract RaindexV6VaultBalanceTest is RaindexV6ExternalMockTest {
    using LibDecimalFloat for Float;

    /// Test that reading the vault balance without deposits is always zero.
    function testVaultBalanceNoDeposits(address owner, address token, bytes32 vaultId) external view {
        vm.assume(vaultId != bytes32(0));
        assertTrue(iRaindex.vaultBalance2(owner, token, vaultId).isZero());
    }
}
