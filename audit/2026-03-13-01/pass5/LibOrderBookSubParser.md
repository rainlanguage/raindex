# Pass 5: Correctness — LibOrderBookSubParser.sol

**Agent:** A13
**File:** `src/lib/LibOrderBookSubParser.sol` (599 lines)

## Evidence of Thorough Reading

- Read all 599 lines in full
- Verified all imports (lines 1-46): `AuthoringMetaV2`, `OperandV2`, `LibUint256Matrix`, `LibSubParse`, and all 38 `CONTEXT_*` constants from `LibOrderBook.sol`
- Verified all 21 word constants (lines 51-80): `WORD_ORDER_CLEARER` through `WORD_WITHDRAW_TARGET_AMOUNT`
- Verified all 13 deposit/withdraw word index constants (lines 82-95)
- Verified `SUB_PARSER_WORD_PARSERS_LENGTH = 2` and `EXTERN_PARSE_META_BUILD_DEPTH = 1` (lines 48-49)
- Verified all 20 sub-parser functions (lines 101-366)
- Verified the `authoringMetaV2()` function (lines 369-598)

## Correctness Verification

### Word-to-Context Mapping (Sub-Parser Functions)

Every sub-parser function maps a word to a specific `(column, row)` in the context grid via `LibSubParse.subParserContext`. Verified each:

| Function | Word | Column | Row | Correct? |
|---|---|---|---|---|
| `subParserSender` | order-clearer / depositor / withdrawer | `CONTEXT_BASE_COLUMN` (0) | `CONTEXT_BASE_ROW_SENDER` (0) | Yes |
| `subParserCallingContract` | orderbook | `CONTEXT_BASE_COLUMN` (0) | `CONTEXT_BASE_ROW_CALLING_CONTRACT` (1) | Yes |
| `subParserOrderHash` | order-hash | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_ORDER_HASH` (0) | Yes |
| `subParserOrderOwner` | order-owner | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_ORDER_OWNER` (1) | Yes |
| `subParserOrderCounterparty` | order-counterparty | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_ORDER_COUNTERPARTY` (2) | Yes |
| `subParserMaxOutput` | calculated-max-output | `CONTEXT_CALCULATIONS_COLUMN` (2) | `..ROW_MAX_OUTPUT` (0) | Yes |
| `subParserIORatio` | calculated-io-ratio | `CONTEXT_CALCULATIONS_COLUMN` (2) | `..ROW_IO_RATIO` (1) | Yes |
| `subParserInputToken` | input-token | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_TOKEN` (0) | Yes |
| `subParserInputTokenDecimals` | input-token-decimals | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `..TOKEN_DECIMALS` (1) | Yes |
| `subParserInputVaultId` | input-vault-id | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `..VAULT_ID` (2) | Yes |
| `subParserInputBalanceBefore` | input-vault-before | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `..BALANCE_BEFORE` (3) | Yes |
| `subParserInputBalanceDiff` | input-vault-increase | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `..BALANCE_DIFF` (4) | Yes |
| `subParserOutputToken` | output-token | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_TOKEN` (0) | Yes |
| `subParserOutputTokenDecimals` | output-token-decimals | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `..TOKEN_DECIMALS` (1) | Yes |
| `subParserOutputVaultId` | output-vault-id | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `..VAULT_ID` (2) | Yes |
| `subParserOutputBalanceBefore` | output-vault-before | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `..BALANCE_BEFORE` (3) | Yes |
| `subParserOutputBalanceDiff` | output-vault-decrease | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `..BALANCE_DIFF` (4) | Yes |
| `subParserSigners` | signer | `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` (5) | operand-derived | Yes |
| `subParserSignedContext` | signed-context | `CONTEXT_SIGNED_CONTEXT_START_COLUMN + (operand & 0xFF)` | `(operand >> 8) & 0xFF` | Yes |

#### Deposit Sub-Parser Functions

| Function | Word | Column | Row |
|---|---|---|---|
| `subParserDepositToken` | deposit-token | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_DEPOSIT_TOKEN` (0) |
| `subParserDepositVaultId` | deposit-vault-id | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_DEPOSIT_VAULT_ID` (1) |
| `subParserDepositVaultBalanceBefore` | deposit-vault-before | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_DEPOSIT_VAULT_BEFORE` (2) |
| `subParserDepositVaultBalanceAfter` | deposit-vault-after | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_DEPOSIT_VAULT_AFTER` (3) |

Cross-referenced with `OrderBookV6.deposit4` context construction:
- `arrayFrom(token, vaultId, beforeBalance, afterBalance, decimals)` at context column index 0 (pre-build), which becomes column 1 after `LibContext.build`. Rows match.

#### Withdraw Sub-Parser Functions

| Function | Word | Column | Row |
|---|---|---|---|
| `subParserWithdrawToken` | withdraw-token | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_WITHDRAW_TOKEN` (0) |
| `subParserWithdrawVaultId` | withdraw-vault-id | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_WITHDRAW_VAULT_ID` (1) |
| `subParserWithdrawVaultBalanceBefore` | withdraw-vault-before | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_WITHDRAW_VAULT_BEFORE` (2) |
| `subParserWithdrawVaultBalanceAfter` | withdraw-vault-after | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_WITHDRAW_VAULT_AFTER` (3) |
| `subParserWithdrawTargetAmount` | withdraw-target-amount | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `..ROW_WITHDRAW_TARGET_AMOUNT` (4) |

Cross-referenced with `OrderBookV6.withdraw4` context construction:
- `arrayFrom(token, vaultId, beforeBalance, afterBalance, targetAmount, decimals)` at context column index 0 (pre-build), which becomes column 1 after `LibContext.build`. Rows match.

### `authoringMetaV2` Function (lines 369-598)

**Array size:** `CONTEXT_COLUMNS + 2 + 1 + 1 = 9` -- matches `buildSubParserWordParsers` and `buildOperandHandlerFunctionPointers` in `OrderBookV6SubParser.sol`.

**Meta indexing for deposit section (lines 518-546):**
- `depositMeta[0]` = WORD_DEPOSITOR -- matches `DEPOSIT_WORD_DEPOSITOR = 0`
- `depositMeta[CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN + 1]` = `depositMeta[1]` = WORD_DEPOSIT_TOKEN -- matches `DEPOSIT_WORD_TOKEN = 1`
- `depositMeta[CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID + 1]` = `depositMeta[2]` = WORD_DEPOSIT_VAULT_ID -- matches `DEPOSIT_WORD_VAULT_ID = 2`
- `depositMeta[CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE + 1]` = `depositMeta[3]` = WORD_DEPOSIT_VAULT_BEFORE -- matches `DEPOSIT_WORD_VAULT_BEFORE = 3`
- `depositMeta[CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER + 1]` = `depositMeta[4]` = WORD_DEPOSIT_VAULT_AFTER -- matches `DEPOSIT_WORD_VAULT_AFTER = 4`

All indices are numerically correct. However, the `+1` offset pattern relies on the numerical coincidence that `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_*` constants (0-3) are one less than the corresponding `DEPOSIT_WORD_*` constants (1-4).

**Meta indexing for withdraw section (lines 549-583):**
- Uses `WITHDRAW_WORD_*` constants directly. All match correctly.

**Authoring meta word bytes:**
- `WORD_ORDER_CLEARER` = "order-clearer" for sender -- matches semantics
- `WORD_ORDERBOOK` = "orderbook" for calling contract -- matches
- `WORD_ORDER_HASH`, `WORD_ORDER_OWNER`, `WORD_ORDER_COUNTERPARTY` -- all match
- `WORD_CALCULATED_MAX_OUTPUT`, `WORD_CALCULATED_IO_RATIO` -- match
- Input/output vault words -- all match
- "signer" and "signed-context" -- match

**Assembly block (lines 588-594):** Standard `memory-safe` cast from `AuthoringMetaV2[][]` to `uint256[][]`, flatten, cast back. This is the same pattern used in `OrderBookV6SubParser.sol`. Note: the first assembly block at line 588 is NOT marked `memory-safe` (unlike the ones in `OrderBookV6SubParser.sol`), but this is in a pure function and the cast is semantically safe.

### `subParserSignedContext` Operand Parsing (lines 357-366)

The function extracts column from `operand & 0xFF` and row from `(operand >> 8) & 0xFF`. This matches the `handleOperandDoublePerByteNoDefault` handler assigned in `OrderBookV6SubParser.buildOperandHandlerFunctionPointers()`. The double-per-byte operand packs two bytes: low byte = column offset, high byte = row. Correct.

### `subParserSigners` Operand Parsing (lines 251-258)

Uses `uint256(OperandV2.unwrap(operand))` as the row in the signers column. This matches `handleOperandSingleFullNoDefault` which allows a single operand value. Correct.

## Findings

### P5-A13-01 (INFO): Inconsistent Indexing Style in `authoringMetaV2` Deposit Section

**Severity:** INFO
**File:** `src/lib/LibOrderBookSubParser.sol`, lines 524-541

**Details:** The deposit meta section uses `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_* + 1` to index `depositMeta`, while the withdraw meta section uses `WITHDRAW_WORD_*` constants directly. The deposit section could equivalently use `DEPOSIT_WORD_TOKEN`, `DEPOSIT_WORD_VAULT_ID`, `DEPOSIT_WORD_VAULT_BEFORE`, `DEPOSIT_WORD_VAULT_AFTER` for clarity and consistency. The `+1` offset works only because the numerical relationship between `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_*` (0-3) and `DEPOSIT_WORD_*` (1-4) is exactly +1. If either set changed independently, this would silently produce wrong meta entries.

**Impact:** Maintainability concern. No current correctness issue since the values are correct. Previous passes (P3, P4) did not flag this specific inconsistency.

### P5-A13-02 (INFO): Missing `memory-safe` Annotation on Assembly Block

**Severity:** INFO
**File:** `src/lib/LibOrderBookSubParser.sol`, line 588

**Details:** The `assembly` block at line 588 is not annotated with `("memory-safe")`, unlike the equivalent blocks in `OrderBookV6SubParser.sol` (lines 179-181, 300-303). The cast is semantically safe (same layout pointer reinterpretation) and occurs in a `pure` function, so this has no practical impact on the optimizer. However, consistency with the rest of the codebase would be improved by adding the annotation.

## Summary

| ID | Severity | Description |
|---|---|---|
| P5-A13-01 | INFO | Inconsistent indexing style in `authoringMetaV2` deposit section vs withdraw section |
| P5-A13-02 | INFO | Missing `memory-safe` annotation on assembly block at line 588 |

All 20 sub-parser functions correctly map words to their intended context (column, row) positions. The `authoringMetaV2` function produces correct metadata for all words across all context columns. The deposit/withdraw sub-parser functions correctly reference the context rows that match the actual context arrays constructed in `OrderBookV6.deposit4` and `OrderBookV6.withdraw4`.
