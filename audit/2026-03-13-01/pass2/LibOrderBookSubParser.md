# Pass 2: Test Coverage -- LibOrderBookSubParser.sol
**Agent:** A13
**Date:** 2026-03-13
**File:** `src/lib/LibOrderBookSubParser.sol` (599 lines)

## Evidence of Thorough Reading

### Library
- `LibOrderBookSubParser` (line 98)

### Constants (lines 48-95)
- `SUB_PARSER_WORD_PARSERS_LENGTH` (line 48) = 2
- `EXTERN_PARSE_META_BUILD_DEPTH` (line 49) = 1
- Word constants (lines 51-80): `WORD_ORDER_CLEARER`, `WORD_ORDERBOOK`, `WORD_ORDER_HASH`, `WORD_ORDER_OWNER`, `WORD_ORDER_COUNTERPARTY`, `WORD_CALCULATED_MAX_OUTPUT`, `WORD_CALCULATED_IO_RATIO`, `WORD_INPUT_TOKEN`, `WORD_INPUT_TOKEN_DECIMALS`, `WORD_INPUT_VAULT_ID`, `WORD_INPUT_VAULT_BALANCE_BEFORE`, `WORD_INPUT_VAULT_BALANCE_INCREASE`, `WORD_OUTPUT_TOKEN`, `WORD_OUTPUT_TOKEN_DECIMALS`, `WORD_OUTPUT_VAULT_ID`, `WORD_OUTPUT_VAULT_BALANCE_BEFORE`, `WORD_OUTPUT_VAULT_BALANCE_DECREASE`, `WORD_DEPOSITOR`, `WORD_DEPOSIT_TOKEN`, `WORD_DEPOSIT_VAULT_ID`, `WORD_DEPOSIT_VAULT_BEFORE`, `WORD_DEPOSIT_VAULT_AFTER`, `WORD_WITHDRAWER`, `WORD_WITHDRAW_TOKEN`, `WORD_WITHDRAW_VAULT_ID`, `WORD_WITHDRAW_VAULT_BEFORE`, `WORD_WITHDRAW_VAULT_AFTER`, `WORD_WITHDRAW_TARGET_AMOUNT`
- Deposit index constants (lines 82-87): `DEPOSIT_WORD_DEPOSITOR` through `DEPOSIT_WORDS_LENGTH`
- Withdraw index constants (lines 89-95): `WITHDRAW_WORD_WITHDRAWER` through `WITHDRAW_WORDS_LENGTH`

### Functions (all `internal pure`)
| # | Function | Line | Purpose |
|---|----------|------|---------|
| 1 | `subParserSender` | 101 | Context: base column, sender row |
| 2 | `subParserCallingContract` | 106 | Context: base column, calling contract row |
| 3 | `subParserOrderHash` | 115 | Context: calling context column, order hash row |
| 4 | `subParserOrderOwner` | 124 | Context: calling context column, order owner row |
| 5 | `subParserOrderCounterparty` | 133 | Context: calling context column, order counterparty row |
| 6 | `subParserMaxOutput` | 143 | Context: calculations column, max output row |
| 7 | `subParserIORatio` | 152 | Context: calculations column, IO ratio row |
| 8 | `subParserInputToken` | 161 | Context: vault inputs column, token row |
| 9 | `subParserInputTokenDecimals` | 170 | Context: vault inputs column, token decimals row |
| 10 | `subParserInputVaultId` | 179 | Context: vault inputs column, vault ID row |
| 11 | `subParserInputBalanceBefore` | 188 | Context: vault inputs column, balance before row |
| 12 | `subParserInputBalanceDiff` | 197 | Context: vault inputs column, balance diff row |
| 13 | `subParserOutputToken` | 206 | Context: vault outputs column, token row |
| 14 | `subParserOutputTokenDecimals` | 215 | Context: vault outputs column, token decimals row |
| 15 | `subParserOutputVaultId` | 224 | Context: vault outputs column, vault ID row |
| 16 | `subParserOutputBalanceBefore` | 233 | Context: vault outputs column, balance before row |
| 17 | `subParserOutputBalanceDiff` | 242 | Context: vault outputs column, balance diff row |
| 18 | `subParserSigners` | 251 | Context: signers column, operand-as-row (dynamic) |
| 19 | `subParserDepositToken` | 260 | Context: calling context column, deposit token row |
| 20 | `subParserDepositVaultId` | 269 | Context: calling context column, deposit vault ID row |
| 21 | `subParserDepositVaultBalanceBefore` | 279 | Context: calling context column, deposit vault before row |
| 22 | `subParserDepositVaultBalanceAfter` | 291 | Context: calling context column, deposit vault after row |
| 23 | `subParserWithdrawToken` | 303 | Context: calling context column, withdraw token row |
| 24 | `subParserWithdrawVaultId` | 312 | Context: calling context column, withdraw vault ID row |
| 25 | `subParserWithdrawVaultBalanceBefore` | 322 | Context: calling context column, withdraw vault before row |
| 26 | `subParserWithdrawVaultBalanceAfter` | 334 | Context: calling context column, withdraw vault after row |
| 27 | `subParserWithdrawTargetAmount` | 346 | Context: calling context column, withdraw target amount row |
| 28 | `subParserSignedContext` | 357 | Context: signed context start column + operand-low-byte, operand-high-byte-as-row |
| 29 | `authoringMetaV2` | 369 | Builds authoring metadata for all context words |

