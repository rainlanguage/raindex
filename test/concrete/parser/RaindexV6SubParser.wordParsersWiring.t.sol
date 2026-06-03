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

    // Calling-context column (col 1) slots. Deposit/withdraw row words route to
    // the SAME (CONTEXT_CALLING_CONTEXT_COLUMN, row) as these order words, so
    // solc deduplicates them to a shared 16-bit pointer. This lets the
    // deposit/withdraw column OFFSETS be pinned by cross-column pointer equality.
    uint256 internal constant SLOT_CALLING_ORDER_HASH = 2; // calling-context row 0
    uint256 internal constant SLOT_CALLING_ORDER_OWNER = 3; // calling-context row 1
    uint256 internal constant SLOT_CALLING_ORDER_COUNTERPARTY = 4; // calling-context row 2

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

    /// Pins the DEPOSIT column to flat offset +1 of the signed-context start
    /// column (i.e. column 7, slots 19..23). The deposit row words route into
    /// CONTEXT_CALLING_CONTEXT_COLUMN rows 0/1/2, the SAME (col,row) as
    /// order-hash/order-owner/order-counterparty, so solc deduplicates them to a
    /// shared 16-bit pointer. Asserting that the deposit token/vault-id/before
    /// slots equal the corresponding calling-context order slots fixes BOTH that
    /// the deposit sub-array is assigned to this column AND that the column sits
    /// at the expected flat offset. A mutation that assigns the deposit array to
    /// `CONTEXT_SIGNED_CONTEXT_START_COLUMN + 2` (overwriting/displacing the
    /// withdraw column) shifts these pointers off the asserted slots and is
    /// caught here (independently of the table-length guard).
    function testDepositColumnAtOffsetPlusOne() external view {
        assertEq(
            pointerAt(SLOT_DEPOSIT_TOKEN),
            pointerAt(SLOT_CALLING_ORDER_HASH),
            "deposit-token (col 7 row 0) aliases calling-context order-hash (row 0)"
        );
        assertEq(
            pointerAt(SLOT_DEPOSIT_VAULT_ID),
            pointerAt(SLOT_CALLING_ORDER_OWNER),
            "deposit-vault-id (col 7 row 1) aliases calling-context order-owner (row 1)"
        );
        assertEq(
            pointerAt(SLOT_DEPOSIT_VAULT_BEFORE),
            pointerAt(SLOT_CALLING_ORDER_COUNTERPARTY),
            "deposit-vault-before (col 7 row 2) aliases calling-context order-counterparty (row 2)"
        );
        // The three calling-context pointers are themselves pairwise distinct, so
        // the equalities above genuinely pin each deposit row to its own row, not
        // a coincidence of three equal functions.
        assertTrue(
            pointerAt(SLOT_CALLING_ORDER_HASH) != pointerAt(SLOT_CALLING_ORDER_OWNER), "order-hash != order-owner"
        );
        assertTrue(
            pointerAt(SLOT_CALLING_ORDER_OWNER) != pointerAt(SLOT_CALLING_ORDER_COUNTERPARTY),
            "order-owner != order-counterparty"
        );
    }

    /// Pins the WITHDRAW column to flat offset +2 of the signed-context start
    /// column (i.e. column 8, slots 24..29). Like deposits, the withdraw row
    /// words 0/1/2 alias the calling-context order words, but the withdraw column
    /// has a SIXTH slot — withdraw-target-amount — routing to
    /// CONTEXT_CALLING_CONTEXT_COLUMN row 4, a row reached by NO order word and NO
    /// deposit word, so its pointer is unique to this slot. A mutation that swaps
    /// the deposit and withdraw column offsets places the withdraw column at +1
    /// (5-slot deposit array follows), so slot 29 no longer holds the unique
    /// row-4 target pointer and the asserted aliases land on the wrong rows.
    function testWithdrawColumnAtOffsetPlusTwo() external view {
        assertEq(
            pointerAt(SLOT_WITHDRAW_TOKEN),
            pointerAt(SLOT_CALLING_ORDER_HASH),
            "withdraw-token (col 8 row 0) aliases calling-context order-hash (row 0)"
        );
        assertEq(
            pointerAt(SLOT_WITHDRAW_VAULT_ID),
            pointerAt(SLOT_CALLING_ORDER_OWNER),
            "withdraw-vault-id (col 8 row 1) aliases calling-context order-owner (row 1)"
        );
        assertEq(
            pointerAt(SLOT_WITHDRAW_VAULT_BEFORE),
            pointerAt(SLOT_CALLING_ORDER_COUNTERPARTY),
            "withdraw-vault-before (col 8 row 2) aliases calling-context order-counterparty (row 2)"
        );
        // withdraw-target-amount routes to calling-context row 4, a row no order
        // word and no deposit slot reaches, so its pointer is unique to slot 29.
        uint16 target = pointerAt(SLOT_WITHDRAW_TARGET_AMOUNT);
        assertTrue(target != pointerAt(SLOT_CALLING_ORDER_HASH), "withdraw-target != order-hash (row 0)");
        assertTrue(target != pointerAt(SLOT_CALLING_ORDER_OWNER), "withdraw-target != order-owner (row 1)");
        assertTrue(
            target != pointerAt(SLOT_CALLING_ORDER_COUNTERPARTY), "withdraw-target != order-counterparty (row 2)"
        );
        assertTrue(target != pointerAt(SLOT_DEPOSIT_TOKEN), "withdraw-target != any deposit row pointer (token)");
        assertTrue(target != pointerAt(SLOT_DEPOSIT_VAULT_ID), "withdraw-target != any deposit row pointer (vault-id)");
        assertTrue(target != pointerAt(SLOT_DEPOSIT_VAULT_BEFORE), "withdraw-target != deposit before");
        assertTrue(target != pointerAt(SLOT_DEPOSIT_VAULT_AFTER), "withdraw-target != deposit after");
    }
}
