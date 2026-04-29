# A13: LibOrderBookSubParser.sol - Pass 1 (Security)

## Evidence of Thorough Reading

**File:** `src/lib/LibOrderBookSubParser.sol` (633 lines)

### Library
- `LibOrderBookSubParser` (line 101)

### Functions
- `subParserSender(uint256, uint256, OperandV2)` - line 105
- `subParserCallingContract(uint256, uint256, OperandV2)` - line 111
- `subParserOrderHash(uint256, uint256, OperandV2)` - line 121
- `subParserOrderOwner(uint256, uint256, OperandV2)` - line 130
- `subParserOrderCounterparty(uint256, uint256, OperandV2)` - line 141
- `subParserMaxOutput(uint256, uint256, OperandV2)` - line 152
- `subParserIORatio(uint256, uint256, OperandV2)` - line 162
- `subParserInputToken(uint256, uint256, OperandV2)` - line 172
- `subParserInputTokenDecimals(uint256, uint256, OperandV2)` - line 182
- `subParserInputVaultId(uint256, uint256, OperandV2)` - line 192
- `subParserInputBalanceBefore(uint256, uint256, OperandV2)` - line 202
- `subParserInputBalanceDiff(uint256, uint256, OperandV2)` - line 212
- `subParserOutputToken(uint256, uint256, OperandV2)` - line 222
- `subParserOutputTokenDecimals(uint256, uint256, OperandV2)` - line 232
- `subParserOutputVaultId(uint256, uint256, OperandV2)` - line 242
- `subParserOutputBalanceBefore(uint256, uint256, OperandV2)` - line 252
- `subParserOutputBalanceDiff(uint256, uint256, OperandV2)` - line 262
- `subParserSigners(uint256, uint256, OperandV2 operand)` - line 273
- `subParserDepositToken(uint256, uint256, OperandV2)` - line 283
- `subParserDepositVaultId(uint256, uint256, OperandV2)` - line 293
- `subParserDepositVaultBalanceBefore(uint256, uint256, OperandV2)` - line 304
- `subParserDepositVaultBalanceAfter(uint256, uint256, OperandV2)` - line 317
- `subParserWithdrawToken(uint256, uint256, OperandV2)` - line 330
- `subParserWithdrawVaultId(uint256, uint256, OperandV2)` - line 340
- `subParserWithdrawVaultBalanceBefore(uint256, uint256, OperandV2)` - line 350
- `subParserWithdrawVaultBalanceAfter(uint256, uint256, OperandV2)` - line 364
- `subParserWithdrawTargetAmount(uint256, uint256, OperandV2)` - line 377
- `subParserSignedContext(uint256, uint256, OperandV2 operand)` - line 390
- `authoringMetaV2()` - line 406

### Constants (file-level)
- `SUB_PARSER_WORD_PARSERS_LENGTH` = 2 (line 49)
- `EXTERN_PARSE_META_BUILD_DEPTH` = 1 (line 50)
- `WORD_ORDER_CLEARER` through `WORD_WITHDRAW_TARGET_AMOUNT` (lines 52-81, 18 word constants)
- `DEPOSIT_WORD_DEPOSITOR` = 0 (line 83)
- `DEPOSIT_WORD_TOKEN` = 1 (line 84)
- `DEPOSIT_WORD_VAULT_ID` = 2 (line 85)
- `DEPOSIT_WORD_VAULT_BEFORE` = 3 (line 86)
- `DEPOSIT_WORD_VAULT_AFTER` = 4 (line 87)
- `DEPOSIT_WORDS_LENGTH` = 5 (line 88)
- `WITHDRAW_WORD_WITHDRAWER` = 0 (line 90)
- `WITHDRAW_WORD_TOKEN` = 1 (line 91)
- `WITHDRAW_WORD_VAULT_ID` = 2 (line 92)
- `WITHDRAW_WORD_VAULT_BEFORE` = 3 (line 93)
- `WITHDRAW_WORD_VAULT_AFTER` = 4 (line 94)
- `WITHDRAW_WORD_TARGET_AMOUNT` = 5 (line 95)
- `WITHDRAW_WORDS_LENGTH` = 6 (line 96)

### Imports
- `AuthoringMetaV2`, `OperandV2` from ISubParserV4
- `LibUint256Matrix` from rain.solmem
- `LibSubParse` from rain.interpreter
- All CONTEXT_* constants from LibOrderBook

### Assembly blocks
- Lines 622-624: `metaUint256 := meta` (type cast `AuthoringMetaV2[][]` to `uint256[][]`)
- Lines 627-629: `metaFlattened := metaUint256Flattened` (type cast `uint256[]` to `AuthoringMetaV2[]`)

## Findings

No security findings. Analysis:

- **All subParser functions are `pure`**: They only map word names to context column/row indices. No state, no external calls, no side effects.
- **Assembly type casts** (lines 622-624, 627-629): These reinterpret `AuthoringMetaV2[][]` as `uint256[][]` for flattening, then cast back. This is safe because both types have identical memory layout (array of 32-byte slots). The `AuthoringMetaV2` struct has two `bytes32` fields, and `uint256` is also 32 bytes, so the pointer cast is layout-compatible for the purpose of `flatten()`.
- **`subParserSigners`** (line 273): Uses raw operand as row index. No bounds check, but this is a parser-time operation that produces context references -- out-of-bounds access would be caught at runtime by the interpreter when the context array is indexed.
- **`subParserSignedContext`** (line 390): Extracts column from low byte and row from next byte of operand. Same parser-time safety as above.
- **No string reverts**: No revert statements in this file.
