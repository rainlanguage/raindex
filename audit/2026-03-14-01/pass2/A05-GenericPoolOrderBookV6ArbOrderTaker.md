# Pass 2: Test Coverage -- A05 GenericPoolOrderBookV6ArbOrderTaker

**File:** `src/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sol`

## Evidence of Reading

### Source File (47 lines)

**Contract:** `GenericPoolOrderBookV6ArbOrderTaker is OrderBookV6ArbOrderTaker`

| Item | Kind | Line |
|------|------|------|
| `constructor(OrderBookV6ArbConfig memory config)` | constructor | 18 |
| `onTakeOrders2(address inputToken, address outputToken, Float inputAmountSent, Float totalOutputAmount, bytes calldata takeOrdersData)` | function (public virtual override) | 21-41 |
| `receive()` | special function (external payable) | 45 |
| `fallback()` | special function (external payable) | 46 |

**Imports used:** `IERC20`, `SafeERC20`, `Address`, `OrderBookV6ArbOrderTaker`, `OrderBookV6ArbConfig`, `Float`

**Inherited from `OrderBookV6ArbOrderTaker` (lines relevant to this concrete contract):**
- `arb5(IRaindexV6, TakeOrdersConfigV5, TaskV2)` -- entry point (line 38-64 of parent)
- `onTakeOrders2(...)` -- base no-op (line 70 of parent), overridden by this contract
- `supportsInterface(bytes4)` -- ERC165 (line 32-35 of parent)
- `nonReentrant` modifier from `ReentrancyGuard`
- `onlyValidTask(task)` modifier from `OrderBookV6ArbCommon`

**`onTakeOrders2` logic (lines 27-41):**
1. Calls `super.onTakeOrders2(...)` (no-op in parent)
2. `abi.decode(takeOrdersData, (address, address, bytes))` into `(spender, pool, encodedFunctionCall)`
3. `IERC20(inputToken).forceApprove(spender, type(uint256).max)` -- line 35
4. `pool.functionCallWithValue(encodedFunctionCall, address(this).balance)` -- line 39
5. `IERC20(inputToken).forceApprove(spender, 0)` -- line 40 (revoke)

### Test Files Read in Full

| # | Test file | Tests | What it covers |
|---|-----------|-------|----------------|
| 1 | `test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.expression.t.sol` | `testGenericPoolTakeOrdersWrongExpression`, `testGenericPoolTakeOrdersExpression` | WrongTask revert when task mismatch; correct expression passthrough via arb5 with mocked interpreter |
| 2 | `test/concrete/arb/GenericPoolOrderBookV6ArbOrderTaker.sender.t.sol` | `testGenericPoolTakeOrdersSender` | arb5 end-to-end with mocked OB (FlashLendingMockOrderBook + Refundoor) |
| 3 | `test/abstract/OrderBookV6ArbCommon.fallback.t.sol` | `testFallbackAcceptsCalldata`, `testFallbackAcceptsEmptyCalldata`, `testReceiveAcceptsETH`, `testFallbackAcceptsETHWithCalldata` | receive() and fallback() accept ETH and arbitrary calldata |
| 4 | `test/abstract/OrderBookV6ArbOrderTaker.onTakeOrders2.t.sol` | `testArb5RealTokenTransfers` | Full arb5 cycle with real ERC20 transfers through RealisticOrderTakerMockOrderBook + MockExchange; asserts all balances zero on arb at end |
| 5 | `test/abstract/OrderBookV6ArbOrderTaker.onTakeOrders2Direct.t.sol` | `testOnTakeOrders2DirectCallSucceeds` | Direct external call to onTakeOrders2 by arbitrary address succeeds (no access control by design); verifies contract remains empty |
| 6 | `test/abstract/OrderBookV6ArbOrderTaker.noOrders.t.sol` | `testArb5NoOrders` | arb5 reverts with `NoOrders` on empty orders array |
| 7 | `test/abstract/OrderBookV6ArbOrderTaker.reentrancy.t.sol` | `testArb5Reentrancy` | arb5 reverts with `ReentrancyGuardReentrantCall` when mock OB re-enters arb5 during takeOrders4 callback |
| 8 | `test/abstract/OrderBookV6ArbOrderTaker.ierc165.t.sol` | `testOrderBookV6ArbOrderTakerIERC165` | ERC165 interface reporting for IERC165, IRaindexV6ArbOrderTaker, IRaindexV6OrderTaker; fuzz rejects unknown IDs |
| 9 | `test/abstract/OrderBookV6ArbOrderTaker.context.t.sol` | `testOrderBookV6ArbOrderTakerContext` | Verifies context column values (input token, output token, gas balance) passed to finalizeArb task |
| 10 | `test/abstract/OrderBookV6ArbCommon.iTaskHash.t.sol` | `testITaskHashNonEmpty`, `testITaskHashEmpty` | iTaskHash set correctly for non-empty bytecode, zero for empty bytecode |
| 11 | `test/lib/LibOrderBookArb.finalizeArbTokenTransfers.t.sol` | `testFinalizeArbTransfersInputTokenProfit` | 20e18 profit swept to msg.sender; arb contract empty after |
| 12 | `test/lib/LibOrderBookArb.finalizeArbNativeGas.t.sol` | `testFinalizeArbSendsNativeGas` | ETH sent with arb5{value:}, forwarded to exchange via functionCallWithValue, returned, swept by finalizeArb |

