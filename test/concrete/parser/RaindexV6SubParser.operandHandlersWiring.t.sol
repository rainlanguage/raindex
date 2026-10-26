// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {RaindexV6SubParser} from "../../../src/concrete/parser/RaindexV6SubParser.sol";

/// @title RaindexV6SubParserOperandHandlersWiringTest
/// @notice Pins the SHAPE and per-slot operand-handler wiring of
/// `buildOperandHandlerFunctionPointers()`. The build function flattens a
/// `[column][row]` matrix of internal operand-handler function pointers into a
/// flat table of 16-bit pointers (two bytes per slot, column-major then row
/// order), exactly mirroring the word-parser table. EVERY plain context word
/// (base / calling / calculations / vault-in / vault-out / deposit / withdraw)
/// is wired to the SAME `LibParseOperand.handleOperandDisallowed` function, so
/// those slots all share one 16-bit pointer. Only THREE slots are different:
///   - signers (column 5, row 0) -> `handleOperandSingleFullNoDefault`
///   - signer domains (column 6, row 0) -> `handleOperandSingleFullNoDefault`
///   - signed-context (column 7, row 0) -> `handleOperandDoublePerByteNoDefault`
/// Because the build function is a freshly compiled `external pure`, this test
/// runs against a fresh deployment WITHOUT etching or pointer regeneration. The
/// absolute pointer values are unstable across source edits (solc reorders its
/// dispatch table), so the assertions are all RELATIONS within a single build
/// output: table length, which slots equal the shared disallowed pointer, and
/// which slots are the distinct signers / signer-domain / signed-context
/// handlers. Any mutation of the outer length, the column assignments, or the
/// deposit/withdraw column offsets changes the table length and/or relocates
/// the distinct handler pointers off slots 17/18/19.
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
    //   col 6 signer domains  (1 row)  -> 18
    //   col 7 signed context  (1 row)  -> 19
    //   col 8 deposit         (6 rows) -> 20..25
    //   col 9 withdraw        (7 rows) -> 26..32
    uint256 internal constant TABLE_SLOTS = 33;
    uint256 internal constant SLOT_SIGNERS = 17;
    uint256 internal constant SLOT_SIGNER_DOMAINS = 18;
    uint256 internal constant SLOT_SIGNED_CONTEXT = 19;

    /// @dev Read the big-endian 16-bit function pointer at flat `slot` from the
    /// freshly built operand-handler table, asserting the table is the expected
    /// 66 bytes (33 16-bit slots).
    function pointerAt(uint256 slot) internal view returns (uint16) {
        bytes memory table = subParser.buildOperandHandlerFunctionPointers();
        assertEq(table.length, TABLE_SLOTS * 2, "operand handler table must hold 33 16-bit pointers");
        uint256 byteIndex = slot * 2;
        return (uint16(uint8(table[byteIndex])) << 8) | uint16(uint8(table[byteIndex + 1]));
    }

    /// The operand-handler table holds exactly 33 slots (66 bytes), matching the
    /// extended context column/row layout. A mutation to the outer matrix length
    /// (`new ...[][](CONTEXT_COLUMNS_EXTENDED)`) or to the deposit/withdraw column
    /// offset (which displaces or drops a whole column) changes this length and
    /// is caught by the guard inside `pointerAt`. Reading slot 0 exercises it.
    function testOperandHandlerTableLength() external view {
        pointerAt(0);
    }

    /// The signers operand handler (column 5, row 0) is
    /// `handleOperandSingleFullNoDefault`, different from the
    /// `handleOperandDisallowed` used by every plain context word in slots
    /// 0..16. A column-assignment mutation that moves the signers column off
    /// slot 17, or that assigns a disallowed sub-array to it, is caught.
    function testSignersOperandHandlerIsDistinct() external view {
        uint16 disallowed = pointerAt(0);
        for (uint256 slot = 0; slot <= 16; slot++) {
            assertEq(pointerAt(slot), disallowed, "plain context slot uses handleOperandDisallowed");
        }
        assertTrue(pointerAt(SLOT_SIGNERS) != disallowed, "signers handler must differ from handleOperandDisallowed");
    }

    /// The signer-domain operand handler (column 6, row 0) is the same
    /// `handleOperandSingleFullNoDefault` as signers: one row index operand.
    function testSignerDomainsOperandHandlerMatchesSigners() external view {
        assertTrue(
            pointerAt(SLOT_SIGNER_DOMAINS) != pointerAt(0),
            "signer-domain handler must differ from handleOperandDisallowed"
        );
        assertEq(
            pointerAt(SLOT_SIGNER_DOMAINS), pointerAt(SLOT_SIGNERS), "signer-domain handler must equal signers handler"
        );
    }

    /// The signed-context operand handler (column 7, row 0) is
    /// `handleOperandDoublePerByteNoDefault`, different from BOTH the
    /// disallowed default AND the single-operand signers/signer-domain handler.
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

    /// Every deposit (slots 20..25) and withdraw (slots 26..32) operand-handler
    /// slot is the shared `handleOperandDisallowed` pointer. With the length
    /// guard and the distinct-handler tests above this pins the deposit column
    /// at offset +1 and the withdraw column at offset +2 of the signed-context
    /// start column: a swapped or dropped column either changes the table
    /// length or moves a distinct handler into the deposit/withdraw range.
    function testDepositWithdrawOperandHandlersAllDisallowed() external view {
        uint16 disallowed = pointerAt(0);
        for (uint256 slot = 20; slot <= 32; slot++) {
            assertEq(pointerAt(slot), disallowed, "deposit/withdraw operand slot uses handleOperandDisallowed");
        }
        assertTrue(pointerAt(SLOT_SIGNERS) != disallowed, "signers slot 17 is a distinct handler");
        assertTrue(pointerAt(SLOT_SIGNER_DOMAINS) != disallowed, "signer-domain slot 18 is a distinct handler");
        assertTrue(pointerAt(SLOT_SIGNED_CONTEXT) != disallowed, "signed-context slot 19 is a distinct handler");
    }
}
