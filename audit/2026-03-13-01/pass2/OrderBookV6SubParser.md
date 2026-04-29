# Pass 2: Test Coverage -- OrderBookV6SubParser.sol
**Agent:** A09
**Date:** 2026-03-13
**File:** `src/concrete/parser/OrderBookV6SubParser.sol` (lines 1-307)

## 1. Evidence of Thorough Reading

### Contract
- `OrderBookV6SubParser` (line 71), inherits `BaseRainterpreterSubParser`

### Functions (with line numbers)
| # | Function | Line | Visibility |
|---|----------|------|------------|
| 1 | `describedByMetaV1()` | 75 | external pure |
| 2 | `subParserParseMeta()` | 80 | internal pure virtual override |
| 3 | `subParserWordParsers()` | 85 | internal pure virtual override |
| 4 | `subParserOperandHandlers()` | 90 | internal pure virtual override |
| 5 | `buildLiteralParserFunctionPointers()` | 95 | external pure |
| 6 | `buildOperandHandlerFunctionPointers()` | 100 | external pure |
| 7 | `buildSubParserWordParsers()` | 186 | external pure |

### Types / Errors / Constants (imported, not locally defined)
- `OperandV2` (from rain.interpreter)
- `BadDynamicLength` (imported but not directly used in this contract -- used in BaseRainterpreterSubParser)
- All `CONTEXT_*` constants from `LibOrderBook.sol`
- All `DEPOSIT_WORD_*` and `WITHDRAW_WORD_*` constants from `LibOrderBookSubParser.sol`
- `DESCRIBED_BY_META_HASH`, `SUB_PARSER_PARSE_META`, `SUB_PARSER_WORD_PARSERS`, `SUB_PARSER_OPERAND_HANDLERS` from generated pointers

### Key Operand Handler Configuration
- **Context base** (2 words): sender, calling-contract -- `handleOperandDisallowed`
- **Calling context** (3 words): order-hash, order-owner, order-counterparty -- `handleOperandDisallowed`
- **Calculations** (2 words): max-output, io-ratio -- `handleOperandDisallowed`
- **Vault inputs** (5 words): token, decimals, vault-id, balance-before, balance-diff -- `handleOperandDisallowed`
- **Vault outputs** (5 words): same as inputs -- `handleOperandDisallowed`
- **Signers** (1 word): `handleOperandSingleFullNoDefault`
- **Signed context** (1 word): `handleOperandDoublePerByteNoDefault`
- **Deposit** (5 words): depositor, token, vault-id, vault-before, vault-after -- `handleOperandDisallowed`
- **Withdraw** (6 words): withdrawer, token, vault-id, vault-before, vault-after, target-amount -- `handleOperandDisallowed`

## 2. Test Files Reviewed

| Test File | What It Tests |
|-----------|---------------|
| `OrderBookV6SubParser.ierc165.t.sol` | ERC165 interface detection for IERC165, ISubParserV4, IDescribedByMetaV1, IParserToolingV1, ISubParserToolingV1 |
| `OrderBookV6SubParser.describedByMeta.t.sol` | `describedByMetaV1()` returns correct hash |
| `OrderBookV6SubParser.pointers.t.sol` | Parse meta matches authoring meta; word parsers match `buildSubParserWordParsers()`; operand handlers match `buildOperandHandlerFunctionPointers()`; length equivalence |
| `OrderBookV6SubParser.signedContext.t.sol` | 4 happy-path tests (0-0, 0-1, 1-0, 1-1), no-operand error, too-many-operands error, disallowed-input error |
| `OrderBookV6SubParser.signers.t.sol` | 2 happy-path tests (signer-0, signer-1), no-operand error, too-many-operands error, disallowed-input error |
| `OrderBookV6SubParser.contextCalculatedIORatio.t.sol` | happy path, disallowed operand, disallowed inputs (via abstract base) |
| `OrderBookV6SubParser.contextCalculatedMaxOutput.t.sol` | same pattern |
| `OrderBookV6SubParser.contextInputToken.t.sol` | same pattern |
| `OrderBookV6SubParser.contextInputTokenDecimals.t.sol` | same pattern |
| `OrderBookV6SubParser.contextInputVaultBalanceBefore.t.sol` | same pattern |
| `OrderBookV6SubParser.contextInputVaultBalanceIncrease.t.sol` | same pattern |
| `OrderBookV6SubParser.contextInputVaultId.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOrderBook.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOrderClearer.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOrderCounterparty.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOrderHash.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOrderOwner.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOutputToken.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOutputTokenDecimals.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOutputVaultBalanceBefore.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOutputVaultBalanceDecrease.t.sol` | same pattern |
| `OrderBookV6SubParser.contextOutputVaultId.t.sol` | same pattern |

