# A13 - Pass 5: Correctness / Intent Verification
## File: `src/lib/LibOrderBookSubParser.sol`

### Evidence: Contract Inventory

**Library**: `LibOrderBookSubParser`

**Constants** (lines 49-96):
- `SUB_PARSER_WORD_PARSERS_LENGTH = 2` (line 49)
- `EXTERN_PARSE_META_BUILD_DEPTH = 1` (line 50)
- `WORD_ORDER_CLEARER = "order-clearer"` (line 52)
- `WORD_ORDERBOOK = "orderbook"` (line 53)
- `WORD_ORDER_HASH = "order-hash"` (line 54)
- `WORD_ORDER_OWNER = "order-owner"` (line 55)
- `WORD_ORDER_COUNTERPARTY = "order-counterparty"` (line 56)
- `WORD_CALCULATED_MAX_OUTPUT = "calculated-max-output"` (line 57)
- `WORD_CALCULATED_IO_RATIO = "calculated-io-ratio"` (line 58)
- `WORD_INPUT_TOKEN = "input-token"` (line 59)
- `WORD_INPUT_TOKEN_DECIMALS = "input-token-decimals"` (line 60)
- `WORD_INPUT_VAULT_ID = "input-vault-id"` (line 61)
- `WORD_INPUT_VAULT_BALANCE_BEFORE = "input-vault-before"` (line 62)
- `WORD_INPUT_VAULT_BALANCE_INCREASE = "input-vault-increase"` (line 63)
- `WORD_OUTPUT_TOKEN = "output-token"` (line 64)
- `WORD_OUTPUT_TOKEN_DECIMALS = "output-token-decimals"` (line 65)
- `WORD_OUTPUT_VAULT_ID = "output-vault-id"` (line 66)
- `WORD_OUTPUT_VAULT_BALANCE_BEFORE = "output-vault-before"` (line 67)
- `WORD_OUTPUT_VAULT_BALANCE_DECREASE = "output-vault-decrease"` (line 68)
- `WORD_DEPOSITOR = "depositor"` (line 70)
- `WORD_DEPOSIT_TOKEN = "deposit-token"` (line 71)
- `WORD_DEPOSIT_VAULT_ID = "deposit-vault-id"` (line 72)
- `WORD_DEPOSIT_VAULT_BEFORE = "deposit-vault-before"` (line 73)
- `WORD_DEPOSIT_VAULT_AFTER = "deposit-vault-after"` (line 74)
- `WORD_WITHDRAWER = "withdrawer"` (line 76)
- `WORD_WITHDRAW_TOKEN = "withdraw-token"` (line 77)
- `WORD_WITHDRAW_VAULT_ID = "withdraw-vault-id"` (line 78)
- `WORD_WITHDRAW_VAULT_BEFORE = "withdraw-vault-before"` (line 79)
- `WORD_WITHDRAW_VAULT_AFTER = "withdraw-vault-after"` (line 80)
- `WORD_WITHDRAW_TARGET_AMOUNT = "withdraw-target-amount"` (line 81)
- `DEPOSIT_WORD_DEPOSITOR = 0` (line 83)
- `DEPOSIT_WORD_TOKEN = 1` (line 84)
- `DEPOSIT_WORD_VAULT_ID = 2` (line 85)
- `DEPOSIT_WORD_VAULT_BEFORE = 3` (line 86)
- `DEPOSIT_WORD_VAULT_AFTER = 4` (line 87)
- `DEPOSIT_WORDS_LENGTH = 5` (line 88)
- `WITHDRAW_WORD_WITHDRAWER = 0` (line 90)
- `WITHDRAW_WORD_TOKEN = 1` (line 91)
- `WITHDRAW_WORD_VAULT_ID = 2` (line 92)
- `WITHDRAW_WORD_VAULT_BEFORE = 3` (line 93)
- `WITHDRAW_WORD_VAULT_AFTER = 4` (line 94)
- `WITHDRAW_WORD_TARGET_AMOUNT = 5` (line 95)
- `WITHDRAW_WORDS_LENGTH = 6` (line 96)

