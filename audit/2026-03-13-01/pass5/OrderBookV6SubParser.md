# Pass 5: Correctness — OrderBookV6SubParser.sol

**Agent:** A09
**File:** `src/concrete/parser/OrderBookV6SubParser.sol` (307 lines)

## Evidence of Thorough Reading

- Read all 307 lines in full
- Verified all imports (lines 1-68): `LibParseOperand`, `BaseRainterpreterSubParser`, `OperandV2`, `IParserToolingV1`, `LibConvert`, `BadDynamicLength`, `LibExternOpContextSender`, `LibUint256Matrix`, plus all `DEPOSIT_WORD_*`, `WITHDRAW_WORD_*`, and `CONTEXT_*` constants from `LibOrderBookSubParser.sol` and `LibOrderBook.sol`, and generated pointers
- Verified `describedByMetaV1()` returns `DESCRIBED_BY_META_HASH` (line 75-77)
- Verified `subParserParseMeta()`, `subParserWordParsers()`, `subParserOperandHandlers()` return their respective generated pointers (lines 80-92)
- Verified `buildLiteralParserFunctionPointers()` returns empty bytes (lines 95-97)
- Verified `buildOperandHandlerFunctionPointers()` array size = `CONTEXT_COLUMNS + 2 + 1 + 1 = 5 + 4 = 9` columns (line 105)
- Verified `buildSubParserWordParsers()` array size = `CONTEXT_COLUMNS + 2 + 1 + 1 = 9` columns (line 194)
- Cross-checked every handler/parser slot against its context constant index

## Correctness Verification

### Constants and Column/Row Mapping in `buildOperandHandlerFunctionPointers`

| Handler Array | Column Index | Size Constant | Row Indices Verified |
|---|---|---|---|
| `contextBaseHandlers` | `CONTEXT_BASE_COLUMN` (0) | `CONTEXT_BASE_ROWS` (2) | `CONTEXT_BASE_ROW_SENDER` (0), `CONTEXT_BASE_ROW_CALLING_CONTRACT` (1) |
| `contextCallingContextHandlers` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROWS` (3) | `..ROW_ORDER_HASH` (0), `..ROW_ORDER_OWNER` (1), `..ROW_ORDER_COUNTERPARTY` (2) |
| `contextCalculationsHandlers` | `CONTEXT_CALCULATIONS_COLUMN` (2) | `CONTEXT_CALCULATIONS_ROWS` (2) | `..ROW_MAX_OUTPUT` (0), `..ROW_IO_RATIO` (1) |
| `contextVaultInputsHandlers` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_ROWS` (5) | TOKEN(0), DECIMALS(1), VAULT_ID(2), BALANCE_BEFORE(3), BALANCE_DIFF(4) |
| `contextVaultOutputsHandlers` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_ROWS` (5) | TOKEN(0), DECIMALS(1), VAULT_ID(2), BALANCE_BEFORE(3), BALANCE_DIFF(4) |
| `contextSignersHandlers` | `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` (5) | `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` (1) | `..SIGNERS_ROW` (0) -> `handleOperandSingleFullNoDefault` |
| `contextSignedContextHandlers` | `CONTEXT_SIGNED_CONTEXT_START_COLUMN` (6) | `CONTEXT_SIGNED_CONTEXT_START_ROWS` (1) | `..START_ROW` (0) -> `handleOperandDoublePerByteNoDefault` |
| `contextDepositContextHandlers` | `CONTEXT_SIGNED_CONTEXT_START_COLUMN + 1` (7) | `DEPOSIT_WORDS_LENGTH` (5) | All 5 deposit word indices verified |
| `contextWithdrawContextHandlers` | `CONTEXT_SIGNED_CONTEXT_START_COLUMN + 2` (8) | `WITHDRAW_WORDS_LENGTH` (6) | All 6 withdraw word indices verified |

All handler assignments use `LibParseOperand.handleOperandDisallowed` except signers (`handleOperandSingleFullNoDefault`) and signed-context (`handleOperandDoublePerByteNoDefault`). This is correct -- signers need a single operand to select which signer, and signed-context needs two bytes (column and row).

### Constants and Column/Row Mapping in `buildSubParserWordParsers`

Verified every parser function assignment matches the corresponding handler:

| Parser Array Slot | Parser Function | Target Context |
|---|---|---|
| `contextBaseParsers[0]` | `subParserSender` | col 0, row 0 |
| `contextBaseParsers[1]` | `subParserCallingContract` | col 0, row 1 |
| `contextCallingContextParsers[0]` | `subParserOrderHash` | col 1, row 0 |
| `contextCallingContextParsers[1]` | `subParserOrderOwner` | col 1, row 1 |
| `contextCallingContextParsers[2]` | `subParserOrderCounterparty` | col 1, row 2 |
| `contextCalculationsParsers[0]` | `subParserMaxOutput` | col 2, row 0 |
| `contextCalculationsParsers[1]` | `subParserIORatio` | col 2, row 1 |
| Vault inputs [0-4] | subParser{Input*} | col 3, rows 0-4 |
| Vault outputs [0-4] | subParser{Output*} | col 4, rows 0-4 |
| Signers [0] | `subParserSigners` | col 5, operand-derived row |
| Signed context [0] | `subParserSignedContext` | col 6+operand byte, operand-derived row |
| Deposit [0-4] | `subParserSender`, `subParserDeposit{Token,VaultId,VaultBalanceBefore,VaultBalanceAfter}` | col 0/row 0 for depositor, col 1/rows 0-3 for rest |
| Withdraw [0-5] | `subParserSender`, `subParserWithdraw{Token,VaultId,...,TargetAmount}` | col 0/row 0 for withdrawer, col 1/rows 0-4 for rest |

All parsers and handlers arrays are placed into the outer `parsers`/`handlers` arrays at the same column indices. Verified correct.

### Memory-Safety of Assembly Blocks

- Lines 179-181: `assembly ("memory-safe")` casts `handlers` to `handlersUint256`. Since both are `uint256[][]`, this is a safe no-op pointer cast.
- Lines 300-303: Same pattern for `parsers` to `parsersUint256`.

Both are followed by `LibConvert.unsafeTo16BitBytes(*.flatten())` which is the standard pattern for building function pointer tables.

## Findings

### P5-A09-01 (INFO): Unused Imports `BadDynamicLength` and `LibExternOpContextSender`

**Severity:** INFO
**File:** `src/concrete/parser/OrderBookV6SubParser.sol`, lines 12-13

**Details:** `BadDynamicLength` (imported from `rain.interpreter/error/ErrOpList.sol`) and `LibExternOpContextSender` (imported from `rain.interpreter/lib/extern/reference/op/LibExternOpContextSender.sol`) are imported but never referenced in any function or declaration in the contract body. `SUB_PARSER_WORD_PARSERS_LENGTH` was already flagged in pass 4 (P4-A09-03).

**Impact:** No runtime impact. Compiler may warn but these are dead imports that add noise.

## Summary

| ID | Severity | Description |
|---|---|---|
| P5-A09-01 | INFO | Unused imports `BadDynamicLength` and `LibExternOpContextSender` |

All column/row index mappings between operand handlers and word parsers are internally consistent. The handler types (disallowed, single-full, double-per-byte) match the semantic requirements of each context word. No correctness issues found.
