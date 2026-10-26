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
    //   col 6 withdraw        (7 rows) -> 23..29
    //   col 7 signers         (1 row)  -> 30
    //   col 8 signed context  (1 row)  -> 31
    uint256 internal constant TABLE_SLOTS = 32;
    uint256 internal constant SLOT_DEPOSIT_FIRST = 17;
    uint256 internal constant SLOT_WITHDRAW_LAST = 29;

    /// @dev Read the big-endian 16-bit function pointer at flat `slot` from the
    /// freshly built word-parser table.
    function pointerAt(uint256 slot) internal view returns (uint16) {
        bytes memory table = subParser.buildSubParserWordParsers();
        assertEq(table.length, TABLE_SLOTS * 2, "word parser table must hold 32 16-bit pointers");
        uint256 byteIndex = slot * 2;
        return (uint16(uint8(table[byteIndex])) << 8) | uint16(uint8(table[byteIndex + 1]));
    }

    /// Every deposit and withdraw word, depositor and withdrawer included, reads
    /// its own cell, so each of those slots has a pointer no other slot shares.
    function testDepositWithdrawSlotsAreUnique() external view {
        for (uint256 slot = SLOT_DEPOSIT_FIRST; slot <= SLOT_WITHDRAW_LAST; slot++) {
            uint16 pointer = pointerAt(slot);
            for (uint256 other = 0; other < TABLE_SLOTS; other++) {
                if (other != slot) {
                    assertTrue(pointer != pointerAt(other), "deposit/withdraw slot shares a parser with another slot");
                }
            }
        }
    }
}