### Proposed Fix File Already Present

| Fix file | Covers |
|----------|--------|
| `audit/2026-03-14-01/pass2/.fixes/GAP-A05-1-OrderTakerApprovalRevoked.t.sol` | Test asserting spender allowance is zero after onTakeOrders2 completes (approve-call-revoke) |

## Coverage Analysis

### Well-Covered

1. **Constructor** -- deployed in every test via `GenericPoolOrderBookV6ArbOrderTakerTest` base or directly. Construct event tested in `ArbTest`.
2. **`onTakeOrders2` full flow** -- `testArb5RealTokenTransfers` (test #4) drives the complete decode -> approve -> pool call -> revoke cycle with real ERC20 transfers and asserts final balances.
3. **`onTakeOrders2` direct call safety** -- `testOnTakeOrders2DirectCallSucceeds` (test #5) confirms an arbitrary caller can invoke it without extracting value.
4. **`receive()` and `fallback()`** -- four tests (test #3) covering ETH-only, calldata-only, ETH+calldata, and empty calldata.
5. **WrongTask revert** -- fuzz test (test #1) validates revert when evaluable does not match stored task.
6. **Expression passthrough** -- fuzz test (test #1) validates interpreter eval4 call with correct task.
7. **NoOrders revert** -- test #6.
8. **Reentrancy guard** -- test #7 via `ReentrantMockOrderBook`.
9. **ERC165 interface support** -- fuzz test (test #8).
10. **Context values in finalizeArb** -- test #9 with real interpreter.
11. **iTaskHash construction** -- test #10.
12. **Token profit sweep** -- test #11.
13. **Native gas sweep** -- test #12 with `arb5{value: 1 ether}`, confirming ETH flows through `functionCallWithValue` and back via `finalizeArb`.

### Gaps

#### GAP-A05-1: No dedicated test that spender allowance is zero after onTakeOrders2 (approve-call-revoke)

**Severity:** LOW

**Details:** The `GenericPoolOrderBookV6FlashBorrower` has a dedicated test (`GenericPoolOrderBookV6FlashBorrower.approvalRevoked.t.sol`) verifying the spender's allowance is zero after `_exchange` completes. The equivalent approve-call-revoke pattern in `GenericPoolOrderBookV6ArbOrderTaker.onTakeOrders2` (lines 35-40) has no dedicated assertion verifying the allowance is revoked to zero after the pool call. The existing `onTakeOrders2.t.sol` test exercises the full flow but does not assert the post-call allowance state.

A proposed fix is already present at `audit/2026-03-14-01/pass2/.fixes/GAP-A05-1-OrderTakerApprovalRevoked.t.sol` using `AllowanceCheckingExchange` to record `lastAllowance` during the call and then assert `allowance == 0` after arb5 completes.

#### GAP-A05-2: No test for pool call revert propagation (order-taker path)

**Severity:** LOW

**Details:** The flash borrower path has a dedicated test (`GenericPoolOrderBookV6FlashBorrower.exchangeRevert.t.sol`) verifying that when the exchange/pool call reverts, the revert reason bubbles up through the entire arb transaction. The order-taker path uses `Address.functionCallWithValue` on line 39, which also bubbles reverts, but there is no corresponding test for this contract. Since `Address.functionCallWithValue` is well-tested OpenZeppelin code and the flash borrower already validates the pattern, the practical risk is minimal, but parity with the flash borrower test suite would improve confidence.

#### GAP-A05-3: No test with spender != pool

**Severity:** INFO

**Details:** All existing tests use `spender == pool` (or `spender == exchange`). The `onTakeOrders2` logic supports `spender != pool` -- the approval goes to `spender` while the call goes to `pool`. No test exercises this split-address scenario. This is a design feature documented in the contract (the caller controls both addresses), but a test with distinct spender and pool addresses would confirm the approve targets the correct address independently of the call target.

## Findings

| ID | Severity | Title |
|----|----------|-------|
| A05-1 | LOW | No dedicated test for approval revocation after onTakeOrders2 |
| A05-2 | LOW | No test for pool call revert propagation (order-taker path) |
| A05-3 | INFO | No test with spender != pool |
