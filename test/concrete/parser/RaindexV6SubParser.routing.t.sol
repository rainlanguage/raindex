// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.1/src/Test.sol";
import {OperandV2} from "rain-interpreter-interface-0.1.0/src/interface/ISubParserV4.sol";
import {OPCODE_CONTEXT} from "rain-interpreter-interface-0.1.0/src/interface/IInterpreterV4.sol";
import {LibRaindexSubParser} from "../../../src/lib/LibRaindexSubParser.sol";

/// @title RaindexV6SubParserRoutingTest
/// @notice Pins the exact (column, row) routing of every context word parser in
/// `LibRaindexSubParser`. Each `subParser*` function returns the bytecode blob
/// produced by `LibSubParse.subParserContext(column, row)`, whose 4 bytes are
/// `[opIndex, ioByte, row, column]`. By decoding bytes 2 and 3 we assert the
/// column AND row independently, so a swap of either constant in the source is
/// caught here directly (no etched pointers / regen required). This complements
/// the per-word happy tests (which run the etched parser and assert the keccak
/// of the resolved context value) by isolating the col-vs-row decision and by
/// covering the deposit/withdraw words that have no fixture context columns.
contract RaindexV6SubParserRoutingTest is Test {
    /// @dev Assert that a parser function routes to the exact (column, row).
    /// Decodes the 4-byte context bytecode blob returned by the parser:
    /// byte 0 = opcode index, byte 1 = io byte (1 output), byte 2 = row,
    /// byte 3 = column.
    function checkRoute(
        function(uint256, uint256, OperandV2) internal pure returns (bool, bytes memory, bytes32[] memory) parser,
        uint256 expectedColumn,
        uint256 expectedRow
    ) internal {
        (bool success, bytes memory bytecode, bytes32[] memory constants) = parser(0, 0, OperandV2.wrap(0));
        assertTrue(success, "parser must succeed");
        assertEq(bytecode.length, 4, "context bytecode must be 4 bytes");
        assertEq(constants.length, 0, "context has no constants");
        assertEq(uint8(bytecode[0]), OPCODE_CONTEXT, "opcode index");
        // 0 inputs, 1 output is encoded as the high nibble of byte 1.
        assertEq(uint8(bytecode[1]), 0x10, "io byte (0 in, 1 out)");
        assertEq(uint8(bytecode[2]), uint8(expectedRow), "row");
        assertEq(uint8(bytecode[3]), uint8(expectedColumn), "column");
    }

    /// @dev Same as `checkRoute` but the parser consumes the operand to select
    /// the row/column, so an explicit operand is supplied.
    function checkRouteOperand(
        function(uint256, uint256, OperandV2) internal pure returns (bool, bytes memory, bytes32[] memory) parser,
        OperandV2 operand,
        uint256 expectedColumn,
        uint256 expectedRow
    ) internal {
        (bool success, bytes memory bytecode,) = parser(0, 0, operand);
        assertTrue(success, "parser must succeed");
        assertEq(bytecode.length, 4, "context bytecode must be 4 bytes");
        assertEq(uint8(bytecode[2]), uint8(expectedRow), "row");
        assertEq(uint8(bytecode[3]), uint8(expectedColumn), "column");
    }

    // Base column (column 0).

    function testRouteSender() external {
        // order-clearer -> base column 0, sender row 0.
        checkRoute(LibRaindexSubParser.subParserSender, 0, 0);
    }

    function testRouteCallingContract() external {
        // raindex -> base column 0, calling-contract row 1.
        checkRoute(LibRaindexSubParser.subParserCallingContract, 0, 1);
    }

    // Calling context column (column 1).

    function testRouteOrderHash() external {
        checkRoute(LibRaindexSubParser.subParserOrderHash, 1, 0);
    }

    function testRouteOrderOwner() external {
        checkRoute(LibRaindexSubParser.subParserOrderOwner, 1, 1);
    }

    function testRouteOrderCounterparty() external {
        checkRoute(LibRaindexSubParser.subParserOrderCounterparty, 1, 2);
    }

    // Calculations column (column 2).

    function testRouteMaxOutput() external {
        checkRoute(LibRaindexSubParser.subParserMaxOutput, 2, 0);
    }

    function testRouteIORatio() external {
        checkRoute(LibRaindexSubParser.subParserIORatio, 2, 1);
    }

    // Vault inputs column (column 3).

    function testRouteInputToken() external {
        checkRoute(LibRaindexSubParser.subParserInputToken, 3, 0);
    }

    function testRouteInputTokenDecimals() external {
        checkRoute(LibRaindexSubParser.subParserInputTokenDecimals, 3, 1);
    }

    function testRouteInputVaultId() external {
        checkRoute(LibRaindexSubParser.subParserInputVaultId, 3, 2);
    }

    function testRouteInputBalanceBefore() external {
        checkRoute(LibRaindexSubParser.subParserInputBalanceBefore, 3, 3);
    }

    function testRouteInputBalanceDiff() external {
        checkRoute(LibRaindexSubParser.subParserInputBalanceDiff, 3, 4);
    }

    // Vault outputs column (column 4) — same rows as inputs but different
    // column, so this pins input-vs-output column isolation.

    function testRouteOutputToken() external {
        checkRoute(LibRaindexSubParser.subParserOutputToken, 4, 0);
    }

    function testRouteOutputTokenDecimals() external {
        checkRoute(LibRaindexSubParser.subParserOutputTokenDecimals, 4, 1);
    }

    function testRouteOutputVaultId() external {
        checkRoute(LibRaindexSubParser.subParserOutputVaultId, 4, 2);
    }

    function testRouteOutputBalanceBefore() external {
        checkRoute(LibRaindexSubParser.subParserOutputBalanceBefore, 4, 3);
    }

    function testRouteOutputBalanceDiff() external {
        checkRoute(LibRaindexSubParser.subParserOutputBalanceDiff, 4, 4);
    }

    // Deposit words route to the calling context column (column 1), deposit
    // row enum. No fixture context columns exist for these, so this is the
    // only coverage of their routing.

    function testRouteDepositToken() external {
        checkRoute(LibRaindexSubParser.subParserDepositToken, 1, 0);
    }

    function testRouteDepositVaultId() external {
        checkRoute(LibRaindexSubParser.subParserDepositVaultId, 1, 1);
    }

    function testRouteDepositVaultBalanceBefore() external {
        checkRoute(LibRaindexSubParser.subParserDepositVaultBalanceBefore, 1, 2);
    }

    function testRouteDepositVaultBalanceAfter() external {
        checkRoute(LibRaindexSubParser.subParserDepositVaultBalanceAfter, 1, 3);
    }

    // Withdraw words route to the calling context column (column 1), withdraw
    // row enum.

    function testRouteWithdrawToken() external {
        checkRoute(LibRaindexSubParser.subParserWithdrawToken, 1, 0);
    }

    function testRouteWithdrawVaultId() external {
        checkRoute(LibRaindexSubParser.subParserWithdrawVaultId, 1, 1);
    }

    function testRouteWithdrawVaultBalanceBefore() external {
        checkRoute(LibRaindexSubParser.subParserWithdrawVaultBalanceBefore, 1, 2);
    }

    function testRouteWithdrawVaultBalanceAfter() external {
        checkRoute(LibRaindexSubParser.subParserWithdrawVaultBalanceAfter, 1, 3);
    }

    function testRouteWithdrawTargetAmount() external {
        checkRoute(LibRaindexSubParser.subParserWithdrawTargetAmount, 1, 4);
    }

    // Operand-driven words.

    /// signers routes to the signers column (column 5) and uses the raw
    /// operand as the row.
    function testRouteSignersRowFromOperand() external {
        checkRouteOperand(LibRaindexSubParser.subParserSigners, OperandV2.wrap(bytes32(uint256(0))), 5, 0);
        checkRouteOperand(LibRaindexSubParser.subParserSigners, OperandV2.wrap(bytes32(uint256(1))), 5, 1);
        checkRouteOperand(LibRaindexSubParser.subParserSigners, OperandV2.wrap(bytes32(uint256(7))), 5, 7);
    }

    /// signed-context low operand byte selects a column offset added to the
    /// signed context start column (column 6); the second byte selects the row.
    function testRouteSignedContextColumnMaskAndRowShift() external {
        // operand 0x0000 -> column 6 + 0, row 0.
        checkRouteOperand(LibRaindexSubParser.subParserSignedContext, OperandV2.wrap(bytes32(uint256(0x0000))), 6, 0);
        // operand 0x0001 -> low byte 1 -> column 6 + 1 = 7, row 0.
        checkRouteOperand(LibRaindexSubParser.subParserSignedContext, OperandV2.wrap(bytes32(uint256(0x0001))), 7, 0);
        // operand 0x0100 -> low byte 0 -> column 6, second byte 1 -> row 1.
        checkRouteOperand(LibRaindexSubParser.subParserSignedContext, OperandV2.wrap(bytes32(uint256(0x0100))), 6, 1);
        // operand 0x0201 -> column 6 + 1 = 7, row 2. Proves column and row are
        // taken from distinct operand bytes (mask vs shift not swapped).
        checkRouteOperand(LibRaindexSubParser.subParserSignedContext, OperandV2.wrap(bytes32(uint256(0x0201))), 7, 2);
    }
}
