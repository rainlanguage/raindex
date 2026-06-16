# Pass 2: Test Coverage -- OrderBookV6.sol
**Agent:** A08
**File:** src/concrete/ob/OrderBookV6.sol

## Evidence of Thorough Reading

### Contract
- **Name:** `OrderBookV6` (line 198)
- **Inherits:** `IRaindexV6`, `IMetaV1_2`, `ReentrancyGuard`, `Multicall`, `OrderBookV6FlashLender`
- **Pragma:** `solidity =0.8.25` (line 3)

### State Variables
| Name | Type | Line |
|------|------|------|
| `sOrders` | `mapping(bytes32 => uint256)` | 215 |
| `sVaultBalances` | `mapping(address => mapping(address => mapping(bytes32 => Float)))` | 222 |

### Constants
| Name | Value | Line |
|------|-------|------|
| `ORDER_LIVE` | `1` | 129 |
| `ORDER_DEAD` | `0` | 134 |
| `CALCULATE_ORDER_ENTRYPOINT` | `SourceIndexV2.wrap(0)` | 137 |
| `HANDLE_IO_ENTRYPOINT` | `SourceIndexV2.wrap(1)` | 140 |
| `CALCULATE_ORDER_MIN_OUTPUTS` | `2` | 143 |
| `CALCULATE_ORDER_MAX_OUTPUTS` | `2` | 145 |
| `HANDLE_IO_MIN_OUTPUTS` | `0` | 148 |
| `HANDLE_IO_MAX_OUTPUTS` | `0` | 150 |

### Errors (all defined at file scope, lines 69-125)
| Error | Line |
|-------|------|
| `ReentrancyGuardReentrantCall()` | 69 |
| `NotOrderOwner(address)` | 73 |
| `TokenMismatch()` | 76 |
| `TokenSelfTrade()` | 79 |
| `TokenDecimalsMismatch()` | 83 |
| `MinimumIO(Float, Float)` | 88 |
| `SameOwner()` | 91 |
| `UnsupportedCalculateInputs(uint256)` | 95 |
| `UnsupportedCalculateOutputs(uint256)` | 99 |
| `NegativeInput()` | 102 |
| `NegativeOutput()` | 105 |
| `NegativeVaultBalance(Float)` | 109 |
| `NegativeVaultBalanceChange(Float)` | 113 |
| `NegativePull()` | 116 |
| `NegativePush()` | 119 |
| `NegativeBounty()` | 122 |
| `ClearZeroAmount()` | 125 |

### Types
| Name | Line |
|------|------|
| `OrderIOCalculationV4` (struct) | 181 |
| `Output18Amount` (type alias) | 192 |
| `Input18Amount` (type alias) | 194 |

### Public/External Functions
| Function | Visibility | Line | Modifier(s) |
|----------|------------|------|-------------|
| `vaultBalance2` | external view | 226 | -- |
| `orderExists` | external view | 249 | -- |
| `entask2` | external | 254 | nonReentrant |
| `deposit4` | external | 259 | nonReentrant, nonZeroVaultId |
| `withdraw4` | external | 294 | nonReentrant, nonZeroVaultId |
| `addOrder4` | external | 333 | nonReentrant |
| `removeOrder3` | external | 383 | nonReentrant |
| `quote2` | external view | 413 | -- |
| `takeOrders4` | external | 436 | nonReentrant |
| `clear3` | external | 596 | nonReentrant |

### Internal Functions
| Function | Visibility | Line |
|----------|------------|------|
| `_vaultBalance` | internal view | 231 |
| `checkTokenSelfTrade` | internal pure | 406 |
| `calculateOrderIO` | internal view | 701 |
| `increaseVaultBalance` | internal | 823 |
| `decreaseVaultBalance` | internal | 854 |
| `recordVaultIO` | internal | 892 |
| `handleIO` | internal | 916 |
| `calculateClearStateChange` | internal pure | 971 |
| `calculateClearStateAlice` | internal pure | 984 |
| `pullTokens` | internal | 1003 |
| `pushTokens` | internal | 1025 |
| `_nonZeroVaultId` | internal pure | 1045 |

### Modifier
| Modifier | Line |
|----------|------|
| `nonZeroVaultId` | 1051 |

