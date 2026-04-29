# A12: LibOrderBookArb.sol -- Test Coverage Audit

## Source File
`src/lib/LibOrderBookArb.sol`

## Evidence of Thorough Reading

### Function: `finalizeArb` (lines 20-75)
- Takes `task`, `ordersInputToken`, `inputDecimals`, `ordersOutputToken`, `outputDecimals`
- Creates a 1-column context array with 3 entries (input balance as Float, output balance as Float, gas balance as Float)
- Block 1 (lines 31-39): Gets input token balance, transfers if > 0, converts to Float via `fromFixedDecimalLossyPacked`
- Block 2 (lines 41-51): Gets output token balance, transfers if > 0, converts to Float via `fromFixedDecimalLossyPacked`
- Block 3 (lines 53-68): Gets native gas balance, sends via `Address.sendValue` if > 0, converts to Float via `packLossless` with exponent -18
- Wraps task into a single-element `TaskV2[]` and calls `LibOrderBook.doPost(context, post)`

## Test Files Found
- `test/lib/LibOrderBookArb.finalizeArbTokenTransfers.t.sol` -- 1 test: `testFinalizeArbTransfersInputTokenProfit`
- `test/lib/LibOrderBookArb.finalizeArbNativeGas.t.sol` -- 1 test: `testFinalizeArbSendsNativeGas`

## Coverage Gaps

### GAP-A12-1: No test for zero-balance token transfers (both tokens empty)
**Severity**: Low
**Details**: `finalizeArb` checks `if (inputBalance > 0)` and `if (outputBalance > 0)` before transferring. No test exercises the path where both token balances are zero after arb (i.e., the OB consumed everything). The existing `testFinalizeArbTransfersInputTokenProfit` test has 20e18 input token profit remaining and 0 output token remaining, but there is no explicit assertion that `safeTransfer` was NOT called for the output token with zero balance.

### GAP-A12-2: No test for output token profit (only input token profit tested)
**Severity**: Medium
**Details**: `testFinalizeArbTransfersInputTokenProfit` only tests the case where the arber profits in the input token. There is no test where the arber has remaining output token balance that needs to be swept. While the code path is symmetric, the output token sweep at line 45 is untested with a nonzero balance.

### GAP-A12-3: No test for post-arb task evaluation with context verification
**Severity**: Medium
**Details**: Both existing tests pass an empty task (zero-address interpreter, empty bytecode) so `doPost` is effectively a no-op. There is no test that verifies the context column passed to the post-arb task contains the correct Float-encoded balances (col[0] = input, col[1] = output, col[2] = gas). The context data is constructed but never verified by a test.

### GAP-A12-4: No fuzz testing on finalizeArb
**Severity**: Low
**Details**: Both tests use fixed values (80e18, 100e18, 1 ether). There is no fuzz test covering different decimal values, edge-case amounts, or the interaction between `fromFixedDecimalLossyPacked` and `packLossless` with boundary values.
