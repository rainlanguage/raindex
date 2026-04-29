# Pass 2: Test Coverage -- A08 OrderBookV6

**File:** src/concrete/ob/OrderBookV6.sol

## Evidence of Reading

### Contract: `OrderBookV6` (line 191)
Inherits: `IRaindexV6`, `IMetaV1_2`, `ReentrancyGuard`, `Multicall`, `OrderBookV6FlashLender`

### Errors (lines 69-125)
- `ReentrancyGuardReentrantCall` (line 69)
- `NotOrderOwner(address owner)` (line 73)
- `TokenMismatch` (line 76)
- `TokenSelfTrade` (line 79)
- `TokenDecimalsMismatch` (line 83)
- `MinimumIO(Float minimumIO, Float actualIO)` (line 88)
- `SameOwner` (line 91)
- `UnsupportedCalculateInputs(uint256 inputs)` (line 95)
- `UnsupportedCalculateOutputs(uint256 outputs)` (line 99)
- `NegativeInput` (line 102)
- `NegativeOutput` (line 105)
- `NegativeVaultBalance(Float vaultBalance)` (line 109)
- `NegativeVaultBalanceChange(Float amount)` (line 113)
- `NegativePull` (line 116)
- `NegativePush` (line 119)
- `NegativeBounty` (line 122)
- `ClearZeroAmount` (line 125)

### Constants (lines 129-150)
- `ORDER_LIVE = 1` (line 129)
- `ORDER_DEAD = 0` (line 134)
- `CALCULATE_ORDER_ENTRYPOINT = SourceIndexV2.wrap(0)` (line 137)
- `HANDLE_IO_ENTRYPOINT = SourceIndexV2.wrap(1)` (line 140)
- `CALCULATE_ORDER_MIN_OUTPUTS = 2` (line 143)
- `CALCULATE_ORDER_MAX_OUTPUTS = 2` (line 145)
- `HANDLE_IO_MIN_OUTPUTS = 0` (line 148)
- `HANDLE_IO_MAX_OUTPUTS = 0` (line 150)

### Struct (lines 178-187)
- `OrderIOCalculationV4` (line 178)

### State Variables (lines 208-216)
- `sOrders` mapping (line 208)
- `sVaultBalances` mapping (line 215)

### Functions
- `vaultBalance2(address, address, bytes32) external view` (line 219)
- `_vaultBalance(address, address, bytes32) internal view` (line 226)
- `orderExists(bytes32) external view` (line 244)
- `entask2(TaskV2[] calldata) external` (line 249)
- `deposit4(address, bytes32, Float, TaskV2[] calldata) external` (line 254)
- `withdraw4(address, bytes32, Float, TaskV2[] calldata) external` (line 289)
- `addOrder4(OrderConfigV4 calldata, TaskV2[] calldata) external` (line 328)
- `removeOrder3(OrderV4 calldata, TaskV2[] calldata) external` (line 378)
- `checkTokenSelfTrade(OrderV4, uint256, uint256) internal pure` (line 403)
- `quote2(QuoteV2 calldata) external view` (line 410)
- `takeOrders4(TakeOrdersConfigV5 calldata) external` (line 433)
- `clear3(OrderV4, OrderV4, ClearConfigV2, SignedContextV1[], SignedContextV1[]) external` (line 593)
- `calculateOrderIO(OrderV4, uint256, uint256, address, SignedContextV1[]) internal view` (line 698)
- `increaseVaultBalance(address, address, bytes32, Float) internal` (line 822)
- `decreaseVaultBalance(address, address, bytes32, Float) internal` (line 855)
- `recordVaultIO(Float, Float, OrderIOCalculationV4) internal` (line 893)
- `handleIO(OrderIOCalculationV4) internal` (line 919)
- `calculateClearStateChange(OrderIOCalculationV4, OrderIOCalculationV4) internal pure` (line 971)
- `calculateClearStateAlice(OrderIOCalculationV4, OrderIOCalculationV4) internal pure` (line 987)
- `pullTokens(address, address, Float) internal` (line 1008)
- `pushTokens(address, address, Float) internal` (line 1032)
- `_nonZeroVaultId(address, address, bytes32) internal pure` (line 1052)
- `nonZeroVaultId modifier` (line 1058)

