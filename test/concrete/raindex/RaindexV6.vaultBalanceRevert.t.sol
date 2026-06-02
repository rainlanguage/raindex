// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {
    RaindexV6,
    NegativeVaultBalance,
    NegativeVaultBalanceChange,
    NegativePull,
    NegativePush
} from "src/concrete/raindex/RaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";
import {MockToken} from "test/util/concrete/MockToken.sol";
import {LibRainDeploy} from "rain-deploy-0.1.2/src/lib/LibRainDeploy.sol";
import {LibTOFUTokenDecimals} from "rain-tofu-erc20-decimals-0.1.1/src/lib/LibTOFUTokenDecimals.sol";

/// @dev Exposes RaindexV6's internal vault-balance / token helpers for testing.
contract RaindexV6VaultBalanceHarness is RaindexV6 {
    function exposedIncrease(address owner, address token, bytes32 vaultId, Float amount)
        external
        returns (Float, Float)
    {
        return increaseVaultBalance(owner, token, vaultId, amount);
    }

    function exposedDecrease(address owner, address token, bytes32 vaultId, Float amount)
        external
        returns (Float, Float)
    {
        return decreaseVaultBalance(owner, token, vaultId, amount);
    }

    function exposedPull(address account, address token, Float amount) external returns (uint256, uint8) {
        return pullTokens(account, token, amount);
    }

    function exposedPush(address account, address token, Float amount) external returns (uint256, uint8) {
        return pushTokens(account, token, amount);
    }
}

/// @title RaindexV6VaultBalanceRevertTest
/// @notice Audit A08-5 / A08-6 (#2535): the vault-balance helpers reject a
/// negative `amount` (`NegativeVaultBalanceChange`) and a decrease that would
/// drive the balance below zero (`NegativeVaultBalance`).
contract RaindexV6VaultBalanceRevertTest is Test {
    RaindexV6VaultBalanceHarness internal harness;
    address internal owner = makeAddr("owner");
    address internal token = makeAddr("token");
    // Non-zero vault id so the internal-balance branch (not the vault-0 token
    // transfer branch) is exercised.
    bytes32 internal constant VAULT = bytes32(uint256(1));

    function setUp() external {
        // pullTokens/pushTokens read token decimals via the deployed TOFU
        // contract, so it must exist before the negative-amount check is reached.
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LibTOFUTokenDecimals.TOFU_DECIMALS_EXPECTED_CREATION_CODE);

        // Etch the runtime code so the large RaindexV6-derived harness isn't
        // subject to the EIP-170 creation size limit.
        harness = RaindexV6VaultBalanceHarness(payable(address(uint160(uint256(keccak256("vault.balance.harness"))))));
        vm.etch(address(harness), type(RaindexV6VaultBalanceHarness).runtimeCode);
    }

    /// A08-6: increase with a negative amount reverts.
    function testIncreaseNegativeAmountReverts() external {
        vm.expectRevert(
            abi.encodeWithSelector(NegativeVaultBalanceChange.selector, LibDecimalFloat.packLossless(-1, 0))
        );
        harness.exposedIncrease(owner, token, VAULT, LibDecimalFloat.packLossless(-1, 0));
    }

    /// A08-6: decrease with a negative amount reverts.
    function testDecreaseNegativeAmountReverts() external {
        vm.expectRevert(
            abi.encodeWithSelector(NegativeVaultBalanceChange.selector, LibDecimalFloat.packLossless(-1, 0))
        );
        harness.exposedDecrease(owner, token, VAULT, LibDecimalFloat.packLossless(-1, 0));
    }

    /// A08-5: decreasing below the (zero) balance reverts NegativeVaultBalance.
    function testDecreaseBelowZeroReverts() external {
        vm.expectRevert(abi.encodeWithSelector(NegativeVaultBalance.selector, LibDecimalFloat.packLossless(-1, 0)));
        harness.exposedDecrease(owner, token, VAULT, LibDecimalFloat.packLossless(1, 0));
    }

    /// A08-7: pulling a negative amount reverts NegativePull (a real token is
    /// used so the TOFU decimals read passes before the negative check).
    function testPullNegativeAmountReverts() external {
        MockToken realToken = new MockToken("Token", "TKN", 18);
        vm.expectRevert(NegativePull.selector);
        harness.exposedPull(owner, address(realToken), LibDecimalFloat.packLossless(-1, 0));
    }

    /// A08-7: pushing a negative amount reverts NegativePush.
    function testPushNegativeAmountReverts() external {
        MockToken realToken = new MockToken("Token", "TKN", 18);
        vm.expectRevert(NegativePush.selector);
        harness.exposedPush(owner, address(realToken), LibDecimalFloat.packLossless(-1, 0));
    }
}
