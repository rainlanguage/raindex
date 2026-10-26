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
/// those slots all share one 16-bit pointer. Only TWO slots are different:
///   - signers (column 6, row 0) -> `handleOperandSingleFullNoDefault`
///   - signed-context (column 7, row 0) -> `handleOperandDoublePerByteNoDefault`
/// Because the build function is a freshly compiled `external pure`, this test
/// runs against a fresh deployment WITHOUT etching or pointer regeneration. The
/// absolute pointer values are unstable across source edits (solc reorders its
/// dispatch table), so the assertions are all RELATIONS within a single build
/// output: table length, which slots equal the shared disallowed pointer, and
/// which two slots are the distinct signers / signed-context handlers. This
/// deterministically covers the operand-handler outer length, the column
/// assignments, and the deposit/withdraw column offsets: any of those mutations
/// changes the table length and/or relocates the two distinct handler pointers
/// off slots 23/24.
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
    //   col 5 deposit         (6 rows) -> 17..22
    //   col 6 signers         (1 row)  -> 23
    //   col 7 signed context  (1 row)  -> 24
    //   col 8 withdraw        (7 rows) -> 25..31
    uint256 internal constant TABLE_SLOTS = 32;
    uint256 internal constant SLOT_SIGNERS = 23;
    uint256 internal constant SLOT_SIGNED_CONTEXT = 24;

    /// @dev Read the big-endian 16-bit function pointer at flat `slot` from the
    /// freshly built operand-handler table, asserting the table is the expected
    /// 64 bytes (32 16-bit slots).
    function pointerAt(uint256 slot) internal view returns (uint16) {
        bytes memory table = subParser.buildOperandHandlerFunctionPointers();
        assertEq(table.length, TABLE_SLOTS * 2, "operand handler table must hold 32 16-bit pointers");
        uint256 byteIndex = slot * 2;
        return (uint16(uint8(table[byteIndex])) << 8) | uint16(uint8(table[byteIndex + 1]));
    }

    /// The operand-handler table holds exactly 32 slots (64 bytes); any outer
    /// length or column offset mutation that drops a column is caught by the
    /// guard inside `pointerAt`.
    function testOperandHandlerTableLength() external view {
        pointerAt(0);
    }

    /// Every slot before signers (base through deposit) is the shared
    /// disallowed pointer; the signers slot (column 6, row 0) differs.
    function testSignersOperandHandlerIsDistinct() external view {
        uint16 disallowed = pointerAt(0);
        for (uint256 slot = 0; slot < SLOT_SIGNERS; slot++) {
            assertEq(pointerAt(slot), disallowed, "plain context slot uses handleOperandDisallowed");
        }
        assertTrue(pointerAt(SLOT_SIGNERS) != disallowed, "signers handler must differ from handleOperandDisallowed");
    }

    /// The signed-context handler (column 7, row 0) differs from both the
    /// disallowed default and the signers handler.
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

    /// Every deposit (slots 17..22) and withdraw (slots 25..31) slot is the
    /// shared disallowed pointer.
    function testDepositWithdrawOperandHandlersAllDisallowed() external view {
        uint16 disallowed = pointerAt(0);
        for (uint256 slot = 17; slot < SLOT_SIGNERS; slot++) {
            assertEq(pointerAt(slot), disallowed, "deposit operand slot uses handleOperandDisallowed");
        }
        for (uint256 slot = SLOT_SIGNED_CONTEXT + 1; slot < TABLE_SLOTS; slot++) {
            assertEq(pointerAt(slot), disallowed, "withdraw operand slot uses handleOperandDisallowed");
        }
    }
}
