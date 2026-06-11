// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {OperandV2} from "rain-interpreter-interface-0.1.0/src/interface/ISubParserV4.sol";
import {OPCODE_CONTEXT} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterV4.sol";
import {ContextGridOverflow} from "rainlang-0.1.2/src/error/ErrSubParse.sol";
import {LibSubParse} from "rainlang-0.1.2/src/lib/parse/LibSubParse.sol";
import {LibRaindexSubParser} from "../../../src/lib/LibRaindexSubParser.sol";

/// @title RaindexV6SubParserRoutingTokenDecimalsAndMasksTest
/// @notice Pins routing that the main `RaindexV6SubParser.routing.t.sol` suite
/// leaves underconstrained, all against FRESH-compiled source (the
/// `LibRaindexSubParser.subParser*` functions are called directly, so no etched
/// pointers / regen are involved and any source-constant change is observed):
///
/// 1. `subParserWithdrawTokenDecimals` absolute (column, row). The
///    word-parser-wiring suite only pins this slot RELATIVELY (it asserts the
///    16-bit pointer is `!=` withdraw-target-amount / deposit-token-decimals /
///    sender), so an absolute (column, row) shift that lands on a pointer still
///    distinct from those three survives there. The deposit counterpart is
///    pinned absolutely by the wiring suite (its pointer must EQUAL
///    withdraw-target-amount at (1,4)), but withdraw-token-decimals has no such
///    equality anchor, so it needs a direct (column, row) assertion.
///
/// 2. The `& 0xFF` ROW mask in `subParserSignedContext`. The operand row is
///    `(operand >> 8) & 0xFF`. Because `LibSubParse.subParserContext` REVERTS
///    `ContextGridOverflow` when row > 255, dropping the mask turns a wide
///    operand from a successful in-range row into a revert. A wide operand whose
///    masked row is a small in-range value discriminates the masked code
///    (returns 4-byte context bytecode) from the unmasked code (reverts).
contract RaindexV6SubParserRoutingTokenDecimalsAndMasksTest is Test {
    /// @dev Decode the 4-byte context bytecode blob returned by a parser:
    /// byte 0 = opcode index, byte 1 = io byte, byte 2 = row, byte 3 = column.
    function decode(bytes memory bytecode) internal pure returns (uint8 row, uint8 column) {
        assertEq(bytecode.length, 4, "context bytecode must be 4 bytes");
        assertEq(uint8(bytecode[0]), OPCODE_CONTEXT, "opcode index");
        assertEq(uint8(bytecode[1]), 0x10, "io byte (0 in, 1 out)");
        row = uint8(bytecode[2]);
        column = uint8(bytecode[3]);
    }

    /// withdraw-token-decimals routes to the calling context column (1), row 5.
    /// A mutation of either the column or the row constant lands a different
    /// byte in the decoded bytecode and fails here.
    function testRouteWithdrawTokenDecimalsAbsolute() external pure {
        (bool success, bytes memory bytecode, bytes32[] memory constants) =
            LibRaindexSubParser.subParserWithdrawTokenDecimals(0, 0, OperandV2.wrap(0));
        assertTrue(success, "parser must succeed");
        assertEq(constants.length, 0, "context has no constants");
        (uint8 row, uint8 column) = decode(bytecode);
        assertEq(column, 1, "withdraw-token-decimals column is calling-context (1)");
        assertEq(row, 5, "withdraw-token-decimals row is 5");
    }

    /// deposit-token-decimals routes to the calling context column (1), row 4.
    /// The wiring suite pins this via pointer equality to withdraw-target-amount;
    /// this is the direct (column, row) assertion as a self-contained anchor.
    function testRouteDepositTokenDecimalsAbsolute() external pure {
        (bool success, bytes memory bytecode,) =
            LibRaindexSubParser.subParserDepositTokenDecimals(0, 0, OperandV2.wrap(0));
        assertTrue(success, "parser must succeed");
        (uint8 row, uint8 column) = decode(bytecode);
        assertEq(column, 1, "deposit-token-decimals column is calling-context (1)");
        assertEq(row, 4, "deposit-token-decimals row is 4");
    }

    /// The signed-context ROW is `(operand >> 8) & 0xFF`. An operand of
    /// `0x010000` has low byte 0 (column offset 0 -> column 6) and `>> 8` =
    /// `0x0100` = 256. WITH the mask the row is `0x0100 & 0xFF` = 0, so the
    /// parser succeeds and returns (column 6, row 0). WITHOUT the mask the row is
    /// 256, which exceeds uint8 and reverts `ContextGridOverflow(6, 256)`. This
    /// asserts the masked, non-reverting outcome, so dropping the row mask flips
    /// success into a revert and fails the test.
    function testSignedContextRowMaskKeepsRowInRange() external pure {
        (bool success, bytes memory bytecode,) =
            LibRaindexSubParser.subParserSignedContext(0, 0, OperandV2.wrap(bytes32(uint256(0x010000))));
        assertTrue(success, "parser must succeed (row mask keeps row in range)");
        (uint8 row, uint8 column) = decode(bytecode);
        assertEq(column, 6, "column is signed-context start (6) + low byte 0");
        assertEq(row, 0, "row is (0x010000 >> 8) & 0xFF = 0");
    }

    /// Confirms the discriminator above is genuine: the SAME wide operand, with
    /// the row mask DROPPED (`>> 8` without `& 0xFF`), drives the row to 256 and
    /// `LibSubParse.subParserContext` reverts `ContextGridOverflow(6, 256)`. This
    /// documents the exact revert the production mask prevents, so the pairing of
    /// these two tests pins the mask: removing it makes the happy test above
    /// revert with this very selector/args.
    function testSignedContextUnmaskedRowWouldOverflow() external {
        uint256 column = uint256(0x010000) & 0xFF; // low byte 0
        uint256 unmaskedRow = uint256(0x010000) >> 8; // 256
        vm.expectRevert(abi.encodeWithSelector(ContextGridOverflow.selector, column + 6, unmaskedRow));
        this.callContext(column + 6, unmaskedRow);
    }

    /// @dev External wrapper so `vm.expectRevert` can catch the library revert.
    function callContext(uint256 column, uint256 row) external pure returns (bool, bytes memory, bytes32[] memory) {
        return LibSubParse.subParserContext(column, row);
    }
}
