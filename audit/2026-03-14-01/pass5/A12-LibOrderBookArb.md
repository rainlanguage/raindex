# A12 - LibOrderBookArb.sol - Pass 5 (Correctness / Intent Verification)

**Source file:** `src/lib/LibOrderBookArb.sol` (77 lines)
**Test files:**
- `test/lib/LibOrderBookArb.finalizeArbNativeGas.t.sol` (110 lines)
- `test/lib/LibOrderBookArb.finalizeArbTokenTransfers.t.sol` (105 lines)

## Evidence Inventory

### LibOrderBookArb.sol
- **Library:** `LibOrderBookArb` (line 14)
- **Using:** `SafeERC20 for IERC20` (line 15)
- **Function:** `finalizeArb(TaskV2, address, uint8, address, uint8)` (line 20) - internal

### Imports
- `TaskV2` from `rain.raindex.interface`
- `IERC20` from OpenZeppelin
- `LibOrderBook` from `./LibOrderBook.sol`
- `Address` from OpenZeppelin
- `SafeERC20` from OpenZeppelin
- `IERC20Metadata` from OpenZeppelin -- **unused import** (see A12-1)
- `LibDecimalFloat, Float` from `rain.math.float`

---

## NatSpec vs. Implementation

### `finalizeArb` function (lines 17-75)

**NatSpec claim (line 17-19):** "Sends all remaining token balances and native gas to `msg.sender`, then evaluates the post-arb task with a context column containing the amounts sent as Floats."

**Verification step-by-step:**

1. **Input token transfer (lines 30-39):**
   - Gets `inputBalance = IERC20(ordersInputToken).balanceOf(address(this))` (line 32)
   - If > 0, transfers to `msg.sender` via `safeTransfer` (line 34)
   - Converts `inputBalance` to a `Float` via `fromFixedDecimalLossyPacked` using `inputDecimals` (line 37)
   - Stores in `col[0]` (line 38)
   - **Correct:** Sends all remaining input tokens to sender and records the amount as a Float.

2. **Output token transfer (lines 41-51):**
   - Gets `outputBalance = IERC20(ordersOutputToken).balanceOf(address(this))` (line 43)
   - If > 0, transfers to `msg.sender` via `safeTransfer` (line 45)
   - Converts to Float via `fromFixedDecimalLossyPacked` using `outputDecimals` (line 49)
   - Stores in `col[1]` (line 50)
   - **Correct:** Sends all remaining output tokens to sender and records the amount as a Float.

3. **Native gas transfer (lines 53-68):**
   - Gets `gasBalance = address(this).balance` (line 60)
   - If > 0, sends via `Address.sendValue` (line 62)
   - Converts to Float via `packLossless(int256(gasBalance), -18)` (line 67)
   - The `-18` exponent means the value is interpreted as having 18 decimals (native ETH convention)
   - Stores in `col[2]` (line 67)
   - **Correct:** Sends all remaining native gas to sender and records as a Float with 18-decimal precision.

4. **Post-arb task evaluation (lines 70-74):**
   - Creates context matrix with single column `col` (line 70)
   - Wraps task into a single-element `post` array (lines 72-73)
   - Calls `LibOrderBook.doPost(context, post)` (line 74)
   - **Correct:** Evaluates the post-arb task with the amounts as context.

**NatSpec accuracy:** Fully matches. The function does exactly what it claims -- sends all balances, then runs the post task with Float amounts as context.

### `int256(gasBalance)` cast safety (line 67)
- **Comment claim:** "gasBalance can't overflow int256 because there isn't enough gas in existence for that to happen on every production chain."
- **Verified:** `int256` max is ~5.78e76. The total supply of any native gas token on production chains (including ETH at ~120M * 1e18 = ~1.2e26) is far below this. The cast is safe.

### `packLossless` vs `fromFixedDecimalLossyPacked` (lines 37, 49, 67)
- For ERC20 tokens: uses `fromFixedDecimalLossyPacked` which accepts lossy conversion. This is appropriate because arbitrary token amounts may not fit precisely in the Float representation.
- For native gas: uses `packLossless` which reverts on lossy conversion. This is appropriate because the value and exponent are well-bounded for native gas amounts.
- Both approaches are intentional and documented.

---

## Test Correctness

### `testFinalizeArbSendsNativeGas` (lines 37-106 in finalizeArbNativeGas.t.sol)
- **Claim:** "finalizeArb MUST send native gas balance to msg.sender."
- **Setup:** Creates mock tokens, orderbook (pulls 100e18), exchange, arb contract. Sends 1 ETH with the `arb5` call. The MockExchange returns ETH back to the arb contract during the swap.
- **Assertions:**
  - `address(this).balance == senderBalanceBefore` -- sender gets ETH back (net zero)
  - `address(arb).balance == 0` -- arb contract is drained
  - `address(exchange).balance == 0` -- exchange is drained
- **Verified:** Correctly tests that `finalizeArb` sweeps all native gas back to `msg.sender`.

### `testFinalizeArbTransfersInputTokenProfit` (lines 33-104 in finalizeArbTokenTransfers.t.sol)
- **Claim:** "finalizeArb MUST transfer remaining input token profit to msg.sender."
- **Setup:** Creates mock tokens, orderbook (pulls only 80e18 instead of 100e18), exchange (gives 100e18 inputToken). This leaves 20e18 profit on the arb contract.
- **Assertions:**
  - `inputToken.balanceOf(address(this)) == 20e18` -- 20e18 profit swept to sender
  - `inputToken.balanceOf(address(arb)) == 0` -- arb contract is empty
  - `outputToken.balanceOf(address(arb)) == 0` -- arb contract is empty
  - `inputToken.balanceOf(address(orderBook)) == 80e18` -- OB got what it pulled
  - `outputToken.balanceOf(address(exchange)) == 100e18` -- exchange got its swap
- **Verified:** Correctly tests that `finalizeArb` sweeps all input token profit.

### Test coverage gaps
- No test verifies that `col[0]`, `col[1]`, `col[2]` context values are correctly passed to the post-arb task. The tests use empty-bytecode tasks (no interpreter), so the context contents are never asserted.
- No test covers the case where `ordersInputToken == ordersOutputToken` (same token for input and output). In this case, `finalizeArb` would transfer the full balance on the first `safeTransfer` call, then `balanceOf` returns 0 for the second check, which is correct behavior but untested.

---

## Findings

### A12-1: Unused import `IERC20Metadata` (INFO)

**File:** `src/lib/LibOrderBookArb.sol`, line 10

`IERC20Metadata` is imported but never referenced in the library. It was likely needed in a previous version that called `decimals()` directly, but the current code receives decimals as a parameter.

No production impact; purely a code hygiene issue.