### Test Files Reviewed (30 files)
- `OrderBookV6.deposit.t.sol` -- 7 tests
- `OrderBookV6.deposit.entask.t.sol` -- 8 tests
- `OrderBookV6.withdraw.t.sol` -- 6 tests
- `OrderBookV6.withdraw.entask.t.sol` -- 9 tests
- `OrderBookV6.addOrder.t.sol` -- 9 tests
- `OrderBookV6.addOrder.mock.t.sol` -- 8 tests
- `OrderBookV6.addOrder.nonce.t.sol` -- 3 tests
- `OrderBookV6.addOrder.owner.t.sol` -- 2 tests
- `OrderBookV6.addOrder.entask.t.sol` -- 9 tests
- `OrderBookV6.removeOrder.mock.t.sol` -- 5 tests
- `OrderBookV6.removeOrder.owner.t.sol` -- 3 tests
- `OrderBookV6.removeOrder.entask.t.sol` -- 8 tests
- `OrderBookV6.takeOrder.badStack.t.sol` -- 2 tests
- `OrderBookV6.takeOrder.handleIO.revert.t.sol` -- 14 tests
- `OrderBookV6.takeOrder.maximumInput.t.sol` -- 6 tests
- `OrderBookV6.takeOrder.maximumOutput.t.sol` -- 11 tests
- `OrderBookV6.takeOrder.noop.t.sol` -- 3 tests
- `OrderBookV6.takeOrder.precision.t.sol` -- 12 tests
- `OrderBookV6.takeOrder.sameToken.t.sol` -- 1 test
- `OrderBookV6.takeOrder.tokenMismatch.t.sol` -- 2 tests
- `OrderBookV6.clear.badStack.t.sol` -- 6 tests
- `OrderBookV6.clear.context.t.sol` -- 2 tests
- `OrderBookV6.clear.handleIO.revert.t.sol` -- 10 tests
- `OrderBookV6.clear.mock.t.sol` -- ~20 tests
- `OrderBookV6.clear.sameToken.t.sol` -- 1 test
- `OrderBookV6.clear.zeroAmount.t.sol` -- 1 test
- `OrderBookV6.quote.t.sol` -- 4 tests
- `OrderBookV6.quote.sameToken.t.sol` -- 1 test
- `OrderBookV6.vaultBalance.t.sol` -- 1 test
- `OrderBookV6.entask.t.sol` -- 5 tests

---

## Findings

### A08-1 [INFO] No direct test for `SameOwner` error in `clear3`

**Description:** The `clear3` function reverts with `SameOwner()` at line 605 when `aliceOrder.owner == bobOrder.owner`. No test file directly exercises this error path. The `clear.mock.t.sol` imports `SameOwner` but every test uses `vm.assume(alice != bob)` to avoid it. A test confirming the revert would strengthen coverage.

**Impact:** Informational. The check is simple and correct by inspection; this is purely a test gap.

**Test files searched:** All `OrderBookV6.clear.*.t.sol`

---

### A08-2 [INFO] No direct test for `TokenMismatch` error in `clear3`

**Description:** The `clear3` function at lines 607-614 checks that Alice's output token equals Bob's input token and vice-versa, reverting with `TokenMismatch()`. No clear test file exercises this path. The `TokenMismatch` error is tested only in `takeOrder.tokenMismatch.t.sol` for `takeOrders4`. The `clear3` code path is separate and could have a regression independently.

**Impact:** Informational. The logic is straightforward, but a regression test would be useful since `clear3` has its own token matching code.

**Test files searched:** All `OrderBookV6.clear.*.t.sol`

---

### A08-3 [INFO] No test for `MinimumIO` revert path in `takeOrders4`

**Description:** The `takeOrders4` function at lines 562-566 checks that the actual relevant IO meets the caller-specified `minimumIO`, reverting with `MinimumIO(minimumIO, actualRelevantTakerIO)`. No test exercises this error. Every test sets `minimumIO` to 0, meaning this revert path is never triggered.

**Impact:** Informational. The `minimumIO` check is the taker's primary protection against slippage. Without testing, a regression (e.g., comparing wrong values or wrong direction) could go undetected.

**Test files searched:** All `OrderBookV6.takeOrder.*.t.sol` and grep across all test files for `MinimumIO`.

---

### A08-4 [INFO] No test for `OrderExceedsMaxRatio` event emission in `takeOrders4`

