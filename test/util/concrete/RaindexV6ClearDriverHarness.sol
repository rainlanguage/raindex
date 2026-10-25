// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6} from "src/concrete/raindex/RaindexV6.sol";
import {IRaindexV6ClearDriverHarness} from "test/util/interface/IRaindexV6ClearDriverHarness.sol";
import {Float} from "rain-math-float-0.3.10/src/lib/LibDecimalFloat.sol";

/// @dev RaindexV6 subclass that exposes order-liveness and vault seeding so a
/// full `clear4` can be driven against fresh-compiled source bytecode.
contract RaindexV6ClearDriverHarness is RaindexV6, IRaindexV6ClearDriverHarness {
    function setOrderLive(bytes32 orderHash) external {
        sOrders[orderHash] = 1;
    }

    function seedVault(address owner, address token, bytes32 vaultId, Float amount) external {
        sVaultBalances[owner][token][vaultId] = amount;
    }
}
