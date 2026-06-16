# Pass 5: Correctness — LibOrderBookArb.sol

**Agent:** A12
**File:** `src/lib/LibOrderBookArb.sol` (77 lines)

## Evidence of Thorough Reading

- Read all 77 lines in full
- Verified all imports (lines 5-11): `TaskV2`, `IERC20`, `LibOrderBook`, `Address`, `SafeERC20`, `IERC20Metadata`, `LibDecimalFloat`, `Float`
- Verified error declarations (lines 14-18): `NonZeroBeforeArbStack`, `BadLender`
- Verified library declaration with `using SafeERC20 for IERC20` (lines 20-21)
- Verified the single function `finalizeArb` (lines 23-76)
- Cross-referenced against both callers: `OrderBookV6ArbOrderTaker.arb5` (line 68) and `OrderBookV6FlashBorrower.arb4` (line 164)

## Correctness Verification

### Error Declarations

- `NonZeroBeforeArbStack()` (line 14): NatSpec says "Thrown when the stack is not empty after the access control dispatch." This error is defined here but used in `OrderBookV6ArbCommon` (the base contract for both arb contracts). The name and documentation match: it's for when the before-arb eval returns a non-empty stack. Correct.
- `BadLender(address badLender)` (line 17-18): NatSpec says "Thrown when the lender is not the trusted `OrderBook`." This error is also imported by the flash borrower. The parameter name `badLender` and documentation match. Correct.

### `finalizeArb` Function (lines 23-76)

**Parameters:**
- `task`: The post-arb task to evaluate
- `ordersInputToken`: The token that was input to the orders (i.e., what the taker sent to fill orders)
- `inputDecimals`: Decimals for the input token
- `ordersOutputToken`: The token that was output from the orders (i.e., what the taker received)
- `outputDecimals`: Decimals for the output token

**Step-by-step verification:**

1. **Context array creation** (lines 30-31): Creates a 1-column context with 3 rows. This will become column 1 (after `LibContext.build` prepends base at column 0 in `doPost`).

2. **Input token handling** (lines 34-42):
   - Gets balance of `ordersInputToken` held by `address(this)` (the arb contract)
   - If balance > 0, transfers all to `msg.sender` (the arb initiator)
   - Converts to float and stores in `col[0]`
   - NatSpec comment "Send all unspent input tokens to the sender" is accurate

3. **Output token handling** (lines 44-54):
   - Gets balance of `ordersOutputToken` held by `address(this)`
   - If balance > 0, transfers all to `msg.sender`
   - Converts to float and stores in `col[1]`
   - NatSpec comment "Send all unspent output tokens to the sender" is accurate

4. **Native gas handling** (lines 56-69):
   - Gets ETH balance of `address(this)`
   - Sends all to `msg.sender` via `Address.sendValue`
   - Converts to float with 18 decimals and stores in `col[2]`
   - NatSpec comment explains why sending all balance is correct (contract should be empty between uses)
   - The cast `int256(gasBalance)` is safe because there isn't enough ETH in existence to overflow int256

5. **Post task execution** (lines 71-75):
   - Sets `context[0] = col` (the 3-row column)
   - Wraps task in a 1-element array
   - Calls `LibOrderBook.doPost(context, post)`

**Caller verification:**

`OrderBookV6ArbOrderTaker.arb5` (lines 60-74):
- Sets `ordersInputToken` from `takeOrders.orders[0].order.validInputs[takeOrders.orders[0].inputIOIndex].token`
- Sets `ordersOutputToken` from `takeOrders.orders[0].order.validOutputs[takeOrders.orders[0].outputIOIndex].token`
- These are from the perspective of the ORDER (input = what the order receives, output = what the order sends)
- From the TAKER's perspective: the taker sends `ordersInputToken` and receives `ordersOutputToken`
- After `takeOrders4`, the arb contract may have leftover `ordersInputToken` (if the taker overpaid) and `ordersOutputToken` (profit)
- `finalizeArb` correctly sweeps both to the sender

`OrderBookV6FlashBorrower.arb4` (lines 130-165):
- Same token extraction logic
- Flash borrows `ordersOutputToken` (what the orders will send out, which the flash borrower needs to sell on external liq)
- After exchange + takeOrders + flash loan repayment, leftover tokens are swept by `finalizeArb`
- Correct behavior

### Float Conversion

- `LibDecimalFloat.fromFixedDecimalLossyPacked(inputBalance, inputDecimals)` -- uses the "lossy" variant, discarding the `lossless` return value. This is acceptable because the context column is informational for the post-arb task, not used for accounting.
- `LibDecimalFloat.packLossless(int256(gasBalance), -18)` -- uses lossless for ETH (18 decimals is standard). This will revert if the value can't be represented losslessly, which is acceptable.

### Unused Import

`IERC20Metadata` is imported (line 10) but never used in the library. This was noted in previous passes.

## Findings

No new findings at LOW or above. The function correctly:
1. Sweeps all remaining tokens and ETH to the arb initiator
2. Provides swept amounts as context to the post-arb task
3. Delegates to `LibOrderBook.doPost` for task evaluation

The context layout (input balance at row 0, output balance at row 1, gas balance at row 2) is self-consistent and matches the expectations of callers.

## Summary

| ID | Severity | Description |
|---|---|---|
| (none) | -- | -- |

`finalizeArb` behavior matches its callers' expectations precisely. Both `OrderBookV6ArbOrderTaker.arb5` and `OrderBookV6FlashBorrower.arb4` call it after their respective order-taking strategies, and it correctly sweeps residual balances and runs the post-arb task.