**Description:** At line 503-504, when an order's IO ratio exceeds `config.maximumIORatio`, the order is skipped and the event `OrderExceedsMaxRatio` is emitted. No test exercises this path. All tests set `maximumIORatio` to `type(int224).max`, so this branch is never taken.

**Impact:** Informational. This is the taker's protection against overpaying for an order. An untested path could silently break.

**Test files searched:** All `OrderBookV6.takeOrder.*.t.sol` -- grep for `OrderExceedsMaxRatio` returned no matches.

---

### A08-5 [INFO] No test for `OrderZeroAmount` event emission in `takeOrders4`

**Description:** At lines 505-506, when `orderIOCalculation.outputMax.isZero()`, the order is skipped and `OrderZeroAmount` is emitted. No test directly triggers this event in the `takeOrders4` context. While `clear.zeroAmount.t.sol` tests zero output in `clear3`, the `takeOrders4` path is separate.

**Impact:** Informational. This path represents skipping zero-output orders; a test would confirm that correctly-evaluated zero-output orders are indeed skipped without reverting.

---

### A08-6 [INFO] No test for `onTakeOrders2` callback path in `takeOrders4`

**Description:** At lines 581-584, `takeOrders4` calls `IRaindexV6OrderTaker(msg.sender).onTakeOrders2(...)` when `config.data.length > 0`. All tests pass an empty `data` field, so this callback is never tested in the `OrderBookV6.takeOrder.*.t.sol` files. (There are separate arb-related test files that exercise similar callbacks but from the arb contract's perspective, not from `OrderBookV6.takeOrders4` directly.)

**Impact:** Informational. The callback is the mechanism for flash-swap-style integrations. Untested paths could mask bugs in parameter ordering or token direction.

---

### A08-7 [INFO] Unused error: `TokenDecimalsMismatch`

**Description:** The error `TokenDecimalsMismatch()` is declared at line 83 but is never used in `OrderBookV6.sol`. It is not thrown anywhere in the contract. This is dead code.

**Impact:** Informational. Dead error declarations create confusion about contract behavior.

---

### A08-8 [INFO] Unused errors: `NegativeInput`, `NegativeOutput`

**Description:** The errors `NegativeInput()` (line 102) and `NegativeOutput()` (line 105) are declared but never thrown in `OrderBookV6.sol`. The functions `increaseVaultBalance` and `decreaseVaultBalance` use `NegativeVaultBalanceChange` instead. These appear to be vestigial from an earlier design.

**Impact:** Informational. Dead error declarations; no test coverage needed but they should be cleaned up.

---

### A08-9 [INFO] Unused error: `UnsupportedCalculateInputs`

**Description:** The error `UnsupportedCalculateInputs(uint256)` is declared at line 95 but is never thrown in `OrderBookV6.sol`. In `calculateOrderIO` (line 788), only `UnsupportedCalculateOutputs` is checked. The `addOrder.t.sol` imports `UnsupportedCalculateInputs` but no test actually triggers it because the contract never throws it.

**Impact:** Informational. Dead code that creates false expectations about runtime behavior.

---

### A08-10 [INFO] No test for `NegativeVaultBalanceChange` revert in `increaseVaultBalance`/`decreaseVaultBalance`

**Description:** Both `increaseVaultBalance` (line 827) and `decreaseVaultBalance` (line 858) check for negative amounts and revert with `NegativeVaultBalanceChange`. No test exercises these paths. While negative Float values reaching these functions would be an unusual condition, the guard is important for correctness.

**Impact:** Informational. These are safety guards on internal functions. Testing would confirm they work as expected if an expression evaluates to a negative output or ratio.

---

### A08-11 [INFO] No test for `NegativePull`/`NegativePush` revert paths

**Description:** `pullTokens` (line 1008) reverts with `NegativePull()` and `pushTokens` (line 1031) reverts with `NegativePush()` when the Float amount is negative. No test triggers these paths.

**Impact:** Informational. These are safety guards. In normal operation, negative amounts should not reach these functions, but a crafted expression could potentially return negative values. Testing confirms the guard works.

---

### A08-12 [INFO] No test for `NegativeVaultBalance` revert in `increaseVaultBalance`

**Description:** At line 845, `increaseVaultBalance` checks that `newBalance` is not negative after adding a positive amount. The comment states "This should never be possible" but the check is there for safety. No test exercises this path.

**Impact:** Informational. While this condition is theoretically unreachable (adding a positive amount to any Float should not produce a negative result), the guard exists and should ideally have a test demonstrating it cannot be reached or demonstrating the revert if it could.

---

### A08-13 [INFO] No test for `NegativeVaultBalance` revert in `decreaseVaultBalance`

**Description:** At line 877, `decreaseVaultBalance` checks that `newBalance` is not negative after subtracting. No test exercises this path for the case where a withdrawal exceeds the vault balance through an internal path (not via `withdraw4`, which caps the amount). This could be triggered through `recordVaultIO` if an order expression returns values that cause the output to exceed the vault balance.

**Impact:** Informational. This is a critical safety check -- it prevents a vault from going negative. The `withdraw4` function caps withdrawal at the balance, but `clear3` and `takeOrders4` paths through `recordVaultIO` do not explicitly cap at the vault balance before calling `decreaseVaultBalance`. The cap is instead via `orderOutputMax` being min'd with `ownerVaultBalance` in `calculateOrderIO` (line 804). A test confirming the revert behavior would be valuable.

---

### A08-14 [INFO] No test for `NegativeBounty` revert in `clear3`

**Description:** At line 658, `clear3` checks if either the Alice or Bob bounty is negative and reverts with `NegativeBounty()`. While `clear.mock.t.sol` imports `NegativeBounty` and tests it in `testClearFuzzIoRatioError` (line 839 of the mock test), this test uses a mock interpreter. No real-interpreter test exercises this path.

**Impact:** Informational. The mock test provides adequate coverage for the revert logic. A real-interpreter test would add confidence.

---

### A08-15 [INFO] No test for dead order early-return paths in `clear3`

**Description:** At lines 626-633, `clear3` returns early (emitting `OrderNotFound`) if either Alice's or Bob's order is dead. No test exercises these specific paths. `takeOrders4` dead-order handling IS tested in `takeOrder.noop.t.sol`, but the `clear3` early-return paths are separate.

**Impact:** Informational. The early-return design (instead of reverting) is intentional for Multicall compatibility. Testing would confirm the `OrderNotFound` event is correctly emitted and no state changes occur.

---

### A08-16 [INFO] Unused type aliases `Output18Amount` and `Input18Amount`

**Description:** The type aliases `Output18Amount` (line 192) and `Input18Amount` (line 194) are declared but never used anywhere in the contract.

**Impact:** Informational. Dead code that should be cleaned up.

---

## Summary

The test suite is extensive and covers the major functional paths well:

**Well-tested areas:**
- Deposit (happy path, zero deposit, zero vault ID, reentrancy, many deposits, events, entask context)
- Withdraw (happy path, zero target, empty vault, full/partial, failure, many withdrawals, entask context)
- Add order (no inputs/outputs reverts, valid order emission, meta validation, nonce behavior, owner isolation, entask, idempotency)
- Remove order (owner check, add/remove cycles, dead order noop, different owners, entask, revert-in-action)
- Take orders (zero orders, dead orders, token mismatch, same-token, bad stack, handle IO revert, maximum input/output, precision, vault ID 0)
- Clear (bad stack, context, handle IO revert, same token, zero amount, vault ID 0 combinations, mock matching with various ratios)
- Quote (dead order, simple, vault-capped output, context access, same-token)
- Vault balance (empty read)
- Entask (stateless, stateful, namespaced)
- Flash lending (separate test directory)

**Coverage gaps (all INFO severity):**
- `SameOwner` error in `clear3` -- no test
- `TokenMismatch` error in `clear3` -- no test (only tested for `takeOrders4`)
- `MinimumIO` revert in `takeOrders4` -- no test (all tests use minimumIO=0)
- `OrderExceedsMaxRatio` skip in `takeOrders4` -- no test
- `OrderZeroAmount` skip in `takeOrders4` -- no test
- `onTakeOrders2` callback path -- no test in OB test files
- `NegativeVaultBalanceChange`, `NegativePull`, `NegativePush` reverts -- no tests
- `NegativeVaultBalance` reverts in increase/decrease -- no tests
- Dead order early-return in `clear3` -- no test
- Several unused/dead error declarations and type aliases

No MEDIUM or higher findings. The existing tests provide strong coverage of the core economic logic, and the gaps are limited to secondary error paths and guard rails.
