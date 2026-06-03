// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {RaindexV6SubParser} from "../../../src/concrete/parser/RaindexV6SubParser.sol";

/// @title RaindexV6SubParserOperandHandlersWiringTest
/// @notice Pins the SHAPE and per-slot operand-handler wiring of
/// `buildOperandHandlerFunctionPointers()`. The build function flattens a
/// `[column][row]` matrix of internal operand-handler function pointers into a
/// flat table of 16-bit pointers (two bytes per slot, column-major then row
/// order), exactly mirroring the word-parser table. EVERY plain context word
/// (base / calling / calculations / vault-in / vault-out / deposit / withdraw)
/// is wired to the SAME `LibParseOperand.handleOperandDisallowed` function, so
/// those slots all share one 16-bit pointer. Only TWO slots are different:
///   - signers (column 5, row 0) -> `handleOperandSingleFullNoDefault`
///   - signed-context (column 6, row 0) -> `handleOperandDoublePerByteNoDefault`
/// Because the build function is a freshly compiled `external pure`, this test
/// runs against a fresh deployment WITHOUT etching or pointer regeneration. The
/// absolute pointer values are unstable across source edits (solc reorders its
/// dispatch table), so the assertions are all RELATIONS within a single build
/// output: table length, which slots equal the shared disallowed pointer, and
/// which two slots are the distinct signers / signed-context handlers. This
/// deterministically covers the operand-handler outer length, the column
/// assignments, and the deposit/withdraw column offsets: any of those mutations
/// changes the table length and/or relocates the two distinct handler pointers
/// off slots 17/18.
contract RaindexV6SubParserOperandHandlersWiringTest is Test {
    RaindexV6SubParser internal subParser;

    function setUp() public {
        subParser = new RaindexV6SubParser();
    }

    // Flat slot indices into the operand-handler pointer table (column-major):
    //   col 0 base            (2 rows) -> 0, 1
    //   col 1 calling context (3 rows) -> 2, 3, 4
    //   col 2 calculations    (2 rows) -> 5, 6
    //   col 3 vault inputs    (5 rows) -> 7..11
    //   col 4 vault outputs   (5 rows) -> 12..16
    //   col 5 signers         (1 row)  -> 17
    //   col 6 signed context  (1 row)  -> 18
    //   col 7 deposit         (5 rows) -> 19..23
    //   col 8 withdraw        (6 rows) -> 24..29
    uint256 internal constant TABLE_SLOTS = 30;
    uint256 internal constant SLOT_SIGNERS = 17;
    uint256 internal constant SLOT_SIGNED_CONTEXT = 18;

    /// @dev Read the big-endian 16-bit function pointer at flat `slot` from the
    /// freshly built operand-handler table, asserting the table is the expected
    /// 60 bytes (30 16-bit slots).
    function pointerAt(uint256 slot) internal view returns (uint16) {
        bytes memory table = subParser.buildOperandHandlerFunctionPointers();
        assertEq(table.length, TABLE_SLOTS * 2, "operand handler table must hold 30 16-bit pointers");
        uint256 byteIndex = slot * 2;
        return (uint16(uint8(table[byteIndex])) << 8) | uint16(uint8(table[byteIndex + 1]));
    }

    /// The operand-handler table holds exactly 30 slots (60 bytes), matching the
    /// extended context column/row layout. A mutation to the outer matrix length
    /// (`new ...[][](CONTEXT_COLUMNS_EXTENDED)`) or to the deposit/withdraw column
    /// offset (which displaces or drops a whole column) changes this length and
    /// is caught by the guard inside `pointerAt`. Reading slot 0 exercises it.
    function testOperandHandlerTableLength() external view {
        // pointerAt asserts table.length == 60; any wrong length reverts here.
        pointerAt(0);
    }

    /// The signers operand handler (column 5, row 0) is the FIRST distinct
    /// handler: `handleOperandSingleFullNoDefault`, different from the
    /// `handleOperandDisallowed` used by every plain context word. Slots 0..16
    /// (base, calling, calculations, vault-in, vault-out) are all the shared
    /// disallowed pointer, so the signers slot must differ from each of them. A
    /// column-assignment mutation that moves the signers column off slot 17, or
    /// that assigns a disallowed sub-array to the signers column, collapses this
    /// distinction and is caught.
    function testSignersOperandHandlerIsDistinct() external view {
        uint16 disallowed = pointerAt(0);
        // Every plain context slot before signers shares the disallowed pointer.
        for (uint256 slot = 0; slot <= 16; slot++) {
            assertEq(pointerAt(slot), disallowed, "plain context slot uses handleOperandDisallowed");
        }
        assertTrue(pointerAt(SLOT_SIGNERS) != disallowed, "signers handler must differ from handleOperandDisallowed");
    }

    /// The signed-context operand handler (column 6, row 0) is the SECOND
    /// distinct handler: `handleOperandDoublePerByteNoDefault`, different from
    /// BOTH the disallowed default AND the signers handler. This pins that the
    /// signers and signed-context columns are wired to two genuinely different
    /// handlers at slots 17 and 18 respectively (not swapped, not collapsed).
    function testSignedContextOperandHandlerIsDistinct() external view {
        uint16 disallowed = pointerAt(0);
        assertTrue(
            pointerAt(SLOT_SIGNED_CONTEXT) != disallowed,
            "signed-context handler must differ from handleOperandDisallowed"
        );
        assertTrue(
            pointerAt(SLOT_SIGNED_CONTEXT) != pointerAt(SLOT_SIGNERS),
            "signed-context handler must differ from signers handler"
        );
    }

    /// Every deposit (slots 19..23) and withdraw (slots 24..29) operand-handler
    /// slot is the shared `handleOperandDisallowed` pointer: deposit/withdraw
    /// words are plain context words whose operand is disallowed. This, combined
    /// with the two distinct-handler tests above and the length guard, pins that
    /// the deposit column sits at offset +1 and the withdraw column at offset +2
    /// of the signed-context start column: a mutation swapping those offsets, or
    /// dropping a column, either changes the table length (caught by the guard)
    /// or relocates the two distinct signers/signed-context handlers into the
    /// deposit/withdraw range (caught because slots 19..29 would no longer all
    /// equal the disallowed pointer).
    function testDepositWithdrawOperandHandlersAllDisallowed() external view {
        uint16 disallowed = pointerAt(0);
        for (uint256 slot = 19; slot <= 29; slot++) {
            assertEq(pointerAt(slot), disallowed, "deposit/withdraw operand slot uses handleOperandDisallowed");
        }
        // And neither distinct handler leaks into the deposit/withdraw range:
        // the only two non-disallowed slots are 17 (signers) and 18 (signed
        // context), strictly before the deposit column at slot 19.
        assertTrue(pointerAt(SLOT_SIGNERS) != disallowed, "signers slot 17 is the distinct signers handler");
        assertTrue(pointerAt(SLOT_SIGNED_CONTEXT) != disallowed, "signed-context slot 18 is the distinct handler");
    }
}