### Test Files Read
- `test/concrete/ob/OrderBookV6.deposit.t.sol` -- deposit basic, zero, zero vault id, gas, failure, many, event, reentrancy
- `test/concrete/ob/OrderBookV6.deposit.entask.t.sol` -- deposit entask, context, revert in action
- `test/concrete/ob/OrderBookV6.withdraw.t.sol` -- withdraw zero vault id, zero, empty vault, full, partial, failure, many
- `test/concrete/ob/OrderBookV6.withdraw.entask.t.sol` -- withdraw entask, context, revert in action, zero amount eval
- `test/concrete/ob/OrderBookV6.addOrder.t.sol` -- add order real (various stack sizes, inputs)
- `test/concrete/ob/OrderBookV6.addOrder.mock.t.sol` -- add without calculations/inputs/outputs, with meta, two accounts
- `test/concrete/ob/OrderBookV6.addOrder.entask.t.sol` -- add order entask, context, revert blocks add, noop on live
- `test/concrete/ob/OrderBookV6.addOrder.nonce.t.sol` -- nonce same order noop, different nonce state change
- `test/concrete/ob/OrderBookV6.addOrder.owner.t.sol` -- owner same order noop, different owner state change
- `test/concrete/ob/OrderBookV6.removeOrder.mock.t.sol` -- only owner, add/remove multi, does not exist, different orders/owners
- `test/concrete/ob/OrderBookV6.removeOrder.owner.t.sol` -- owner noop, different owner, wrong owner
- `test/concrete/ob/OrderBookV6.removeOrder.entask.t.sol` -- remove entask, context, dead order noop, revert blocks removal
- `test/concrete/ob/OrderBookV6.entask.t.sol` -- entask basic, stateless, read state, write state, namespaced
- `test/concrete/ob/OrderBookV6.takeOrder.noop.t.sol` -- zero orders, non-live order (one and two)
- `test/concrete/ob/OrderBookV6.takeOrder.tokenMismatch.t.sol` -- input/output token mismatch
- `test/concrete/ob/OrderBookV6.takeOrder.sameToken.t.sol` -- same token self trade
- `test/concrete/ob/OrderBookV6.takeOrder.badStack.t.sol` -- empty and one stack
- `test/concrete/ob/OrderBookV6.takeOrder.handleIO.revert.t.sol` -- handle IO revert scenarios (0-10), missing handle IO, vault 0
- `test/concrete/ob/OrderBookV6.takeOrder.maximumInput.t.sol` -- max input single/multi order, vault limited, vault 0
- `test/concrete/ob/OrderBookV6.takeOrder.maximumOutput.t.sol` -- max output single/multi order, vault limited, vault 0
- `test/concrete/ob/OrderBookV6.takeOrder.precision.t.sol` -- precision known-bad values, various decimal combos, vault 0
- `test/concrete/ob/OrderBookV6.clear.mock.t.sol` -- comprehensive clear mocking with various IO ratios, NegativeBounty
- `test/concrete/ob/OrderBookV6.clear.zeroAmount.t.sol` -- clear zero amount
- `test/concrete/ob/OrderBookV6.clear.sameToken.t.sol` -- clear same token
- `test/concrete/ob/OrderBookV6.clear.badStack.t.sol` -- clear bad stack (empty, one, mixed)
- `test/concrete/ob/OrderBookV6.clear.handleIO.revert.t.sol` -- clear handle IO revert, missing handle IO, vault 0
- `test/concrete/ob/OrderBookV6.clear.context.t.sol` (not fully read but referenced in glob)
- `test/concrete/ob/OrderBookV6.quote.t.sol` -- quote dead order, simple, max output, context/sender, vault 0
- `test/concrete/ob/OrderBookV6.quote.sameToken.t.sol` -- quote same token error
- `test/concrete/ob/OrderBookV6.vaultBalance.t.sol` -- vault balance no deposits

## Findings

### A08-1: No test for `MinimumIO` revert in `takeOrders4`

**Severity:** MEDIUM

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 558-563

The `takeOrders4` function checks whether the actual relevant taker IO meets the `minimumIO` threshold (line 560) and reverts with `MinimumIO(config.minimumIO, actualRelevantTakerIO)` if it does not. There are no tests anywhere in the test suite that trigger this `MinimumIO` revert. All test configurations set `minimumIO` to 0. This is a financially significant guard -- if untested, a regression could allow takers to accept trades below their minimum threshold without error, causing unexpected losses.

---

### A08-2: No test for `SameOwner` revert in `clear3`

**Severity:** MEDIUM

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 601-603

The `clear3` function checks `aliceOrder.owner == bobOrder.owner` and reverts with `SameOwner()` on line 602. There is no dedicated test that exercises this code path. The `clear.mock.t.sol` test always ensures `alice != bob`. A missing guard for self-clearing could allow an owner to clear against themselves, which may have unintended economic consequences (e.g., manipulating vault balances or extracting bounties).

---

### A08-3: No test for `OrderExceedsMaxRatio` event/skip path in `takeOrders4`

**Severity:** MEDIUM

**Location:** `src/concrete/ob/OrderBookV6.sol` line 500-501

When an order's `IORatio` exceeds `config.maximumIORatio`, the order is skipped and `OrderExceedsMaxRatio` is emitted. There are no tests that set a meaningful `maximumIORatio` below the order's ratio to verify this skip path. All tests set `maximumIORatio` to `type(int224).max`. This means the taker's max-ratio protection is untested -- a regression allowing expensive orders to execute despite the cap would go unnoticed.

