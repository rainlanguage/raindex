# Pass 2: Test Coverage -- A09 OrderBookV6SubParser

**File:** src/concrete/parser/OrderBookV6SubParser.sol

## Evidence of Reading

### Contract: `OrderBookV6SubParser` (inherits `BaseRainterpreterSubParser`)

**Functions:**
| # | Function | Line | Visibility | Mutability |
|---|----------|------|------------|------------|
| 1 | `describedByMetaV1()` | 73 | external | pure |
| 2 | `subParserParseMeta()` | 78 | internal | pure virtual override |
| 3 | `subParserWordParsers()` | 83 | internal | pure virtual override |
| 4 | `subParserOperandHandlers()` | 88 | internal | pure virtual override |
| 5 | `buildLiteralParserFunctionPointers()` | 93 | external | pure |
| 6 | `buildOperandHandlerFunctionPointers()` | 98 | external | pure |
| 7 | `buildSubParserWordParsers()` | 183 | external | pure |

**Key imports/types:**
- `BaseRainterpreterSubParser` (parent contract)
- `LibParseOperand` (operand handling helpers)
- `LibOrderBookSubParser` (word parser functions)
- `LibConvert`, `LibUint256Matrix` (encoding utilities)
- `OperandV2` (operand type)
- Generated constants: `DESCRIBED_BY_META_HASH`, `SUB_PARSER_PARSE_META`, `SUB_PARSER_WORD_PARSERS`, `SUB_PARSER_OPERAND_HANDLERS`

**Context columns configured in `buildOperandHandlerFunctionPointers()` (line 98):**
- `CONTEXT_BASE_COLUMN` -- 2 rows, all `handleOperandDisallowed`
- `CONTEXT_CALLING_CONTEXT_COLUMN` -- 3 rows, all `handleOperandDisallowed`
- `CONTEXT_CALCULATIONS_COLUMN` -- 2 rows, all `handleOperandDisallowed`
- `CONTEXT_VAULT_INPUTS_COLUMN` -- 5 rows, all `handleOperandDisallowed`
- `CONTEXT_VAULT_OUTPUTS_COLUMN` -- 5 rows, all `handleOperandDisallowed`
- `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` -- 1 row, `handleOperandSingleFullNoDefault`
- `CONTEXT_SIGNED_CONTEXT_START_COLUMN` -- 1 row, `handleOperandDoublePerByteNoDefault`
- Deposit column (CONTEXT_SIGNED_CONTEXT_START_COLUMN + 1) -- 5 rows, all `handleOperandDisallowed`
- Withdraw column (CONTEXT_SIGNED_CONTEXT_START_COLUMN + 2) -- 6 rows, all `handleOperandDisallowed`

**Context columns configured in `buildSubParserWordParsers()` (line 183):**
- Base: `subParserSender`, `subParserCallingContract`
- Calling context: `subParserOrderHash`, `subParserOrderOwner`, `subParserOrderCounterparty`
- Calculations: `subParserMaxOutput`, `subParserIORatio`
- Vault inputs: 5 sub-parsers (token, decimals, vaultId, balanceBefore, balanceDiff)
- Vault outputs: 5 sub-parsers (token, decimals, vaultId, balanceBefore, balanceDiff)
- Signers: `subParserSigners`
- Signed context: `subParserSignedContext`
- Deposit: 5 sub-parsers (sender, depositToken, depositVaultId, depositVaultBalanceBefore, depositVaultBalanceAfter)
- Withdraw: 6 sub-parsers (sender, withdrawToken, withdrawVaultId, withdrawVaultBalanceBefore, withdrawVaultBalanceAfter, withdrawTargetAmount)

### Test files read:
- `test/util/abstract/OrderBookV6SubParserContextTest.sol` -- abstract base for context word tests
- `test/util/fixture/LibOrderBookSubParserContextFixture.sol` -- context fixture
- `test/concrete/parser/OrderBookV6SubParser.describedByMeta.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.ierc165.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.pointers.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.signedContext.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.signers.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputToken.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputTokenDecimals.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputVaultBalanceBefore.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputVaultBalanceDecrease.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputVaultId.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderClearer.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderCounterparty.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderHash.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderOwner.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputToken.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputTokenDecimals.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputVaultBalanceBefore.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputVaultBalanceIncrease.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputVaultId.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderBook.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextCalculatedIORatio.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextCalculatedMaxOutput.t.sol`
- `test/lib/deploy/LibOrderBookDeploy.t.sol`