### Types/Errors
No custom errors or types are defined in this library. All types (`AuthoringMetaV2`, `OperandV2`) are imported.

## Existing Test Coverage

### Unit-level context word tests (via `OrderBookV6SubParserContextTest` abstract base)
Each test file extends `OrderBookV6SubParserContextTest` and overrides `word()`. The base provides three tests per word:
1. `testSubParserContextHappy` -- parses word, evaluates against fixture context, checks stack matches `keccak256(word)`.
2. `testSubParserContextUnhappyDisallowedOperand` -- verifies operand `<1>` is rejected.
3. `testSubParserContextUnhappyDisallowedInputs` -- verifies input `(1)` causes `StackAllocationMismatch`.

**Covered words (17 test files):**
- `order-clearer` (subParserSender)
- `orderbook` (subParserCallingContract)
- `order-hash` (subParserOrderHash)
- `order-owner` (subParserOrderOwner)
- `order-counterparty` (subParserOrderCounterparty)
- `calculated-max-output` (subParserMaxOutput)
- `calculated-io-ratio` (subParserIORatio)
- `input-token` (subParserInputToken)
- `input-token-decimals` (subParserInputTokenDecimals)
- `input-vault-id` (subParserInputVaultId)
- `input-vault-before` (subParserInputBalanceBefore)
- `input-vault-increase` (subParserInputBalanceDiff)
- `output-token` (subParserOutputToken)
- `output-token-decimals` (subParserOutputTokenDecimals)
- `output-vault-id` (subParserOutputVaultId)
- `output-vault-before` (subParserOutputBalanceBefore)
- `output-vault-decrease` (subParserOutputBalanceDiff)

### Signer tests (`OrderBookV6SubParser.signers.t.sol`)
- Happy path for `signer<0>()` and `signer<1>()`.
- Unhappy: no operand, too many operands, disallowed input.

### Signed context tests (`OrderBookV6SubParser.signedContext.t.sol`)
- Happy path for `signed-context<0 0>()`, `<0 1>`, `<1 0>`, `<1 1>`.
- Unhappy: no operand, too many operands, disallowed input.

### Pointer / meta tests (`OrderBookV6SubParser.pointers.t.sol`)
- `testSubParserParseMeta` -- verifies `authoringMetaV2()` output matches generated parse meta.
- `testSubParserFunctionPointers` -- verifies word parser function pointers match.
- `testSubParserOperandParsers` -- verifies operand handler pointers match.
- `testWordOperandLengthEquivalence` -- checks length consistency.

