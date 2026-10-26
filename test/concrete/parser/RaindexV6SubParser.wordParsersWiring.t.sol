// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {RaindexV6SubParser} from "../../../src/concrete/parser/RaindexV6SubParser.sol";

/// @title RaindexV6SubParserWordParsersWiringTest
/// @notice Pins which library word-parser function is wired into every
/// deposit and withdraw slot of `buildSubParserWordParsers()`. The build
/// function flattens a `[column][row]` matrix of internal-function pointers
/// into a flat table of 16-bit pointers (two bytes per slot, in column-major
/// then row order). The ABSOLUTE pointer value of any slot is unstable across
/// source edits (solc reorders its internal dispatch table), so this test
/// asserts only RELATIONS BETWEEN slots WITHIN A SINGLE build output, which are
/// invariant under that reordering: a slot wired to function F has the same
/// 16-bit pointer as every other slot wired to F, and a different pointer from
/// any slot wired to a distinct function.
contract RaindexV6SubParserWordParsersWiringTest is Test {
    RaindexV6SubParser internal subParser;

    function setUp() public {
        subParser = new RaindexV6SubParser();
    }

    // Flat slot indices into the word-parser pointer table (column-major):
    //   col 0 base            (2 rows) -> 0, 1
    //   col 1 calling context (3 rows) -> 2, 3, 4
    //   col 2 calculations    (2 rows) -> 5, 6
    //   col 3 vault inputs    (5 rows) -> 7..11
    //   col 4 vault outputs   (5 rows) -> 12..16
    //   col 5 deposit         (6 rows) -> 17..22
    //   col 6 signers         (1 row)  -> 23
    //   col 7 signed context  (1 row)  -> 24
    //   col 8 withdraw        (7 rows) -> 25..31
    uint256 internal constant TABLE_SLOTS = 32;
    uint256 internal constant SLOT_BASE_SENDER = 0;

    // Withdraw row words route to the SAME (CONTEXT_CALLING_CONTEXT_COLUMN, row)
    // as these order words, so solc deduplicates them to a shared 16-bit
    // pointer, which pins the withdraw column offset by pointer equality.
    uint256 internal constant SLOT_CALLING_ORDER_HASH = 2;
    uint256 internal constant SLOT_CALLING_ORDER_OWNER = 3;
    uint256 internal constant SLOT_CALLING_ORDER_COUNTERPARTY = 4;

    uint256 internal constant SLOT_DEPOSIT_FIRST = 17;
    uint256 internal constant SLOT_DEPOSIT_TOKEN_DECIMALS = 22;

    uint256 internal constant SLOT_WITHDRAW_WITHDRAWER = 25;
    uint256 internal constant SLOT_WITHDRAW_TOKEN = 26;
    uint256 internal constant SLOT_WITHDRAW_VAULT_ID = 27;
    uint256 internal constant SLOT_WITHDRAW_VAULT_BEFORE = 28;
    uint256 internal constant SLOT_WITHDRAW_VAULT_AFTER = 29;
    uint256 internal constant SLOT_WITHDRAW_TARGET_AMOUNT = 30;
    uint256 internal constant SLOT_WITHDRAW_TOKEN_DECIMALS = 31;

    /// @dev Read the big-endian 16-bit function pointer at flat `slot` from the
    /// freshly built word-parser table.
    function pointerAt(uint256 slot) internal view returns (uint16) {
        bytes memory table = subParser.buildSubParserWordParsers();
        assertEq(table.length, TABLE_SLOTS * 2, "word parser table must hold 32 16-bit pointers");
        uint256 byteIndex = slot * 2;
        return (uint16(uint8(table[byteIndex])) << 8) | uint16(uint8(table[byteIndex + 1]));
    }

    /// Every deposit word, depositor included, reads its own deposit column
    /// cell, so each deposit slot's pointer differs from every other slot.
    function testDepositSlotsAreUnique() external view {
        for (uint256 slot = SLOT_DEPOSIT_FIRST; slot <= SLOT_DEPOSIT_TOKEN_DECIMALS; slot++) {
            uint16 pointer = pointerAt(slot);
            for (uint256 other = 0; other < TABLE_SLOTS; other++) {
                if (other != slot) {
                    assertTrue(pointer != pointerAt(other), "deposit slot shares a parser with another slot");
                }
            }
        }
    }

    /// The withdrawer slot of the withdraw column is wired to `subParserSender`.
    function testWithdrawerAliasesSender() external view {
        assertEq(pointerAt(SLOT_WITHDRAW_WITHDRAWER), pointerAt(SLOT_BASE_SENDER), "withdrawer must alias base sender");
        assertTrue(
            pointerAt(SLOT_WITHDRAW_WITHDRAWER) != pointerAt(SLOT_WITHDRAW_TOKEN),
            "withdrawer must differ from withdraw-token parser"
        );
    }

    /// Every non-withdrawer withdraw slot is wired to its own DISTINCT withdraw
    /// parser function: token, vault-id, balance-before, balance-after,
    /// target-amount are five different functions, none of them
    /// `subParserSender`.
    function testWithdrawSlotsAreDistinctFunctions() external view {
        uint16 sender = pointerAt(SLOT_BASE_SENDER);
        uint16 token = pointerAt(SLOT_WITHDRAW_TOKEN);
        uint16 vaultId = pointerAt(SLOT_WITHDRAW_VAULT_ID);
        uint16 before = pointerAt(SLOT_WITHDRAW_VAULT_BEFORE);
        uint16 afterPtr = pointerAt(SLOT_WITHDRAW_VAULT_AFTER);
        uint16 target = pointerAt(SLOT_WITHDRAW_TARGET_AMOUNT);

        assertTrue(token != sender, "withdraw-token must not be sender");
        assertTrue(vaultId != sender, "withdraw-vault-id must not be sender");
        assertTrue(before != sender, "withdraw-before must not be sender");
        assertTrue(afterPtr != sender, "withdraw-after must not be sender");
        assertTrue(target != sender, "withdraw-target must not be sender");

        assertTrue(token != vaultId, "withdraw token vs vault-id distinct");
        assertTrue(token != before, "withdraw token vs before distinct");
        assertTrue(token != afterPtr, "withdraw token vs after distinct");
        assertTrue(token != target, "withdraw token vs target distinct");
        assertTrue(vaultId != before, "withdraw vault-id vs before distinct");
        assertTrue(vaultId != afterPtr, "withdraw vault-id vs after distinct");
        assertTrue(vaultId != target, "withdraw vault-id vs target distinct");
        assertTrue(before != afterPtr, "withdraw before vs after distinct");
        assertTrue(before != target, "withdraw before vs target distinct");
        assertTrue(afterPtr != target, "withdraw after vs target distinct");
    }

    /// Pins the withdraw column to the table after the signed context start
    /// column (slots 25..31): its row 0/1/2 words alias the order words, and
    /// withdraw-target-amount (row 4) is reached by no order word.
    function testWithdrawColumnAfterSignedContext() external view {
        assertEq(
            pointerAt(SLOT_WITHDRAW_TOKEN),
            pointerAt(SLOT_CALLING_ORDER_HASH),
            "withdraw-token aliases calling-context order-hash (row 0)"
        );
        assertEq(
            pointerAt(SLOT_WITHDRAW_VAULT_ID),
            pointerAt(SLOT_CALLING_ORDER_OWNER),
            "withdraw-vault-id aliases calling-context order-owner (row 1)"
        );
        assertEq(
            pointerAt(SLOT_WITHDRAW_VAULT_BEFORE),
            pointerAt(SLOT_CALLING_ORDER_COUNTERPARTY),
            "withdraw-vault-before aliases calling-context order-counterparty (row 2)"
        );
        assertTrue(
            pointerAt(SLOT_CALLING_ORDER_HASH) != pointerAt(SLOT_CALLING_ORDER_OWNER), "order-hash != order-owner"
        );
        assertTrue(
            pointerAt(SLOT_CALLING_ORDER_OWNER) != pointerAt(SLOT_CALLING_ORDER_COUNTERPARTY),
            "order-owner != order-counterparty"
        );
        uint16 target = pointerAt(SLOT_WITHDRAW_TARGET_AMOUNT);
        assertTrue(target != pointerAt(SLOT_CALLING_ORDER_HASH), "withdraw-target != order-hash (row 0)");
        assertTrue(target != pointerAt(SLOT_CALLING_ORDER_OWNER), "withdraw-target != order-owner (row 1)");
        assertTrue(
            target != pointerAt(SLOT_CALLING_ORDER_COUNTERPARTY), "withdraw-target != order-counterparty (row 2)"
        );
    }

    /// withdraw-token-decimals routes to calling-context row 5, reached by no
    /// other word.
    function testWithdrawTokenDecimalsWiring() external view {
        uint16 withdrawDecimals = pointerAt(SLOT_WITHDRAW_TOKEN_DECIMALS);
        assertTrue(
            withdrawDecimals != pointerAt(SLOT_WITHDRAW_TARGET_AMOUNT),
            "withdraw-token-decimals (row 5) != withdraw-target-amount (row 4)"
        );
        assertTrue(
            withdrawDecimals != pointerAt(SLOT_DEPOSIT_TOKEN_DECIMALS),
            "withdraw-token-decimals != deposit-token-decimals"
        );
        assertTrue(withdrawDecimals != pointerAt(SLOT_BASE_SENDER), "withdraw-token-decimals != sender");
    }
}