## Findings

### A09-1: No dedicated sub-parser unit tests for deposit context words [LOW]

**Description:** The contract registers 5 deposit context words (`depositor`, `deposit-token`, `deposit-vault-id`, `deposit-vault-before`, `deposit-vault-after`) in both `buildSubParserWordParsers()` (lines 269-273) and `buildOperandHandlerFunctionPointers()` (lines 154-158). Every other context word category (base, calling, calculations, vault inputs, vault outputs, signers, signed-context) has dedicated sub-parser unit tests that verify parsing, operand disallowing, and input disallowing via the `OrderBookV6SubParserContextTest` abstract pattern. Deposit words have none of these.

The deposit words are exercised only via integration-level entask tests in `test/concrete/ob/OrderBookV6.deposit.entask.t.sol`, which test them in the context of a full deposit transaction. This provides coverage for the happy path but does not isolate and verify sub-parser behavior (operand disallowing, input rejection) at the unit level.

**Impact:** A regression in sub-parser operand handling for deposit words (e.g., accidentally allowing operands when they should be disallowed) would not be caught by existing unit tests.

**Recommendation:** Add unit test files following the `OrderBookV6SubParserContextTest` pattern for each deposit word. This requires extending `LibOrderBookSubParserContextFixture.hashedNamesContext()` to include the deposit context column. Example files: `OrderBookV6SubParser.contextDepositor.t.sol`, `OrderBookV6SubParser.contextDepositToken.t.sol`, etc.

---

### A09-2: No dedicated sub-parser unit tests for withdraw context words [LOW]

**Description:** The contract registers 6 withdraw context words (`withdrawer`, `withdraw-token`, `withdraw-vault-id`, `withdraw-vault-before`, `withdraw-vault-after`, `withdraw-target-amount`) in both `buildSubParserWordParsers()` (lines 285-290) and `buildOperandHandlerFunctionPointers()` (lines 164-169). Like deposit words (A09-1), every other context word category has dedicated sub-parser unit tests, but withdraw words do not.

The withdraw words are exercised only via integration-level entask tests in `test/concrete/ob/OrderBookV6.withdraw.entask.t.sol`.

Additionally, the test fixture `LibOrderBookSubParserContextFixture.hashedNamesContext()` allocates `CONTEXT_COLUMNS + 3` columns (8 total), but `CONTEXT_COLUMNS_EXTENDED` is 9. The fixture does not populate the withdraw context column at index 8, making it impossible to test withdraw words with the current fixture without extending it.

**Impact:** Same as A09-1 -- a regression in sub-parser operand handling for withdraw words would not be caught at the unit level.

**Recommendation:** Extend the fixture to populate all `CONTEXT_COLUMNS_EXTENDED` columns and add unit tests for each withdraw word following the existing pattern.

---

### A09-3: `buildLiteralParserFunctionPointers()` has no direct test [INFO]

**Description:** The function `buildLiteralParserFunctionPointers()` at line 93 returns an empty `bytes("")`. There is no test that directly calls this function or asserts its return value. The function is required by the `IParserToolingV1` interface and is trivially correct (it returns empty bytes because the sub-parser defines no literal parsers), but it still lacks test coverage.

**Impact:** Negligible. The function is trivial and stateless. It returns a constant empty bytes value.

**Recommendation:** Consider adding a simple assertion in the pointers test file:
```solidity
function testBuildLiteralParserFunctionPointers() external pure {
    OrderBookV6SubParser extern = OrderBookV6SubParser(LibOrderBookDeploy.SUB_PARSER_DEPLOYED_ADDRESS);
    assertEq(extern.buildLiteralParserFunctionPointers(), "");
}
```