### Abstract Base Test
`test/util/abstract/OrderBookV6SubParserContextTest.sol` provides three tests for each word:
1. `testSubParserContextHappy` -- parses the word, checks it reads the correct context value
2. `testSubParserContextUnhappyDisallowedOperand` -- ensures operand `<1>` is rejected
3. `testSubParserContextUnhappyDisallowedInputs` -- ensures input `(1)` is rejected

### Fixture
`test/util/fixture/LibOrderBookSubParserContextFixture.sol` provides `hashedNamesContext()` which builds a context array of `CONTEXT_COLUMNS + 3 = 8` columns (indices 0-7). It populates the base, calling-context, calculations, vault-inputs, vault-outputs, signers, and two signed-context columns. It does NOT include deposit context (column 7 per sub-parser, but fixture uses 7 for signed-context-1) or withdraw context (column 8).

## 3. Coverage Gaps

### GAP-1: No tests for deposit word parsing (11 words, 0 tests) [LOW]
**Finding ID:** P2-A09-01

The sub-parser registers 5 deposit words (`depositor`, `deposit-token`, `deposit-vault-id`, `deposit-vault-before`, `deposit-vault-after`) and 6 withdraw words (`withdrawer`, `withdraw-token`, `withdraw-vault-id`, `withdraw-vault-before`, `withdraw-vault-after`, `withdraw-target-amount`) in both `buildSubParserWordParsers()` (lines 157-176 / 267-298) and `buildOperandHandlerFunctionPointers()` (lines 157-176 in operand section).

No test files exist for any of these 11 words. There are:
- No happy-path tests verifying they parse and resolve to the correct context column/row
- No operand-disallowed tests
- No input-disallowed tests

These words use `LibOrderBookSubParser.subParserSender`, `subParserDepositToken`, `subParserDepositVaultId`, `subParserDepositVaultBalanceBefore`, `subParserDepositVaultBalanceAfter`, `subParserWithdrawToken`, `subParserWithdrawVaultId`, `subParserWithdrawVaultBalanceBefore`, `subParserWithdrawVaultBalanceAfter`, `subParserWithdrawTargetAmount` -- all untested at the sub-parser level.

**Risk:** If any deposit/withdraw word-to-context mapping is wrong (wrong column or row index), a rainlang expression referencing e.g. `deposit-token()` in a deposit entask would silently read the wrong context value. The pointer tests confirm the generated pointers match the builder output, but they do NOT verify that the builder output is semantically correct.

**Severity:** LOW -- the mapping logic in `LibOrderBookSubParser` is straightforward (each function calls `LibSubParse.subParserContext` with constant column/row), and the pointer test verifies structural correctness. However, a semantic error (wrong constant used) would be invisible without word-level tests.

### GAP-2: `buildLiteralParserFunctionPointers()` has no direct test [INFO]
**Finding ID:** P2-A09-02

`buildLiteralParserFunctionPointers()` (line 95) returns empty bytes `""`. No test verifies this. The risk is negligible since the function body is a single return statement, but completeness would benefit from a one-line assertion.

### GAP-3: Test fixture does not cover deposit/withdraw context columns [INFO]
**Finding ID:** P2-A09-03

`LibOrderBookSubParserContextFixture.hashedNamesContext()` allocates `CONTEXT_COLUMNS + 3 = 8` entries. The sub-parser's `buildSubParserWordParsers()` and `buildOperandHandlerFunctionPointers()` allocate `CONTEXT_COLUMNS + 2 + 1 + 1 = 9` entries. The fixture is structurally smaller than the sub-parser's expected context grid. If deposit/withdraw tests were added, the fixture would need to be extended to 9+ columns.

### GAP-4: No fuzz tests on signed-context operand encoding [INFO]
**Finding ID:** P2-A09-04

The `subParserSignedContext` function (LibOrderBookSubParser.sol line 357-366) extracts column and row from the operand via bitmasking (`operand & 0xFF` for column, `(operand >> 8) & 0xFF` for row). Tests only cover 4 specific (column, row) pairs: (0,0), (0,1), (1,0), (1,1). A fuzz test with arbitrary column/row values would verify the operand encoding is correct across the full range.

### GAP-5: No fuzz tests on signer operand encoding [INFO]
**Finding ID:** P2-A09-05

Similar to GAP-4, `subParserSigners` (line 251-258) uses the full operand as a row index. Only two specific values (0, 1) are tested.

## 4. Summary

| Category | Count |
|----------|-------|
| Functions in contract | 7 |
| Words registered | 28 (17 context + 1 signer + 1 signed-context + 5 deposit + 6 withdraw = 30 word slots, but 28 unique words counting signer/signed-context parameterized) |
| Test files | 22 |
| Words with happy-path tests | 19 out of 30 word slots (17 disallowed-operand words + signer + signed-context) |
| Words with NO tests | 11 (all deposit + withdraw words) |
| Functions with no direct test | 1 (`buildLiteralParserFunctionPointers`) |
| Findings | 1 LOW, 4 INFO |
