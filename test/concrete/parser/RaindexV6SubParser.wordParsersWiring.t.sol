// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
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
/// any slot wired to a distinct function. This deterministically covers the
/// deposit/withdraw slot wiring (which has no fixture context column and so is
/// invisible to the etched per-word happy suite) WITHOUT etching or pointer
/// regeneration, because the freshly compiled `external pure` build function is
/// what runs on a fresh deployment.
contract RaindexV6SubParserWordParsersWiringTest is Test {
    RaindexV6SubParser internal subParser;

    function setUp() public {
        subParser = new RaindexV6SubParser();
    }

    // Flat slot indices into the word-parser pointer table. The matrix is
    // flattened column-major; cumulative row counts per column are:
    //   col 0 base            (2 rows) -> 0, 1
    //   col 1 calling context (3 rows) -> 2, 3, 4
    //   col 2 calculations    (2 rows) -> 5, 6
    //   col 3 vault inputs    (5 rows) -> 7..11
    //   col 4 vault outputs   (5 rows) -> 12..16
    //   col 5 signers         (1 row)  -> 17
    //   col 6 signed context  (1 row)  -> 18
    //   col 7 deposit         (5 rows) -> 19..23
    //   col 8 withdraw        (6 rows) -> 24..29
    uint256 internal constant SLOT_BASE_SENDER = 0;

    uint256 internal constant SLOT_DEPOSIT_DEPOSITOR = 19;
    uint256 internal constant SLOT_DEPOSIT_TOKEN = 20;
    uint256 internal constant SLOT_DEPOSIT_VAULT_ID = 21;
    uint256 internal constant SLOT_DEPOSIT_VAULT_BEFORE = 22;
    uint256 internal constant SLOT_DEPOSIT_VAULT_AFTER = 23;

    uint256 internal constant SLOT_WITHDRAW_WITHDRAWER = 24;
    uint256 internal constant SLOT_WITHDRAW_TOKEN = 25;
    uint256 internal constant SLOT_WITHDRAW_VAULT_ID = 26;
    uint256 internal constant SLOT_WITHDRAW_VAULT_BEFORE = 27;
    uint256 internal constant SLOT_WITHDRAW_VAULT_AFTER = 28;
    uint256 internal constant SLOT_WITHDRAW_TARGET_AMOUNT = 29;

    /// @dev Read the big-endian 16-bit function pointer at flat `slot` from the
    /// freshly built word-parser table.
    function pointerAt(uint256 slot) internal view returns (uint16) {
        bytes memory table = subParser.buildSubParserWordParsers();
        // 30 slots, 2 bytes each.
        assertEq(table.length, 60, "word parser table must hold 30 16-bit pointers");
        uint256 byteIndex = slot * 2;
        return (uint16(uint8(table[byteIndex])) << 8) | uint16(uint8(table[byteIndex + 1]));
    }

    /// The depositor slot of the deposit column is wired to `subParserSender`,
    /// the SAME function as the base sender slot, NOT a deposit-specific parser.
    /// Asserting pointer equality with the base sender slot catches any mutation
    /// that swaps the depositor slot to a deposit row function (e.g.
    /// `subParserDepositToken`), since that pointer differs from sender's.
    function testDepositorAliasesSender() external view {
        assertEq(pointerAt(SLOT_DEPOSIT_DEPOSITOR), pointerAt(SLOT_BASE_SENDER), "depositor must alias base sender");
        // And it is genuinely distinct from the deposit-token slot, so the
        // equality above is not a coincidence of two equal functions.
        assertTrue(
            pointerAt(SLOT_DEPOSIT_DEPOSITOR) != pointerAt(SLOT_DEPOSIT_TOKEN),
            "depositor must differ from deposit-token parser"
        );
    }

    /// The withdrawer slot of the withdraw column is wired to `subParserSender`.
    function testWithdrawerAliasesSender() external view {
        assertEq(pointerAt(SLOT_WITHDRAW_WITHDRAWER), pointerAt(SLOT_BASE_SENDER), "withdrawer must alias base sender");
        assertTrue(
            pointerAt(SLOT_WITHDRAW_WITHDRAWER) != pointerAt(SLOT_WITHDRAW_TOKEN),
            "withdrawer must differ from withdraw-token parser"
        );
    }

    /// Every non-depositor deposit slot is wired to its own DISTINCT deposit
    /// parser function: token, vault-id, balance-before, balance-after are four
    /// different functions, none of them `subParserSender`. A mutation that
    /// duplicates one deposit function across two slots, or that drops a slot
    /// back to sender, collapses a pair to equal pointers and is caught.
    function testDepositSlotsAreDistinctFunctions() external view {
        uint16 sender = pointerAt(SLOT_BASE_SENDER);
        uint16 token = pointerAt(SLOT_DEPOSIT_TOKEN);
        uint16 vaultId = pointerAt(SLOT_DEPOSIT_VAULT_ID);
        uint16 before = pointerAt(SLOT_DEPOSIT_VAULT_BEFORE);
        uint16 afterPtr = pointerAt(SLOT_DEPOSIT_VAULT_AFTER);

        assertTrue(token != sender, "deposit-token must not be sender");
        assertTrue(vaultId != sender, "deposit-vault-id must not be sender");
        assertTrue(before != sender, "deposit-before must not be sender");
        assertTrue(afterPtr != sender, "deposit-after must not be sender");

        assertTrue(token != vaultId, "deposit token vs vault-id distinct");
        assertTrue(token != before, "deposit token vs before distinct");
        assertTrue(token != afterPtr, "deposit token vs after distinct");
        assertTrue(vaultId != before, "deposit vault-id vs before distinct");
        assertTrue(vaultId != afterPtr, "deposit vault-id vs after distinct");
        assertTrue(before != afterPtr, "deposit before vs after distinct");
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

    /// Deposit and withdraw words that share the same calling-context row resolve
    /// to the SAME library function and so the SAME 16-bit pointer. Both
    /// `subParserDepositToken` and `subParserWithdrawToken` route to
    /// (CONTEXT_CALLING_CONTEXT_COLUMN, row 0), producing byte-identical context
    /// bytecode, so solc deduplicates them into one function. This pins that the
    /// deposit/withdraw token slots are wired to the row-0 calling-context parser
    /// (a mutation routing either to a different row, e.g. swapping in the
    /// vault-id parser, breaks the equality because the rows would differ).
    function testDepositAndWithdrawTokenShareRowZeroFunction() external view {
        assertEq(
            pointerAt(SLOT_DEPOSIT_TOKEN),
            pointerAt(SLOT_WITHDRAW_TOKEN),
            "deposit-token and withdraw-token both route to calling-context row 0"
        );
        // The shared row-0 function is genuinely distinct from sender and from
        // the row-1 (vault-id) function, so the equality is meaningful.
        assertTrue(pointerAt(SLOT_DEPOSIT_TOKEN) != pointerAt(SLOT_BASE_SENDER), "row-0 token parser is not sender");
        assertTrue(
            pointerAt(SLOT_DEPOSIT_TOKEN) != pointerAt(SLOT_DEPOSIT_VAULT_ID),
            "row-0 token parser is not row-1 vault-id"
        );
    }
}
