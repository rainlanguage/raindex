# A13: LibOrderBookSubParser.sol -- Test Coverage Audit

## Source File
`src/lib/LibOrderBookSubParser.sol`

## Evidence of Thorough Reading

### Constants (lines 49-96)
- `SUB_PARSER_WORD_PARSERS_LENGTH = 2`, `EXTERN_PARSE_META_BUILD_DEPTH = 1`
- Word constants for all context words: order-clearer, orderbook, order-hash, order-owner, order-counterparty, calculated-max-output, calculated-io-ratio, input-token, input-token-decimals, input-vault-id, input-vault-before, input-vault-increase, output-token, output-token-decimals, output-vault-id, output-vault-before, output-vault-decrease
- Deposit word constants: depositor, deposit-token, deposit-vault-id, deposit-vault-before, deposit-vault-after
- Withdraw word constants: withdrawer, withdraw-token, withdraw-vault-id, withdraw-vault-before, withdraw-vault-after, withdraw-target-amount
- Index constants for deposit words (0-4) and withdraw words (0-5)

### Sub-parser functions (lines 101-399)
All `internal pure` functions returning `(bool, bytes memory, bytes32[] memory)`:
- `subParserSender` -> `CONTEXT_BASE_COLUMN, CONTEXT_BASE_ROW_SENDER`
- `subParserCallingContract` -> `CONTEXT_BASE_COLUMN, CONTEXT_BASE_ROW_CALLING_CONTRACT`
- `subParserOrderHash` -> `CONTEXT_CALLING_CONTEXT_COLUMN, CONTEXT_CALLING_CONTEXT_ROW_ORDER_HASH`
- `subParserOrderOwner` -> `CONTEXT_CALLING_CONTEXT_COLUMN, CONTEXT_CALLING_CONTEXT_ROW_ORDER_OWNER`
- `subParserOrderCounterparty` -> `CONTEXT_CALLING_CONTEXT_COLUMN, CONTEXT_CALLING_CONTEXT_ROW_ORDER_COUNTERPARTY`
- `subParserMaxOutput` -> `CONTEXT_CALCULATIONS_COLUMN, CONTEXT_CALCULATIONS_ROW_MAX_OUTPUT`
- `subParserIORatio` -> `CONTEXT_CALCULATIONS_COLUMN, CONTEXT_CALCULATIONS_ROW_IO_RATIO`
- `subParserInputToken` -> `CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN`
- `subParserInputTokenDecimals` -> `CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN_DECIMALS`
- `subParserInputVaultId` -> `CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_VAULT_ID`
- `subParserInputBalanceBefore` -> `CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_BEFORE`
- `subParserInputBalanceDiff` -> `CONTEXT_VAULT_INPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_DIFF`
- `subParserOutputToken` -> `CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN`
- `subParserOutputTokenDecimals` -> `CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_TOKEN_DECIMALS`
- `subParserOutputVaultId` -> `CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_VAULT_ID`
- `subParserOutputBalanceBefore` -> `CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_BEFORE`
- `subParserOutputBalanceDiff` -> `CONTEXT_VAULT_OUTPUTS_COLUMN, CONTEXT_VAULT_IO_BALANCE_DIFF`
- `subParserSigners` -> `CONTEXT_SIGNED_CONTEXT_SIGNERS_COLUMN`, row from operand
- `subParserDepositToken` -> deposit token
- `subParserDepositVaultId` -> deposit vault id
- `subParserDepositVaultBalanceBefore` -> deposit vault before
- `subParserDepositVaultBalanceAfter` -> deposit vault after
- `subParserWithdrawToken` -> withdraw token
- `subParserWithdrawVaultId` -> withdraw vault id
- `subParserWithdrawVaultBalanceBefore` -> withdraw vault before
- `subParserWithdrawVaultBalanceAfter` -> withdraw vault after
- `subParserWithdrawTargetAmount` -> withdraw target amount
- `subParserSignedContext` -> column from low byte, row from next byte of operand

### `authoringMetaV2()` (lines 406-632)
- Builds `AuthoringMetaV2[][]` for all context columns including base, calling, calculations, vault IO, signers, signed, deposit, withdraw
- Flattens via `LibUint256Matrix.flatten()` and ABI-encodes

## Test Files Found

### Order-clearing context words (each has its own test file via `OrderBookV6SubParserContextTest` abstract):
- `test/concrete/parser/OrderBookV6SubParser.contextOrderClearer.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderBook.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderHash.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderOwner.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOrderCounterparty.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextCalculatedMaxOutput.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextCalculatedIORatio.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputToken.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputTokenDecimals.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputVaultId.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputVaultBalanceBefore.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextInputVaultBalanceIncrease.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputToken.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputTokenDecimals.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputVaultId.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputVaultBalanceBefore.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.contextOutputVaultBalanceDecrease.t.sol`

### Signed context and signers:
- `test/concrete/parser/OrderBookV6SubParser.signedContext.t.sol`
- `test/concrete/parser/OrderBookV6SubParser.signers.t.sol`

### Pointers/meta:
- `test/concrete/parser/OrderBookV6SubParser.pointers.t.sol` (tests `authoringMetaV2()`)

### Indirect coverage for deposit/withdraw words:
- `test/concrete/ob/OrderBookV6.deposit.entask.t.sol` (tests depositor, deposit-token, deposit-vault-id, deposit-vault-before, deposit-vault-after)
- `test/concrete/ob/OrderBookV6.withdraw.entask.t.sol` (tests withdrawer, withdraw-token, withdraw-vault-id, withdraw-vault-before, withdraw-vault-after, withdraw-target-amount)

## Coverage Gaps

### GAP-A13-1: No dedicated sub-parser unit tests for deposit/withdraw context words
**Severity**: Low
**Details**: Every order-clearing context word (order-clearer, orderbook, order-hash, etc.) has its own dedicated test file under `test/concrete/parser/` using the `OrderBookV6SubParserContextTest` abstract. The deposit words (depositor, deposit-token, deposit-vault-id, deposit-vault-before, deposit-vault-after) and withdraw words (withdrawer, withdraw-token, withdraw-vault-id, withdraw-vault-before, withdraw-vault-after, withdraw-target-amount) have NO corresponding sub-parser test files. They are only tested indirectly through integration tests in `OrderBookV6.deposit.entask.t.sol` and `OrderBookV6.withdraw.entask.t.sol`. The integration tests do verify correctness, but the gap in the pattern is notable -- a sub-parser regression would only be caught by full integration tests, not targeted unit tests.

Note: The deposit/withdraw sub-parser functions use different context column mappings from the order-clearing words (they map to `CONTEXT_CALLING_CONTEXT_COLUMN` with deposit/withdraw row constants, but the context fixture `LibOrderBookSubParserContextFixture.hashedNamesContext()` does not include deposit/withdraw context columns). This means these words cannot be tested via the existing `OrderBookV6SubParserContextTest` pattern without extending the fixture.