---

### A08-4: No test for `OrderZeroAmount` event/skip path in `takeOrders4`

**Severity:** LOW

**Location:** `src/concrete/ob/OrderBookV6.sol` line 502-503

When an order's `outputMax` is zero, the order is skipped with `OrderZeroAmount` emitted. While the clear.mock.t.sol tests exercise zero-amount outputs in clearing, no `takeOrders4` test specifically verifies that a live order with zero `outputMax` is properly skipped (as opposed to dead orders, which are tested). The take-order code path handles this differently from dead orders -- it evaluates the expression first, then skips based on the result.

---

### A08-5: No test for `NegativeVaultBalance` revert in `increaseVaultBalance` / `decreaseVaultBalance`

**Severity:** LOW

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 844-846 (increase) and 878-879 (decrease)

The `increaseVaultBalance` function has a guard against a negative resulting balance (line 844), though the comment acknowledges this "should never be possible." The `decreaseVaultBalance` function guards against the balance going negative (line 878), which can happen if an order tries to output more than its vault holds. Neither `NegativeVaultBalance` revert path is directly tested. For `decreaseVaultBalance`, the outputMax is capped at vault balance in `calculateOrderIO` (line 801), but a direct test that the guard fires correctly if somehow triggered would improve confidence.

---

### A08-6: No test for `NegativeVaultBalanceChange` revert

**Severity:** LOW

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 826-828 and 859-861

Both `increaseVaultBalance` and `decreaseVaultBalance` check that the `amount` parameter is not negative and revert with `NegativeVaultBalanceChange(amount)` if it is. These guards are defensive checks for internal invariants. No tests exercise these revert paths.

---

### A08-7: No test for `NegativePull` and `NegativePush` reverts

**Severity:** LOW

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 1013-1014 (NegativePull) and 1038-1039 (NegativePush)

The `pullTokens` and `pushTokens` functions guard against negative amounts. These are defensive checks since the code should never call these with negative values, but no tests verify the revert messages are correct or that the guards fire.

---

### A08-8: No test for `IOIsInput = false` branch correctness in `takeOrders4` with `minimumIO` check

**Severity:** LOW

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 522-530, 559

When `IOIsInput` is false, `remainingTakerIO` tracks the output side and the `minimumIO` check applies to `totalTakerOutput`. The `takeOrder.maximumOutput.t.sol` file tests the output path mechanics, but `minimumIO` is always set to 0. There is no test verifying that the `minimumIO` check works correctly in `IOIsInput = false` mode -- i.e., that the minimum is correctly checked against `totalTakerOutput` rather than `totalTakerInput`.

---

### A08-9: `TokenDecimalsMismatch` error is declared but never used

**Severity:** INFO

**Location:** `src/concrete/ob/OrderBookV6.sol` line 83

The error `TokenDecimalsMismatch` is declared at line 83 but is never thrown anywhere in the contract. It appears to be dead code. No test references it either.

---

### A08-10: `NegativeInput` and `NegativeOutput` errors are declared but never used

**Severity:** INFO

**Location:** `src/concrete/ob/OrderBookV6.sol` lines 102, 105

The errors `NegativeInput` (line 102) and `NegativeOutput` (line 105) are declared but never thrown anywhere in the contract. They appear to be dead code from a previous version. No tests reference them.

---

### A08-11: `UnsupportedCalculateInputs` error is declared but never used

**Severity:** INFO

**Location:** `src/concrete/ob/OrderBookV6.sol` line 95

The error `UnsupportedCalculateInputs(uint256 inputs)` is declared and imported in `OrderBookV6AddOrderTest` but is never actually thrown by the contract. The `calculateOrderIO` function only checks minimum outputs (line 785), not inputs. The interpreter itself enforces input handling. This is dead code.

---

### A08-12: No test for `Multicall` functionality

**Severity:** LOW

**Location:** `src/concrete/ob/OrderBookV6.sol` line 191

`OrderBookV6` inherits `Multicall` from OpenZeppelin, which allows batching multiple calls. There are no tests verifying that `multicall` works correctly with OrderBookV6's functions, particularly around reentrancy guard interactions (each sub-call in multicall runs as a delegatecall in the same transaction context). The clear function comment at line 620-622 mentions using `Multicall` for bulk clearing, but this pattern is not tested.

---

### A08-13: No test for `supportsInterface` (ERC165)

**Severity:** INFO

**Location:** Inherited via `OrderBookV6FlashLender` -> `ERC165`

The `OrderBookV6FlashLender` tests in `test/abstract/` do test `supportsInterface` for the flash lender interface, but there is no test verifying `supportsInterface` on the concrete `OrderBookV6` contract directly. Since `OrderBookV6` does not override `supportsInterface`, this is covered by the abstract tests, but a direct integration test on the deployed contract would confirm correct behavior.
