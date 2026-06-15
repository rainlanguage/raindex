// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RaindexV6} from "src/concrete/raindex/RaindexV6.sol";
import {Float} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @dev RaindexV6 subclass that exposes order-liveness and vault seeding so a
/// full `clear3` can be driven against fresh-compiled source bytecode.
contract RaindexV6ClearDriverHarness is RaindexV6 {
    function setOrderLive(bytes32 orderHash) external {
        sOrders[orderHash] = 1;
    }

    function seedVault(address owner, address token, bytes32 vaultId, Float amount) external {
        sVaultBalances[owner][token][vaultId] = amount;
    }
}