### Integration tests for deposit/withdraw context words
- `OrderBookV6.deposit.entask.t.sol` (`testOrderDepositContext`): Tests `depositor()`, `deposit-token()`, `deposit-vault-id()`, `deposit-vault-before()`, `deposit-vault-after()` via actual `deposit4()` call with fuzzed values.
- `OrderBookV6.withdraw.entask.t.sol` (`testOrderWithdrawContext`): Tests `withdrawer()`, `withdraw-token()`, `withdraw-vault-id()`, `withdraw-vault-before()`, `withdraw-vault-after()`, `withdraw-target-amount()` via actual `withdraw4()` call with fuzzed values.

### Other
- `OrderBookV6SubParser.ierc165.t.sol` -- ERC165 interface support tests.
- `OrderBookV6SubParser.describedByMeta.t.sol` -- verifies described-by meta hash matches on-disk meta file.

## Coverage Gap Analysis

### GAP-1: No isolated unit tests for deposit/withdraw sub-parser functions
**Severity:** INFO
**Details:** The 11 deposit/withdraw-related sub-parser functions (`subParserDepositToken`, `subParserDepositVaultId`, `subParserDepositVaultBalanceBefore`, `subParserDepositVaultBalanceAfter`, `subParserWithdrawToken`, `subParserWithdrawVaultId`, `subParserWithdrawVaultBalanceBefore`, `subParserWithdrawVaultBalanceAfter`, `subParserWithdrawTargetAmount`) lack the same isolated `OrderBookV6SubParserContextTest`-style unit tests that the other 17 context words have. However, these functions ARE tested end-to-end through `OrderBookV6.deposit.entask.t.sol` and `OrderBookV6.withdraw.entask.t.sol`, which verify the words produce correct values in real deposit/withdraw flows with fuzzed inputs.

The difference in test approach is structurally justified: the deposit/withdraw words are served to a different context (the entask/post-action context) which is populated differently from the order-clearing context. The abstract `OrderBookV6SubParserContextTest` base builds a fixture context from `LibOrderBookSubParserContextFixture.hashedNamesContext()` that only covers the order-clearing columns, not the deposit/withdraw columns. The integration tests provide stronger coverage because they verify correctness of both the sub-parser routing AND the actual context population by the orderbook.

No fix required. The integration tests provide adequate coverage.

### GAP-2: No test for `authoringMetaV2()` internal consistency of deposit/withdraw metadata
**Severity:** INFO
**Details:** The `testSubParserParseMeta` test in `OrderBookV6SubParser.pointers.t.sol` indirectly validates that `authoringMetaV2()` produces valid metadata (by verifying the parse meta built from it matches the pre-generated constant). This covers the deposit and withdraw metadata entries since they are part of the same `authoringMetaV2()` output. The test also serves as a regression check: if any word is added/removed without updating the generated pointers, this test fails.

No fix required.

### GAP-3: `subParserSignedContext` operand encoding boundary values not fuzz-tested
**Severity:** INFO
**Details:** `subParserSignedContext` (line 357-366) extracts column from the low byte (`& 0xFF`) and row from the next byte (`>> 8 & 0xFF`) of the operand. While the happy-path tests cover four concrete combinations (0,0), (0,1), (1,0), (1,1), there is no fuzz test exercising boundary values of the operand encoding (e.g., column=255, row=255, or operands where higher bits beyond byte 0 and byte 1 are set).

However, the operand values are controlled by the parser's `handleOperandDoublePerByteNoDefault` handler, which restricts input to two single-byte values. Arbitrary operand values cannot reach `subParserSignedContext` through the normal parsing flow. The risk is near-zero in practice.

No fix required.

## Summary

| Category | Count |
|----------|-------|
| Functions in library | 29 |
| Functions with direct unit tests | 19 (17 context words + signers + signed-context) |
| Functions with integration test coverage | 9 (deposit/withdraw words via entask tests) |
| Functions with indirect test coverage | 1 (`authoringMetaV2` via pointer test) |
| Untested functions | 0 |
| Coverage gaps requiring fixes | 0 |

**Overall Assessment:** Test coverage for `LibOrderBookSubParser.sol` is comprehensive. All 29 functions have at least one form of test coverage. The 17 order-clearing context words have thorough isolated unit tests (happy + unhappy paths). The deposit/withdraw context words have fuzzed integration tests. The `authoringMetaV2` function is indirectly validated through the parse meta consistency test. No LOW+ findings.