**Functions** (lines 105-632):
- `subParserSender` (line 105) -- maps "sender" word to base context column sender row
- `subParserCallingContract` (line 111) -- maps "calling-contract" word to base context column
- `subParserOrderHash` (line 121) -- maps "order-hash" word to calling context column
- `subParserOrderOwner` (line 131) -- maps "order-owner" word to calling context column
- `subParserOrderCounterparty` (line 140) -- maps "order-counterparty" word
- `subParserMaxOutput` (line 152) -- maps "max-output" word to calculations column
- `subParserIORatio` (line 162) -- maps "io-ratio" word to calculations column
- `subParserInputToken` (line 172) -- maps "input-token" word to vault inputs column
- `subParserInputTokenDecimals` (line 182) -- maps "input-token-decimals" word
- `subParserInputVaultId` (line 191) -- maps "input-vault-id" word
- `subParserInputBalanceBefore` (line 201) -- maps "input-balance-before" word
- `subParserInputBalanceDiff` (line 211) -- maps "input-balance-diff" word
- `subParserOutputToken` (line 221) -- maps "output-token" word
- `subParserOutputTokenDecimals` (line 231) -- maps "output-token-decimals" word
- `subParserOutputVaultId` (line 241) -- maps "output-vault-id" word
- `subParserOutputBalanceBefore` (line 251) -- maps "output-balance-before" word
- `subParserOutputBalanceDiff` (line 261) -- maps "output-balance-diff" word
- `subParserSigners` (line 273) -- maps "signers" word, uses operand for row selection
- `subParserDepositToken` (line 283) -- maps "deposit-token" word
- `subParserDepositVaultId` (line 293) -- maps "deposit-vault-id" word
- `subParserDepositVaultBalanceBefore` (line 304) -- maps "deposit-vault-balance-before" word
- `subParserDepositVaultBalanceAfter` (line 317) -- maps "deposit-vault-balance-after" word
- `subParserWithdrawToken` (line 330) -- maps "withdraw-token" word
- `subParserWithdrawVaultId` (line 340) -- maps "withdraw-vault-id" word
- `subParserWithdrawVaultBalanceBefore` (line 351) -- maps "withdraw-vault-balance-before" word
- `subParserWithdrawVaultBalanceAfter` (line 364) -- maps "withdraw-vault-balance-after" word
- `subParserWithdrawTargetAmount` (line 377) -- maps "withdraw-target-amount" word
- `subParserSignedContext` (line 390) -- maps "signed-context" word, operand encodes column+row
- `authoringMetaV2` (line 406) -- builds complete authoring metadata

### Verification: Sub-parser function column/row mappings

Every sub-parser function was verified against the context constants in `LibOrderBook.sol`:

| Function | Column | Row | Correct? |
|---|---|---|---|
| `subParserSender` | `CONTEXT_BASE_COLUMN` (0) | `CONTEXT_BASE_ROW_SENDER` (0) | Yes |
| `subParserCallingContract` | `CONTEXT_BASE_COLUMN` (0) | `CONTEXT_BASE_ROW_CALLING_CONTRACT` (1) | Yes |
| `subParserOrderHash` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH` (0) | Yes |
| `subParserOrderOwner` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER` (1) | Yes |
| `subParserOrderCounterparty` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY` (2) | Yes |
| `subParserMaxOutput` | `CONTEXT_CALCULATIONS_COLUMN` (2) | `CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT` (0) | Yes |
| `subParserIORatio` | `CONTEXT_CALCULATIONS_COLUMN` (2) | `CONTEXT_CALCULATIONS_ROW_IO_RATIO` (1) | Yes |
| `subParserInputToken` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_TOKEN` (0) | Yes |
| `subParserInputTokenDecimals` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_TOKEN_DECIMALS` (1) | Yes |
| `subParserInputVaultId` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_VAULT_ID` (2) | Yes |
| `subParserInputBalanceBefore` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_BALANCE_BEFORE` (3) | Yes |
| `subParserInputBalanceDiff` | `CONTEXT_VAULT_INPUTS_COLUMN` (3) | `CONTEXT_VAULT_IO_BALANCE_DIFF` (4) | Yes |
| `subParserOutputToken` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_TOKEN` (0) | Yes |
| `subParserOutputTokenDecimals` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_TOKEN_DECIMALS` (1) | Yes |
| `subParserOutputVaultId` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_VAULT_ID` (2) | Yes |
| `subParserOutputBalanceBefore` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_BALANCE_BEFORE` (3) | Yes |
| `subParserOutputBalanceDiff` | `CONTEXT_VAULT_OUTPUTS_COLUMN` (4) | `CONTEXT_VAULT_IO_BALANCE_DIFF` (4) | Yes |
| `subParserSigners` | `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN` (5) | operand (dynamic) | Yes |
| `subParserDepositToken` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_TOKEN` (0) | Yes |
| `subParserDepositVaultId` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_ID` (1) | Yes |
| `subParserDepositVaultBalanceBefore` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_BEFORE` (2) | Yes |
| `subParserDepositVaultBalanceAfter` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_DEPOSIT_VAULT_AFTER` (3) | Yes |
| `subParserWithdrawToken` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TOKEN` (0) | Yes |
| `subParserWithdrawVaultId` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_ID` (1) | Yes |
| `subParserWithdrawVaultBalanceBefore` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_BEFORE` (2) | Yes |
| `subParserWithdrawVaultBalanceAfter` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_VAULT_AFTER` (3) | Yes |
| `subParserWithdrawTargetAmount` | `CONTEXT_CALLING_CONTEXT_COLUMN` (1) | `CONTEXT_CALLING_CONTEXT_ROW_WITHDRAW_TARGET_AMOUNT` (4) | Yes |
| `subParserSignedContext` | `CONTEXT_SIGNED_CONTEXT_START_COLUMN` (6) + low byte | row from bits 8-15 | Yes |

### Verification: `authoringMetaV2()` array indexing

- Array size = `CONTEXT_COLUMNS_EXTENDED` = 9. Correct (columns 0-8 are all populated).
- `contextBaseMeta` size = `CONTEXT_BASE_ROWS` (2). Indices 0 and 1 populated. Correct.
- `contextCallingContextMeta` size = `CONTEXT_CALLING_CONTEXT_ROWS` (3). Indices 0, 1, 2 populated. Correct.
- `contextCalculationsMeta` size = `CONTEXT_CALCULATIONS_ROWS` (2). Indices 0 and 1 populated. Correct.
- `contextVaultInputsMeta` size = `CONTEXT_VAULT_IO_ROWS` (5). Indices 0-4 populated. Correct.
- `contextVaultOutputsMeta` size = `CONTEXT_VAULT_IO_ROWS` (5). Indices 0-4 populated. Correct.
- `contextSignersMeta` size = `CONTEXT_SIGNED_CONTEXT_SIGNERS_ROWS` (1). Index 0 populated. Correct.
- `contextSignedMeta` size = `CONTEXT_SIGNED_CONTEXT_START_ROWS` (1). Index 0 populated. Correct.
- `depositMeta` size = `DEPOSIT_WORDS_LENGTH` (5). Indices 0-4 populated. Correct.
- `withdrawMeta` size = `WITHDRAW_WORDS_LENGTH` (6). Indices 0-5 populated. Correct.

### Verification: Word constants match authoring metadata

All `WORD_*` constants correctly correspond to the bytes32-casted values in the authoring metadata. Each constant string is under 32 bytes, making the `bytes32()` cast safe.

### Verification: Tests vs Claims

Tests exercise 17 context words via `OrderBookV6SubParserContextTest`:
- `order-clearer`, `orderbook`, `order-hash`, `order-owner`, `order-counterparty`, `calculated-max-output`, `calculated-io-ratio`, `input-token`, `input-token-decimals`, `input-vault-id`, `input-vault-before`, `input-vault-increase`, `output-token`, `output-token-decimals`, `output-vault-id`, `output-vault-before`, `output-vault-decrease`

Each test verifies: (1) happy path produces correct context value, (2) operand is disallowed, (3) inputs are disallowed.

Signer tests: verify operand-based row selection with `signer<0>()` and `signer<1>()`.
Signed context tests: verify two-byte operand selection with all four combinations of column 0-1, row 0-1.

Tests correctly match behavior described in NatSpec.

### Verification: `EXTERN_PARSE_META_BUILD_DEPTH` usage

The constant is defined as `1` at line 50. It is used in `script/BuildPointers.sol` line 75 when calling `LibGenParseMeta.parseMetaConstantString`. However, the test in `OrderBookV6SubParser.pointers.t.sol` line 24 hardcodes `2` instead of using the constant:
```solidity
bytes memory expected = LibGenParseMeta.buildParseMetaV2(authoringMeta, 2);
```

### Findings

#### A13-1: Test uses hardcoded depth `2` instead of `EXTERN_PARSE_META_BUILD_DEPTH` (value `1`)
**Severity**: LOW

**Location**: `test/concrete/parser/OrderBookV6SubParser.pointers.t.sol`, line 24

**Description**: `testSubParserParseMeta()` calls `LibGenParseMeta.buildParseMetaV2(authoringMeta, 2)` with a hardcoded depth of `2`, while the build script (`script/BuildPointers.sol` line 75) uses `EXTERN_PARSE_META_BUILD_DEPTH` which equals `1`. The test compares the result against the pre-generated `SUB_PARSER_PARSE_META` constant (which was built with depth `1`).

If both depth values happen to produce the same bloom filter for this set of words, the test passes coincidentally. If they produce different results, the test would fail and catch the inconsistency. In either case, the test intent does not match the production build parameter: it should use the same constant to verify the build is reproducible.

**Impact**: The test does not faithfully verify the production build configuration. A future change to the word set could cause the two depths to diverge silently, and this test would fail for reasons unrelated to the change, or worse, pass when it should fail.

#### A13-2: `SUB_PARSER_WORD_PARSERS_LENGTH` imported but unused in `OrderBookV6SubParser.sol`
**Severity**: INFO

**Location**: `src/concrete/parser/OrderBookV6SubParser.sol`, line 16

**Description**: The constant `SUB_PARSER_WORD_PARSERS_LENGTH` is imported from `LibOrderBookSubParser.sol` but never referenced in the body of `OrderBookV6SubParser.sol`. This is a dead import.

**Impact**: No functional impact. Minor code hygiene issue.

### No Missing Test Coverage for Core Logic

All 17 standard context words, the signer word, and the signed-context word have dedicated happy-path and unhappy-path tests. The `authoringMetaV2()` function is indirectly tested via `testSubParserParseMeta` (pointers test). The `describedByMeta` test verifies meta hash consistency. IERC165 interface conformance is tested.

Deposit and withdraw context words (`depositor`, `deposit-token`, `deposit-vault-id`, `deposit-vault-before`, `deposit-vault-after`, `withdrawer`, `withdraw-token`, `withdraw-vault-id`, `withdraw-vault-before`, `withdraw-vault-after`, `withdraw-target-amount`) do not have dedicated sub-parser context tests of the same form as the core 17 words. This is noted but not flagged as a finding since the word-parser pointer and operand-handler tables are verified in the pointers test, and the underlying `subParser*` functions are trivial delegations to `LibSubParse.subParserContext`.
