# Pass 1: Security -- LibOrderBookSubParser.sol

**Agent:** A13
**File:** src/lib/LibOrderBookSubParser.sol

## Evidence of Thorough Reading

**Library name:** `LibOrderBookSubParser` (line 98)

**Functions (with line numbers):**
1. `subParserSender(uint256, uint256, OperandV2)` -- line 101
2. `subParserCallingContract(uint256, uint256, OperandV2)` -- line 106
3. `subParserOrderHash(uint256, uint256, OperandV2)` -- line 115
4. `subParserOrderOwner(uint256, uint256, OperandV2)` -- line 124
5. `subParserOrderCounterparty(uint256, uint256, OperandV2)` -- line 133
6. `subParserMaxOutput(uint256, uint256, OperandV2)` -- line 143
7. `subParserIORatio(uint256, uint256, OperandV2)` -- line 152
8. `subParserInputToken(uint256, uint256, OperandV2)` -- line 161
9. `subParserInputTokenDecimals(uint256, uint256, OperandV2)` -- line 170
10. `subParserInputVaultId(uint256, uint256, OperandV2)` -- line 179
11. `subParserInputBalanceBefore(uint256, uint256, OperandV2)` -- line 188
12. `subParserInputBalanceDiff(uint256, uint256, OperandV2)` -- line 197
13. `subParserOutputToken(uint256, uint256, OperandV2)` -- line 206
14. `subParserOutputTokenDecimals(uint256, uint256, OperandV2)` -- line 215
15. `subParserOutputVaultId(uint256, uint256, OperandV2)` -- line 224
16. `subParserOutputBalanceBefore(uint256, uint256, OperandV2)` -- line 233
17. `subParserOutputBalanceDiff(uint256, uint256, OperandV2)` -- line 242
18. `subParserSigners(uint256, uint256, OperandV2 operand)` -- line 251
19. `subParserDepositToken(uint256, uint256, OperandV2)` -- line 260
20. `subParserDepositVaultId(uint256, uint256, OperandV2)` -- line 269
21. `subParserDepositVaultBalanceBefore(uint256, uint256, OperandV2)` -- line 279
22. `subParserDepositVaultBalanceAfter(uint256, uint256, OperandV2)` -- line 291
23. `subParserWithdrawToken(uint256, uint256, OperandV2)` -- line 303
24. `subParserWithdrawVaultId(uint256, uint256, OperandV2)` -- line 312
25. `subParserWithdrawVaultBalanceBefore(uint256, uint256, OperandV2)` -- line 322
26. `subParserWithdrawVaultBalanceAfter(uint256, uint256, OperandV2)` -- line 334
27. `subParserWithdrawTargetAmount(uint256, uint256, OperandV2)` -- line 346
28. `subParserSignedContext(uint256, uint256, OperandV2 operand)` -- line 357
29. `authoringMetaV2()` -- line 369

**Constants defined (file-level):**
- `SUB_PARSER_WORD_PARSERS_LENGTH` (line 48) = 2
- `EXTERN_PARSE_META_BUILD_DEPTH` (line 49) = 1
- Word constants: `WORD_ORDER_CLEARER`, `WORD_ORDERBOOK`, `WORD_ORDER_HASH`, `WORD_ORDER_OWNER`, `WORD_ORDER_COUNTERPARTY`, `WORD_CALCULATED_MAX_OUTPUT`, `WORD_CALCULATED_IO_RATIO`, `WORD_INPUT_TOKEN`, `WORD_INPUT_TOKEN_DECIMALS`, `WORD_INPUT_VAULT_ID`, `WORD_INPUT_VAULT_BALANCE_BEFORE`, `WORD_INPUT_VAULT_BALANCE_INCREASE`, `WORD_OUTPUT_TOKEN`, `WORD_OUTPUT_TOKEN_DECIMALS`, `WORD_OUTPUT_VAULT_ID`, `WORD_OUTPUT_VAULT_BALANCE_BEFORE`, `WORD_OUTPUT_VAULT_BALANCE_DECREASE` (lines 51-67)
- Deposit word constants: `WORD_DEPOSITOR`, `WORD_DEPOSIT_TOKEN`, `WORD_DEPOSIT_VAULT_ID`, `WORD_DEPOSIT_VAULT_BEFORE`, `WORD_DEPOSIT_VAULT_AFTER` (lines 69-73)
- Withdraw word constants: `WORD_WITHDRAWER`, `WORD_WITHDRAW_TOKEN`, `WORD_WITHDRAW_VAULT_ID`, `WORD_WITHDRAW_VAULT_BEFORE`, `WORD_WITHDRAW_VAULT_AFTER`, `WORD_WITHDRAW_TARGET_AMOUNT` (lines 75-80)
- Deposit word index constants: `DEPOSIT_WORD_DEPOSITOR` through `DEPOSIT_WORDS_LENGTH` (lines 82-87)
- Withdraw word index constants: `WITHDRAW_WORD_WITHDRAWER` through `WITHDRAW_WORDS_LENGTH` (lines 89-95)

