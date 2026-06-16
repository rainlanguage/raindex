# A09 - Pass 5: Correctness / Intent Verification
## File: `src/concrete/parser/OrderBookV6SubParser.sol`

## Source File Evidence

**Contract:** `OrderBookV6SubParser` (inherits `BaseRainterpreterSubParser`)

### Functions
- Line 73: `describedByMetaV1()` external pure - returns `DESCRIBED_BY_META_HASH`
- Line 78: `subParserParseMeta()` internal pure override - returns `SUB_PARSER_PARSE_META`
- Line 83: `subParserWordParsers()` internal pure override - returns `SUB_PARSER_WORD_PARSERS`
- Line 88: `subParserOperandHandlers()` internal pure override - returns `SUB_PARSER_OPERAND_HANDLERS`
- Line 93: `buildLiteralParserFunctionPointers()` external pure - returns empty bytes
- Line 98: `buildOperandHandlerFunctionPointers()` external pure - builds operand handler table
- Line 183: `buildSubParserWordParsers()` external pure - builds word parser table

**Library:** `LibOrderBookSubParser` (`src/lib/LibOrderBookSubParser.sol`)
- 25 `subParser*` functions mapping words to context columns/rows
- `authoringMetaV2()` function building authoring metadata

### Constants verified from `LibOrderBookSubParser`
- `SUB_PARSER_WORD_PARSERS_LENGTH = 2` (line 49)
- `EXTERN_PARSE_META_BUILD_DEPTH = 1` (line 50)
- All DEPOSIT_WORD_* and WITHDRAW_WORD_* index constants (lines 83-96)

## Test Files Reviewed

1. `test/concrete/parser/OrderBookV6SubParser.pointers.t.sol`
2. `test/concrete/parser/OrderBookV6SubParser.ierc165.t.sol`
3. `test/concrete/parser/OrderBookV6SubParser.describedByMeta.t.sol`
4. `test/concrete/parser/OrderBookV6SubParser.signedContext.t.sol`
5. `test/concrete/parser/OrderBookV6SubParser.signers.t.sol`
6. `test/concrete/parser/OrderBookV6SubParser.contextInputVaultBalanceIncrease.t.sol`
7. `test/concrete/parser/OrderBookV6SubParser.contextOutputVaultBalanceDecrease.t.sol`
8. `test/util/abstract/OrderBookV6SubParserContextTest.sol`
9. `test/util/fixture/LibOrderBookSubParserContextFixture.sol`

(Also reviewed all other context*.t.sol files for completeness)

## Findings

### No findings at LOW or above.

All named items match their documented behavior:

### Verified Correct Behavior

