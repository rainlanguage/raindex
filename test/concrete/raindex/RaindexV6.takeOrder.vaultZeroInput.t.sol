// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {IERC20} from "@openzeppelin-contracts-5.6.1/token/ERC20/IERC20.sol";
import {IERC20Errors} from "@openzeppelin-contracts-5.6.1/interfaces/draft-IERC6093.sol";
import {RaindexV6ExternalRealTest} from "test/util/abstract/RaindexV6ExternalRealTest.sol";
import {LibTestTakeOrder} from "test/util/lib/LibTestTakeOrder.sol";
import {
    OrderConfigV4,
    OrderV4,
    TakeOrderConfigV4,
    TakeOrdersConfigV5,
    ClearConfigV2,
    SignedContextV1,
    IOV2,
    EvaluableV4,
    TaskV2
} from "raindex-interface-0.1.1/src/interface/IRaindexV6.sol";
import {Float, LibDecimalFloat} from "rain-math-float-0.1.1/src/lib/LibDecimalFloat.sol";

/// @title RaindexV6TakeOrderVaultZeroInputTest
/// @notice Audit Protofire M01 (#2618): `vaultId == 0` is "vaultless" mode,
/// where settlement is a direct wallet transfer rather than an internal vault
/// balance. `recordVaultIO` runs `increaseVaultBalance` (credit the order
/// owner's INPUT) before `decreaseVaultBalance` (debit the OUTPUT). For a
/// `vaultId == 0` INPUT this credit is a direct `pushTokens` (ERC20 `transfer`)
/// of the input token to the owner. But in `takeOrders4` the taker only pays the
/// input token at the very END (`pullTokens(msg.sender, ...)` after the per-order
/// loop), and in `clear3` Alice's `recordVaultIO` runs before Bob's. So Raindex
/// transfers the input token to the owner while holding none of it: a logically
/// fundable trade reverts unless Raindex already holds ambient balance of that
/// token from unrelated positions.
///
/// The vault-0 token movements are mocked: the input push to the owner is left
/// unmocked (so it reverts against the unfunded etched token) for the "no
/// ambient balance" cases, and mocked to succeed for the "ambient balance" case.
/// #2435 (rouzwelt, pre-rename) first reproduced this; same bug on current
/// `RaindexV6`.
contract RaindexV6TakeOrderVaultZeroInputTest is RaindexV6ExternalRealTest {
    using LibDecimalFloat for Float;

    address internal owner = address(uint160(uint256(keccak256("owner.rain.test"))));
    address internal bob = address(uint160(uint256(keccak256("bob.rain.test"))));
    address internal alice = address(uint160(uint256(keccak256("alice.rain.test"))));

    bytes32 internal constant OUTPUT_VAULT_ID = bytes32(uint256(0x01));

    /// Mock the calculate-phase reads for a vault-0 (wallet) IO: `balanceOf` and
    /// `allowance` of `who` for `token`.
    function _mockVault0Reads(address token, address who, uint256 balance) internal {
        vm.mockCall(token, abi.encodeWithSelector(IERC20.balanceOf.selector, who), abi.encode(balance));
        vm.mockCall(
            token, abi.encodeWithSelector(IERC20.allowance.selector, who, address(iRaindex)), abi.encode(balance)
        );
    }

    /// Build an order owned by `owner` whose INPUT is `iToken0` at `vaultId == 0`
    /// (vaultless) and OUTPUT is `iToken1` at a normal vault, funded so output is
    /// available. Mocks the calculate reads and the two settlement legs that
    /// always succeed (order output to the taker, and the taker's after-loop
    /// payment pull), so the ONLY thing that can fail is the in-loop push of the
    /// input token to the owner.
    function _buildVaultZeroInputOrder() internal returns (TakeOrdersConfigV5 memory) {
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, owner, address(iRaindex)),
            abi.encode(true)
        );
        vm.prank(owner);
        iRaindex.deposit4(address(iToken1), OUTPUT_VAULT_ID, LibDecimalFloat.packLossless(10, 0), new TaskV2[](0));

        // Vault-0 input read for the owner's wallet (zero balance is fine; the
        // outputMax is capped by the OUTPUT vault, not this).
        _mockVault0Reads(address(iToken0), owner, 0);

        OrderV4 memory order = LibTestTakeOrder.addOrderWithExpression(
            vm, owner, "_ _: 1 1;:;", address(iToken0), bytes32(0), address(iToken1), OUTPUT_VAULT_ID
        );

        // Order output to the taker, and the taker's after-loop payment pull.
        // Both succeed: the taker is willing and able, so the trade is fundable.
        vm.mockCall(address(iToken1), abi.encodeWithSelector(IERC20.transfer.selector, bob), abi.encode(true));
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iRaindex)),
            abi.encode(true)
        );

        TakeOrderConfigV4[] memory orders = new TakeOrderConfigV4[](1);
        orders[0] = TakeOrderConfigV4({
            order: order, inputIOIndex: 0, outputIOIndex: 0, signedContext: new SignedContextV1[](0)
        });
        return LibTestTakeOrder.defaultTakeConfig(orders);
    }

    /// M01: with no ambient `iToken0` on Raindex, the in-loop push of the input
    /// token to the owner reverts — even though the taker can pay (the after-loop
    /// pull is mocked to succeed). The vault-0 input push is left unmocked so it
    /// reverts against the unfunded token. This revert IS the bug: a fundable
    /// trade fails purely on push-before-pull execution order.
    function testM01VaultZeroInputRevertsWithoutAmbientBalance() external {
        TakeOrdersConfigV5 memory config = _buildVaultZeroInputOrder();

        // The orderbook holds no ambient iToken0, so the in-loop push to the
        // owner reverts with insufficient balance (as a real ERC20 would).
        vm.mockCallRevert(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, owner),
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, address(iRaindex), uint256(0), 1e18)
        );

        vm.prank(bob);
        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, address(iRaindex), uint256(0), 1e18)
        );
        iRaindex.takeOrders4(config);
    }

    /// The identical trade SUCCEEDS once the in-loop push of the input token to
    /// the owner is funded (ambient balance present). M01's fragile cross-
    /// position dependency: success hinges on unrelated balances, not on the
    /// trade being fundable by the taker.
    function testM01VaultZeroInputSucceedsWithAmbientBalance() external {
        TakeOrdersConfigV5 memory config = _buildVaultZeroInputOrder();

        // Ambient balance: the in-loop push of the input token to the owner now succeeds.
        vm.mockCall(address(iToken0), abi.encodeWithSelector(IERC20.transfer.selector, owner), abi.encode(true));

        vm.prank(bob);
        (Float totalTakerInput, Float totalTakerOutput) = iRaindex.takeOrders4(config);

        assertTrue(totalTakerInput.eq(LibDecimalFloat.packLossless(1, 0)), "taker input filled with ambient balance");
        assertTrue(totalTakerOutput.eq(LibDecimalFloat.packLossless(1, 0)), "taker output filled with ambient balance");
    }

    /// M01 also applies to `clear3`. Alice's `recordVaultIO` runs before Bob's,
    /// so a `vaultId == 0` INPUT for Alice pushes her input token to her wallet
    /// as the first token movement — before Bob's output (which logically funds
    /// it) is debited. With no ambient balance on Raindex this reverts, even
    /// though the clear is logically balanced.
    ///
    ///   Alice: input = iToken0 (vault 0, wallet),  output = iToken1 (vault 0x01)
    ///   Bob:   input = iToken1 (vault 0x01),        output = iToken0 (vault 0, wallet)
    function testM01ClearVaultZeroInputRevertsWithoutAmbientBalance() external {
        // Alice: iToken0 input (vault 0), iToken1 output (vault 0x01).
        IOV2[] memory aliceInputs = new IOV2[](1);
        aliceInputs[0] = IOV2({token: address(iToken0), vaultId: bytes32(0)});
        IOV2[] memory aliceOutputs = new IOV2[](1);
        aliceOutputs[0] = IOV2({token: address(iToken1), vaultId: OUTPUT_VAULT_ID});
        OrderConfigV4 memory aliceConfig = OrderConfigV4({
            evaluable: EvaluableV4({
                bytecode: iParserV2.parse2("_ _: 1 1;:;"), interpreter: iInterpreter, store: iStore
            }),
            validInputs: aliceInputs,
            validOutputs: aliceOutputs,
            nonce: 0,
            secret: 0,
            meta: ""
        });

        // Bob: iToken1 input (vault 0x01), iToken0 output (vault 0, wallet).
        IOV2[] memory bobInputs = new IOV2[](1);
        bobInputs[0] = IOV2({token: address(iToken1), vaultId: OUTPUT_VAULT_ID});
        IOV2[] memory bobOutputs = new IOV2[](1);
        bobOutputs[0] = IOV2({token: address(iToken0), vaultId: bytes32(0)});
        OrderConfigV4 memory bobConfig = OrderConfigV4({
            evaluable: EvaluableV4({
                bytecode: iParserV2.parse2("_ _: 1 1;:;"), interpreter: iInterpreter, store: iStore
            }),
            validInputs: bobInputs,
            validOutputs: bobOutputs,
            nonce: 0,
            secret: 0,
            meta: ""
        });

        vm.prank(alice);
        iRaindex.addOrder4(aliceConfig, new TaskV2[](0));
        vm.prank(bob);
        iRaindex.addOrder4(bobConfig, new TaskV2[](0));

        OrderV4 memory aliceOrder =
            OrderV4(alice, aliceConfig.evaluable, aliceConfig.validInputs, aliceConfig.validOutputs, aliceConfig.nonce);
        OrderV4 memory bobOrder =
            OrderV4(bob, bobConfig.evaluable, bobConfig.validInputs, bobConfig.validOutputs, bobConfig.nonce);

        // Fund Alice's iToken1 output vault so her outputMax is nonzero.
        vm.mockCall(
            address(iToken1),
            abi.encodeWithSelector(IERC20.transferFrom.selector, alice, address(iRaindex)),
            abi.encode(true)
        );
        vm.prank(alice);
        iRaindex.deposit4(address(iToken1), OUTPUT_VAULT_ID, LibDecimalFloat.packLossless(10, 0), new TaskV2[](0));

        // Alice's vault-0 INPUT read (her wallet), and Bob's vault-0 OUTPUT read
        // (his wallet, large so his outputMax is nonzero -> the clear is a
        // nonzero, logically-balanced trade).
        _mockVault0Reads(address(iToken0), alice, 0);
        _mockVault0Reads(address(iToken0), bob, 1000e18);

        // Bob's vault-0 output debit (pull iToken0 from his wallet) would succeed.
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, bob, address(iRaindex)),
            abi.encode(true)
        );

        // The push of iToken0 to Alice (her vault-0 input credit) reverts with
        // insufficient balance: Raindex holds no ambient iToken0, and this is
        // the first token movement — before Bob's iToken0 output is pulled to
        // fund it.
        vm.mockCallRevert(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, alice),
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, address(iRaindex), uint256(0), 1e18)
        );

        vm.expectRevert(
            abi.encodeWithSelector(IERC20Errors.ERC20InsufficientBalance.selector, address(iRaindex), uint256(0), 1e18)
        );
        iRaindex.clear3(
            aliceOrder, bobOrder, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }
}