**Types, errors, custom types defined:** None in this file. Types and errors are imported from dependencies.

**Imports:**
- `AuthoringMetaV2`, `OperandV2` from `rain.interpreter.interface`
- `LibUint256Matrix` from `rain.solmem`
- `LibSubParse` from `rain.interpreter`
- Numerous context constants from `LibOrderBook.sol`

**Using directives:**
- `LibUint256Matrix for uint256[][]` (line 99)

## Findings

No findings. The file is secure.

**Analysis summary:**

1. **All sub-parser functions (lines 101-355):** Functions 1-17 and 19-27 are trivial wrappers around `LibSubParse.subParserContext(column, row)` with compile-time constant arguments. The called function validates both column and row fit in uint8 before encoding. All constant pairs were verified against `LibOrderBook.sol` definitions and are within range.

2. **`subParserSigners` (line 251):** Passes the full `uint256(OperandV2.unwrap(operand))` as the row to `subParserContext`. Since `OperandV2 is bytes32`, this could be a large value, but `subParserContext` validates `row > type(uint8).max` and reverts with `ContextGridOverflow` if violated. Safe.

3. **`subParserSignedContext` (line 357):** Extracts column from low byte and row from second byte of operand via bitmasking (`& 0xFF`). Both are bounded to 0-255. The column is then offset by `CONTEXT_SIGNED_CONTEXT_START_COLUMN` (6), yielding max 261, which `subParserContext` validates against uint8 max. Operand column values >= 250 will revert. This is correct behavior.

4. **`authoringMetaV2` (line 369):** The two `assembly` blocks at lines 588-590 and 593-595 perform pointer aliasing (reinterpreting `AuthoringMetaV2[][]` as `uint256[][]` and `uint256[]` as `AuthoringMetaV2[]`). These are read-only pointer casts with no memory writes, consistent with the rain codebase pattern. The `AuthoringMetaV2` struct layout (`bytes32 word; string description;`) maps to two 256-bit slots per element, matching the `uint256[]` element size after flattening.

5. **Array bounds in `authoringMetaV2`:** The `meta` array is allocated with size `CONTEXT_COLUMNS + 2 + 1 + 1 = 9`. All 9 indices (0 through 8) are populated. Deposit meta uses `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_* + 1` indexing which was verified correct against the constant values (0-3, yielding indices 1-4 in a length-5 array). Withdraw meta uses `WITHDRAW_WORD_*` indices (0-5 in a length-6 array). All within bounds.

6. **`bytes32` conversions from `bytes` constants:** All `WORD_*` constants are string literals shorter than 32 bytes (longest is 21 characters). The `bytes32(WORD_*)` conversions are safe and annotated with `forge-lint: disable-next-line(unsafe-typecast)` comments.

7. **No string reverts:** The library does not define or use any `require` with string messages or `revert` with string arguments. All error handling is delegated to `LibSubParse.subParserContext` which uses custom errors (`ContextGridOverflow`).

8. **No arithmetic operations** beyond array index computation with compile-time constants. No unchecked blocks. No external calls.