1. **Word-to-Context Mapping Correctness:**

   Every sub-parser word function was verified to map to the correct context column and row:

   | Word | Function | Column | Row | Correct? |
   |------|----------|--------|-----|----------|
   | `order-clearer` | `subParserSender` | `CONTEXT_BASE_COLUMN` (0) | `CONTEXT_BASE_ROW_SENDER` (0) | Yes |
   | `orderbook` | `subParserCallingContract` | `CONTEXT_BASE_COLUMN` (0) | `CONTEXT_BASE_ROW_CALLING_CONTRACT` (1) | Yes |
   | `order-hash` | `subParserOrderHash` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH` (0) | Yes |
   | `order-owner` | `subParserOrderOwner` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER` (1) | Yes |
   | `order-counterparty` | `subParserOrderCounterparty` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY` (2) | Yes |
   | `calculated-max-output` | `subParserMaxOutput` | `CONTEXT_CALCULATIONS_COLUMN` (2) | `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT` (0) | Yes |
   | `calculated-io-ratio` | `subParserIORatio` | `CONTEXT_CALCULATIONS_COLUMN` (2) | `CONTEXT_CALCULATIONS_ROW_IO_RATIO` (1) | Yes |
   | `input-token` | `subParserInputToken` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_TOKEN` (0) | Yes |
   | `input-token-decimals` | `subParserInputTokenDecimals` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_TOKEN_DECIMALS` (1) | Yes |
   | `input-vault-id` | `subParserInputVaultId` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_VAULT_ID` (2) | Yes |
   | `input-vault-before` | `subParserInputBalanceBefore` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_BALANCE_BEFORE` (3) | Yes |
   | `input-vault-increase` | `subParserInputBalanceDiff` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_BALANCE_DIFF` (4) | Yes |
   | `output-token` | `subParserOutputToken` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_TOKEN` (0) | Yes |
   | `output-token-decimals` | `subParserOutputTokenDecimals` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_TOKEN_DECIMALS` (1) | Yes |
   | `output-vault-id` | `subParserOutputVaultId` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_VAULT_ID` (2) | Yes |
   | `output-vault-before` | `subParserOutputBalanceBefore` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_BALANCE_BEFORE` (3) | Yes |
   | `output-vault-decrease` | `subParserOutputBalanceDiff` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_BALANCE_DIFF` (4) | Yes |
   | `signer` | `subParserSigners` | `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` (5) | operand value | Yes |
   | `signed-context` | `subParserSignedContext` | `CONTEXT_SIGNED_CONTEXT_START_COLUMN` (6) + low byte | high byte | Yes |

   Deposit words (column `CONTEXT_SIGNED_CONTEXT_START_COLUMN + 1` = 7 in the sub-parser table, but actual runtime context maps to `CONTEXT_CALLING_CONTEXT_COLUMN` via the sub-parser dispatch):
   | Word | Function | Maps to Column | Row | Correct? |
   |------|----------|---------------|-----|----------|
   | `depositor` | `subParserSender` | `CONTEXT_BASE_COLUMN` (0) | 0 | Yes |
   | `deposit-token` | `subParserDepositToken` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 0 | Yes |
   | `deposit-vault-id` | `subParserDepositVaultId` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 1 | Yes |
   | `deposit-vault-before` | `subParserDepositVaultBalanceBefore` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 2 | Yes |
   | `deposit-vault-after` | `subParserDepositVaultBalanceAfter` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 3 | Yes |

   Withdraw words (column `CONTEXT_SIGNED_CONTEXT_START_COLUMN + 2` = 8):
   | Word | Function | Maps to Column | Row | Correct? |
   |------|----------|---------------|-----|----------|
   | `withdrawer` | `subParserSender` | `CONTEXT_BASE_COLUMN` (0) | 0 | Yes |
   | `withdraw-token` | `subParserWithdrawToken` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 0 | Yes |
   | `withdraw-vault-id` | `subParserWithdrawVaultId` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 1 | Yes |
   | `withdraw-vault-before` | `subParserWithdrawVaultBalanceBefore` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 2 | Yes |
   | `withdraw-vault-after` | `subParserWithdrawVaultBalanceAfter` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 3 | Yes |
   | `withdraw-target-amount` | `subParserWithdrawTargetAmount` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | 4 | Yes |

2. **Operand Handler Correctness:**
   - All standard context words use `handleOperandDisallowed` -- no operand allowed. Correct, as these are fixed column/row references.
   - `signer` uses `handleOperandSingleFullNoDefault` -- requires exactly one operand (the signer index). Correct.
   - `signed-context` uses `handleOperandDoublePerByteNoDefault` -- requires two operands packed into bytes (column offset and row). Correct.

3. **`buildOperandHandlerFunctionPointers` vs. `buildSubParserWordParsers` Symmetry:**
   Both functions build identically structured arrays of the same dimensions across all context columns. The operand handlers and word parsers are paired 1:1. Verified in `testWordOperandLengthEquivalence` which asserts `SUB_PARSER_WORD_PARSERS.length == SUB_PARSER_OPERAND_HANDLERS.length`.

4. **Pointer Table Verification:**
   - `testSubParserParseMeta`: Rebuilds parse meta from authoring meta and asserts it equals the pre-computed `SUB_PARSER_PARSE_META`. This ensures the generated pointers match the runtime build.
   - `testSubParserFunctionPointers`: Rebuilds word parsers and asserts equality with `SUB_PARSER_WORD_PARSERS`.
   - `testSubParserOperandParsers`: Rebuilds operand handlers and asserts equality with `SUB_PARSER_OPERAND_HANDLERS`.

5. **Interface Conformance:**
   - `IDescribedByMetaV1`: Implemented via `describedByMetaV1()` returning `DESCRIBED_BY_META_HASH`. Tested in `describedByMeta.t.sol` which verifies the hash matches the actual meta file.
   - `IParserToolingV1`: `buildLiteralParserFunctionPointers()` returns empty bytes (no literals). `buildOperandHandlerFunctionPointers()` returns the handler table.
   - `BaseRainterpreterSubParser`: All three required overrides (`subParserParseMeta`, `subParserWordParsers`, `subParserOperandHandlers`) are implemented.
   - ERC165: Tested in `ierc165.t.sol` for `IERC165`, `ISubParserV4`, `IDescribedByMetaV1`, `IParserToolingV1`, `ISubParserToolingV1`.

6. **Test Correctness:**
   - Each context word test (e.g., `OrderBookV6SubParserContextInputVaultBalanceIncreaseTest`) overrides `word()` to return the word string and inherits `OrderBookV6SubParserContextTest` which:
     - `testSubParserContextHappy`: Parses rainlang using the sub-parser, evaluates against a fixture context where each cell contains `keccak256(wordName)`, and checks the stack output matches the expected hash.
     - `testSubParserContextUnhappyDisallowedOperand`: Verifies that adding an operand causes a parse error.
     - `testSubParserContextUnhappyDisallowedInputs`: Verifies that passing an input causes a stack allocation mismatch.
   - The `signer` and `signed-context` tests verify both happy paths (with correct operands) and unhappy paths (missing operands, too many operands, inputs).

7. **`authoringMetaV2` Consistency:**
   The authoring meta in `LibOrderBookSubParser.authoringMetaV2()` correctly maps every word string to its corresponding context position and provides accurate descriptions. The deposit meta uses offset indexing (`CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN + 1`) which correctly accounts for the `depositor` entry at index 0. The withdraw meta uses direct `WITHDRAW_WORD_*` indices which are already 0-based.

8. **Assembly Safety:**
   - Lines 174-176 and 294-296: The `assembly ("memory-safe")` blocks perform type-unsafe casts from function pointer arrays to `uint256[][]`. This is safe because function pointers and `uint256` have the same ABI encoding width.
   - The `flatten()` call on the matrix followed by `unsafeTo16BitBytes` produces the packed function pointer table. This is validated by the pointer tests.

9. **`CONTEXT_COLUMNS_EXTENDED` Calculation:**
   `CONTEXT_COLUMNS_EXTENDED = CONTEXT_COLUMNS + 2 + 1 + 1 = 5 + 2 + 1 + 1 = 9`. This accounts for: 5 base columns + signers column + signed-context-start column + deposit column + withdraw column. The `buildOperandHandlerFunctionPointers` and `buildSubParserWordParsers` both allocate arrays of size `CONTEXT_COLUMNS_EXTENDED` (9) and populate columns 0-8. Correct.
